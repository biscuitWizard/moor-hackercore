use fjall::Partition;
use tracing::{info, warn};
use tokio::sync::mpsc;

use super::{ProviderError, ProviderResult};

    /// Combined Index and Changes provider - manages both change storage and ordering
pub trait IndexProvider: Send + Sync {
    // ===== CHANGE ORDERING METHODS =====
    /// Push a change ID to the top of the stack (end of the ordered list)
    /// The list is ordered oldest first, newest last (like a stack)
    /// This also sets the change as the top_change (active local change)
    fn push_change(&self, change_id: &str) -> ProviderResult<()>;
    
    /// Add a change ID to the end of the change_order list (as merged history)
    /// WITHOUT setting it as top_change. Use this for approving changes from workspace.
    fn append_change_to_order(&self, change_id: &str) -> ProviderResult<()>;
    
    /// Get the ordered list of change IDs (oldest first, newest last)
    fn get_change_order(&self) -> ProviderResult<Vec<String>>;
    
    /// Set the complete change order (for bulk operations like cloning)
    fn set_change_order(&self, order: Vec<String>) -> ProviderResult<()>;
    
    /// Get the top (most recent/current) change ID (the last element in the list)
    fn get_top_change(&self) -> ProviderResult<Option<String>>;
    
    /// Remove a change ID from the working index (change_order list and TOP_KEY)
    /// Note: This only removes from the index metadata, not from history_storage
    /// The caller is responsible for deleting from history_storage if needed (e.g., for abandoned changes)
    /// Use this for abandoning or switching away from a change
    fn remove_from_index(&self, change_id: &str) -> ProviderResult<()>;
    
    /// Clear the top_change pointer if it points to the given change ID
    /// The change remains in change_order (as part of merged history)
    /// Use this when approving a change (converting Local -> Merged)
    fn clear_top_change_if(&self, change_id: &str) -> ProviderResult<()>;
    
    // ===== CHANGE STORAGE METHODS =====
    /// Store a change in the database
    fn store_change(&self, change: &crate::types::Change) -> ProviderResult<()>;
    
    /// Get a change by ID
    fn get_change(&self, change_id: &str) -> ProviderResult<Option<crate::types::Change>>;
    
    /// Update an existing change
    fn update_change(&self, change: &crate::types::Change) -> ProviderResult<()>;
    
    /// Delete a change from permanent storage (use with caution - typically only for abandoned local changes)
    fn delete_change(&self, change_id: &str) -> ProviderResult<()>;
    
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
    
    // ===== SOURCE METHODS =====
    /// Get the source URL if this is a clone
    fn get_source(&self) -> ProviderResult<Option<String>>;
    
    /// Set the source URL for this clone
    fn set_source(&self, url: &str) -> ProviderResult<()>;
    
    // ===== CLEAR METHODS =====
    /// Clear all changes and index data
    fn clear(&self) -> ProviderResult<()>;
}

/// Implementation of the IndexProvider trait using two separate Fjall partitions:
/// 
/// **Architecture:**
/// - `working_index`: Tracks the active working set (change_order list, top_change pointer, source_url)
/// - `history_storage`: Permanent storage for all Change objects (never deleted)
/// 
/// **Key Distinction:**
/// When a change is "removed from index", it's only removed from the working set metadata
/// (working_index), but the Change object remains permanently in history_storage.
pub struct IndexProviderImpl {
    /// **Working Index Metadata** - Stores the change index structure:
    /// - `change_order`: Chronological list of ALL changes (merged history + current local, if any)
    /// - `top_change`: Pointer to the ONE active Local change (if one exists)
    /// - `source_url`: Optional remote source for this repository
    /// 
    /// When a change is approved: stays in change_order (becomes part of history), top_change cleared
    /// When a change is abandoned: removed from change_order entirely, top_change cleared
    working_index: Partition,
    
    /// **Permanent History Storage** - Stores Change objects:
    /// - Stores all committed/merged changes permanently
    /// - Abandoned Local changes are deleted from here
    /// - Indexed by change ID
    /// - Contains changes in all states (Local, Merged, Review, Idle)
    /// 
    /// This is the authoritative source of truth for change data
    history_storage: Partition,
    
    /// Channel for requesting background database flushes
    flush_sender: mpsc::UnboundedSender<()>,
}

impl IndexProviderImpl {
    pub fn new(index_tree: Partition, changes_tree: Partition, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { 
            working_index: index_tree,
            history_storage: changes_tree,
            flush_sender,
        }
    }
    
    const ORDER_KEY: &'static str = "change_order";
    const TOP_KEY: &'static str = "top_change";
    const SOURCE_KEY: &'static str = "source_url";
    
    // ===== DRY HELPER METHODS =====
    
