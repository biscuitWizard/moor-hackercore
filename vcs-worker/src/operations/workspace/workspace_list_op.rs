use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{Change, ChangeStatus, Permission, User};
use moor_var::{E_INVARG, Var, v_error, v_int, v_list, v_map, v_str};

/// Workspace list operation that lists all changes in the workspace (stashed, in review, etc.)
#[derive(Clone)]
pub struct WorkspaceListOperation {
    database: DatabaseRef,
}

impl WorkspaceListOperation {
    /// Create a new workspace list operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the workspace list request
    fn process_workspace_list(
        &self,
        user: &User,
        status_filter: Option<ChangeStatus>,
    ) -> Result<Vec<Change>, ObjectsTreeError> {
        // Check if user has permission to view workspace changes (using SubmitChanges as it's the closest permission)
        if !user.has_permission(&Permission::SubmitChanges) {
            error!(
                "User '{}' does not have permission to view workspace changes",
                user.id
            );
            return Err(ObjectsTreeError::SerializationError(format!(
                "User '{}' does not have permission to view workspace changes",
                user.id
            )));
        }

        info!("User '{}' requesting workspace changes list", user.id);

        // Get changes based on filter
        let changes = if let Some(status) = &status_filter {
            self.database
                .workspace()
                .list_workspace_changes_by_status(status.clone())
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        } else {
            self.database
                .workspace()
                .list_all_workspace_changes()
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        };

        info!(
            "Found {} workspace changes for user '{}'",
            changes.len(),
            user.id
        );

        Ok(changes)
    }

    /// Convert a Change to a MOO map structure
    fn change_to_moo_map(&self, change: &Change) -> Var {
        let mut pairs = Vec::new();

        // id (full hash)
        pairs.push((v_str("id"), v_str(&change.id)));

        // short_id (abbreviated hash)
        let short_id = crate::util::short_hash(&change.id);
        pairs.push((v_str("short_id"), v_str(&short_id)));

        // name
        pairs.push((v_str("name"), v_str(&change.name)));

        // description (optional)
        if let Some(desc) = &change.description {
            pairs.push((v_str("description"), v_str(desc)));
        }

        // author
        pairs.push((v_str("author"), v_str(&change.author)));

        // timestamp
        pairs.push((v_str("timestamp"), v_int(change.timestamp as i64)));

        // status
        let status_str = match change.status {
            ChangeStatus::Review => "Review",
            ChangeStatus::Idle => "Idle",
            ChangeStatus::Merged => "Merged",
            ChangeStatus::Local => "Local",
        };
        pairs.push((v_str("status"), v_str(status_str)));

        // based_on (optional)
        if let Some(index_id) = &change.index_change_id {
            pairs.push((v_str("based_on"), v_str(index_id)));
        }

        // Build changes list (similar to ObjectDiffModel)
        let mut changes_pairs = Vec::new();

        // objects_added
        let added_list: Vec<Var> = change
            .added_objects
            .iter()
            .map(|obj| v_str(&obj.name))
            .collect();
        changes_pairs.push((v_str("objects_added"), v_list(&added_list)));

        // objects_modified
        let modified_list: Vec<Var> = change
            .modified_objects
            .iter()
            .map(|obj| v_str(&obj.name))
            .collect();
        changes_pairs.push((v_str("objects_modified"), v_list(&modified_list)));

        // objects_deleted
        let deleted_list: Vec<Var> = change
            .deleted_objects
            .iter()
            .map(|obj| v_str(&obj.name))
            .collect();
        changes_pairs.push((v_str("objects_deleted"), v_list(&deleted_list)));

        // objects_renamed
        let renamed_list: Vec<Var> = change
            .renamed_objects
            .iter()
            .map(|renamed| {
                let rename_pairs = vec![
                    (v_str("from"), v_str(&renamed.from.name)),
                    (v_str("to"), v_str(&renamed.to.name)),
                ];
                v_map(&rename_pairs)
            })
            .collect();
        changes_pairs.push((v_str("objects_renamed"), v_list(&renamed_list)));

        pairs.push((v_str("changes"), v_map(&changes_pairs)));

        v_map(&pairs)
    }
}

impl Operation for WorkspaceListOperation {
    fn name(&self) -> &'static str {
        "workspace/list"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Lists all changes in the workspace. Optionally filter by status (Review, Idle). Usage: workspace/list [status]"
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/workspace/list".to_string(),
            method: Method::GET,
            is_json: false,
        }]
    }

    fn philosophy(&self) -> &'static str {
        "Lists all changes currently in the workspace, including those awaiting review (Review status) and those that \
        are stashed for later work (Idle status). This operation allows filtering by status to show only specific types \
        of changes. Each change entry includes detailed information about the author, timestamp, affected objects, and \
        change type. This operation requires the SubmitChanges permission."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "List all workspace changes".to_string(),
                moocode: r#"changes = worker_request("vcs", {"workspace/list"});
