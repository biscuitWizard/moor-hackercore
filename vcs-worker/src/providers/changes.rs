use sled::Tree;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use tokio::sync::mpsc;

use super::{ProviderError, ProviderResult};

/// Represents a file rename operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamedObject {
    pub from: String,
    pub to: String,
}

/// Represents an object version override in a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectVersionOverride {
    pub object_name: String,
    pub version: u64,
    pub sha256_key: String,
}

/// Represents a change in the version control system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: String,
    pub timestamp: u64, // Linux UTC epoch
    pub added_objects: Vec<String>,
    pub modified_objects: Vec<String>,
    pub deleted_objects: Vec<String>,
    pub renamed_objects: Vec<RenamedObject>,
    pub version_overrides: Vec<ObjectVersionOverride>, // Object name + version -> SHA256 mappings
}

/// Provider trait for change management
pub trait ChangesProvider: Send + Sync {
    /// Store a change in the database
    fn store_change(&self, change: &Change) -> ProviderResult<()>;
    
    /// Get a change by ID
    fn get_change(&self, change_id: &str) -> ProviderResult<Option<Change>>;
    
    /// Update an existing change
    fn update_change(&self, change: &Change) -> ProviderResult<()>;
    
    /// Get the current active change
    #[allow(dead_code)]
    fn get_current_change(&self) -> ProviderResult<Option<Change>>;
    
    /// Set the current change
    #[allow(dead_code)]
    fn set_current_change(&self, change_id: Option<String>) -> ProviderResult<()>;
    
    /// Create a blank change automatically
    fn create_blank_change(&self) -> ProviderResult<Change>;
    
    /// List all changes
    #[allow(dead_code)]
    fn list_changes(&self) -> ProviderResult<Vec<Change>>;
    
    /// Delete a change
    #[allow(dead_code)]
    fn delete_change(&self, change_id: &str) -> ProviderResult<bool>;
}

/// Implementation of ChangesProvider using Sled
pub struct ChangesProviderImpl {
    changes_tree: Tree,
    flush_sender: mpsc::UnboundedSender<()>,
    
    // Note: This provider needs access to RepositoryProvider for current change tracking
    // In a real implementation, you'd inject this dependency or use events
    #[allow(dead_code)]
    repository_provider: Option<Arc<dyn super::repository::RepositoryProvider>>,
}

impl ChangesProviderImpl {
    /// Create a new changes provider
    pub fn new(changes_tree: Tree, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { 
            changes_tree, 
            flush_sender,
            repository_provider: None,
        }
    }
    
    /// Set the repository provider for current change tracking
    #[allow(dead_code)]
    pub fn set_repository_provider(&mut self, repo_provider: Arc<dyn super::repository::RepositoryProvider>) {
        self.repository_provider = Some(repo_provider);
    }
}

impl ChangesProvider for ChangesProviderImpl {
    fn store_change(&self, change: &Change) -> ProviderResult<()> {
        let json = serde_json::to_string(change)
            .map_err(|e| ProviderError::SerializationError(format!("JSON serialization error: {e}")))?;
        self.changes_tree.insert(change.id.as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Stored change '{}' ({}) in database", change.name, change.id);
        Ok(())
    }

    fn get_change(&self, change_id: &str) -> ProviderResult<Option<Change>> {
        match self.changes_tree.get(change_id.as_bytes())? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let change: Change = serde_json::from_str(&json)
                    .map_err(|e| ProviderError::SerializationError(format!("JSON parse error: {e}")))?;
                Ok(Some(change))
            }
            None => Ok(None),
        }
    }

    fn update_change(&self, change: &Change) -> ProviderResult<()> {
        self.store_change(change)
    }

    fn get_current_change(&self) -> ProviderResult<Option<Change>> {
        // This requires access to repository provider - simplified for now
        // In real implementation, you'd use dependency injection
        Ok(None)
    }

    fn set_current_change(&self, _change_id: Option<String>) -> ProviderResult<()> {
        // This requires access to repository provider - simplified for now
        Ok(())
    }

    fn create_blank_change(&self) -> ProviderResult<Change> {
        let change = Change {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(), // Blank name
            description: None,
            author: "system".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            added_objects: Vec::new(),
            modified_objects: Vec::new(),
            deleted_objects: Vec::new(),
            renamed_objects: Vec::new(),
            version_overrides: Vec::new(),
        };
        
        // Store the change
        self.store_change(&change)?;
        
        info!("Created blank change '{}'", change.id);
        Ok(change)
    }

    fn list_changes(&self) -> ProviderResult<Vec<Change>> {
        let mut changes = Vec::new();
        
        for result in self.changes_tree.iter() {
            let (_, value) = result?;
            if let Ok(json) = String::from_utf8(value.to_vec()) {
                if let Ok(change) = serde_json::from_str::<Change>(&json) {
                    changes.push(change);
                }
            }
        }
        
        Ok(changes)
    }

    fn delete_change(&self, change_id: &str) -> ProviderResult<bool> {
        let result = self.changes_tree.remove(change_id.as_bytes())?;
        Ok(result.is_some())
    }
}
