use sled::Tree;
use tracing::{info, warn};
use tokio::sync::mpsc;

use super::{ProviderError, ProviderResult};

    /// Combined Index and Changes provider - manages both change storage and ordering
pub trait IndexProvider: Send + Sync {
    // ===== CHANGE ORDERING METHODS =====
    /// Add a change ID to the end of the ordered list
    fn append_change(&self, change_id: &str) -> ProviderResult<()>;
    
    /// Insert a change ID at the front of the ordered list (for current working change)
    fn prepend_change(&self, change_id: &str) -> ProviderResult<()>;
    
    /// Get the ordered list of change IDs (most recent first)
    fn get_change_order(&self) -> ProviderResult<Vec<String>>;
    
    /// Get the top (most recent/current) change ID
    fn get_top_change(&self) -> ProviderResult<Option<String>>;
    
    /// Remove a change ID from the order list
    fn remove_change(&self, change_id: &str) -> ProviderResult<()>;
    
    /// Reorder the list when a change status changes
    fn reorder_on_status_change(&self, change_id: &str, was_local: bool, is_now_local: bool) -> ProviderResult<()>;
    
    // ===== CHANGE STORAGE METHODS =====
    /// Store a change in the database
    fn store_change(&self, change: &crate::types::Change) -> ProviderResult<()>;
    
    /// Get a change by ID
    fn get_change(&self, change_id: &str) -> ProviderResult<Option<crate::types::Change>>;
    
    /// Update an existing change
    fn update_change(&self, change: &crate::types::Change) -> ProviderResult<()>;
    
    /// Create a blank change automatically
    fn create_blank_change(&self) -> ProviderResult<crate::types::Change>;
    
    /// List all changes (optional/for debugging)
    #[allow(dead_code)]
    fn list_changes(&self) -> ProviderResult<Vec<crate::types::Change>>;
    
    // ===== COMBINED METHODS =====
    /// Get or create a local change - uses internal change storage now
    fn get_or_create_local_change(&self) -> ProviderResult<crate::types::Change>;
    
    /// Resolve the current state of an object considering the top change in the index.
    fn resolve_object_current_state<F>(&self, object_name: &str, get_ref: F) -> ProviderResult<Option<String>>
    where
        F: Fn(&str) -> ProviderResult<Option<crate::providers::refs::ObjectRef>>;
}

pub struct IndexProviderImpl {
    tree: Tree,
    changes_tree: Tree,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl IndexProviderImpl {
    pub fn new(index_tree: Tree, changes_tree: Tree, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { 
            tree: index_tree,
            changes_tree,
            flush_sender,
        }
    }
    
    const ORDER_KEY: &'static str = "change_order";
    const TOP_KEY: &'static str = "top_change";
}

impl IndexProvider for IndexProviderImpl {
    fn append_change(&self, change_id: &str) -> ProviderResult<()> {
        // Get current order
        let mut order = if let Some(data) = self.tree.get(Self::ORDER_KEY)? {
            serde_json::from_slice::<Vec<String>>(&data)
                .map_err(|e| ProviderError::SerializationError(e.to_string()))?
        } else {
            Vec::new()
        };
        
        // Add to end (oldest)
        if !order.contains(&change_id.to_string()) {
            order.push(change_id.to_string());
            self.tree.insert(Self::ORDER_KEY, serde_json::to_vec(&order)
                .map_err(|e| ProviderError::SerializationError(e.to_string()))?)?;
            info!("Added change '{}' to index order", change_id);
        }
        
        Ok(())
    }
    
    fn prepend_change(&self, change_id: &str) -> ProviderResult<()> {
        // Get current order
        let mut order = if let Some(data) = self.tree.get(Self::ORDER_KEY)? {
            serde_json::from_slice::<Vec<String>>(&data)
                .map_err(|e| ProviderError::SerializationError(e.to_string()))?
        } else {
            Vec::new()
        };
        
        // Remove if already exists and add to front (newest)
        order.retain(|id| id != change_id);
        order.insert(0, change_id.to_string());
        
        self.tree.insert(Self::ORDER_KEY, serde_json::to_vec(&order)
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?)?;
        self.tree.insert(Self::TOP_KEY, change_id.as_bytes())?;
        
        info!("Set change '{}' as top/local change", change_id);
        Ok(())
    }
    
    fn get_change_order(&self) -> ProviderResult<Vec<String>> {
        if let Some(data) = self.tree.get(Self::ORDER_KEY)? {
            serde_json::from_slice(&data)
                .map_err(|e| ProviderError::SerializationError(e.to_string()))
        } else {
            Ok(Vec::new())
        }
    }
    
    fn get_top_change(&self) -> ProviderResult<Option<String>> {
        if let Some(data) = self.tree.get(Self::TOP_KEY)? {
            Ok(Some(String::from_utf8(data.to_vec())
                .map_err(|e| ProviderError::SerializationError(e.to_string()))?))
        } else {
            Ok(None)
        }
    }
    
    fn remove_change(&self, change_id: &str) -> ProviderResult<()> {
        let mut order = self.get_change_order()?;
        order.retain(|id| id != change_id);
        
        self.tree.insert(Self::ORDER_KEY, serde_json::to_vec(&order)
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?)?;
        
        // Update top change if we removed it
        if let Some(top_change) = self.tree.get(Self::TOP_KEY)? {
            if &top_change.to_vec() == change_id.as_bytes() {
                let new_top = order.first().cloned();
                if let Some(new_top_id) = new_top {
                    self.tree.insert(Self::TOP_KEY, new_top_id.as_bytes())?;
                } else {
                    self.tree.remove(Self::TOP_KEY)?;
                }
            }
        }
        
        info!("Removed change '{}' from index order", change_id);
        Ok(())
    }
    
