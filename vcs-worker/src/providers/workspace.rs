use fjall::Partition;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use super::{ProviderError, ProviderResult};
use crate::types::{Change, ChangeStatus};

/// Workspace provider - manages changes that aren't yet on the index
///
/// Workspace is just a pool of changes that aren't on index. This will be used
/// eventually for changes awaiting approval and changes that are idle. We need
/// to record with the changes in workspace from where index change id they were
/// based off from. Also some way to track their approval/review status,
/// probably on change status.
pub trait WorkspaceProvider: Send + Sync {
    /// Store a workspace change
    fn store_workspace_change(&self, change: &Change) -> ProviderResult<()>;

    /// Get a workspace change by ID
    fn get_workspace_change(&self, change_id: &str) -> ProviderResult<Option<Change>>;

    /// Update a workspace change
    fn update_workspace_change(&self, change: &Change) -> ProviderResult<()>;

    /// Delete a workspace change
    fn delete_workspace_change(&self, change_id: &str) -> ProviderResult<bool>;

    /// List all changes in workspace by status
    fn list_workspace_changes_by_status(&self, status: ChangeStatus)
    -> ProviderResult<Vec<Change>>;

    /// List all workspace changes (regardless of status)
    fn list_all_workspace_changes(&self) -> ProviderResult<Vec<Change>>;

    /// Get workspace changes awaiting approval (status = Review)
    fn get_changes_waiting_approval(&self) -> ProviderResult<Vec<Change>>;

    /// Get idle workspace changes (status = Idle)
    fn get_idle_changes(&self) -> ProviderResult<Vec<Change>>;

    /// Move a change from workspace to index (promotes it)
    fn promote_change_to_index(&self, change_id: &str) -> ProviderResult<Change>;

    /// Create a new workspace change based on an existing indexed change
    fn create_workspace_branch(
        &self,
        index_change_id: &str,
        name: String,
        author: String,
    ) -> ProviderResult<Change>;

    /// Update change approval status
    fn update_approval_status(&self, change_id: &str, status: ChangeStatus) -> ProviderResult<()>;

    /// Find workspace changes based on a specific indexed change
    fn get_workspace_changes_for_index_change(
        &self,
        index_change_id: &str,
    ) -> ProviderResult<Vec<Change>>;
}

pub struct WorkspaceProviderImpl {
    workspace_tree: Partition,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl WorkspaceProviderImpl {
    pub fn new(workspace_tree: Partition, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self {
            workspace_tree,
            flush_sender,
        }
    }

    /// Helper function to convert change to workspace format
    fn change_to_workspace_key(change_id: &str) -> Vec<u8> {
        format!("change:{change_id}").as_bytes().to_vec()
    }

    /// Helper function to convert status to search key prefix
    fn status_to_prefix(status: ChangeStatus) -> String {
        format!("status:{status:?}:").to_lowercase()
    }
}

impl WorkspaceProvider for WorkspaceProviderImpl {
    fn store_workspace_change(&self, change: &Change) -> ProviderResult<()> {
        let json = serde_json::to_string(change).map_err(|e| {
            ProviderError::SerializationError(format!("JSON serialization error: {e}"))
        })?;

        let key = Self::change_to_workspace_key(&change.id);
        self.workspace_tree.insert(&key, json.as_bytes())?;

        // Also store under status-based index for efficient querying
        let status_key = format!(
            "{}{}",
            Self::status_to_prefix(change.status.clone()),
            change.id
        );
        self.workspace_tree
            .insert(status_key.as_bytes(), change.id.as_bytes())?;

        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!(
                "Failed to request flush for workspace change '{}'",
                change.id
            );
        }

        debug!(
            "Stored workspace change '{}' with status {:?}, index_change_id: {:?}",
            change.id, change.status, change.index_change_id
        );
        Ok(())
    }

