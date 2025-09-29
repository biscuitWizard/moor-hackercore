use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::ChangeStatus;

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
        // Get the current change from the top of the index
        let changes = self.database.index().list_changes()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(current_change) = changes.first() {
            info!("Getting status for current change: {}", current_change.id);
            
            // Convert ObjectInfo to Var list of maps for added objects
            let added_vars: Vec<moor_var::Var> = current_change.added_objects
                .iter()
                .map(|obj_info| {
                    moor_var::v_map(&[
                        (moor_var::v_str("name"), moor_var::v_str(&obj_info.name)),
                        (moor_var::v_str("version"), moor_var::v_int(obj_info.version as i64)),
                    ])
                })
                .collect();
            
            // Convert ObjectInfo to Var list of maps for modified objects
            let modified_vars: Vec<moor_var::Var> = current_change.modified_objects
                .iter()
                .map(|obj_info| {
                    moor_var::v_map(&[
                        (moor_var::v_str("name"), moor_var::v_str(&obj_info.name)),
                        (moor_var::v_str("version"), moor_var::v_int(obj_info.version as i64)),
                    ])
                })
                .collect();
            
            // Convert ObjectInfo to Var list of maps for deleted objects
            let deleted_vars: Vec<moor_var::Var> = current_change.deleted_objects
                .iter()
                .map(|obj_info| {
                    moor_var::v_map(&[
                        (moor_var::v_str("name"), moor_var::v_str(&obj_info.name)),
                        (moor_var::v_str("version"), moor_var::v_int(obj_info.version as i64)),
                    ])
                })
                .collect();
            
            // Convert renamed objects to Var list of maps
            let renamed_vars: Vec<moor_var::Var> = current_change.renamed_objects
                .iter()
                .map(|renamed| {
                    moor_var::v_map(&[
                        (moor_var::v_str("from"), moor_var::v_map(&[
                            (moor_var::v_str("name"), moor_var::v_str(&renamed.from.name)),
                            (moor_var::v_str("version"), moor_var::v_int(renamed.from.version as i64)),
                        ])),
                        (moor_var::v_str("to"), moor_var::v_map(&[
                            (moor_var::v_str("name"), moor_var::v_str(&renamed.to.name)),
                            (moor_var::v_str("version"), moor_var::v_int(renamed.to.version as i64)),
                        ])),
                    ])
                })
                .collect();
            
            // Create the main status map
            let status_map = moor_var::v_map(&[
                (moor_var::v_str("change_id"), moor_var::v_str(&current_change.id)),
                (moor_var::v_str("change_name"), moor_var::v_str(&current_change.name)),
                (moor_var::v_str("added"), moor_var::v_list(&added_vars)),
                (moor_var::v_str("modified"), moor_var::v_list(&modified_vars)),
                (moor_var::v_str("deleted"), moor_var::v_list(&deleted_vars)),
                (moor_var::v_str("renamed"), moor_var::v_list(&renamed_vars)),
            ]);
            
            info!("Successfully retrieved change status for '{}'", current_change.name);
            Ok(status_map)
        } else {
            info!("No current change to show status for");
            // Return a map indicating no active change
            let empty_vec = Vec::new();
            let no_change_map = moor_var::v_map(&[
                (moor_var::v_str("change_id"), moor_var::v_none()),
                (moor_var::v_str("change_name"), moor_var::v_none()),
                (moor_var::v_str("message"), moor_var::v_str("No active change")),
                (moor_var::v_str("added"), moor_var::v_list(&empty_vec)),
                (moor_var::v_str("modified"), moor_var::v_list(&empty_vec)),
                (moor_var::v_str("deleted"), moor_var::v_list(&empty_vec)),
                (moor_var::v_str("renamed"), moor_var::v_list(&empty_vec)),
            ]);
            Ok(no_change_map)
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