use sled::Tree;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};

use super::{ProviderError, ProviderResult};

/// Index provider manages ordered collections of change IDs
pub trait IndexProvider: Send + Sync {
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
    
    /// Resolve the current state of an object considering the top change in the index.
    /// Takes closure functions to get change and ref data to avoid trait bound issues.
    fn resolve_object_current_state<F1, F2>(&self, object_name: &str, get_change: F1, get_ref: F2) -> ProviderResult<Option<String>>
    where
        F1: Fn(&str) -> ProviderResult<Option<crate::types::Change>>,
        F2: Fn(&str) -> ProviderResult<Option<crate::providers::refs::ObjectRef>>;
}

pub struct IndexProviderImpl {
    tree: Tree,
}

impl IndexProviderImpl {
    pub fn new(tree: Tree) -> Self {
        Self { tree }
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
    
    fn resolve_object_current_state<F1, F2>(&self, object_name: &str, get_change: F1, get_ref: F2) -> ProviderResult<Option<String>>
    where
        F1: Fn(&str) -> ProviderResult<Option<crate::types::Change>>,
        F2: Fn(&str) -> ProviderResult<Option<crate::providers::refs::ObjectRef>>,
    {
        // Get the top change from the index
        if let Some(top_change_id) = self.get_top_change()? {
            if let Some(top_change) = get_change(&top_change_id)? {
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
                        return self.resolve_object_current_state(&renamed.to, get_change, get_ref);
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
