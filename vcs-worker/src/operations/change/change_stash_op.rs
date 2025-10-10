use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{ChangeStashRequest, ChangeStatus, User, Permission};
use crate::object_diff::{ObjectDiffModel, build_abandon_diff_from_change};
use moor_var::{v_error, E_INVARG};

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
    fn process_change_stash(&self, _request: ChangeStashRequest, user: &User) -> Result<ObjectDiffModel, ObjectsTreeError> {
        // Check if user has permission to stash changes
        if !user.has_permission(&Permission::SubmitChanges) {
            error!("User '{}' does not have permission to stash changes", user.id);
            return Err(ObjectsTreeError::SerializationError(
                format!("User '{}' does not have permission to stash changes", user.id)
            ));
        }

        // Get the top change from the index
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError("No change to stash".to_string()))?;

        // Get the change
        let mut change = self.database.index().get_change(&top_change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Change '{}' not found", top_change_id)))?;

        info!("User '{}' attempting to stash change: {} ({})", user.id, change.name, change.id);

        // Check if the change is local
        if change.status != ChangeStatus::Local {
            error!("Cannot stash change '{}' ({}) - it is not local (status: {:?})", 
                   change.name, change.id, change.status);
            return Err(ObjectsTreeError::SerializationError(
                format!("Cannot stash change '{}' - it is not local (status: {:?})", change.name, change.status)
            ));
        }

        // Build the undo diff (like abandon/submit does)
        let undo_diff = build_abandon_diff_from_change(&self.database, &change)?;

        // Change the status to Idle (stashed for later work)
        change.status = ChangeStatus::Idle;

        // Store the change in the workspace (where idle changes live)
        self.database.workspace().store_workspace_change(&change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        info!("Stored change '{}' in workspace with Idle status", change.name);

        // Remove the change from the working index
        self.database.index().remove_from_index(&change.id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        info!("Removed change '{}' from top of index", change.name);

        info!("Successfully stashed change '{}' ({}), moved to workspace as Idle", 
              change.name, change.id);

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
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/change/stash".to_string(),
                method: Method::POST,
                is_json: false, // No body needed
            },
            OperationRoute {
                path: "/api/change/stash".to_string(),
                method: Method::POST,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Change stash operation received {} arguments for user: {}", args.len(), user.id);
        
        let request = ChangeStashRequest {};

        match self.process_change_stash(request, user) {
            Ok(undo_diff) => {
                info!("Change stash operation completed successfully, returning undo diff");
                // Return the ObjectDiffModel as a MOO variable showing what needs to be undone
                undo_diff.to_moo_var()
            }
            Err(e) => {
                error!("Change stash operation failed: {}", e);
                v_error(E_INVARG.msg(&format!("Error: {e}")))
            }
        }
    }
}

