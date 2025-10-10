use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{ChangeApproveRequest, ChangeStatus, User, Permission};
use crate::object_diff::{ObjectDiffModel, build_object_diff_from_change};
use moor_var::{v_error, E_INVARG};

/// Change approve operation that approves a local change and marks it as merged
#[derive(Clone)]
pub struct ChangeApproveOperation {
    database: DatabaseRef,
}

impl ChangeApproveOperation {
    /// Create a new change approve operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change approve request
    fn process_change_approve(&self, request: ChangeApproveRequest, user: &User) -> Result<ObjectDiffModel, ObjectsTreeError> {
        let change_id = request.change_id;
        
        // Check if user has permission to approve changes
        if !user.has_permission(&Permission::ApproveChanges) {
            error!("User '{}' does not have permission to approve changes", user.id);
            return Err(ObjectsTreeError::SerializationError(
                format!("User '{}' does not have permission to approve changes", user.id)
            ));
        }
        
        // Get the change by ID
        let mut change = self.database.index().get_change(&change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Change '{}' not found", change_id)))?;
        
        info!("User '{}' attempting to approve change: {} ({})", user.id, change.name, change.id);
        
        // Check if the change is local
        if change.status != ChangeStatus::Local {
            error!("Cannot approve change '{}' ({}) - it is not local (status: {:?})", 
                   change.name, change.id, change.status);
            return Err(ObjectsTreeError::SerializationError(
                format!("Cannot approve change '{}' - it is not local (status: {:?})", change.name, change.status)
            ));
        }
        
        // Check if there's already a local change on top of the index
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(top_id) = top_change_id {
            if top_id != change_id {
                // There's a different change on top - check if it's local
                if let Some(top_change) = self.database.index().get_change(&top_id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
                    if top_change.status == ChangeStatus::Local {
                        error!("Cannot approve change '{}' - there's already a local change '{}' on top of the index", 
                               change.name, top_change.name);
                        return Err(ObjectsTreeError::SerializationError(
                            format!("Cannot approve change '{}' - there's already a local change '{}' on top of the index", 
                                    change.name, top_change.name)
                        ));
                    }
                }
            }
        }
        
        // Build the ObjectDiffModel before changing the status
        let diff_model = build_object_diff_from_change(&self.database, &change)?;
        
        // Update the change status to Merged
        change.status = ChangeStatus::Merged;
        
        // Update the change in the database
        self.database.index().update_change(&change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Remove the change from workspace if it exists there (as a pending or stashed change)
        if self.database.workspace().get_workspace_change(&change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .is_some() {
            self.database.workspace().delete_workspace_change(&change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            info!("Removed change '{}' from workspace", change.name);
        }
        
        info!("Successfully approved change '{}' ({}), marked as merged", 
              change.name, change.id);
        
        Ok(diff_model)
    }
    
}

impl Operation for ChangeApproveOperation {
    fn name(&self) -> &'static str {
        "change/approve"
    }
    
    fn description(&self) -> &'static str {
        "Approves a local change by marking it as merged and removing it from the workspace if present. Returns a ChangeDiff showing what was approved."
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/change/approve".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/change/approve".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Change approve operation received {} arguments for user: {}", args.len(), user.id);
        
        if args.is_empty() {
            error!("Change approve operation requires a change ID argument");
            return v_error(E_INVARG.msg("Change approve operation requires a change ID argument"));
        }
        
        let change_id = args[0].clone();
        let request = ChangeApproveRequest { change_id };

        match self.process_change_approve(request, user) {
            Ok(diff_model) => {
                info!("Change approve operation completed successfully, returning change diff");
                // Return the ObjectDiffModel as a MOO variable showing what was approved
                diff_model.to_moo_var()
            }
            Err(e) => {
                error!("Change approve operation failed: {}", e);
                v_error(E_INVARG.msg(&format!("Error: {e}")))
            }
        }
    }
}