    /// Get the current change order with error handling
    fn get_change_order_internal(&self) -> ProviderResult<Vec<String>> {
        if let Some(data) = self.working_index.get(Self::ORDER_KEY)? {
            serde_json::from_slice::<Vec<String>>(&data)
                .map_err(|e| ProviderError::SerializationError(e.to_string()))
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Save the change order to storage
    fn save_change_order(&self, order: &Vec<String>) -> ProviderResult<()> {
        self.working_index.insert(Self::ORDER_KEY, serde_json::to_vec(order)
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
        // 1. Handle deletions first (remove from our tracking) - filter to only MooObject types
        for deleted_obj in change.deleted_objects.iter().filter(|o| o.object_type == crate::types::VcsObjectType::MooObject) {
            if self.objects.remove(&deleted_obj.name).is_some() {
                info!("  Deleted object: {} (version {})", deleted_obj.name, deleted_obj.version);
            }
        }
        
        // 2. Handle renames (rename in our tracking) - filter to only MooObject types
        for renamed_obj in change.renamed_objects.iter().filter(|r| r.from.object_type == crate::types::VcsObjectType::MooObject && r.to.object_type == crate::types::VcsObjectType::MooObject) {
            if let Some(version) = self.objects.remove(&renamed_obj.from.name) {
                self.objects.insert(renamed_obj.to.name.clone(), version);
                info!("  Renamed: {} -> {} (version {})", renamed_obj.from.name, renamed_obj.to.name, version);
            }
        }
        
        // 3. Handle additions (add to our tracking with version 1) - filter to only MooObject types
        for added_obj in change.added_objects.iter().filter(|o| o.object_type == crate::types::VcsObjectType::MooObject) {
            if !self.objects.contains_key(&added_obj.name) {
                self.objects.insert(added_obj.name.clone(), 1);
                info!("  Added object: {} (version {})", added_obj.name, added_obj.version);
            }
        }
        
        // 4. Handle modifications (update versions for existing objects) - filter to only MooObject types
        for modified_obj in change.modified_objects.iter().filter(|o| o.object_type == crate::types::VcsObjectType::MooObject) {
            if let Some(&current_version) = self.objects.get(&modified_obj.name) {
                let new_version = current_version + 1;
                self.objects.insert(modified_obj.name.clone(), new_version);
                info!("  Modified object: {} (version {} -> {})", modified_obj.name, current_version, new_version);
            } else {
                // Modified object that doesn't exist yet - treat as addition
                self.objects.insert(modified_obj.name.clone(), 1);
                warn!("  Modified object '{}' not found in tracking - treating as addition", modified_obj.name);
            }
        }
    }
    
    fn finalize(self) -> Vec<crate::types::ObjectInfo> {
        // Convert the HashMap to a sorted list for consistent output
        let mut object_list: Vec<crate::types::ObjectInfo> = self.objects.into_iter()
            .map(|(name, version)| crate::types::ObjectInfo { 
                object_type: crate::types::VcsObjectType::MooObject,  // Default to MooObject for now
                name, 
                version 
            })
            .collect();
        
        // Sort by name for consistent output
        object_list.sort_by(|a, b| a.name.cmp(&b.name));
        
        object_list
    }
}

impl IndexProvider for IndexProviderImpl {
    
    fn push_change(&self, change_id: &str) -> ProviderResult<()> {
        // Get current order using helper method
        let mut order = self.get_change_order_internal()?;
        
        // Remove if already exists and push to top of stack (end of list)
        order.retain(|id| id != change_id);
        order.push(change_id.to_string());
        
        self.save_change_order(&order)?;
        self.working_index.insert(Self::TOP_KEY, change_id.as_bytes())?;
        
        info!("Set change '{}' as top/local change", change_id);
        Ok(())
    }
    
    fn append_change_to_order(&self, change_id: &str) -> ProviderResult<()> {
        info!("append_change_to_order called for change '{}'", change_id);
        
        // Get current order
        let mut order = self.get_change_order_internal()?;
        info!("Current order before append: {:?}", order);
        
        // Remove if already exists (to avoid duplicates)
        order.retain(|id| id != change_id);
        
        // Add to end of list (newest position)
        order.push(change_id.to_string());
        info!("New order after append: {:?}", order);
        
        // Save the order without modifying top_change
        self.save_change_order(&order)?;
        
        info!("Appended change '{}' to change_order (without setting as top)", change_id);
        Ok(())
    }
    
    fn get_change_order(&self) -> ProviderResult<Vec<String>> {
        self.get_change_order_internal()
    }
    
    fn set_change_order(&self, order: Vec<String>) -> ProviderResult<()> {
        self.save_change_order(&order)?;
        
        // Update top change to the last in the order (newest)
        if let Some(top_change_id) = order.last() {
            self.working_index.insert(Self::TOP_KEY, top_change_id.as_bytes())?;
            info!("Set change order with {} changes, top change: {}", order.len(), top_change_id);
        } else {
            self.working_index.remove(Self::TOP_KEY)?;
            info!("Set empty change order");
        }
        
        Ok(())
    }
    
    fn get_top_change(&self) -> ProviderResult<Option<String>> {
        if let Some(data) = self.working_index.get(Self::TOP_KEY)? {
            Ok(Some(String::from_utf8(data.to_vec())
                .map_err(|e| ProviderError::SerializationError(e.to_string()))?))
        } else {
            Ok(None)
        }
    }
    
    fn remove_from_index(&self, change_id: &str) -> ProviderResult<()> {
        let mut order = self.get_change_order_internal()?;
        order.retain(|id| id != change_id);
        
        self.save_change_order(&order)?;
        
        // Clear top_change if we removed it (don't automatically set to last item)
        // top_change should only point to Local changes, not Merged ones
        if let Some(top_change) = self.working_index.get(Self::TOP_KEY)? {
            if &top_change.to_vec() == change_id.as_bytes() {
                self.working_index.remove(Self::TOP_KEY)?;
                info!("Cleared top_change pointer");
            }
        }
        
        info!("Removed change '{}' from change_order (may still be in history_storage)", change_id);
        Ok(())
    }
    
    fn clear_top_change_if(&self, change_id: &str) -> ProviderResult<()> {
        // Only clear top_change if it currently points to this change
        if let Some(top_change) = self.working_index.get(Self::TOP_KEY)? {
            if &top_change.to_vec() == change_id.as_bytes() {
                self.working_index.remove(Self::TOP_KEY)?;
                info!("Cleared top_change pointer (was '{}')", change_id);
            }
        }
        Ok(())
    }
    
    // ===== CHANGE STORAGE METHODS =====
    fn store_change(&self, change: &crate::types::Change) -> ProviderResult<()> {
        let json = serde_json::to_string(change)
            .map_err(|e| ProviderError::SerializationError(format!("JSON serialization error: {e}")))?;
        self.history_storage.insert(change.id.as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request flush for change '{}'", change.id);
        }
        
        Ok(())
    }
    
    fn get_change(&self, change_id: &str) -> ProviderResult<Option<crate::types::Change>> {
        match self.history_storage.get(change_id.as_bytes())? {
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
    
    fn delete_change(&self, change_id: &str) -> ProviderResult<()> {
        self.history_storage.remove(change_id.as_bytes())?;
        info!("Deleted change '{}' from history storage", change_id);
        Ok(())
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
        
        for result in self.history_storage.iter() {
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
        self.push_change(&new_change.id)?;
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
                    // Check if deleted (filter to MooObject types)
                    if top_change.deleted_objects.iter()
                        .filter(|obj| obj.object_type == crate::types::VcsObjectType::MooObject)
                        .any(|obj| obj.name == object_name) {
                        info!("Object '{}' has been deleted in top change", object_name);
                        return Ok(None);
                    }
                    
                    // Check if we're looking up by the NEW name (to.name) - need to map back to old name
                    if let Some(renamed) = top_change.renamed_objects.iter()
                        .filter(|r| r.from.object_type == crate::types::VcsObjectType::MooObject && r.to.object_type == crate::types::VcsObjectType::MooObject)
                        .find(|r| r.to.name == object_name) {
                        info!("Object '{}' is the new name of renamed object '{}', looking up ref by old name", object_name, renamed.from.name);
                        // Look up the ref using the old name (where the ref still is)
                        // Don't recursively call resolve because that would find the rename again and return None
                        return get_sha256(&renamed.from.name);
                    }
                    
                    // Check for renamed object (looking up by old name) (filter to MooObject types)
                    if top_change.renamed_objects.iter()
                        .filter(|r| r.from.object_type == crate::types::VcsObjectType::MooObject && r.to.object_type == crate::types::VcsObjectType::MooObject)
                        .any(|r| r.from.name == object_name) {
                        info!("Object '{}' has been renamed away in top change (old name lookup returns none)", object_name);
                        // If looking up by old name, return None (object no longer exists at this name)
                        return Ok(None);
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
        
        // Get all changes in chronological order (oldest first, newest last)
        let changes_order = self.get_change_order_internal()?;
        
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
    
    fn get_source(&self) -> ProviderResult<Option<String>> {
        if let Some(data) = self.working_index.get(Self::SOURCE_KEY)? {
            Ok(Some(String::from_utf8(data.to_vec())
                .map_err(|e| ProviderError::SerializationError(e.to_string()))?))
        } else {
            Ok(None)
        }
    }
    
    fn set_source(&self, url: &str) -> ProviderResult<()> {
        self.working_index.insert(Self::SOURCE_KEY, url.as_bytes())?;
        info!("Set source URL to: {}", url);
        Ok(())
    }
    
    fn clear(&self) -> ProviderResult<()> {
        // Clear the index tree (change order, top change, source)
        let index_keys: Vec<_> = self.working_index.iter()
            .filter_map(|result| result.ok())
            .map(|(key, _)| key.to_vec())
            .collect();
        
        for key in index_keys {
            self.working_index.remove(&key)?;
        }
        
        // Clear the changes tree
        let changes_keys: Vec<_> = self.history_storage.iter()
            .filter_map(|result| result.ok())
            .map(|(key, _)| key.to_vec())
            .collect();
        
        for key in changes_keys {
            self.history_storage.remove(&key)?;
        }
        
        info!("Cleared all index and changes data");
        Ok(())
    }
}