    fn reorder_on_status_change(&self, change_id: &str, was_local: bool, is_now_local: bool) -> ProviderResult<()> {
        if was_local == is_now_local {
            return Ok(()); // No change needed
        }
        
        if is_now_local {
            // Becoming local/current - move to top
            self.prepend_change(change_id)?;
        } else {
            // Becoming merged - move to bottom
            self.append_change(change_id)?;
            // Update top change if this was the top
            if let Some(top_change) = self.tree.get(Self::TOP_KEY)? {
                if &top_change.to_vec() == change_id.as_bytes() {
                    let order = self.get_change_order()?;
                    let new_top = order.iter().find(|id| **id != change_id).cloned();
                    if let Some(new_top_id) = new_top {
                        self.tree.insert(Self::TOP_KEY, new_top_id.as_bytes())?;
                    } else {
                        self.tree.remove(Self::TOP_KEY)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    
    // ===== CHANGE STORAGE METHODS =====
    fn store_change(&self, change: &crate::types::Change) -> ProviderResult<()> {
        let json = serde_json::to_string(change)
            .map_err(|e| ProviderError::SerializationError(format!("JSON serialization error: {e}")))?;
        self.changes_tree.insert(change.id.as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request flush for change '{}'", change.id);
        }
        
        Ok(())
    }
    
    fn get_change(&self, change_id: &str) -> ProviderResult<Option<crate::types::Change>> {
        match self.changes_tree.get(change_id.as_bytes())? {
            Some(value) => {
                let json = String::from_utf8(value.to_vec())
                    .map_err(|e| ProviderError::SerializationError(format!("UTF-8 error: {e}")))?;
                let change: crate::types::Change = serde_json::from_str(&json)
                    .map_err(|e| ProviderError::SerializationError(format!("JSON deserialization error: {e}")))?;
                Ok(Some(change))
            }
            None => Ok(None),
        }
    }
    
    fn update_change(&self, change: &crate::types::Change) -> ProviderResult<()> {
        self.store_change(change)
    }
    
    fn create_blank_change(&self) -> ProviderResult<crate::types::Change> {
        let change = crate::types::Change {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(), // Blank name
            description: None,
            author: "system".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: crate::types::ChangeStatus::Local,
            added_objects: Vec::new(),
            modified_objects: Vec::new(),
            deleted_objects: Vec::new(),
            renamed_objects: Vec::new(),
            index_change_id: None,
            version_overrides: Vec::new(),
        };
        
        self.store_change(&change)?;
        info!("Created blank change '{}'", change.id);
        Ok(change)
    }
    
    fn list_changes(&self) -> ProviderResult<Vec<crate::types::Change>> {
        let mut changes = Vec::new();
        
        for result in self.changes_tree.iter() {
            let (_, value) = result?;
            if let Ok(json) = String::from_utf8(value.to_vec()) {
                if let Ok(change) = serde_json::from_str::<crate::types::Change>(&json) {
                    changes.push(change);
                }
            }
        }
        
        Ok(changes)
    }
    
    // ===== UPDATED COMBINED METHODS =====
    fn get_or_create_local_change(&self) -> ProviderResult<crate::types::Change> {
        // Check if we have a top change and if it's local
        if let Some(top_change_id) = self.get_top_change()? {
            if let Some(change) = self.get_change(&top_change_id)? {
                if change.status == crate::types::ChangeStatus::Local {
                    info!("Using existing local change '{}' ({})", change.name, change.id);
                    return Ok(change);
                } else {
                    info!("Top change '{}' ({}) is not local, creating new local change", change.name, change.id);
                }
            } else {
                info!("Top change '{}' not found, creating new local change", top_change_id);
            }
        } else {
            info!("No top change found, creating new local change");
        }
        
        // Create new local change and set it as top
        let new_change = self.create_blank_change()?;
        self.prepend_change(&new_change.id)?;
        info!("Created and set new local change '{}' ({})", new_change.name, new_change.id);
        Ok(new_change)
    }
    
    fn resolve_object_current_state<F>(&self, object_name: &str, get_ref: F) -> ProviderResult<Option<String>>
    where
        F: Fn(&str) -> ProviderResult<Option<crate::providers::refs::ObjectRef>>
    {
        // Get the top change from the index
        if let Some(top_change_id) = self.get_top_change()? {
            if let Some(top_change) = self.get_change(&top_change_id)? {
                // Only consider changes that are local (working state)
                if top_change.status == crate::types::ChangeStatus::Local {
                    // Check if deleted
                    if top_change.deleted_objects.contains(&object_name.to_string()) {
                        info!("Object '{}' has been deleted in top change", object_name);
                        return Ok(None);
                    }
                    
                    // Check for renamed object
                    if let Some(renamed) = top_change.renamed_objects.iter()
                        .find(|r| r.from == object_name) {
                        info!("Object '{}' has been renamed to '{}' in top change", object_name, renamed.to);
                        // Recursively resolve the renamed object
                        return self.resolve_object_current_state(&renamed.to, get_ref);
                    }
                }
            }
        }
        
        // No top change or object not modified in top change - resolve through refs
        match get_ref(object_name)? {
            Some(object_ref) => {
                info!("Found object '{}' in refs at version {}", object_name, object_ref.version);
                Ok(Some(object_ref.sha256_key))
            }
            None => {
                info!("Object '{}' not found in refs", object_name);
                Ok(None)
            }
        }
    }
}
