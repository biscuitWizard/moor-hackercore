use sled::Tree;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use tokio::sync::mpsc;

use super::{ProviderError, ProviderResult};

/// Represents a reference to an object with its current version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRef {
    pub object_name: String,
    pub version: u64, // Monotonic version number
    pub sha256_key: String, // SHA256 hash of the object dump
}

/// Provider trait for reference management
pub trait RefsProvider: Send + Sync {
    /// Get reference for an object by name
    fn get_ref(&self, object_name: &str) -> ProviderResult<Option<ObjectRef>>;
    
    /// Update or create a reference
    fn update_ref(&self, object_ref: &ObjectRef) -> ProviderResult<()>;
    
    /// Get the latest version number for an object
    #[allow(dead_code)]
    fn get_latest_version(&self, object_name: &str) -> ProviderResult<u64>;
    
    /// Get the next version number for an object
    fn get_next_version(&self, object_name: &str) -> ProviderResult<u64>;
    
    /// Resolve object name + optional version to SHA256 key
    #[allow(dead_code)]
    fn resolve_to_sha256(&self, object_name: &str, version: Option<u64>) -> ProviderResult<Option<String>>;
    
    /// Delete a reference
    #[allow(dead_code)]
    fn delete_ref(&self, object_name: &str) -> ProviderResult<bool>;
}

/// Implementation of RefsProvider using Sled
pub struct RefsProviderImpl {
    refs_tree: Tree,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl RefsProviderImpl {
    /// Create a new refs provider
    pub fn new(refs_tree: Tree, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { refs_tree, flush_sender }
    }
}

impl RefsProvider for RefsProviderImpl {
    fn get_ref(&self, object_name: &str) -> ProviderResult<Option<ObjectRef>> {
        match self.refs_tree.get(object_name.as_bytes())? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let object_ref: ObjectRef = serde_json::from_str(&json)
                    .map_err(|e| ProviderError::SerializationError(format!("JSON parse error: {e}")))?;
                Ok(Some(object_ref))
            }
            None => Ok(None),
        }
    }

    fn update_ref(&self, object_ref: &ObjectRef) -> ProviderResult<()> {
        let json = serde_json::to_string(object_ref)
            .map_err(|e| ProviderError::SerializationError(format!("JSON serialization error: {e}")))?;
        self.refs_tree.insert(object_ref.object_name.as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Updated ref for object '{}' version {}", object_ref.object_name, object_ref.version);
        Ok(())
    }

    fn get_latest_version(&self, object_name: &str) -> ProviderResult<u64> {
        match self.get_ref(object_name)? {
            Some(ref_) => Ok(ref_.version),
            None => Err(ProviderError::ReferenceNotFound(object_name.to_string())),
        }
    }

    fn get_next_version(&self, object_name: &str) -> ProviderResult<u64> {
        match self.get_ref(object_name)? {
            Some(ref_) => Ok(ref_.version + 1),
            None => Ok(1), // First version
        }
    }

    fn resolve_to_sha256(&self, object_name: &str, version: Option<u64>) -> ProviderResult<Option<String>> {
        if let Some(target_version) = version {
            // Specific version requested
            if let Some(object_ref) = self.get_ref(object_name)? {
                if object_ref.version == target_version {
                    Ok(Some(object_ref.sha256_key))
                } else {
                    Err(ProviderError::VersionNotFound(
                        format!("Object '{}' version {} not found (latest is {})", 
                                object_name, target_version, object_ref.version)
                    ))
                }
            } else {
                Err(ProviderError::ReferenceNotFound(object_name.to_string()))
            }
        } else {
            // Latest version requested
            match self.get_ref(object_name)? {
                Some(object_ref) => Ok(Some(object_ref.sha256_key)),
                None => Ok(None),
            }
        }
    }

    fn delete_ref(&self, object_name: &str) -> ProviderResult<bool> {
        let result = self.refs_tree.remove(object_name.as_bytes())?;
        Ok(result.is_some())
    }
}
