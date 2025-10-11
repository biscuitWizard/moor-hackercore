use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, VcsObjectType};
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
        
        // Validate that the source object exists (using resolve to handle renames)
        let existing_sha256 = self.database.index().resolve_object_current_state(
            &request.object_name,
            |obj_name| self.database.refs().get_ref(VcsObjectType::MooObject, obj_name, None).map_err(|e| crate::providers::ProviderError::SerializationError(e.to_string()))
        ).map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if existing_sha256.is_none() {
            error!("Cannot delete object '{}' - object does not exist", request.object_name);
            return Err(ObjectsTreeError::ObjectNotFound(format!("Object '{}' not found", request.object_name)));
        }
        
        // Get or create a local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Check if this object is the result of a rename in the current change
        // If so, we need to use the ORIGINAL name for deleted_objects
        let original_name = current_change.renamed_objects.iter()
            .filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject)
            .find(|r| r.to.name == request.object_name)
            .map(|r| r.from.name.clone())
            .unwrap_or_else(|| request.object_name.clone());
        
        info!("Deleting object '{}' (original name: '{}')", request.object_name, original_name);
        
        // Get the current version of the object being deleted
        let object_version = self.database.refs().get_ref(VcsObjectType::MooObject, &original_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .and_then(|_| {
                // For now, we'll use version 1 as a placeholder - this should be improved
                Some(1u64)
            }).unwrap_or(1);
        
        // Handle change tracking - remove from added/modified lists if present (filter to MooObject types)
        let was_in_added = current_change.added_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooObject)
            .any(|obj| obj.name == request.object_name);
        let was_in_modified = current_change.modified_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooObject)
            .any(|obj| obj.name == request.object_name);
        
        current_change.added_objects.retain(|obj| !(obj.object_type == VcsObjectType::MooObject && obj.name == request.object_name));
        current_change.modified_objects.retain(|obj| !(obj.object_type == VcsObjectType::MooObject && obj.name == request.object_name));
        
        if was_in_added {
            info!("Removed object '{}' from added_objects (deleted before commit, not adding to deleted_objects)", request.object_name);
        }
        
        if was_in_modified {
            info!("Removed object '{}' from modified_objects (now deleting instead)", request.object_name);
        }
        
        // Only add to deleted_objects if it wasn't just added in this change
        // If it was in added_objects, we just remove it completely
        if !was_in_added {
            // Add to deleted_objects list if not already present (filter to MooObject types)
            // Use original_name (handles renamed objects correctly)
            let obj_info = crate::types::ObjectInfo { 
                object_type: VcsObjectType::MooObject,
                name: original_name.clone(), 
                version: object_version 
            };
            if !current_change.deleted_objects.iter()
                .filter(|obj| obj.object_type == VcsObjectType::MooObject)
                .any(|obj| obj.name == original_name) {
                current_change.deleted_objects.push(obj_info);
                info!("Added object '{}' to deleted_objects in change '{}'", original_name, current_change.name);
            }
        }
        
        // Remove any rename entries for this object since it's being deleted (filter to MooObject types)
        current_change.renamed_objects.retain(|renamed| 
            !(renamed.from.object_type == VcsObjectType::MooObject && 
              renamed.to.object_type == VcsObjectType::MooObject && 
              (renamed.from.name == request.object_name || renamed.to.name == request.object_name)));
        
        // Also handle deleting the corresponding MooMetaObject if it exists
        if let Some(_meta_sha256) = self.database.refs().get_ref(VcsObjectType::MooMetaObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            info!("Found meta for '{}', deleting it as well", request.object_name);
            
            // Get the current version of the meta
            let meta_version = self.database.refs().get_current_version(VcsObjectType::MooMetaObject, &request.object_name)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .unwrap_or(1);
            
            // Remove meta from added/modified lists if present
            current_change.added_objects.retain(|obj| !(obj.object_type == VcsObjectType::MooMetaObject && obj.name == request.object_name));
            current_change.modified_objects.retain(|obj| !(obj.object_type == VcsObjectType::MooMetaObject && obj.name == request.object_name));
            
            // Always add meta to deleted_objects when deleting an object
            // Note: Meta objects are tracked in deleted_objects but filtered out of user-facing diffs
            // This ensures proper cleanup on commit while keeping diffs focused on MOO objects only
            let meta_obj_info = crate::types::ObjectInfo {
                object_type: VcsObjectType::MooMetaObject,
                name: request.object_name.clone(),
                version: meta_version,
            };
            if !current_change.deleted_objects.iter()
                .filter(|obj| obj.object_type == VcsObjectType::MooMetaObject)
                .any(|obj| obj.name == request.object_name) {
                current_change.deleted_objects.push(meta_obj_info);
                info!("Added meta '{}' to deleted_objects in change '{}'", request.object_name, current_change.name);
            }
            
            // Remove any rename entries for the meta
            current_change.renamed_objects.retain(|renamed| 
                !(renamed.from.object_type == VcsObjectType::MooMetaObject && 
                  renamed.to.object_type == VcsObjectType::MooMetaObject && 
                  (renamed.from.name == request.object_name || renamed.to.name == request.object_name)));
            
            info!("Successfully queued deletion of meta for '{}'", request.object_name);
        }
        
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
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
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
