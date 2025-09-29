use sled::Tree;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use tokio::sync::mpsc;

use super::{ProviderError, ProviderResult};

/// Represents the current working state (HEAD) which references object versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadState {
    pub refs: Vec<HeadRef>, // List of object_name + version pairs
}

/// Represents a reference in the HEAD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadRef {
    pub object_name: String,
    pub version: u64,
}

/// Provider trait for HEAD state management
pub trait HeadProvider: Send + Sync {
    /// Get the current HEAD state
    fn get_head(&self) -> ProviderResult<HeadState>;
    
    /// Update the HEAD state
    fn set_head(&self, head: &HeadState) -> ProviderResult<()>;
    
    /// Add or update a reference in the HEAD
    fn update_ref(&self, object_name: &str, version: u64) -> ProviderResult<()>;
    
    /// Remove a reference from HEAD
    #[allow(dead_code)]
    fn remove_ref(&self, object_name: &str) -> ProviderResult<()>;
    
    /// Check if an object is referenced in HEAD
    #[allow(dead_code)]
    fn contains_ref(&self, object_name: &str) -> ProviderResult<bool>;
    
    /// Get HEAD version for a specific object
    #[allow(dead_code)]
    fn get_ref_version(&self, object_name: &str) -> ProviderResult<Option<u64>>;
    
    /// List all HEAD references
    #[allow(dead_code)]
    fn list_refs(&self) -> ProviderResult<Vec<HeadRef>>;
}

/// Implementation of HeadProvider using Sled
pub struct HeadProviderImpl {
    head_tree: Tree,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl HeadProviderImpl {
    /// Create a new head provider
    pub fn new(head_tree: Tree, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { head_tree, flush_sender }
    }
}

impl HeadProvider for HeadProviderImpl {
    fn get_head(&self) -> ProviderResult<HeadState> {
        match self.head_tree.get("state".as_bytes())? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let head: HeadState = serde_json::from_str(&json)
                    .map_err(|e| ProviderError::SerializationError(format!("JSON parse error: {e}")))?;
                Ok(head)
            }
            None => {
                // Return default HEAD (empty)
                Ok(HeadState { refs: Vec::new() })
            }
        }
    }

    fn set_head(&self, head: &HeadState) -> ProviderResult<()> {
        let json = serde_json::to_string(head)
            .map_err(|e| ProviderError::SerializationError(format!("JSON serialization error: {e}")))?;
        self.head_tree.insert("state".as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Updated HEAD state with {} refs", head.refs.len());
        Ok(())
    }

    fn update_ref(&self, object_name: &str, version: u64) -> ProviderResult<()> {
        let mut head = self.get_head()?;
        
        // Check if ref already exists and update, or add new
        if let Some(head_ref) = head.refs.iter_mut().find(|hr| hr.object_name == object_name) {
            head_ref.version = version;
            info!("Updated HEAD ref for object '{}' to version {}", object_name, version);
        } else {
            head.refs.push(HeadRef {
                object_name: object_name.to_string(),
                version,
            });
            info!("Added HEAD ref for object '{}' at version {}", object_name, version);
        }
        
        self.set_head(&head)
    }

    fn remove_ref(&self, object_name: &str) -> ProviderResult<()> {
        let mut head = self.get_head()?;
        head.refs.retain(|hr| hr.object_name != object_name);
        self.set_head(&head)?;
        info!("Removed HEAD ref for object '{}'", object_name);
        Ok(())
    }

    fn contains_ref(&self, object_name: &str) -> ProviderResult<bool> {
        let head = self.get_head()?;
        Ok(head.refs.iter().any(|hr| hr.object_name == object_name))
    }

    fn get_ref_version(&self, object_name: &str) -> ProviderResult<Option<u64>> {
        let head = self.get_head()?;
        Ok(head.refs.iter()
            .find(|hr| hr.object_name == object_name)
            .map(|hr| hr.version))
    }

    fn list_refs(&self) -> ProviderResult<Vec<HeadRef>> {
        let head = self.get_head()?;
        Ok(head.refs)
    }
}
