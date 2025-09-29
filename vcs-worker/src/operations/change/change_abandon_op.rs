use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::{ChangeAbandonRequest, ChangeStatus};
use crate::object_diff::{ObjectDiffModel, ObjectChange, obj_id_to_object_name};
use std::collections::HashMap;

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
        let changes = self.database.index().list_changes()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(change) = changes.first() {
            info!("Attempting to abandon current change: {}", change.id);
            
            if change.status == ChangeStatus::Merged {
                error!("Cannot abandon change '{}' ({}) - it has already been merged", change.name, change.id);
                return Err(ObjectsTreeError::SerializationError(
                    format!("Cannot abandon merged change '{}'", change.name)
                ));
            }
            
            // Create a delta model showing what needs to be undone
            let mut undo_delta = ObjectDiffModel::new();
            
            // Get object name mappings for better display names
            let object_names = self.get_object_names(change);
            
            // Process added objects - to undo, we need to delete them
            for added_obj in &change.added_objects {
                let object_name = obj_id_to_object_name(&added_obj.name, object_names.get(&added_obj.name).map(|s| s.as_str()));
                undo_delta.add_object_deleted(object_name);
            }
            
            // Process deleted objects - to undo, we need to add them back
            for deleted_obj in &change.deleted_objects {
                let object_name = obj_id_to_object_name(&deleted_obj.name, object_names.get(&deleted_obj.name).map(|s| s.as_str()));
                undo_delta.add_object_added(object_name);
            }
            
            // Process renamed objects - to undo, we need to rename them back
            for renamed in &change.renamed_objects {
                let from_name = obj_id_to_object_name(&renamed.from.name, object_names.get(&renamed.from.name).map(|s| s.as_str()));
                let to_name = obj_id_to_object_name(&renamed.to.name, object_names.get(&renamed.to.name).map(|s| s.as_str()));
                undo_delta.add_object_renamed(to_name, from_name);
            }
            
            // Process modified objects - to undo, we need to mark them as modified
            // and create basic ObjectChange entries
            for modified_obj in &change.modified_objects {
                let object_name = obj_id_to_object_name(&modified_obj.name, object_names.get(&modified_obj.name).map(|s| s.as_str()));
                undo_delta.add_object_modified(object_name.clone());
                
                // Create a basic ObjectChange for modified objects
                // In a real implementation, you'd want to track what specifically changed
                let mut object_change = ObjectChange::new(object_name);
                object_change.props_modified.insert("content".to_string());
                undo_delta.add_object_change(object_change);
            }
            
            // Remove from index if it's LOCAL
            if change.status == ChangeStatus::Local {
                self.database.index().remove_change(&change.id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                info!("Removed change '{}' from index", change.name);
            }
            
            info!("Successfully abandoned change '{}' ({}), created undo delta", change.name, change.id);
            Ok(undo_delta)
        } else {
            info!("No current change to abandon");
            // Return empty delta model when no change to abandon
            Ok(ObjectDiffModel::new())
        }
    }

    /// Get object names for the change objects to improve display names
    fn get_object_names(&self, change: &crate::types::Change) -> HashMap<String, String> {
        let mut object_names = HashMap::new();
        
        // Try to get object names from workspace provider
        // This is a simplified implementation - in practice you'd want to
        // query the actual object names from the MOO database
        for obj_info in change.added_objects.iter()
            .chain(change.modified_objects.iter())
            .chain(change.deleted_objects.iter()) {
            
            // For now, we'll just use the object name as the name
            // In a real implementation, you'd query the actual object names
            object_names.insert(obj_info.name.clone(), obj_info.name.clone());
        }
        
        for renamed in &change.renamed_objects {
            object_names.insert(renamed.from.name.clone(), renamed.from.name.clone());
            object_names.insert(renamed.to.name.clone(), renamed.to.name.clone());
        }
        
        object_names
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
    
    fn execute(&self, args: Vec<String>) -> moor_var::Var {
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
