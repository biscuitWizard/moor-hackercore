use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, VcsObjectType};
use crate::providers::index::IndexProvider;
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;
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
        
        // Check that we're not using the same name
        if request.from_name == request.to_name {
            error!("Cannot rename object to the same name");
            return Err(ObjectsTreeError::InvalidOperation("Cannot rename object to the same name".to_string()));
        }
        
        // Get or create a local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Check if source is in added_objects or modified_objects (filter to MooObject types)
        // These cases are handled simply (just update the name) without complex validation
        let source_in_added = current_change.added_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooObject)
            .any(|obj| obj.name == request.from_name);
        let source_in_modified = current_change.modified_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooObject)
            .any(|obj| obj.name == request.from_name);
        
        // Check if we're renaming back to undo a previous rename (filter to MooObject types)
        let is_rename_back = current_change.renamed_objects.iter()
            .filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject)
            .any(|renamed| renamed.from.name == request.to_name && renamed.to.name == request.from_name);
        
        // Only do complex validation if NOT an added/modified object and NOT a rename-back
        if !source_in_added && !source_in_modified && !is_rename_back {
            // Check if the source object exists either in refs or in renamed_objects
            let source_exists_in_refs = self.database.refs().get_ref(VcsObjectType::MooObject, &request.from_name, None)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .is_some();
            
            let source_exists_in_renamed = current_change.renamed_objects.iter()
                .filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject)
                .any(|renamed| renamed.to.name == request.from_name);
            
            if !source_exists_in_refs && !source_exists_in_renamed {
                error!("Cannot rename object '{}' - object does not exist", request.from_name);
                return Err(ObjectsTreeError::ObjectNotFound(format!("Object '{}' not found", request.from_name)));
            }
            
            // Validate that the target object name doesn't already exist
            let target_exists_in_refs = self.database.refs().get_ref(VcsObjectType::MooObject, &request.to_name, None)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .is_some();
            
            let target_exists_in_renamed = current_change.renamed_objects.iter()
                .filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject)
                .any(|renamed| renamed.to.name == request.to_name);
            
            let target_exists_in_added = current_change.added_objects.iter()
                .filter(|obj| obj.object_type == VcsObjectType::MooObject)
                .any(|obj| obj.name == request.to_name);
            
            let target_exists_in_modified = current_change.modified_objects.iter()
                .filter(|obj| obj.object_type == VcsObjectType::MooObject)
                .any(|obj| obj.name == request.to_name);
            
            if target_exists_in_refs || target_exists_in_renamed || target_exists_in_added || target_exists_in_modified {
                error!("Cannot rename to '{}' - object already exists", request.to_name);
                return Err(ObjectsTreeError::InvalidOperation(format!("Object '{}' already exists", request.to_name)));
            }
        }
        
        // The index already manages the current change, so we don't need repository management
        
        // Get the current version of the source object
        let from_version = self.database.refs().get_ref(VcsObjectType::MooObject, &request.from_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .and_then(|_| {
                // Get the latest version number for the source object
                self.database.refs().get_ref(VcsObjectType::MooObject, &request.from_name, None)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))
                    .ok()
                    .flatten()
                    .map(|_| {
                        // We need to find the actual version number
                        // For now, we'll use version 1 as a placeholder - this should be improved
                        1u64
                    })
            }).unwrap_or(1);
        
        // Now handle the rename based on whether it's a rename-back or a normal rename
        if is_rename_back {
            // This is a rename back to the original name - remove the original rename operation
            info!("Detected rename back to original name '{}' -> '{}', removing rename operation", 
                  request.from_name, request.to_name);
            
            // Remove the original rename entry
            current_change.renamed_objects.retain(|renamed| 
                !(renamed.from.name == request.to_name && renamed.to.name == request.from_name));
            
            // Restore the object back to its original name in added/modified lists (filter to MooObject types)
            // (the rename operation had moved it to the new name, now we move it back)
            if let Some(pos) = current_change.added_objects.iter()
                .position(|obj| obj.object_type == VcsObjectType::MooObject && obj.name == request.from_name) {
                current_change.added_objects[pos] = crate::types::ObjectInfo { 
                    object_type: current_change.added_objects[pos].object_type,
                    name: request.to_name.clone(), 
                    version: current_change.added_objects[pos].version 
                };
                info!("Restored object back to '{}' in added_objects", request.to_name);
            }
            
            if let Some(pos) = current_change.modified_objects.iter()
                .position(|obj| obj.object_type == VcsObjectType::MooObject && obj.name == request.from_name) {
                current_change.modified_objects[pos] = crate::types::ObjectInfo { 
                    object_type: current_change.modified_objects[pos].object_type,
                    name: request.to_name.clone(), 
                    version: current_change.modified_objects[pos].version 
                };
                info!("Restored object back to '{}' in modified_objects", request.to_name);
            }
        } else {
            // Normal rename operation
            
            // Check if object is in added_objects or modified_objects (filter to MooObject types)
            // If so, just update the name there and DON'T add a rename entry
            let was_in_added = current_change.added_objects.iter()
                .filter(|obj| obj.object_type == VcsObjectType::MooObject)
                .any(|obj| obj.name == request.from_name);
            let was_in_modified = current_change.modified_objects.iter()
                .filter(|obj| obj.object_type == VcsObjectType::MooObject)
                .any(|obj| obj.name == request.from_name);
            
            if was_in_added {
                // Object was added in this change
                // Special case: if target is the "to.name" of a rename where "from.name" == source,
                // then we're renaming the added object back to the renamed object's name
                // This cancels everything out: delete the added object and delete the rename entry
                let cancels_rename = current_change.renamed_objects.iter()
                    .filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject)
                    .any(|renamed| renamed.from.name == request.from_name && renamed.to.name == request.to_name);
                
                if cancels_rename {
                    // This rename cancels out the previous rename + add
                    info!("Detected rename of added object back to renamed object's name, canceling both operations");
                    
                    // Remove the added object (filter to MooObject types)
                    let removed_obj = current_change.added_objects.iter()
                        .filter(|obj| obj.object_type == VcsObjectType::MooObject)
                        .find(|obj| obj.name == request.from_name)
                        .cloned();
                    current_change.added_objects.retain(|obj| !(obj.object_type == VcsObjectType::MooObject && obj.name == request.from_name));
                    info!("Removed added object '{}'", request.from_name);
                    
                    // Remove the rename entry
                    current_change.renamed_objects.retain(|renamed| 
                        !(renamed.from.name == request.from_name && renamed.to.name == request.to_name));
                    info!("Removed rename entry '{}' -> '{}'", request.from_name, request.to_name);
                    
                    // Clean up the SHA256 and ref for the removed added object
                    if let Some(removed) = removed_obj {
                        // Get the SHA256 for this object
                        if let Some(sha256) = self.database.refs().get_ref(VcsObjectType::MooObject, &request.from_name, Some(removed.version))
                            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
                            
                            // Check if this SHA256 is referenced elsewhere
                            let is_referenced = self.database.refs().is_sha256_referenced_excluding(&sha256, VcsObjectType::MooObject, &request.from_name, removed.version)
                                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                            
                            if !is_referenced {
                                // Delete the orphaned SHA256
                                info!("Deleting orphaned SHA256 '{}' for removed object '{}'", sha256, request.from_name);
                                self.database.objects().delete(&sha256)
                                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                            } else {
                                info!("SHA256 '{}' is still referenced, keeping it", sha256);
                            }
                        }
                        
                        // Delete the ref for this object
                        self.database.refs().delete_ref(VcsObjectType::MooObject, &request.from_name, removed.version)
                            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                        info!("Deleted ref for '{}' version {}", request.from_name, removed.version);
                    }
                } else {
                    // Normal case: just update the name in added_objects (filter to MooObject types)
                    if let Some(pos) = current_change.added_objects.iter()
                        .position(|obj| obj.object_type == VcsObjectType::MooObject && obj.name == request.from_name) {
                        current_change.added_objects[pos].name = request.to_name.clone();
                        info!("Updated added object name from '{}' to '{}'", request.from_name, request.to_name);
                    }
                }
            } else if was_in_modified {
                // Object was modified in this change - just update its name in modified_objects (filter to MooObject types)
                if let Some(pos) = current_change.modified_objects.iter()
                    .position(|obj| obj.object_type == VcsObjectType::MooObject && obj.name == request.from_name) {
                    current_change.modified_objects[pos].name = request.to_name.clone();
                    info!("Updated modified object name from '{}' to '{}'", request.from_name, request.to_name);
                }
                // Don't add to renamed_objects since it's already tracked as modified
            } else {
                // Object exists only in refs (committed history) - add to renamed_objects
                let renamed_object = RenamedObject {
                    from: crate::types::ObjectInfo { 
                        object_type: VcsObjectType::MooObject,
                        name: request.from_name.clone(), 
                        version: from_version 
                    },
                    to: crate::types::ObjectInfo { 
                        object_type: VcsObjectType::MooObject,
                        name: request.to_name.clone(), 
                        version: 1 
                    },
                };
                
                // Remove any existing rename entry for this object
                current_change.renamed_objects.retain(|renamed| renamed.from.name != request.from_name);
                
                // Add the new rename entry
                current_change.renamed_objects.push(renamed_object);
                info!("Added rename '{}' -> '{}' to renamed_objects in change '{}'", request.from_name, request.to_name, current_change.name);
            }
        }
        
        // Also handle renaming the corresponding MooMetaObject if it exists
        if let Some(meta_sha256) = self.database.refs().get_ref(VcsObjectType::MooMetaObject, &request.from_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            info!("Found meta for '{}', renaming it to '{}'", request.from_name, request.to_name);
            
            // Get the current version of the meta
            let meta_version = self.database.refs().get_current_version(VcsObjectType::MooMetaObject, &request.from_name)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .unwrap_or(1);
            
            // Create a new ref for the meta with the new name
            self.database.refs().update_ref(VcsObjectType::MooMetaObject, &request.to_name, 1, &meta_sha256)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            
            // Delete the old meta ref
            self.database.refs().delete_ref(VcsObjectType::MooMetaObject, &request.from_name, meta_version)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            
            // Update meta in the change tracking lists
            // Update added_objects
            for obj in current_change.added_objects.iter_mut() {
                if obj.object_type == VcsObjectType::MooMetaObject && obj.name == request.from_name {
                    obj.name = request.to_name.clone();
                    obj.version = 1;
                }
            }
            
            // Update modified_objects
            for obj in current_change.modified_objects.iter_mut() {
                if obj.object_type == VcsObjectType::MooMetaObject && obj.name == request.from_name {
                    obj.name = request.to_name.clone();
                    obj.version = 1;
                }
            }
            
            // Update renamed_objects
            for renamed in current_change.renamed_objects.iter_mut() {
                if renamed.from.object_type == VcsObjectType::MooMetaObject && renamed.from.name == request.from_name {
                    renamed.from.name = request.to_name.clone();
                }
                if renamed.to.object_type == VcsObjectType::MooMetaObject && renamed.to.name == request.from_name {
                    renamed.to.name = request.to_name.clone();
                }
            }
            
            info!("Successfully renamed meta from '{}' to '{}'", request.from_name, request.to_name);
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
