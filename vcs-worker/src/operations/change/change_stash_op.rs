use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::object_diff::{ObjectDiffModel, build_abandon_diff_from_change};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{ChangeStashRequest, ChangeStatus, Permission, User};
use moor_var::{E_INVARG, v_error};

/// Change stash operation that stashes a local change to workspace for later resumption
#[derive(Clone)]
pub struct ChangeStashOperation {
    database: DatabaseRef,
}

impl ChangeStashOperation {
    /// Create a new change stash operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change stash request
    fn process_change_stash(
        &self,
        _request: ChangeStashRequest,
        user: &User,
    ) -> Result<ObjectDiffModel, ObjectsTreeError> {
        // Check if user has permission to stash changes
        if !user.has_permission(&Permission::SubmitChanges) {
            error!(
                "User '{}' does not have permission to stash changes",
                user.id
            );
            return Err(ObjectsTreeError::SerializationError(format!(
                "User '{}' does not have permission to stash changes",
                user.id
            )));
        }

        // Get the top change from the index
        let top_change_id = self
            .database
            .index()
            .get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError("No change to stash".to_string())
            })?;

        // Get the change
        let mut change = self
            .database
            .index()
            .get_change(&top_change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!("Change '{top_change_id}' not found"))
            })?;

        info!(
            "User '{}' attempting to stash change: {} ({})",
            user.id, change.name, change.id
        );

        // Check if the change is local
        if change.status != ChangeStatus::Local {
            error!(
                "Cannot stash change '{}' ({}) - it is not local (status: {:?})",
                change.name, change.id, change.status
            );
            return Err(ObjectsTreeError::SerializationError(format!(
                "Cannot stash change '{}' - it is not local (status: {:?})",
                change.name, change.status
            )));
        }

        // Build the undo diff (like abandon/submit does)
        let undo_diff = build_abandon_diff_from_change(&self.database, &change)?;

        // Change the status to Idle (stashed for later work)
        change.status = ChangeStatus::Idle;

        // Store the change in the workspace (where idle changes live)
        self.database
            .workspace()
            .store_workspace_change(&change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        info!(
            "Stored change '{}' in workspace with Idle status",
            change.name
        );

        // Remove the change from the working index
        self.database
            .index()
            .remove_from_index(&change.id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        info!("Removed change '{}' from top of index", change.name);

        info!(
            "Successfully stashed change '{}' ({}), moved to workspace as Idle",
            change.name, change.id
        );

        Ok(undo_diff)
    }
}

impl Operation for ChangeStashOperation {
    fn name(&self) -> &'static str {
        "change/stash"
    }

    fn description(&self) -> &'static str {
        "Stashes the top local change to workspace with Idle status for later resumption. Removes from index. Returns an ObjectDiffModel showing what changes need to be undone. Local only (no remote submission)."
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn philosophy(&self) -> &'static str {
        "Temporarily sets aside your current work without abandoning or submitting it. This is useful when you \
        need to switch contexts quickly - perhaps to work on an urgent fix - but want to preserve your current \
        changes for later. Unlike change/switch, stash doesn't require switching to another specific change; it \
        simply saves your current work to the workspace with 'Idle' status and clears your working state. You \
        can resume a stashed change later with change/switch. The operation returns a diff showing what needs \
        to be undone in your MOO database. This is purely a local operation and doesn't involve remote submission."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Stash the current change for later".to_string(),
            moocode: r#"// You're working on a feature but need to switch contexts
diff = worker_request("vcs", {"change/stash"});
// diff is an ObjectDiffModel (MOO map) showing what needs to be undone
// Current change is saved to workspace with Idle status
player:tell("Stashed. Need to revert ", length(diff["modified_objects"]), " objects");

// Later, resume with change/switch
workspace_list = worker_request("vcs", {"workspace/list"});
// Find your stashed change ID from the list (it has Idle status)
stashed_id = "your-change-id";
worker_request("vcs", {"change/switch", stashed_id});"#
                .to_string(),
            http_curl: Some(r#"curl -X POST http://localhost:8081/api/change/stash"#.to_string()),
        }]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/change/stash".to_string(),
            method: Method::POST,
            is_json: false,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#"["objects_renamed" -> [], "objects_deleted" -> {}, "objects_added" -> {}, "objects_modified" -> {"obj1"}, "changes" -> {["obj_id" -> "obj1", "verbs_modified" -> {}, "verbs_added" -> {}, "verbs_renamed" -> [], "verbs_deleted" -> {}, "props_modified" -> {}, "props_added" -> {}, "props_renamed" -> [], "props_deleted" -> {}]}]"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Cannot stash non-local change",
                r#"E_INVARG("Error: Cannot stash change 'my-change' - it is not local (status: Review)")"#,
            ),
            OperationResponse::new(
                403,
                "Forbidden - User lacks permission to stash changes",
                r#"E_INVARG("Error: User 'player123' does not have permission to submit changes)"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - No change to stash",
                r#"E_INVARG("Error: No change to stash")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#"E_INVARG("Error: Database error: failed to stash change")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!(
            "Change stash operation received {} arguments for user: {}",
            args.len(),
            user.id
        );

        let request = ChangeStashRequest {};

        match self.process_change_stash(request, user) {
            Ok(undo_diff) => {
                info!("Change stash operation completed successfully, returning undo diff");
                // Return the ObjectDiffModel as a MOO variable showing what needs to be undone
                undo_diff.to_moo_var()
            }
            Err(e) => {
                error!("Change stash operation failed: {}", e);
                v_error(E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}
