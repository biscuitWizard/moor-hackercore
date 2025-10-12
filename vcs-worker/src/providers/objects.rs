use fjall::Partition;
use moor_compiler::{CompileOptions, ObjFileContext, ObjectDefinition, compile_object_definitions};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{info, warn};

use super::{ProviderError, ProviderResult};
use crate::types::MooMetaObject;

/// Provider trait for object CRUD operations
pub trait ObjectsProvider: Send + Sync {
    /// Parse a MOO object dump string into an ObjectDefinition
    fn parse_object_dump(&self, dump: &str) -> ProviderResult<ObjectDefinition>;

    /// Generate SHA256 hash from object dump
    fn generate_sha256_hash(&self, dump: &str) -> String;

    /// Store object by SHA256 key
    fn store(&self, sha256_key: &str, dump: &str) -> ProviderResult<()>;

    /// Retrieve object by SHA256 key
    fn get(&self, sha256_key: &str) -> ProviderResult<Option<String>>;

    /// Delete object by SHA256 key
    #[allow(dead_code)]
    fn delete(&self, sha256_key: &str) -> ProviderResult<bool>;

    /// Get count of stored objects
    fn count(&self) -> usize;

    /// Get all objects as a HashMap (for export/cloning)
    fn get_all_objects(&self) -> ProviderResult<HashMap<String, String>>;

    /// Clear all objects from storage
    fn clear(&self) -> ProviderResult<()>;

    /// Parse a YAML meta dump string into a MooMetaObject
    fn parse_meta_dump(&self, dump: &str) -> ProviderResult<MooMetaObject>;

    /// Generate YAML dump from a MooMetaObject
    fn generate_meta_dump(&self, meta: &MooMetaObject) -> ProviderResult<String>;

    /// Get the total data size (sum of all keys and values in bytes)
    fn get_data_size(&self) -> u64;
}

/// Implementation of ObjectsProvider using Fjall
pub struct ObjectsProviderImpl {
    objects_tree: Partition,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl ObjectsProviderImpl {
    /// Create a new objects provider
    pub fn new(objects_tree: Partition, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self {
            objects_tree,
            flush_sender,
        }
    }
}

impl ObjectsProvider for ObjectsProviderImpl {
    fn parse_object_dump(&self, dump: &str) -> ProviderResult<ObjectDefinition> {
        let mut context = ObjFileContext::new();
        let compiled_defs =
            compile_object_definitions(dump, &CompileOptions::default(), &mut context)?;

        if compiled_defs.len() != 1 {
            return Err(ProviderError::SerializationError(format!(
                "Expected exactly 1 object definition, got {}",
                compiled_defs.len()
            )));
        }

        Ok(compiled_defs.into_iter().next().unwrap())
    }

    fn generate_sha256_hash(&self, dump: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(dump.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn store(&self, sha256_key: &str, dump: &str) -> ProviderResult<()> {
        self.objects_tree
            .insert(sha256_key.as_bytes(), dump.as_bytes())?;

        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }

        info!(
            "Stored object with SHA256 key '{}' ({} bytes)",
            sha256_key,
            dump.len()
        );
        Ok(())
    }

    fn get(&self, sha256_key: &str) -> ProviderResult<Option<String>> {
        match self.objects_tree.get(sha256_key.as_bytes())? {
            Some(data) => {
                let stored_data = String::from_utf8(data.to_vec())?;
                Ok(Some(stored_data))
            }
            None => Ok(None),
        }
    }

    fn delete(&self, sha256_key: &str) -> ProviderResult<bool> {
        // Check if the key exists first
        let exists = self.objects_tree.get(sha256_key.as_bytes())?.is_some();
        if exists {
            self.objects_tree.remove(sha256_key.as_bytes())?;
        }
        Ok(exists)
    }

    fn count(&self) -> usize {
        self.objects_tree.len().unwrap_or(0)
    }

    fn get_all_objects(&self) -> ProviderResult<HashMap<String, String>> {
        let mut objects = std::collections::HashMap::new();

        for result in self.objects_tree.iter() {
            let (key, value) = result?;
            let sha256 = String::from_utf8(key.to_vec())?;
            let data = String::from_utf8(value.to_vec())?;
            objects.insert(sha256, data);
        }

        Ok(objects)
    }

    fn clear(&self) -> ProviderResult<()> {
        let keys: Vec<_> = self
            .objects_tree
            .iter()
            .filter_map(|result| result.ok())
            .map(|(key, _)| key.to_vec())
            .collect();

        for key in keys {
            self.objects_tree.remove(&key)?;
        }

        info!("Cleared all objects from storage");
        Ok(())
    }

    fn parse_meta_dump(&self, dump: &str) -> ProviderResult<MooMetaObject> {
        let meta: MooMetaObject = serde_yaml::from_str(dump)
            .map_err(|e| ProviderError::SerializationError(format!("YAML parse error: {e}")))?;
        Ok(meta)
    }

    fn generate_meta_dump(&self, meta: &MooMetaObject) -> ProviderResult<String> {
        let yaml = serde_yaml::to_string(meta).map_err(|e| {
            ProviderError::SerializationError(format!("YAML serialization error: {e}"))
        })?;
        Ok(yaml)
    }

    fn get_data_size(&self) -> u64 {
        let mut total_size = 0u64;
        for (key, value) in self.objects_tree.iter().flatten() {
            total_size += key.len() as u64;
            total_size += value.len() as u64;
        }
        total_size
    }
}
