use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{ChangeStatus, User};
use moor_var::{E_INVARG, v_error};

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

    /// Get the data size of a partition in bytes (sum of all keys and values)
    fn get_partition_data_size(&self, partition_name: &str) -> u64 {
        self.database.get_partition_data_size(partition_name)
    }

    /// Process the status request
    fn process_status(&self, user: &User) -> Result<moor_var::Var, ObjectsTreeError> {
        info!("Processing system status request for user: {}", user.id);

        // Get top change ID
        let top_change_id = self
            .database
            .index()
            .get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .unwrap_or_else(String::new);

        // Get change order to count changes in working index
        let change_order = self
            .database
            .index()
            .get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let changes_in_index = change_order.len() as i64;

        // Get idle changes count from workspace
        let idle_changes = self
            .database
            .workspace()
            .get_idle_changes()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let idle_changes_count = idle_changes.len() as i64;

        // Get changes pending review count from workspace
        let pending_review = self
            .database
            .workspace()
            .get_changes_waiting_approval()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let pending_review_count = pending_review.len() as i64;

        // Get current username
        let current_username = moor_var::v_str(&user.id);

        // Get game name
        let game_name = moor_var::v_str(self.database.game_name());

        // Get latest non-local merged change
        let latest_merged_change = self.get_latest_merged_change()?;

        // Get remote repository URL if present
        let remote_url = self
            .database
            .index()
            .get_source()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .unwrap_or_else(String::new);

        // Get partition data sizes (sum of all keys and values in each partition)
        let index_partition_size = self.get_partition_data_size("index") as i64;
        let refs_partition_size = self.get_partition_data_size("refs") as i64;
        let objects_partition_size = self.get_partition_data_size("objects") as i64;

        // Check if we're behind remote (only if we have a source)
        let pending_updates = if !remote_url.is_empty() {
            // TODO: Implement checking remote for pending updates
            // For now, return 0 as we'd need to query the remote
            0i64
        } else {
            0i64
        };

        // Get short IDs
        let top_change_short_id = if !top_change_id.is_empty() {
            crate::util::short_hash(&top_change_id)
        } else {
            String::new()
        };

        // Build the status map
        let status_map = moor_var::v_map(&[
            (moor_var::v_str("game_name"), game_name),
            (
                moor_var::v_str("top_change_id"),
                moor_var::v_str(&top_change_id),
            ),
            (
                moor_var::v_str("top_change_short_id"),
                moor_var::v_str(&top_change_short_id),
            ),
            (
                moor_var::v_str("idle_changes"),
                moor_var::v_int(idle_changes_count),
            ),
            (
                moor_var::v_str("pending_review"),
                moor_var::v_int(pending_review_count),
            ),
            (moor_var::v_str("current_username"), current_username),
            (
                moor_var::v_str("changes_in_index"),
                moor_var::v_int(changes_in_index),
            ),
            (
                moor_var::v_str("latest_merged_change"),
                latest_merged_change,
            ),
            (
                moor_var::v_str("index_partition_size"),
                moor_var::v_int(index_partition_size),
            ),
            (
                moor_var::v_str("refs_partition_size"),
                moor_var::v_int(refs_partition_size),
            ),
            (
                moor_var::v_str("objects_partition_size"),
                moor_var::v_int(objects_partition_size),
            ),
            (moor_var::v_str("remote_url"), moor_var::v_str(&remote_url)),
            (
                moor_var::v_str("pending_updates"),
                moor_var::v_int(pending_updates),
            ),
        ]);

        info!("Status request completed successfully");
        Ok(status_map)
    }

    /// Get the latest merged change (non-local)
    fn get_latest_merged_change(&self) -> Result<moor_var::Var, ObjectsTreeError> {
        let change_order = self
            .database
            .index()
            .get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        // Iterate through changes in reverse order (newest first)
        for change_id in change_order.iter().rev() {
            if let Some(change) = self
                .database
                .index()
                .get_change(change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            {
                // Check if it's merged (not local)
                if change.status == ChangeStatus::Merged {
                    let short_id = crate::util::short_hash(&change.id);
                    // Return a map with change details
                    return Ok(moor_var::v_map(&[
                        (moor_var::v_str("id"), moor_var::v_str(&change.id)),
                        (moor_var::v_str("short_id"), moor_var::v_str(&short_id)),
                        (moor_var::v_str("author"), moor_var::v_str(&change.author)),
                        (
                            moor_var::v_str("timestamp"),
                            moor_var::v_int(change.timestamp as i64),
                        ),
                        (
                            moor_var::v_str("message"),
                            moor_var::v_str(change.description.as_deref().unwrap_or("")),
                        ),
                    ]));
                }
            }
        }

        // No merged changes found - return empty string
        Ok(moor_var::v_str(""))
    }
}

impl Operation for StatusOperation {
    fn name(&self) -> &'static str {
        "status"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Get comprehensive repository status including game name, change counts, partition sizes, and remote repository information"
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
        vec![OperationExample {
            description: "Get repository status".to_string(),
            moocode: r#"status = worker_request("vcs", {"status"});
// Returns a map with:
// - game_name: Name of the game/world
// - top_change_id: ID of current local change (empty if none)
// - idle_changes: Count of idle changes in workspace
// - pending_review: Count of changes awaiting approval
// - current_username: Your username
// - changes_in_index: Total changes in working index
// - latest_merged_change: Info about most recent merged change (empty if none)
// - index_partition_size: Size of index partition in bytes
// - refs_partition_size: Size of refs partition in bytes
// - objects_partition_size: Size of objects partition in bytes
// - remote_url: Remote repository URL (empty if not cloned)
// - pending_updates: Number of updates available from remote"#
                .to_string(),
            http_curl: Some(r#"curl -X GET http://localhost:8081/api/status"#.to_string()),
        }]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/status".to_string(),
            method: Method::GET,
            is_json: false,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#"["game_name" -> "MyGame", "top_change_id" -> "abc123def456...", "top_change_short_id" -> "abc123", "idle_changes" -> 2, "pending_review" -> 1, "current_username" -> "player", "changes_in_index" -> 5, "latest_merged_change" -> ["id" -> "def789ghi012...", "short_id" -> "def789", "author" -> "player", "timestamp" -> 1697040000, "message" -> "Fixed login bug"], "index_partition_size" -> 1048576, "refs_partition_size" -> 4096, "objects_partition_size" -> 8388608, "remote_url" -> "http://example.com/repo", "pending_updates" -> 0]"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database error",
                r#"E_INVARG("Database error: failed to get change order")"#,
            ),
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
                v_error(E_INVARG.msg(format!("{e}")))
            }
        }
    }
}
