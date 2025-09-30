use fjall::Partition;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use tokio::sync::mpsc;
use std::collections::HashMap;

use super::{ProviderError, ProviderResult};
use crate::types::ObjectInfo;


/// Represents the refs storage as a HashMap where key is ObjectInfo and value is sha256
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefsStorage {
    pub refs: HashMap<ObjectInfo, String>, // ObjectInfo -> sha256
}

/// Provider trait for reference management
pub trait RefsProvider: Send + Sync {
    /// Get SHA256 for an object by name and optional version (defaults to latest)
    fn get_ref(&self, object_name: &str, version: Option<u64>) -> ProviderResult<Option<String>>;
    
    /// Update or create a reference
    fn update_ref(&self, object_name: &str, version: u64, sha256: &str) -> ProviderResult<()>;
    
    /// Get the next version number for an object
    fn get_next_version(&self, object_name: &str) -> ProviderResult<u64>;
    
    /// Get all refs as a HashMap (for export/cloning)
    fn get_all_refs(&self) -> ProviderResult<HashMap<ObjectInfo, String>>;
    
    /// Clear all refs from storage
    fn clear(&self) -> ProviderResult<()>;
}

/// Implementation of RefsProvider using Fjall
pub struct RefsProviderImpl {
    refs_tree: Partition,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl RefsProviderImpl {
    /// Create a new refs provider
    pub fn new(refs_tree: Partition, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { refs_tree, flush_sender }
    }
    
    /// Load the refs storage from the database
    fn load_refs_storage(&self) -> ProviderResult<RefsStorage> {
        match self.refs_tree.get(b"refs_storage")? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let storage: RefsStorage = serde_json::from_str(&json)
                    .map_err(|e| ProviderError::SerializationError(format!("JSON parse error: {e}")))?;
                Ok(storage)
            }
            None => Ok(RefsStorage {
                refs: HashMap::new(),
            }),
        }
    }
    
    /// Save the refs storage to the database
    fn save_refs_storage(&self, storage: &RefsStorage) -> ProviderResult<()> {
        let json = serde_json::to_string(storage)
            .map_err(|e| ProviderError::SerializationError(format!("JSON serialization error: {e}")))?;
        self.refs_tree.insert(b"refs_storage", json.as_bytes())?;
        Ok(())
    }
}

impl RefsProvider for RefsProviderImpl {
    fn get_ref(&self, object_name: &str, version: Option<u64>) -> ProviderResult<Option<String>> {
        let storage = self.load_refs_storage()?;
        
        if let Some(target_version) = version {
            // Specific version requested
            let object_info = ObjectInfo {
                name: object_name.to_string(),
                version: target_version,
            };
            Ok(storage.refs.get(&object_info).cloned())
        } else {
            // Latest version requested
            let latest_version = storage.refs.keys()
                .filter(|obj_info| obj_info.name == object_name)
                .map(|obj_info| obj_info.version)
                .max();
                
            if let Some(version) = latest_version {
                let object_info = ObjectInfo {
                    name: object_name.to_string(),
                    version,
                };
                Ok(storage.refs.get(&object_info).cloned())
            } else {
                Ok(None)
            }
        }
    }


    fn update_ref(&self, object_name: &str, version: u64, sha256: &str) -> ProviderResult<()> {
        let mut storage = self.load_refs_storage()?;
        
        // Add the new reference to the HashMap
        let object_info = ObjectInfo {
            name: object_name.to_string(),
            version,
        };
        storage.refs.insert(object_info, sha256.to_string());
        
        // Save the updated storage
        self.save_refs_storage(&storage)?;
        
        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Updated ref for object '{}' version {}", object_name, version);
        Ok(())
    }


    fn get_next_version(&self, object_name: &str) -> ProviderResult<u64> {
        let storage = self.load_refs_storage()?;
        
        let latest_version = storage.refs.keys()
            .filter(|obj_info| obj_info.name == object_name)
            .map(|obj_info| obj_info.version)
            .max();
            
        match latest_version {
            Some(version) => Ok(version + 1),
            None => Ok(1), // First version
        }
    }
    
    fn get_all_refs(&self) -> ProviderResult<HashMap<ObjectInfo, String>> {
        let storage = self.load_refs_storage()?;
        Ok(storage.refs.clone())
    }
    
    fn clear(&self) -> ProviderResult<()> {
        let empty_storage = RefsStorage {
            refs: HashMap::new(),
        };
        self.save_refs_storage(&empty_storage)?;
        
        info!("Cleared all refs from storage");
        Ok(())
    }

}