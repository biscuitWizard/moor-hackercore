use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::User;
use crate::providers::index::IndexProvider;
use crate::types::ChangeStatus;
use crate::object_diff::build_object_diff_from_change;
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
            let diff_model = build_object_diff_from_change(&self.database, &current_change)?;
            
            // Convert to MOO Var and return
            let status_map = diff_model.to_moo_var();
            
            info!("Successfully retrieved change status for '{}'", current_change.name);
            Ok(status_map)
        } else {
            info!("No top change found, returning error");
            return Ok(v_error(E_INVARG.msg("No change on top of index - nothing to do")));
        }
    }
    
    
}

impl Operation for ChangeStatusOperation {
    fn name(&self) -> &'static str {
        "change/status"
    }
    
    fn description(&self) -> &'static str {
        "Lists all objects that have been modified in the current change (added, modified, deleted, renamed)"
    }
    
    fn philosophy(&self) -> &'static str {
        "Provides a summary of all pending changes in your current local changelist. This is your primary \
        tool for reviewing what you've done before submitting - it shows which objects have been added, \
        modified, deleted, or renamed. The operation returns an ObjectDiffModel that categorizes all your \
        changes, making it easy to verify your work is correct before submission. Use this regularly during \
        development to track your progress and ensure you haven't accidentally modified objects you didn't \
        intend to change."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
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
player:tell("Deleted: ", length(diff["deleted_objects"]), " objects");"#.to_string(),
                http_curl: Some(r#"curl -X GET http://localhost:8081/api/change/status"#.to_string()),
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/change/status".to_string(),
                method: Method::GET,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, _args: Vec<String>, _user: &User) -> moor_var::Var {
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