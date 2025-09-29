use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::ObjectsTreeError;
use crate::providers::index::IndexProvider;
use crate::providers::refs::RefsProvider;
use crate::types::ObjectDeleteRequest;

/// Object delete operation that marks an object for deletion within the current change
#[derive(Clone)]
pub struct ObjectDeleteOperation {
    database: DatabaseRef,
}

impl ObjectDeleteOperation {
    /// Create a new object delete operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the object delete request
    fn process_object_delete(&self, request: ObjectDeleteRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing object delete for '{}'", request.object_name);
        
        // Validate that the source object exists
        let existing_sha256 = self.database.refs().get_ref(&request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if existing_sha256.is_none() {
            error!("Cannot delete object '{}' - object does not exist", request.object_name);
            return Err(ObjectsTreeError::ObjectNotFound(format!("Object '{}' not found", request.object_name)));
        }
        
        // Get or create a local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // The index already manages the current change, so we don't need repository management
        
        // Handle change tracking - remove from added/modified lists if present
        let was_in_added = current_change.added_objects.iter().any(|name| name == &request.object_name);
        let was_in_modified = current_change.modified_objects.iter().any(|name| name == &request.object_name);
        
        current_change.added_objects.retain(|name| name != &request.object_name);
        current_change.modified_objects.retain(|name| name != &request.object_name);
        
        if was_in_added {
            info!("Removed object '{}' from added_objects (now deleting instead)", request.object_name);
        }
        
        if was_in_modified {
            info!("Removed object '{}' from modified_objects (now deleting instead)", request.object_name);
        }
        
        // Add to deleted_objects list if not already present
        if !current_change.deleted_objects.contains(&request.object_name) {
            current_change.deleted_objects.push(request.object_name.clone());
            info!("Added object '{}' to deleted_objects in change '{}'", request.object_name, current_change.name);
        }
        
        // Remove any rename entries for this object since it's being deleted
        current_change.renamed_objects.retain(|renamed| renamed.from != request.object_name && renamed.to != request.object_name);
        
        // Update the change in the database
        self.database.index().update_change(&current_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully queued deletion of '{}' for change '{}'", request.object_name, current_change.name);
        Ok(format!("Object '{}' deletion queued successfully in change '{}'", request.object_name, current_change.name))
    }
}

impl Operation for ObjectDeleteOperation {
    fn name(&self) -> &'static str {
        "object/delete"
    }
    
    fn description(&self) -> &'static str {
        "Marks an object for deletion within the current change"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/object/delete".to_string(),
                method: Method::POST,
                is_json: true, // Expects JSON body with object_name
            },
            OperationRoute {
                path: "/api/object/delete".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>) -> moor_var::Var {
        info!("Object delete operation received {} arguments: {:?}", args.len(), args);
        
        if args.len() < 1 {
            error!("Object delete operation requires object name");
            return moor_var::v_str("Error: Object name is required");
        }

        let object_name = args[0].clone();

        let request = ObjectDeleteRequest {
            object_name,
        };

        match self.process_object_delete(request) {
            Ok(result) => {
                info!("Object delete operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Object delete operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
