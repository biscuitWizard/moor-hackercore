use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::User;
use crate::providers::index::IndexProvider;
use crate::types::{ChangeAbandonRequest, ChangeStatus};
use crate::object_diff::{ObjectDiffModel, build_abandon_diff_from_change};

/// Change abandon operation that abandons the top change in the index
#[derive(Clone)]
pub struct ChangeAbandonOperation {
    database: DatabaseRef,
}

impl ChangeAbandonOperation {
    /// Create a new change abandon operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change abandon request and return an ObjectDiffModel showing what needs to be undone
    fn process_change_abandon(&self, _request: ChangeAbandonRequest) -> Result<ObjectDiffModel, ObjectsTreeError> {
        // Get the current change from the top of the index
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(change_id) = top_change_id {
            let change = self.database.index().get_change(&change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Top change '{}' not found", change_id)))?;
            
            info!("Attempting to abandon current change: {}", change.id);
            
            if change.status == ChangeStatus::Merged {
                error!("Cannot abandon change '{}' ({}) - it has already been merged", change.name, change.id);
                return Err(ObjectsTreeError::SerializationError(
                    format!("Cannot abandon merged change '{}'", change.name)
                ));
            }
            
            // Build the abandon diff using shared logic
            let undo_delta = build_abandon_diff_from_change(&self.database, &change)?;
            
            // Remove from working index if it's LOCAL
            if change.status == ChangeStatus::Local {
                self.database.index().remove_from_index(&change.id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                info!("Removed change '{}' from working index", change.name);
                
                // Delete the change from history storage (abandoned changes are not kept)
                self.database.index().delete_change(&change.id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                info!("Deleted abandoned change '{}' from history storage", change.name);
            }
            
            info!("Successfully abandoned change '{}' ({}), created undo delta", change.name, change.id);
            Ok(undo_delta)
        } else {
            info!("No current change to abandon");
            // Return empty delta model when no change to abandon
            Ok(ObjectDiffModel::new())
        }
    }

}

impl Operation for ChangeAbandonOperation {
    fn name(&self) -> &'static str {
        "change/abandon"
    }
    
    fn description(&self) -> &'static str {
        "Abandons the top local change in the index, removing it from index. Returns an ObjectDiffModel showing what changes need to be undone. Cannot abandon merged changes."
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
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Change abandon operation received {} arguments", args.len());
        
        let request = ChangeAbandonRequest {};

        match self.process_change_abandon(request) {
            Ok(delta_model) => {
                info!("Change abandon operation completed successfully, returning undo delta");
                // Return the ObjectDiffModel as a MOO variable showing what needs to be undone
                delta_model.to_moo_var()
            }
            Err(e) => {
                error!("Change abandon operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
