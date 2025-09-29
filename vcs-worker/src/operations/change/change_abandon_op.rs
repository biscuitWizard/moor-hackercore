use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::repository::RepositoryProvider;

/// Request structure for change abandon operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeAbandonRequest {
    // No fields needed - just abandons the current change
}

/// Change abandon operation that clears the current change
#[derive(Clone)]
pub struct ChangeAbandonOperation {
    database: DatabaseRef,
}

impl ChangeAbandonOperation {
    /// Create a new change abandon operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change abandon request
    fn process_change_abandon(&self, _request: ChangeAbandonRequest) -> Result<String, ObjectsTreeError> {
        // Get the current repository state
        let mut repository = self.database.repository().get_repository()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(current_change_id) = repository.current_change {
            info!("Abandoning current change: {}", current_change_id);
            
            // Clear the current change
            repository.current_change = None;
            self.database.repository().set_repository(&repository)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            
            info!("Successfully abandoned change '{}'", current_change_id);
            Ok(format!("Abandoned change '{current_change_id}'"))
        } else {
            info!("No current change to abandon");
            Ok("No current change to abandon".to_string())
        }
    }
}

impl Operation for ChangeAbandonOperation {
    fn name(&self) -> &'static str {
        "change/abandon"
    }
    
    fn description(&self) -> &'static str {
        "Abandons the current change, clearing it from the repository state"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/change/abandon".to_string(),
                method: Method::POST,
                is_json: false, // No body needed
            },
            OperationRoute {
                path: "/api/change/abandon".to_string(),
                method: Method::POST,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>) -> moor_var::Var {
        info!("Change abandon operation received {} arguments", args.len());
        
        let request = ChangeAbandonRequest {};

        match self.process_change_abandon(request) {
            Ok(result) => {
                info!("Change abandon operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Change abandon operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
