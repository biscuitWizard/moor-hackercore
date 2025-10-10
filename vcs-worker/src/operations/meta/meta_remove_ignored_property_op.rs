use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, VcsObjectType, MooMetaObject, MetaRemoveIgnoredPropertyRequest};
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::index::IndexProvider;

/// Meta operation that removes an ignored property from an object's meta
#[derive(Clone)]
pub struct MetaRemoveIgnoredPropertyOperation {
    database: DatabaseRef,
}

impl MetaRemoveIgnoredPropertyOperation {
    /// Create a new meta remove ignored property operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the meta remove ignored property request
    fn process_meta_remove_ignored_property(&self, request: MetaRemoveIgnoredPropertyRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing meta remove ignored property for '{}', property '{}'", request.object_name, request.property_name);
        
        // Check that the MOO object exists
        self.database.refs().get_ref(VcsObjectType::MooObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::ObjectNotFound(format!("Object '{}' not found", request.object_name)))?;
        
        // Get or create the local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Check if meta existed BEFORE we modify it (for tracking purposes)
        let meta_existed_before = self.database.refs().get_ref(VcsObjectType::MooMetaObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .is_some();
        
        // Get existing meta or create default
        let mut meta = if let Some(meta_sha256) = self.database.refs().get_ref(VcsObjectType::MooMetaObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            // Meta exists, load it
            let yaml = self.database.objects().get(&meta_sha256)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .ok_or_else(|| ObjectsTreeError::SerializationError("Meta SHA256 exists but data not found".to_string()))?;
            self.database.objects().parse_meta_dump(&yaml)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        } else {
            // No meta exists yet, create default
            MooMetaObject::default()
        };
        
        // Remove the property from ignored_properties
        let was_removed = meta.ignored_properties.remove(&request.property_name);
        
        if !was_removed {
            info!("Property '{}' was not in ignored list for object '{}'", request.property_name, request.object_name);
            return Ok(format!("Property '{}' was not in ignored list for object '{}'", request.property_name, request.object_name));
        }
        
        // Generate YAML dump and SHA256
        let yaml_dump = self.database.objects().generate_meta_dump(&meta)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let sha256_key = self.database.objects().generate_sha256_hash(&yaml_dump);
        
        // Store the YAML content
        self.database.objects().store(&sha256_key, &yaml_dump)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Check if meta already exists in this change to determine versioning
        let is_already_in_change = current_change.added_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooMetaObject)
            .any(|obj| obj.name == request.object_name) ||
            current_change.modified_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooMetaObject)
            .any(|obj| obj.name == request.object_name);
        
        let version;
        if is_already_in_change {
            // Reuse the current version
            version = self.database.refs().get_current_version(VcsObjectType::MooMetaObject, &request.object_name)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .unwrap_or(1);
        } else {
            // Increment version
            version = self.database.refs().get_next_version(VcsObjectType::MooMetaObject, &request.object_name)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        }
        
        // Update the ref
        self.database.refs().update_ref(VcsObjectType::MooMetaObject, &request.object_name, version, &sha256_key)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Track in change if not already tracked
        if !is_already_in_change {
            let obj_info = crate::types::ObjectInfo {
                object_type: VcsObjectType::MooMetaObject,
                name: request.object_name.clone(),
                version,
            };
            
            if !meta_existed_before {
                current_change.added_objects.push(obj_info);
            } else {
                current_change.modified_objects.push(obj_info);
            }
        }
        
        // Always update the change
        self.database.index().update_change(&current_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully removed property '{}' from ignored list for object '{}'", request.property_name, request.object_name);
        Ok(format!("Property '{}' removed from ignored list for object '{}'", request.property_name, request.object_name))
    }
}

impl Operation for MetaRemoveIgnoredPropertyOperation {
    fn name(&self) -> &'static str {
        "meta/remove_ignored_property"
    }
    
    fn description(&self) -> &'static str {
        "Removes a property from the ignored properties list in the object's meta"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/meta/remove_ignored_property".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/meta/remove_ignored_property".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Meta remove ignored property operation received {} arguments: {:?}", args.len(), args);
        
        if args.len() < 2 {
            error!("Meta remove ignored property operation requires object name and property name");
            return moor_var::v_str("Error: Object name and property name are required");
        }

        let object_name = args[0].clone();
        let property_name = args[1].clone();

        let request = MetaRemoveIgnoredPropertyRequest {
            object_name,
            property_name,
        };

        match self.process_meta_remove_ignored_property(request) {
            Ok(result) => {
                info!("Meta remove ignored property operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta remove ignored property operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
