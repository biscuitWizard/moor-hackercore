use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, VcsObjectType, MetaClearIgnoredVerbsRequest};
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::index::IndexProvider;

/// Meta operation that clears all ignored verbs from an object's meta
#[derive(Clone)]
pub struct MetaClearIgnoredVerbsOperation {
    database: DatabaseRef,
}

impl MetaClearIgnoredVerbsOperation {
    /// Create a new meta clear ignored verbs operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the meta clear ignored verbs request
    fn process_meta_clear_ignored_verbs(&self, request: MetaClearIgnoredVerbsRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing meta clear ignored verbs for '{}'", request.object_name);
        
        // Check that the MOO object exists
        self.database.refs().get_ref(VcsObjectType::MooObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::ObjectNotFound(format!("Object '{}' not found", request.object_name)))?;
        
        // Get or create the local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Check if meta exists - if not, nothing to clear
        let meta_sha256 = match self.database.refs().get_ref(VcsObjectType::MooMetaObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            Some(sha256) => sha256,
            None => {
                info!("No meta exists for object '{}', nothing to clear", request.object_name);
                return Ok(format!("No meta exists for object '{}', cleared 0 ignored verbs", request.object_name));
            }
        };
        
        // Load the existing meta
        let yaml = self.database.objects().get(&meta_sha256)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError("Meta SHA256 exists but data not found".to_string()))?;
        let mut meta = self.database.objects().parse_meta_dump(&yaml)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Clear all ignored verbs
        let count = meta.ignored_verbs.len();
        meta.ignored_verbs.clear();
        
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
        // Since we return early if meta doesn't exist, we know it existed before
        if !is_already_in_change {
            let obj_info = crate::types::ObjectInfo {
                object_type: VcsObjectType::MooMetaObject,
                name: request.object_name.clone(),
                version,
            };
            
            current_change.modified_objects.push(obj_info);
        }
        
        // Always update the change
        self.database.index().update_change(&current_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully cleared {} ignored verbs for object '{}'", count, request.object_name);
        Ok(format!("Cleared {} ignored verbs for object '{}'", count, request.object_name))
    }
}

impl Operation for MetaClearIgnoredVerbsOperation {
    fn name(&self) -> &'static str {
        "meta/clear_ignored_verbs"
    }
    
    fn description(&self) -> &'static str {
        "Clears all ignored verbs from the object's meta"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/meta/clear_ignored_verbs".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/meta/clear_ignored_verbs".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Meta clear ignored verbs operation received {} arguments: {:?}", args.len(), args);
        
        if args.is_empty() {
            error!("Meta clear ignored verbs operation requires object name");
            return moor_var::v_str("Error: Object name is required");
        }

        let object_name = args[0].clone();

        let request = MetaClearIgnoredVerbsRequest {
            object_name,
        };

        match self.process_meta_clear_ignored_verbs(request) {
            Ok(result) => {
                info!("Meta clear ignored verbs operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta clear ignored verbs operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