    fn get_workspace_change(&self, change_id: &str) -> ProviderResult<Option<Change>> {
        let key = Self::change_to_workspace_key(change_id);
        match self.workspace_tree.get(&key)? {
            Some(value) => {
                let json = String::from_utf8(value.to_vec())
                    .map_err(|e| ProviderError::SerializationError(format!("UTF-8 error: {e}")))?;
                let change: Change = serde_json::from_str(&json).map_err(|e| {
                    ProviderError::SerializationError(format!("JSON deserialization error: {e}"))
                })?;
                Ok(Some(change))
            }
            None => Ok(None),
        }
    }

    fn update_workspace_change(&self, change: &Change) -> ProviderResult<()> {
        // Delete old status entry first
        if let Some(old_change) = self.get_workspace_change(&change.id)? {
            let old_status_key =
                format!("{}{}", Self::status_to_prefix(old_change.status), change.id);
            self.workspace_tree.remove(old_status_key.as_bytes())?;
        }

        // Store updated change
        self.store_workspace_change(change)?;

        debug!("Updated workspace change '{}'", change.id);
        Ok(())
    }

    fn delete_workspace_change(&self, change_id: &str) -> ProviderResult<bool> {
        let key = Self::change_to_workspace_key(change_id);

        // Remove main entry
        let exists = self.workspace_tree.get(&key)?.is_some();
        if exists {
            self.workspace_tree.remove(&key)?;
        }
        let removed = exists;

        // Remove from all possible status indexes
        for status in [ChangeStatus::Review, ChangeStatus::Idle] {
            let status_key = format!("{}{}", Self::status_to_prefix(status), change_id);
            self.workspace_tree.remove(status_key.as_bytes())?;
        }

        if removed {
            debug!("Deleted workspace change '{}'", change_id);
        }
        Ok(removed)
    }

    fn list_workspace_changes_by_status(
        &self,
        status: ChangeStatus,
    ) -> ProviderResult<Vec<Change>> {
        let mut changes = Vec::new();
        let prefix = Self::status_to_prefix(status.clone());

        // The status index stores: status:<status>:<id> -> <id>
        // We need to extract the change ID and fetch the actual change
        for result in self.workspace_tree.prefix(prefix.as_bytes()) {
            let (_key, value) = result?;

            // The value is the change ID
            if let Ok(change_id) = String::from_utf8(value.to_vec()) {
                // Fetch the actual change using the ID
                if let Some(change) = self.get_workspace_change(&change_id)? {
                    // Verify this change has the requested status (should always be true)
                    if change.status == status {
                        changes.push(change);
                    }
                }
            }
        }

        debug!(
            "Found {} workspace changes with status {:?}",
            changes.len(),
            status
        );
        Ok(changes)
    }

    fn list_all_workspace_changes(&self) -> ProviderResult<Vec<Change>> {
        let mut changes = Vec::new();

        for result in self.workspace_tree.prefix(b"change:") {
            let (_, value) = result?;
            if let Ok(json) = String::from_utf8(value.to_vec()) {
                if let Ok(change) = serde_json::from_str::<Change>(&json) {
                    changes.push(change);
                }
            }
        }

        debug!("Found {} total workspace changes", changes.len());
        Ok(changes)
    }

    fn get_changes_waiting_approval(&self) -> ProviderResult<Vec<Change>> {
        let changes = self.list_workspace_changes_by_status(ChangeStatus::Review)?;
        info!("Found {} changes waiting for approval", changes.len());
        Ok(changes)
    }

    fn get_idle_changes(&self) -> ProviderResult<Vec<Change>> {
        let changes = self.list_workspace_changes_by_status(ChangeStatus::Idle)?;
        info!("Found {} idle changes", changes.len());
        Ok(changes)
    }

    fn promote_change_to_index(&self, change_id: &str) -> ProviderResult<Change> {
        let change = self.get_workspace_change(change_id)?.ok_or_else(|| {
            ProviderError::InvalidOperation(format!("Workspace change '{change_id}' not found"))
        })?;

        info!("Promoting workspace change '{}' to index", change_id);

        // The caller will handle moving to index - we just prepare the change
        Ok(change)
    }

