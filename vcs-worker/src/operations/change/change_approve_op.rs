use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::{ChangeApproveRequest, ChangeStatus};
use crate::object_diff::{ObjectDiffModel, obj_id_to_object_name, compare_object_versions};
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
    fn process_change_approve(&self, request: ChangeApproveRequest) -> Result<ObjectDiffModel, ObjectsTreeError> {
        let change_id = request.change_id;
        
        // Get the change by ID
        let mut change = self.database.index().get_change(&change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Change '{}' not found", change_id)))?;
        
        info!("Attempting to approve change: {} ({})", change.name, change.id);
        
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
        let diff_model = self.build_object_diff(&change)?;
        
        // Update the change status to Merged
        change.status = ChangeStatus::Merged;
        
        // Update the change in the database
        self.database.index().update_change(&change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Remove the change from the top of the index (since it's no longer local)
        self.database.index().remove_change(&change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully approved change '{}' ({}), marked as merged and removed from index", 
              change.name, change.id);
        
        Ok(diff_model)
    }
    
    /// Build an ObjectDiffModel by comparing the local change against the compiled state below it
    fn build_object_diff(&self, local_change: &crate::types::Change) -> Result<ObjectDiffModel, ObjectsTreeError> {
        let mut diff_model = ObjectDiffModel::new();
        
        // Get the complete object list from the index state (excluding the local change)
        let complete_object_list = self.database.index().compute_complete_object_list()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Using complete object list with {} objects as baseline for change '{}'", 
              complete_object_list.len(), local_change.name);
        
        // Process the local change to build the diff
        self.process_change_for_diff(&mut diff_model, local_change)?;
        
        Ok(diff_model)
    }
    
    /// Process a single change and add its modifications to the diff model
    fn process_change_for_diff(&self, diff_model: &mut ObjectDiffModel, change: &crate::types::Change) -> Result<(), ObjectsTreeError> {
        // Process added objects
        for obj_info in &change.added_objects {
            let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
            diff_model.add_object_added(obj_name);
        }
        
        // Process deleted objects
        for obj_info in &change.deleted_objects {
            let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
            diff_model.add_object_deleted(obj_name);
        }
        
        // Process renamed objects
        for renamed in &change.renamed_objects {
            let from_name = obj_id_to_object_name(&renamed.from.name, Some(&renamed.from.name));
            let to_name = obj_id_to_object_name(&renamed.to.name, Some(&renamed.to.name));
            diff_model.add_object_renamed(from_name, to_name);
        }
        
        // Process modified objects with detailed comparison
        for obj_info in &change.modified_objects {
            let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
            diff_model.add_object_modified(obj_name.clone());
            
            // Get detailed object changes by comparing local vs baseline
            let object_change = compare_object_versions(&self.database, &obj_name, obj_info.version)?;
            diff_model.add_object_change(object_change);
        }
        
        Ok(())
    }
}

impl Operation for ChangeApproveOperation {
    fn name(&self) -> &'static str {
        "change/approve"
    }
    
    fn description(&self) -> &'static str {
        "Approves a local change by marking it as merged and removing it from the top of the index. Returns a ChangeDiff showing what was approved."
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
    
    fn execute(&self, args: Vec<String>) -> moor_var::Var {
        info!("Change approve operation received {} arguments", args.len());
        
        if args.is_empty() {
            error!("Change approve operation requires a change ID argument");
            return v_error(E_INVARG.msg("Change approve operation requires a change ID argument"));
        }
        
        let change_id = args[0].clone();
        let request = ChangeApproveRequest { change_id };

        match self.process_change_approve(request) {
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
