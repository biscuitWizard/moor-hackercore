use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{ChangeSwitchRequest, ChangeStatus, User};
use crate::object_diff::{ObjectDiffModel, build_abandon_diff_from_change, build_object_diff_from_change};
use moor_var::{v_error, E_INVARG};

/// Change switch operation that switches from the current change to a workspace change
#[derive(Clone)]
pub struct ChangeSwitchOperation {
    database: DatabaseRef,
}

impl ChangeSwitchOperation {
    /// Create a new change switch operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change switch request
    fn process_change_switch(&self, request: ChangeSwitchRequest, _user: &User) -> Result<ObjectDiffModel, ObjectsTreeError> {
        let target_change_id = request.change_id;
        
        info!("Attempting to switch to change: {}", target_change_id);
        
        // Get the target change from workspace
        let mut target_change = self.database.workspace().get_workspace_change(&target_change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError(
                format!("Change '{}' not found in workspace", target_change_id)
            ))?;
        
        info!("Found target change '{}' in workspace (status: {:?})", target_change.name, target_change.status);
        
        // Initialize the merged diff
        let mut merged_diff = ObjectDiffModel::new();
        
        // Check if there's a local change on top of the index
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(current_change_id) = top_change_id {
            // Get the current change
            let mut current_change = self.database.index().get_change(&current_change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .ok_or_else(|| ObjectsTreeError::SerializationError(
                    format!("Current change '{}' not found", current_change_id)
                ))?;
            
            // Only handle local changes - can't switch away from merged/review changes
            if current_change.status == ChangeStatus::Local {
                info!("Found local change '{}' on top of index, moving to workspace", current_change.name);
                
                // Build the abandon diff (to undo the current change)
                let abandon_diff = build_abandon_diff_from_change(&self.database, &current_change)?;
                
                // Merge the abandon diff first (this undoes the current change)
                merged_diff.merge(abandon_diff);
                
                // Change status to Idle (unfinished change moved to workspace)
                current_change.status = ChangeStatus::Idle;
                
                // Store the current change in workspace
                self.database.workspace().store_workspace_change(&current_change)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                
                info!("Stored current change '{}' in workspace with Idle status", current_change.name);
                
                // Remove from working index
                self.database.index().remove_from_index(&current_change_id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                
                info!("Removed current change '{}' from index", current_change.name);
            } else {
                error!("Cannot switch away from non-local change '{}' (status: {:?})", 
                       current_change.name, current_change.status);
                return Err(ObjectsTreeError::SerializationError(
                    format!("Cannot switch away from non-local change '{}' (status: {:?})", 
                            current_change.name, current_change.status)
                ));
            }
        }
        
        // Build the object diff for the target change (to apply it)
        let target_diff = build_object_diff_from_change(&self.database, &target_change)?;
        
        // Merge the target diff (this applies the new change)
        merged_diff.merge(target_diff);
        
        // Change target status to Local (it's now the active change)
        target_change.status = ChangeStatus::Local;
        
        // Store the target change in index
        self.database.index().store_change(&target_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Stored target change '{}' in index", target_change.name);
        
        // Add to top of index
        self.database.index().push_change(&target_change.id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Added target change '{}' to top of index", target_change.name);
        
        // Remove from workspace
        self.database.workspace().delete_workspace_change(&target_change.id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Removed target change '{}' from workspace", target_change.name);
        
        info!("Successfully switched to change '{}' ({})", target_change.name, target_change.id);
        
        Ok(merged_diff)
    }
}

impl Operation for ChangeSwitchOperation {
    fn name(&self) -> &'static str {
        "change/switch"
    }
    
    fn description(&self) -> &'static str {
        "Switches to a different change from workspace. If there's a local change on top of the index, moves it to workspace as Idle. Returns a merged ObjectDiffModel with first the undo diff for the current change (if any), then the apply diff for the target change."
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/change/switch".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/change/switch".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Workspace change switch operation received {} arguments for user: {}", args.len(), user.id);
        
        if args.is_empty() {
            error!("Workspace change switch operation requires a change_id argument");
            return v_error(E_INVARG.msg("Workspace change switch operation requires a change_id argument"));
        }
        
        let change_id = args[0].clone();
        let request = ChangeSwitchRequest { change_id };

        match self.process_change_switch(request, user) {
            Ok(merged_diff) => {
                info!("Workspace change switch operation completed successfully, returning merged diff");
                // Return the merged ObjectDiffModel as a MOO variable
                merged_diff.to_moo_var()
            }
            Err(e) => {
                error!("Workspace change switch operation failed: {}", e);
                v_error(E_INVARG.msg(&format!("Error: {e}")))
            }
        }
    }
}
