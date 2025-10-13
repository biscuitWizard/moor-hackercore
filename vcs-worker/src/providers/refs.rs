use fjall::Partition;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, warn};

use super::{ProviderError, ProviderResult};
use crate::types::{ObjectInfo, VcsObjectType};

/// Represents the refs storage as a HashMap where key is ObjectInfo and value is sha256
/// Custom serialization converts HashMap to Vec for JSON compatibility
#[derive(Debug, Clone)]
pub struct RefsStorage {
    pub refs: HashMap<ObjectInfo, String>, // ObjectInfo -> sha256
}

impl Serialize for RefsStorage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Convert HashMap to Vec of tuples for JSON serialization
        let refs_vec: Vec<(&ObjectInfo, &String)> = self.refs.iter().collect();
        refs_vec.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RefsStorage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize as Vec of tuples and convert back to HashMap
        let refs_vec: Vec<(ObjectInfo, String)> = Vec::deserialize(deserializer)?;
        let refs: HashMap<ObjectInfo, String> = refs_vec.into_iter().collect();
        Ok(RefsStorage { refs })
    }
}

/// Provider trait for reference management
pub trait RefsProvider: Send + Sync {
    /// Get SHA256 for an object by name and optional version (defaults to latest)
    fn get_ref(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
        version: Option<u64>,
    ) -> ProviderResult<Option<String>>;

    /// Update or create a reference
    fn update_ref(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
        version: u64,
        sha256: &str,
    ) -> ProviderResult<()>;

    /// Get the next version number for an object
    fn get_next_version(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
    ) -> ProviderResult<u64>;

    /// Get the current version number for an object (returns None if object doesn't exist)
    fn get_current_version(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
    ) -> ProviderResult<Option<u64>>;

    /// Check if a SHA256 is referenced by any ref excluding a specific object:version
    fn is_sha256_referenced_excluding(
        &self,
        sha256: &str,
        exclude_object_type: VcsObjectType,
        exclude_object: &str,
        exclude_version: u64,
    ) -> ProviderResult<bool>;

    /// Get all refs as a HashMap (for export/cloning)
    fn get_all_refs(&self) -> ProviderResult<HashMap<ObjectInfo, String>>;

    /// Clear all refs from storage
    fn clear(&self) -> ProviderResult<()>;

    /// Delete a specific ref by object name and version
    fn delete_ref(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
        version: u64,
    ) -> ProviderResult<()>;

    /// Get the total data size (sum of all keys and values in bytes)
    fn get_data_size(&self) -> u64;
}

/// Implementation of RefsProvider using Fjall
pub struct RefsProviderImpl {
    refs_tree: Partition,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl RefsProviderImpl {
    /// Create a new refs provider
    pub fn new(refs_tree: Partition, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self {
            refs_tree,
            flush_sender,
        }
    }

    /// Load the refs storage from the database
    fn load_refs_storage(&self) -> ProviderResult<RefsStorage> {
        match self.refs_tree.get(b"refs_storage")? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let storage: RefsStorage = serde_json::from_str(&json).map_err(|e| {
                    ProviderError::SerializationError(format!("JSON parse error: {e}"))
                })?;
                Ok(storage)
            }
            None => Ok(RefsStorage {
                refs: HashMap::new(),
            }),
        }
    }

    /// Save the refs storage to the database
    fn save_refs_storage(&self, storage: &RefsStorage) -> ProviderResult<()> {
        let json = serde_json::to_string(storage).map_err(|e| {
            ProviderError::SerializationError(format!("JSON serialization error: {e}"))
        })?;
        self.refs_tree.insert(b"refs_storage", json.as_bytes())?;
        Ok(())
    }
}

impl RefsProvider for RefsProviderImpl {
    fn get_ref(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
        version: Option<u64>,
    ) -> ProviderResult<Option<String>> {
        let storage = self.load_refs_storage()?;

        if let Some(target_version) = version {
            // Specific version requested
            let key = ObjectInfo {
                object_type,
                name: object_name.to_string(),
                version: target_version,
            };
            Ok(storage.refs.get(&key).cloned())
        } else {
            // Latest version requested
            let latest_version = storage
                .refs
                .keys()
                .filter(|info| info.object_type == object_type && info.name == object_name)
                .map(|info| info.version)
                .max();

            if let Some(ver) = latest_version {
                let key = ObjectInfo {
                    object_type,
                    name: object_name.to_string(),
                    version: ver,
                };
                Ok(storage.refs.get(&key).cloned())
            } else {
                Ok(None)
            }
        }
    }

    fn update_ref(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
        version: u64,
        sha256: &str,
    ) -> ProviderResult<()> {
        let mut storage = self.load_refs_storage()?;

        // Add the new reference to the HashMap
        let key = ObjectInfo {
            object_type,
            name: object_name.to_string(),
            version,
        };
        storage.refs.insert(key, sha256.to_string());

        // Save the updated storage
        self.save_refs_storage(&storage)?;

        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }

        info!(
            "Updated ref for {:?} object '{}' version {}",
            object_type, object_name, version
        );
        Ok(())
    }

    fn get_next_version(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
    ) -> ProviderResult<u64> {
        let storage = self.load_refs_storage()?;

        let latest_version = storage
            .refs
            .keys()
            .filter(|info| info.object_type == object_type && info.name == object_name)
            .map(|info| info.version)
            .max();

        match latest_version {
            Some(version) => Ok(version + 1),
            None => Ok(1), // First version
        }
    }

    fn get_current_version(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
    ) -> ProviderResult<Option<u64>> {
        let storage = self.load_refs_storage()?;

        let latest_version = storage
            .refs
            .keys()
            .filter(|info| info.object_type == object_type && info.name == object_name)
            .map(|info| info.version)
            .max();

        Ok(latest_version)
    }

    fn is_sha256_referenced_excluding(
        &self,
        sha256: &str,
        exclude_object_type: VcsObjectType,
        exclude_object: &str,
        exclude_version: u64,
    ) -> ProviderResult<bool> {
        let storage = self.load_refs_storage()?;

        let exclude_key = ObjectInfo {
            object_type: exclude_object_type,
            name: exclude_object.to_string(),
            version: exclude_version,
        };

        // Check if any ref (except the excluded one) points to this SHA256
        for (key, ref_sha256) in &storage.refs {
            if key != &exclude_key && ref_sha256 == sha256 {
                return Ok(true);
            }
        }

        Ok(false)
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

    fn delete_ref(
        &self,
        object_type: VcsObjectType,
        object_name: &str,
        version: u64,
    ) -> ProviderResult<()> {
        let mut storage = self.load_refs_storage()?;

        // Remove the reference
        let key = ObjectInfo {
            object_type,
            name: object_name.to_string(),
            version,
        };
        storage.refs.remove(&key);

        // Save the updated storage
        self.save_refs_storage(&storage)?;

        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }

        info!(
            "Deleted ref for {:?} object '{}' version {}",
            object_type, object_name, version
        );
        Ok(())
    }

    fn get_data_size(&self) -> u64 {
        let mut total_size = 0u64;
        for (key, value) in self.refs_tree.iter().flatten() {
            total_size += key.len() as u64;
            total_size += value.len() as u64;
        }
        total_size
    }
}
