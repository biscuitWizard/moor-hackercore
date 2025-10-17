use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::object_diff::build_object_diff_from_change;
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::ChangeStatus;
use crate::types::User;
use moor_var::{E_INVARG, v_error};

/// Request structure for change status operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatusRequest {
    /// Optional change ID to query. If not provided, queries the current top local change.
    #[serde(default)]
    pub change_id: Option<String>,
}

/// Response structure for change status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatusResponse {
    pub change_id: Option<String>,
    pub change_message: Option<String>,
    pub status: ChangeStatus,
}

/// Change status operation that lists all objects modified in the current change
#[derive(Clone)]
pub struct ChangeStatusOperation {
    database: DatabaseRef,
}

impl ChangeStatusOperation {
    /// Create a new change status operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change status request
    fn process_change_status(
        &self,
        request: ChangeStatusRequest,
    ) -> Result<moor_var::Var, ObjectsTreeError> {
        // If a specific change_id was provided, fetch that change
        let current_change = if let Some(ref change_id) = request.change_id {
            info!("Fetching status for specific change: {}", change_id);
            
            // Resolve short or full hash to full hash
            let resolved_change_id = self.database.resolve_change_id(change_id)?;
            info!("Resolved '{}' to full change ID: {}", change_id, resolved_change_id);
            
            // Try to find the change in index first
            let change_from_index = self
                .database
                .index()
                .get_change(&resolved_change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            
            if let Some(change) = change_from_index {
                change
            } else {
                // Try workspace if not in index
                let change_from_workspace = self
                    .database
                    .workspace()
                    .get_workspace_change(&resolved_change_id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                
                change_from_workspace.ok_or_else(|| {
                    ObjectsTreeError::SerializationError(format!(
                        "Change '{}' not found in index or workspace",
                        resolved_change_id
                    ))
                })?
            }
        } else {
            // Default behavior: get the top change from the index
            let top_change_id = self
                .database
                .index()
                .get_top_change()
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

            if let Some(change_id) = top_change_id {
                // Get the actual change object
                let change = self
                    .database
                    .index()
                    .get_change(&change_id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                    .ok_or_else(|| {
                        ObjectsTreeError::SerializationError(format!("Change '{change_id}' not found"))
                    })?;

                // Check if the top change is local status
                if change.status != ChangeStatus::Local {
                    info!(
                        "Top change '{}' is not local status (status: {:?}), returning error",
                        change.name, change.status
                    );
                    return Ok(v_error(
                        E_INVARG.msg("No local change on top of index - nothing to do"),
                    ));
                }
                
                change
            } else {
                info!("No top change found, returning error");
                return Ok(v_error(
                    E_INVARG.msg("No change on top of index - nothing to do"),
                ));
            }
        };

        info!("Getting status for change: {} ({})", current_change.name, current_change.id);

        // Build the ObjectDiffModel by comparing change against the compiled state
        let diff_model = build_object_diff_from_change(&self.database, &current_change)?;

        // Convert to MOO Var and return
        let status_map = diff_model.to_moo_var();

        info!(
            "Successfully retrieved change status for '{}'",
            current_change.name
        );
        Ok(status_map)
    }
}

impl Operation for ChangeStatusOperation {
    fn name(&self) -> &'static str {
        "change/status"
    }

    fn description(&self) -> &'static str {
        "Lists all objects that have been modified in a change (added, modified, deleted, renamed). Can query current change or a specific change by ID."
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn philosophy(&self) -> &'static str {
        "Provides a summary of all changes in a changelist. By default, it shows your current local changelist, \
        which is your primary tool for reviewing what you've done before submitting. You can also query any \
        specific change by ID to review historical changes, review submissions, or idle changes. The operation \
        shows which objects have been added, modified, deleted, or renamed. The operation returns an ObjectDiffModel \
        that categorizes all changes, making it easy to verify work is correct. Use this regularly during development \
        to track your progress and ensure you haven't accidentally modified objects you didn't intend to change."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "change_id".to_string(),
                description: "Optional change ID (full 64-char hash or short 12-char prefix) to query. If not provided, queries the current top local change.".to_string(),
                required: false,
            },
        ]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Get status of current change".to_string(),
                moocode: r#"diff = worker_request("vcs", {"change/status"});
// Returns an ObjectDiffModel map like:
// [#<added_objects => {object_name => objdef}, ...>, #<modified_objects => {...}>, ...]
player:tell("Added: ", length(diff["added_objects"]), " objects");
player:tell("Modified: ", length(diff["modified_objects"]), " objects");
player:tell("Deleted: ", length(diff["deleted_objects"]), " objects");"#
                    .to_string(),
                http_curl: Some(r#"curl -X GET http://localhost:8081/api/change/status"#.to_string()),
            },
            OperationExample {
                description: "Get status of a specific change by ID (full or short hash)".to_string(),
                moocode: r#"// Use full 64-character Blake3 hash
change_id = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
diff = worker_request("vcs", {"change/status", change_id});

// Or use short 12-character hash prefix
short_id = "abcdef123456";
diff = worker_request("vcs", {"change/status", short_id});

// Returns the ObjectDiffModel for the specific change
player:tell("Added: ", length(diff["added_objects"]), " objects");
player:tell("Modified: ", length(diff["modified_objects"]), " objects");"#
                    .to_string(),
                http_curl: Some(r#"curl -X GET 'http://localhost:8081/api/change/status?change_id=abcdef123456'"#.to_string()),
            },
        ]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/change/status".to_string(),
            method: Method::GET,
            is_json: false,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#"["objects_renamed" -> ["old_obj" -> "new_obj"], "objects_deleted" -> {"obj1"}, "objects_added" -> {"obj2"}, "objects_modified" -> {"obj3"}, "changes" -> {["obj_id" -> "obj3", "verbs_modified" -> {"verb1"}, "verbs_added" -> {}, "verbs_renamed" -> [], "verbs_deleted" -> {}, "props_modified" -> {"prop1"}, "props_added" -> {}, "props_renamed" -> [], "props_deleted" -> {}]}]"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - No local change available",
                r#"E_INVARG("No local change on top of index - nothing to do)"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - No change to query",
                r#"E_INVARG("Error: No change on top of index - nothing to do)"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#"E_INVARG("Error: Database error: failed to build diff model")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Change status operation executed with {} args", args.len());

        let request = ChangeStatusRequest {
            change_id: args.get(0).map(|s| s.to_string()),
        };

        if let Some(ref cid) = request.change_id {
            info!("Requesting status for specific change: {}", cid);
        } else {
            info!("Requesting status for current top local change");
        }

        match self.process_change_status(request) {
            Ok(result_var) => {
                info!("Change status operation completed successfully");
                result_var
            }
            Err(e) => {
                error!("Change status operation failed: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}