// Returns a list of all changes in the workspace with full details
for change in (changes)
    player:tell("Change: ", change["name"], " by ", change["author"], 
                " - Status: ", change["status"]);
    player:tell("  Objects added: ", tostr(change["changes"]["objects_added"]));
endfor"#
                    .to_string(),
                http_curl: Some(r#"curl -X GET http://localhost:8081/api/workspace/list"#.to_string()),
            },
            OperationExample {
                description: "List only changes awaiting review".to_string(),
                moocode: r#"review_changes = worker_request("vcs", {"workspace/list", "review"});
// Returns only changes with Review status
player:tell("Changes awaiting review: ", length(review_changes));"#
                    .to_string(),
                http_curl: Some(
                    r#"curl -X GET "http://localhost:8081/api/workspace/list?status=review""#
                        .to_string(),
                ),
            },
        ]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully - multiple changes found",
                concat!(
                    r#"{["id" -> "f8a3c2e1b9d04567a890e1f2c3d4e5f6a7b8c9d0", "short_id" -> "f8a3c2e1", "name" -> "my-feature-change", "#,
                    r#""description" -> "Added new login system", "author" -> "wizard", "timestamp" -> 1728651045, "status" -> "Review", "#,
                    r#""based_on" -> "e1f2c3d4e5f6a7b8c9d0f1a2b3c4d5e6f7a8b9c0", "changes" -> ["objects_added" -> {"obj123", "obj124"}, "#,
                    r#""objects_modified" -> {"obj100", "obj101"}, "objects_deleted" -> {}, "objects_renamed" -> {["from" -> "obj50", "to" -> "obj51"]}]], "#,
                    r#"["id" -> "a1b2c3d4e5f6a7b8c9d0e1f2c3d4e5f6a7b8c9d0", "short_id" -> "a1b2c3d4", "name" -> "bugfix-auth", "#,
                    r#""description" -> "Fixed authentication bug", "author" -> "programmer", "timestamp" -> 1728657930, "status" -> "Review", "#,
                    r#""changes" -> ["objects_added" -> {}, "objects_modified" -> {"obj200", "obj201", "obj202"}, "objects_deleted" -> {}, "objects_renamed" -> {}]]}"#
                ),
            ),
            OperationResponse::success(
                "Operation executed successfully - single change found",
                concat!(
                    r#"{["id" -> "a1b2c3d4e5f6a7b8c9d0e1f2c3d4e5f6a7b8c9d0", "short_id" -> "a1b2c3d4", "name" -> "feature-work", "#,
                    r#""author" -> "wizard", "timestamp" -> 1728651045, "status" -> "Idle", "#,
                    r#""based_on" -> "e1f2c3d4e5f6a7b8c9d0f1a2b3c4d5e6f7a8b9c0", "#,
                    r#""changes" -> ["objects_added" -> {"obj123"}, "objects_modified" -> {}, "objects_deleted" -> {}, "objects_renamed" -> {}]]}"#
                ),
            ),
            OperationResponse::success(
                "Operation executed successfully - no changes found",
                r#"{}"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Invalid status filter",
                r#"E_INVARG("Invalid status filter: pending. Valid options: review, idle")"#,
            ),
            OperationResponse::new(
                403,
                "Forbidden - User lacks permission to view workspace changes",
                r#"E_INVARG("User 'player123' does not have permission to view workspace changes")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#"E_INVARG("Database error: failed to list workspace changes")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!(
            "Workspace list operation received {} arguments for user: {}",
            args.len(),
            user.id
        );

        // Parse optional status filter
        let status_filter = if args.is_empty() {
            None
        } else {
            match args[0].to_lowercase().as_str() {
                "review" => Some(ChangeStatus::Review),
                "idle" => Some(ChangeStatus::Idle),
                _ => {
                    error!(
                        "Invalid status filter: {}. Valid options: review, idle",
                        args[0]
                    );
                    return v_error(E_INVARG.msg(format!(
                        "Invalid status filter: {}. Valid options: review, idle",
                        args[0]
                    )));
                }
            }
        };

        match self.process_workspace_list(user, status_filter) {
            Ok(changes) => {
                info!(
                    "Workspace list operation completed successfully, found {} changes",
                    changes.len()
                );
                // Convert each change to a MOO map and return as a list
                let change_maps: Vec<Var> = changes
                    .iter()
                    .map(|change| self.change_to_moo_map(change))
                    .collect();
                v_list(&change_maps)
            }
            Err(e) => {
                error!("Workspace list operation failed: {}", e);
                v_error(E_INVARG.msg(format!("{e}")))
            }
        }
    }
}
