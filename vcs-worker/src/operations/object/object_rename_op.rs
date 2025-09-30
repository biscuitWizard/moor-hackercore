use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User};
use crate::providers::index::IndexProvider;
use crate::providers::refs::RefsProvider;
use crate::types::{ObjectRenameRequest, RenamedObject};

/// Object rename operation that renames an object from one name to another
#[derive(Clone)]
pub struct ObjectRenameOperation {
    database: DatabaseRef,
}

impl ObjectRenameOperation {
    /// Create a new object rename operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the object rename request
    fn process_object_rename(&self, request: ObjectRenameRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing object rename from '{}' to '{}'", request.from_name, request.to_name);
        
        // Validate that the source object exists
        let existing_sha256 = self.database.refs().get_ref(&request.from_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if existing_sha256.is_none() {
            error!("Cannot rename object '{}' - object does not exist", request.from_name);
            return Err(ObjectsTreeError::ObjectNotFound(format!("Object '{}' not found", request.from_name)));
        }
        
        // Validate that the target object name doesn't already exist
        let target_sha256 = self.database.refs().get_ref(&request.to_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if target_sha256.is_some() {
            error!("Cannot rename to '{}' - object already exists", request.to_name);
            return Err(ObjectsTreeError::InvalidOperation(format!("Object '{}' already exists", request.to_name)));
        }
        
        // Check that we're not using the same name
        if request.from_name == request.to_name {
            error!("Cannot rename object to the same name");
            return Err(ObjectsTreeError::InvalidOperation("Cannot rename object to the same name".to_string()));
        }
        
        // Get or create a local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // The index already manages the current change, so we don't need repository management
        
        // Get the current version of the source object
        let from_version = self.database.refs().get_ref(&request.from_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .and_then(|_| {
                // Get the latest version number for the source object
                self.database.refs().get_ref(&request.from_name, None)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))
                    .ok()
                    .flatten()
                    .map(|_| {
                        // We need to find the actual version number
                        // For now, we'll use version 1 as a placeholder - this should be improved
                        1u64
                    })
            }).unwrap_or(1);
        
        // Handle change tracking - remove from added/modified lists if present
        current_change.added_objects.retain(|obj| obj.name != request.from_name);
        current_change.modified_objects.retain(|obj| obj.name != request.from_name);
        
        // Add to renamed_objects list
        let renamed_object = RenamedObject {
            from: crate::types::ObjectInfo { name: request.from_name.clone(), version: from_version },
            to: crate::types::ObjectInfo { name: request.to_name.clone(), version: 1 }, // New object starts at version 1
        };
        
        // Remove any existing rename entry for this object
        current_change.renamed_objects.retain(|renamed| renamed.from.name != request.from_name);
        
        // Add the new rename entry
        current_change.renamed_objects.push(renamed_object);
        info!("Added rename '{}' -> '{}' to renamed_objects in change '{}'", request.from_name, request.to_name, current_change.name);
        
        // If the source object was added in this change, move it to the new name in added_objects
        if let Some(pos) = current_change.added_objects.iter().position(|obj| obj.name == request.from_name) {
            current_change.added_objects[pos] = crate::types::ObjectInfo { name: request.to_name.clone(), version: 1 };
            info!("Moved renamed object '{}' -> '{}' in added_objects", request.from_name, request.to_name);
        }
        
        // If the source object was modified in this change, move it to the new name in modified_objects
        if let Some(pos) = current_change.modified_objects.iter().position(|obj| obj.name == request.from_name) {
            current_change.modified_objects[pos] = crate::types::ObjectInfo { name: request.to_name.clone(), version: 1 };
            info!("Moved renamed object '{}' -> '{}' in modified_objects", request.from_name, request.to_name);
        }
        
        // Update the change in the database
        self.database.index().update_change(&current_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully queued rename '{}' -> '{}' for change '{}'", request.from_name, request.to_name, current_change.name);
        Ok(format!("Object '{}' rename to '{}' queued successfully in change '{}'", request.from_name, request.to_name, current_change.name))
    }
}

impl Operation for ObjectRenameOperation {
    fn name(&self) -> &'static str {
        "object/rename"
    }
    
    fn description(&self) -> &'static str {
        "Renames an object from one name to another within the current change"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/object/rename".to_string(),
                method: Method::POST,
                is_json: true, // Expects JSON body with from_name and to_name
            },
            OperationRoute {
                path: "/api/object/rename".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Object rename operation received {} arguments: {:?}", args.len(), args);
        
        if args.len() < 2 {
            error!("Object rename operation requires at least from_name and to_name");
            return moor_var::v_str("Error: From name and to name are required");
        }

        let from_name = args[0].clone();
        let to_name = args[1].clone();

        let request = ObjectRenameRequest {
            from_name,
            to_name,
        };

        match self.process_object_rename(request) {
            Ok(result) => {
                info!("Object rename operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Object rename operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
