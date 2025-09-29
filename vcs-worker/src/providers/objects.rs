use sled::Tree;
use moor_compiler::{compile_object_definitions, ObjFileContext, CompileOptions, ObjectDefinition};
use tracing::{info, warn};
use tokio::sync::mpsc;
use sha2::{Sha256, Digest};

use super::{ProviderError, ProviderResult};

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
}

/// Implementation of ObjectsProvider using Sled
pub struct ObjectsProviderImpl {
    objects_tree: Tree,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl ObjectsProviderImpl {
    /// Create a new objects provider
    pub fn new(objects_tree: Tree, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { objects_tree, flush_sender }
    }
}

impl ObjectsProvider for ObjectsProviderImpl {
    fn parse_object_dump(&self, dump: &str) -> ProviderResult<ObjectDefinition> {
        let mut context = ObjFileContext::new();
        let compiled_defs = compile_object_definitions(
            dump,
            &CompileOptions::default(),
            &mut context,
        )?;
        
        if compiled_defs.len() != 1 {
            return Err(ProviderError::SerializationError(
                format!("Expected exactly 1 object definition, got {}", compiled_defs.len())
            ));
        }
        
        Ok(compiled_defs.into_iter().next().unwrap())
    }

    fn generate_sha256_hash(&self, dump: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(dump.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn store(&self, sha256_key: &str, dump: &str) -> ProviderResult<()> {
        self.objects_tree.insert(sha256_key.as_bytes(), dump.as_bytes())?;
        
        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Stored object with SHA256 key '{}' ({} bytes)", sha256_key, dump.len());
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
        let result = self.objects_tree.remove(sha256_key.as_bytes())?;
        Ok(result.is_some())
    }

    fn count(&self) -> usize {
        self.objects_tree.len()
    }
}
