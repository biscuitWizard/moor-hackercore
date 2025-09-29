use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info, warn};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::repository::RepositoryProvider;
use crate::providers::index::IndexProvider;
use crate::types::{ChangeAbandonRequest, ChangeStatus};

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
            info!("Attempting to abandon current change: {}", current_change_id);
            
            // Get the change to check its status
            let change_opt = self.database.index().get_change(&current_change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            
            match change_opt {
                Some(change) => {
                    if change.status == ChangeStatus::Merged {
                        error!("Cannot abandon change '{}' ({}) - it has already been merged", change.name, change.id);
                        return Err(ObjectsTreeError::SledError(sled::Error::Unsupported(
                            format!("Cannot abandon merged change '{}'", change.name)
                        )));
                    }
                    
                    // Remove from index if it's LOCAL
                    if change.status == ChangeStatus::Local {
                        self.database.index().remove_change(&current_change_id)
                            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                        info!("Removed change '{}' from index", change.name);
                    }
                    
                    // Clear the current change from repository
                    repository.current_change = None;
                    self.database.repository().set_repository(&repository)
                        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                    
                    info!("Successfully abandoned change '{}' ({})", change.name, change.id);
                    Ok(format!("Abandoned change '{}' ({})", change.name, change.id))
                }
                None => {
                    // Change ID exists in repository but change not found - clean up
                    warn!("Change '{}' referenced in repository but not found, clearing reference", current_change_id);
                    repository.current_change = None;
                    self.database.repository().set_repository(&repository)
                        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                    Ok(format!("Cleared orphaned change reference '{current_change_id}'"))
                }
            }
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
        "Abandons the current local change, removing it from index and clearing repository state. Cannot abandon merged changes."
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
