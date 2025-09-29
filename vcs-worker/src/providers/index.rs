use fjall::Partition;
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
    fn resolve_object_current_state<F>(&self, object_name: &str, get_sha256: F) -> ProviderResult<Option<String>>
    where
        F: Fn(&str) -> ProviderResult<Option<String>>;
    
    /// Compute complete object list by walking through all changes chronologically
    fn compute_complete_object_list(&self) -> ProviderResult<Vec<crate::types::ObjectInfo>>;
}

pub struct IndexProviderImpl {
    tree: Partition,
    changes_tree: Partition,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl IndexProviderImpl {
    pub fn new(index_tree: Partition, changes_tree: Partition, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { 
            tree: index_tree,
            changes_tree,
            flush_sender,
        }
    }
    
    const ORDER_KEY: &'static str = "change_order";
    const TOP_KEY: &'static str = "top_change";
    
    // ===== DRY HELPER METHODS =====
    
    /// Get the current change order with error handling
    fn get_change_order_internal(&self) -> ProviderResult<Vec<String>> {
        if let Some(data) = self.tree.get(Self::ORDER_KEY)? {
            serde_json::from_slice::<Vec<String>>(&data)
                .map_err(|e| ProviderError::SerializationError(e.to_string()))
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Save the change order to storage
    fn save_change_order(&self, order: &Vec<String>) -> ProviderResult<()> {
        self.tree.insert(Self::ORDER_KEY, serde_json::to_vec(order)
            .map_err(|e| ProviderError::SerializationError(e.to_string()))?)?;
        Ok(())
    }
    
    /// Format change status for logging/output
    fn format_change_status(status: &crate::types::ChangeStatus) -> &'static str {
        match status {
            crate::types::ChangeStatus::Merged => "MERGED",
            crate::types::ChangeStatus::Local => "LOCAL",
            crate::types::ChangeStatus::Review => "REVIEW",
            crate::types::ChangeStatus::Idle => "IDLE",
        }
    }
    
    /// Create a change change operation processor for the object list computation
    fn create_change_processor() -> ChangeOperationProcessor {
        ChangeOperationProcessor::new()
    }
}

/// Extracted change operation processor for better separation of concerns
struct ChangeOperationProcessor {
    objects: std::collections::HashMap<String, u64>,
}

impl ChangeOperationProcessor {
    fn new() -> Self {
        Self {
            objects: std::collections::HashMap::new(),
        }
    }
    
    fn process_change(&mut self, change: &crate::types::Change) {
        // 1. Handle deletions first (remove from our tracking)
        for deleted_name in &change.deleted_objects {
            if self.objects.remove(deleted_name).is_some() {
                info!("  Deleted object: {}", deleted_name);
            }
        }
        
        // 2. Handle renames (rename in our tracking)
        for renamed_obj in &change.renamed_objects {
            if let Some(version) = self.objects.remove(&renamed_obj.from) {
                self.objects.insert(renamed_obj.to.clone(), version);
                info!("  Renamed: {} -> {} (version {})", renamed_obj.from, renamed_obj.to, version);
            }
        }
        
        // 3. Handle additions (add to our tracking with version 1)
        for added_name in &change.added_objects {
            if !self.objects.contains_key(added_name as &str) {
                self.objects.insert(added_name.clone(), 1);
                info!("  Added object: {}", added_name);
            }
        }
        
        // 4. Handle modifications (update versions for existing objects)
        for modified_name in &change.modified_objects {
            if let Some(&current_version) = self.objects.get(modified_name as &str) {
                let new_version = current_version + 1;
                self.objects.insert(modified_name.clone(), new_version);
                info!("  Modified object: {} (version {} -> {})", modified_name, current_version, new_version);
            } else {
                // Modified object that doesn't exist yet - treat as addition
                self.objects.insert(modified_name.clone(), 1);
                warn!("  Modified object '{}' not found in tracking - treating as addition", modified_name);
            }
        }
    }
    
    fn finalize(mut self) -> Vec<crate::types::ObjectInfo> {
        // Convert the HashMap to a sorted list for consistent output
        let mut object_list: Vec<crate::types::ObjectInfo> = self.objects.into_iter()
            .map(|(name, version)| crate::types::ObjectInfo { name, version })
            .collect();
        
        // Sort by name for consistent output
        object_list.sort_by(|a, b| a.name.cmp(&b.name));
        
        object_list
    }
}

impl IndexProvider for IndexProviderImpl {
    fn append_change(&self, change_id: &str) -> ProviderResult<()> {
        // Get current order using helper method
        let mut order = self.get_change_order_internal()?;
        
        // Add to end (oldest)
        if !order.contains(&change_id.to_string()) {
            order.push(change_id.to_string());
            self.save_change_order(&order)?;
            info!("Added change '{}' to index order", change_id);
        }
        
        Ok(())
    }
    
    fn prepend_change(&self, change_id: &str) -> ProviderResult<()> {
        // Get current order using helper method
        let mut order = self.get_change_order_internal()?;
        
        // Remove if already exists and add to front (newest)
        order.retain(|id| id != change_id);
        order.insert(0, change_id.to_string());
        
        self.save_change_order(&order)?;
        self.tree.insert(Self::TOP_KEY, change_id.as_bytes())?;
        
        info!("Set change '{}' as top/local change", change_id);
        Ok(())
    }
    
    fn get_change_order(&self) -> ProviderResult<Vec<String>> {
        self.get_change_order_internal()
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
        let mut order = self.get_change_order_internal()?;
        order.retain(|id| id != change_id);
        
        self.save_change_order(&order)?;
        
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
    
    fn resolve_object_current_state<F>(&self, object_name: &str, get_sha256: F) -> ProviderResult<Option<String>>
    where
        F: Fn(&str) -> ProviderResult<Option<String>>
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
                        return self.resolve_object_current_state(&renamed.to, get_sha256);
                    }
                }
            }
        }
        
        // No top change or object not modified in top change - resolve through refs
        match get_sha256(object_name)? {
            Some(sha256) => {
                info!("Found object '{}' in refs with SHA256 {}", object_name, sha256);
                Ok(Some(sha256))
            }
            None => {
                info!("Object '{}' not found in refs", object_name);
                Ok(None)
            }
        }
    }
    
    fn compute_complete_object_list(&self) -> ProviderResult<Vec<crate::types::ObjectInfo>> {
        info!("Computing complete object list by walking change history");
        
        // Get all changes in chronological order (oldest first)
        let mut changes_order = self.get_change_order_internal()?;
        changes_order.reverse(); // Reverse to get oldest first
        
        info!("Walking through {} changes chronologically", changes_order.len());
        
        // Use the change operation processor for cleaner separation of concerns
        let mut processor = Self::create_change_processor();
        
        // Walk through each change chronologically
        for change_id in changes_order {
            match self.get_change(&change_id)? {
                Some(change) => {
                    info!("Processing change '{}' ({}): {} added, {} modified, {} deleted, {} renamed", 
                        change.id, 
                        Self::format_change_status(&change.status),
                        change.added_objects.len(),
                        change.modified_objects.len(), 
                        change.deleted_objects.len(),
                        change.renamed_objects.len());
                    
                    // Delegate to the change processor
                    processor.process_change(&change);
                }
                None => {
                    warn!("Change '{}' not found in database", change_id);
                }
            }
        }
        
        // Finalize and get the result
        let object_list = processor.finalize();
        
        info!("Final object list contains {} objects", object_list.len());
        
        Ok(object_list)
    }
}
