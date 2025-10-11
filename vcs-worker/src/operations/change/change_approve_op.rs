use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
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
        
        // Try to get the change from workspace first (it has the most recent version)
        // If a change is submitted to remote, it's moved to workspace with Review status
        // but the old Local version may still be in history_storage
        let mut change = match self.database.workspace().get_workspace_change(&change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            Some(ch) => ch,
            None => {
                // Not in workspace, try index
                self.database.index().get_change(&change_id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                    .ok_or_else(|| ObjectsTreeError::SerializationError(
                        format!("Change '{}' not found in workspace or index", change_id)
                    ))?
            }
        };
        
        info!("User '{}' attempting to approve change: {} ({}) with status {:?}", 
              user.id, change.name, change.id, change.status);
        
        // Check if the change is local or review
        if change.status != ChangeStatus::Local && change.status != ChangeStatus::Review {
            error!("Cannot approve change '{}' ({}) - it must be Local or Review status (current: {:?})", 
                   change.name, change.id, change.status);
            return Err(ObjectsTreeError::SerializationError(
                format!("Cannot approve change '{}' - it must be Local or Review status (current: {:?})", 
                        change.name, change.status)
            ));
        }
        
        // Check if there's already a local change on top of the index
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        let is_top_change = top_change_id.as_ref() == Some(&change_id);
        
        if let Some(top_id) = &top_change_id {
            if top_id != &change_id {
                // There's a different change on top - check if it's local
                if let Some(top_change) = self.database.index().get_change(top_id)
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
        // If this is the top change (current working change), return an empty diff
        // because there are no NEW changes relative to the current state
        let diff_model = if is_top_change {
            info!("Approving top change - returning empty diff (no new changes relative to current state)");
            ObjectDiffModel::new()
        } else {
            info!("Approving non-top change - building diff model");
            build_object_diff_from_change(&self.database, &change)?
        };
        
        // Remember original status to determine if we need to add to change_order
        let was_in_workspace = change.status == ChangeStatus::Review;
        
        info!("Change status before approval: {:?}, was_in_workspace: {}", change.status, was_in_workspace);
        
        // Update the change status to Merged
        change.status = ChangeStatus::Merged;
        
        // If the change was in workspace (Review status), add it back to the index
        if was_in_workspace {
            info!("Processing workspace approval - will add back to index");
            
            // Store the change in the index
            self.database.index().store_change(&change)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            
            info!("Stored change in index, now calling append_change_to_order");
            
            // Add the change to change_order (as merged history)
            // Use append_change_to_order which adds to the end without setting as top_change
            self.database.index().append_change_to_order(&change_id)
                .map_err(|e| {
                    error!("Failed to append change to order: {}", e);
                    ObjectsTreeError::SerializationError(e.to_string())
                })?;
            
            info!("Added change '{}' back to index as merged", change.name);
        } else {
            info!("Processing local approval - will update in place");
            
            // Change was already in index (Local status), just update it
            self.database.index().update_change(&change)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        }
        
        // Clear the top_change pointer (change stays in history as merged)
        self.database.index().clear_top_change_if(&change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Remove the change from workspace if it exists there
        if self.database.workspace().get_workspace_change(&change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .is_some() {
            self.database.workspace().delete_workspace_change(&change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            info!("Removed change '{}' from workspace", change.name);
        }
        
        if was_in_workspace {
            info!("Successfully approved change '{}' ({}) from workspace, added to index as merged", 
                  change.name, change.id);
        } else {
            info!("Successfully approved change '{}' ({}), marked as merged", 
                  change.name, change.id);
        }
        
        Ok(diff_model)
    }
    
}

impl Operation for ChangeApproveOperation {
    fn name(&self) -> &'static str {
        "change/approve"
    }
    
    fn description(&self) -> &'static str {
        "Approves a change (Local or Review status) by marking it as merged. If the change is in workspace (Review status), it's added back to the index. If it's already in the index (Local status), it's updated in place. Returns a ChangeDiff showing what was approved."
    }
    
    fn philosophy(&self) -> &'static str {
        "Finalizes the review workflow by approving a submitted change and merging it into the repository \
        history. This operation is typically used by reviewers or administrators to accept changes that have \
        been submitted for review (with 'Review' status). Once approved, the change becomes part of the permanent \
        repository history with 'Merged' status. For changes in workspace (submitted remotely), approval adds \
        them to the index. For local changes, approval updates them in place. This is a privileged operation - \
        users must have the ApproveChanges permission to execute it."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "change_id".to_string(),
                description: "The ID of the change to approve (get from workspace/list)".to_string(),
                required: true,
            }
        ]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Approve a change that's been submitted for review".to_string(),
                moocode: r#"// List workspace changes to find the one to approve
workspace_json = worker_request("vcs", {"workspace/list"});
changes = parse_json(workspace_json)["changes"];
// Find a change with Review status
change_id = changes[1]["id"];

// Approve it
diff = worker_request("vcs", {"change/approve", change_id});
// Change is now merged into repository history"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/change/approve \
  -H "Content-Type: application/json" \
  -d '{"operation": "change/approve", "args": ["abc-123-def..."]}'"#.to_string()),
            }
        ]
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