    fn create_workspace_branch(
        &self,
        index_change_id: &str,
        name: String,
        author: String,
    ) -> ProviderResult<Change> {
        let new_change = Change {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            author,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            status: ChangeStatus::Idle, // Start as idle until activated
            added_objects: Vec::new(),
            modified_objects: Vec::new(),
            deleted_objects: Vec::new(),
            renamed_objects: Vec::new(),
            index_change_id: Some(index_change_id.to_string()),
            verb_rename_hints: Vec::new(),
            property_rename_hints: Vec::new(),
        };

        self.store_workspace_change(&new_change)?;
        info!(
            "Created workspace branch '{}' based on index change '{}'",
            new_change.name, index_change_id
        );
        Ok(new_change)
    }

    fn update_approval_status(&self, change_id: &str, status: ChangeStatus) -> ProviderResult<()> {
        let mut change = self.get_workspace_change(change_id)?.ok_or_else(|| {
            ProviderError::InvalidOperation(format!("Workspace change '{change_id}' not found"))
        })?;

        let old_status = change.status.clone();
        change.status = status.clone();

        self.update_workspace_change(&change)?;

        info!(
            "Updated approval status for workspace change '{}' from {:?} to {:?}",
            change_id, old_status, status
        );
        Ok(())
    }

    fn get_workspace_changes_for_index_change(
        &self,
        index_change_id: &str,
    ) -> ProviderResult<Vec<Change>> {
        let all_changes = self.list_all_workspace_changes()?;
        let related_changes: Vec<Change> = all_changes
            .into_iter()
            .filter(|change| {
                change
                    .index_change_id
                    .as_ref()
                    .map(|id| id == index_change_id)
                    .unwrap_or(false)
            })
            .collect();

        debug!(
            "Found {} workspace changes based on index change '{}'",
            related_changes.len(),
            index_change_id
        );
        Ok(related_changes)
    }
}

// Helper trait extension
impl<T: WorkspaceProvider> WorkspaceProvider for Arc<T> {
    fn store_workspace_change(&self, change: &Change) -> ProviderResult<()> {
        (**self).store_workspace_change(change)
    }

    fn get_workspace_change(&self, change_id: &str) -> ProviderResult<Option<Change>> {
        (**self).get_workspace_change(change_id)
    }

    fn update_workspace_change(&self, change: &Change) -> ProviderResult<()> {
        (**self).update_workspace_change(change)
    }

    fn delete_workspace_change(&self, change_id: &str) -> ProviderResult<bool> {
        (**self).delete_workspace_change(change_id)
    }

    fn list_workspace_changes_by_status(
        &self,
        status: ChangeStatus,
    ) -> ProviderResult<Vec<Change>> {
        (**self).list_workspace_changes_by_status(status)
    }

    fn list_all_workspace_changes(&self) -> ProviderResult<Vec<Change>> {
        (**self).list_all_workspace_changes()
    }

    fn get_changes_waiting_approval(&self) -> ProviderResult<Vec<Change>> {
        (**self).get_changes_waiting_approval()
    }

    fn get_idle_changes(&self) -> ProviderResult<Vec<Change>> {
        (**self).get_idle_changes()
    }

    fn promote_change_to_index(&self, change_id: &str) -> ProviderResult<Change> {
        (**self).promote_change_to_index(change_id)
    }

    fn create_workspace_branch(
        &self,
        index_change_id: &str,
        name: String,
        author: String,
    ) -> ProviderResult<Change> {
        (**self).create_workspace_branch(index_change_id, name, author)
    }

    fn update_approval_status(&self, change_id: &str, status: ChangeStatus) -> ProviderResult<()> {
        (**self).update_approval_status(change_id, status)
    }

    fn get_workspace_changes_for_index_change(
        &self,
        index_change_id: &str,
    ) -> ProviderResult<Vec<Change>> {
        (**self).get_workspace_changes_for_index_change(index_change_id)
    }
}
