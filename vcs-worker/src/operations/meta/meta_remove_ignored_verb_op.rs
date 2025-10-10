use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, VcsObjectType, MooMetaObject, MetaRemoveIgnoredVerbRequest};
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::index::IndexProvider;

/// Meta operation that removes an ignored verb from an object's meta
#[derive(Clone)]
pub struct MetaRemoveIgnoredVerbOperation {
    database: DatabaseRef,
}

impl MetaRemoveIgnoredVerbOperation {
    /// Create a new meta remove ignored verb operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the meta remove ignored verb request
    fn process_meta_remove_ignored_verb(&self, request: MetaRemoveIgnoredVerbRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing meta remove ignored verb for '{}', verb '{}'", request.object_name, request.verb_name);
        
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
        
        // Remove the verb from ignored_verbs
        let was_removed = meta.ignored_verbs.remove(&request.verb_name);
        
        if !was_removed {
            info!("Verb '{}' was not in ignored list for object '{}'", request.verb_name, request.object_name);
            return Ok(format!("Verb '{}' was not in ignored list for object '{}'", request.verb_name, request.object_name));
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
        
        info!("Successfully removed verb '{}' from ignored list for object '{}'", request.verb_name, request.object_name);
        Ok(format!("Verb '{}' removed from ignored list for object '{}'", request.verb_name, request.object_name))
    }
}

impl Operation for MetaRemoveIgnoredVerbOperation {
    fn name(&self) -> &'static str {
        "meta/remove_ignored_verb"
    }
    
    fn description(&self) -> &'static str {
        "Removes a verb from the ignored verbs list in the object's meta"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/meta/remove_ignored_verb".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/meta/remove_ignored_verb".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Meta remove ignored verb operation received {} arguments: {:?}", args.len(), args);
        
        if args.len() < 2 {
            error!("Meta remove ignored verb operation requires object name and verb name");
            return moor_var::v_str("Error: Object name and verb name are required");
        }

        let object_name = args[0].clone();
        let verb_name = args[1].clone();

        let request = MetaRemoveIgnoredVerbRequest {
            object_name,
            verb_name,
        };

        match self.process_meta_remove_ignored_verb(request) {
            Ok(result) => {
                info!("Meta remove ignored verb operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta remove ignored verb operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
