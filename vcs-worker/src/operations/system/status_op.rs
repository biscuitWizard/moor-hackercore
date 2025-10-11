use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::{User, ChangeStatus};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;

/// System status operation that provides comprehensive repository status information
#[derive(Clone)]
pub struct StatusOperation {
    database: DatabaseRef,
}

impl StatusOperation {
    /// Create a new status operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Get the size of a directory in bytes
    fn get_directory_size(path: &std::path::Path) -> Result<u64, std::io::Error> {
        let mut total_size = 0u64;
        
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_file() {
                    total_size += entry.metadata()?.len();
                } else if path.is_dir() {
                    total_size += Self::get_directory_size(&path)?;
                }
            }
        }
        
        Ok(total_size)
    }
    
    /// Get the size of a partition directory in bytes
    fn get_partition_size(&self, partition_name: &str) -> u64 {
        // Partitions are stored in subdirectories of the database path
        // The database path is stored in the Database struct
        // We need to construct the full path to the partition
        let db_path = self.database.db_path();
        let partition_path = db_path.join(partition_name);
        
        match Self::get_directory_size(&partition_path) {
            Ok(size) => size,
            Err(e) => {
                error!("Failed to get size for partition '{}': {}", partition_name, e);
                0
            }
        }
    }

    /// Process the status request
    fn process_status(&self, user: &User) -> Result<moor_var::Var, ObjectsTreeError> {
        info!("Processing system status request for user: {}", user.id);
        
        // Get top change ID
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .unwrap_or_else(|| "none".to_string());
        
        // Get change order to count changes in working index
        let change_order = self.database.index().get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let changes_in_index = change_order.len() as i64;
        
        // Get idle changes count from workspace
        let idle_changes = self.database.workspace().get_idle_changes()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let idle_changes_count = idle_changes.len() as i64;
        
        // Get changes pending review count from workspace
        let pending_review = self.database.workspace().get_changes_waiting_approval()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let pending_review_count = pending_review.len() as i64;
        
        // Get current username
        let current_username = moor_var::v_str(&user.id);
        
        // Get latest non-local merged change
        let latest_merged_change = self.get_latest_merged_change()?;
        
        // Get remote repository URL if present
        let remote_url = self.database.index().get_source()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .unwrap_or_else(|| "none".to_string());
        
        // Get partition sizes
        let index_partition_size = self.get_partition_size("index") as i64;
        let refs_partition_size = self.get_partition_size("refs") as i64;
        let objects_partition_size = self.get_partition_size("objects") as i64;
        
        // Check if we're behind remote (only if we have a source)
        let pending_updates = if remote_url != "none" {
            // TODO: Implement checking remote for pending updates
            // For now, return 0 as we'd need to query the remote
            0i64
        } else {
            0i64
        };
        
        // Build the status map
        let status_map = moor_var::v_map(&[
            (moor_var::v_str("top_change_id"), moor_var::v_str(&top_change_id)),
            (moor_var::v_str("idle_changes"), moor_var::v_int(idle_changes_count)),
            (moor_var::v_str("pending_review"), moor_var::v_int(pending_review_count)),
            (moor_var::v_str("current_username"), current_username),
            (moor_var::v_str("changes_in_index"), moor_var::v_int(changes_in_index)),
            (moor_var::v_str("latest_merged_change"), latest_merged_change),
            (moor_var::v_str("index_partition_size"), moor_var::v_int(index_partition_size)),
            (moor_var::v_str("refs_partition_size"), moor_var::v_int(refs_partition_size)),
            (moor_var::v_str("objects_partition_size"), moor_var::v_int(objects_partition_size)),
            (moor_var::v_str("remote_url"), moor_var::v_str(&remote_url)),
            (moor_var::v_str("pending_updates"), moor_var::v_int(pending_updates)),
        ]);
        
        info!("Status request completed successfully");
        Ok(status_map)
    }
    
    /// Get the latest merged change (non-local)
    fn get_latest_merged_change(&self) -> Result<moor_var::Var, ObjectsTreeError> {
        let change_order = self.database.index().get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Iterate through changes in reverse order (newest first)
        for change_id in change_order.iter().rev() {
            if let Some(change) = self.database.index().get_change(change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
                
                // Check if it's merged (not local)
                if change.status == ChangeStatus::Merged {
                    // Return a map with change details
                    return Ok(moor_var::v_map(&[
                        (moor_var::v_str("id"), moor_var::v_str(&change.id)),
                        (moor_var::v_str("author"), moor_var::v_str(&change.author)),
                        (moor_var::v_str("timestamp"), moor_var::v_int(change.timestamp as i64)),
                        (moor_var::v_str("message"), moor_var::v_str(change.description.as_deref().unwrap_or(""))),
                    ]));
                }
            }
        }
        
        // No merged changes found
        Ok(moor_var::v_str("none"))
    }
}

impl Operation for StatusOperation {
    fn name(&self) -> &'static str {
        "system/status"
    }
    
    fn description(&self) -> &'static str {
        "Get comprehensive system status including change counts, partition sizes, and remote repository information"
    }
    
    fn philosophy(&self) -> &'static str {
        "Provides a complete overview of the VCS repository state, including local changes, workspace status, \
        partition sizes, and remote repository information. This operation is useful for monitoring system health, \
        understanding current repository state, and determining if synchronization with remote repositories is needed."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Get system status".to_string(),
                moocode: r#"status = worker_request("vcs", {"system/status"});
// Returns a map with:
// - top_change_id: ID of current local change
// - idle_changes: Count of idle changes in workspace
// - pending_review: Count of changes awaiting approval
// - current_username: Your username
// - changes_in_index: Total changes in working index
// - latest_merged_change: Info about most recent merged change
// - index_partition_size: Size of index partition in bytes
// - refs_partition_size: Size of refs partition in bytes
// - objects_partition_size: Size of objects partition in bytes
// - remote_url: Remote repository URL (if cloned)
// - pending_updates: Number of updates available from remote"#.to_string(),
                http_curl: Some(r#"curl -X GET http://localhost:8081/api/system/status"#.to_string()),
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/system/status".to_string(),
                method: Method::GET,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, _args: Vec<String>, user: &User) -> moor_var::Var {
        info!("System status operation received");
        
        match self.process_status(user) {
            Ok(result) => {
                info!("System status operation completed successfully");
                result
            }
            Err(e) => {
                error!("System status operation failed: {}", e);
                moor_var::v_str(&format!("Error: {}", e))
            }
        }
    }
}

