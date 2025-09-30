use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::ChangeStatus;
use crate::object_diff::{ObjectDiffModel, obj_id_to_object_name, compare_object_versions};
use moor_var::{v_error, E_INVARG};

/// Request structure for change status operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatusRequest {
    // No fields needed - lists status of current change
}

/// Response structure for change status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatusResponse {
    pub change_id: Option<String>,
    pub change_name: Option<String>,
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
    fn process_change_status(&self, _request: ChangeStatusRequest) -> Result<moor_var::Var, ObjectsTreeError> {
        // Get the top change from the index
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(change_id) = top_change_id {
            // Get the actual change object
            let current_change = self.database.index().get_change(&change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Change '{}' not found", change_id)))?;
            
            // Check if the top change is local status
            if current_change.status != ChangeStatus::Local {
                info!("Top change '{}' is not local status (status: {:?}), returning error", 
                      current_change.name, current_change.status);
                return Ok(v_error(E_INVARG.msg("No local change on top of index - nothing to do")));
            }
            
            info!("Getting status for local change: {}", current_change.id);
            
            // Build the ObjectDiffModel by comparing local change against the compiled state below it
            let diff_model = self.build_object_diff(&current_change)?;
            
            // Convert to MOO Var and return
            let status_map = diff_model.to_moo_var();
            
            info!("Successfully retrieved change status for '{}'", current_change.name);
            Ok(status_map)
        } else {
            info!("No top change found, returning error");
            return Ok(v_error(E_INVARG.msg("No change on top of index - nothing to do")));
        }
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

impl Operation for ChangeStatusOperation {
    fn name(&self) -> &'static str {
        "change/status"
    }
    
    fn description(&self) -> &'static str {
        "Lists all objects that have been modified in the current change (added, modified, deleted, renamed)"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/change/status".to_string(),
                method: Method::GET,
                is_json: false, // No body needed
            },
            OperationRoute {
                path: "/api/change/status".to_string(),
                method: Method::GET,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, _args: Vec<String>) -> moor_var::Var {
        info!("Change status operation executed");
        
        let request = ChangeStatusRequest {};

        match self.process_change_status(request) {
            Ok(result_var) => {
                info!("Change status operation completed successfully");
                result_var
            }
            Err(e) => {
                error!("Change status operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}