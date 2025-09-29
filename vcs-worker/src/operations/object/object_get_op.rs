use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::changes::ChangesProvider;
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::head::HeadProvider;
use crate::providers::repository::RepositoryProvider;

/// Request structure for object get operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectGetRequest {
    pub object_name: String,
}

/// Object get operation that retrieves a stored object definition by name
#[derive(Clone)]
pub struct ObjectGetOperation {
    database: DatabaseRef,
}

impl ObjectGetOperation {
    /// Create a new object get operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the object get request
    fn process_object_get(&self, request: ObjectGetRequest) -> Result<String, ObjectsTreeError> {
        info!("Retrieving object '{}'", request.object_name);
        
        // First check current change for overrides
        let repository = self.database.repository().get_repository()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        if let Some(change_id) = repository.current_change {
            if let Ok(Some(current_change)) = self.database.changes().get_change(&change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string())) {
                // Check if deleted
                if current_change.deleted_objects.contains(&request.object_name) {
                    error!("Object '{}' has been deleted in current change", request.object_name);
                    return Err(ObjectsTreeError::SledError(sled::Error::Unsupported(
                        format!("Object '{}' has been deleted", request.object_name)
                    )));
                }
                
                // Check for renamed object
                if let Some(renamed) = current_change.renamed_objects.iter()
                    .find(|r| r.from == request.object_name) {
                    info!("Object '{}' has been renamed to '{}' in current change", request.object_name, renamed.to);
                    return self.process_object_get(ObjectGetRequest { object_name: renamed.to.clone() });
                }
                
                // Check for version override
                if let Some(version_override) = current_change.version_overrides.iter()
                    .find(|vo| vo.object_name == request.object_name) {
                    info!("Found version override for object '{}' in current change", request.object_name);
                    return self.database.objects().get(&version_override.sha256_key)
                        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))
                        .and_then(|opt| opt.ok_or_else(|| ObjectsTreeError::SerializationError(
                            format!("Object '{}' content not found", request.object_name)
                        )));
                }
            }
        }
        
        // No override found, resolve through HEAD
        let head = self.database.head().get_head()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        match head.refs.iter().find(|head_ref| head_ref.object_name == request.object_name) {
            Some(head_ref) => {
                info!("Found object '{}' in HEAD at version {}", request.object_name, head_ref.version);
                // Verify the reference exists and get the SHA256
                match self.database.refs().get_ref(&request.object_name)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
                    Some(object_ref) => {
                        if object_ref.version == head_ref.version {
                            self.database.objects().get(&object_ref.sha256_key)
                                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))
                                .and_then(|opt| opt.ok_or_else(|| ObjectsTreeError::SerializationError(
                                    format!("Object '{}' content not found", request.object_name)
                                )))
                        } else {
                            error!("Version mismatch: HEAD has version {}, but ref has version {}", 
                                   head_ref.version, object_ref.version);
                            Err(ObjectsTreeError::SerializationError(
                                "Version mismatch between HEAD and refs".to_string()
                            ))
                        }
                    }
                    None => {
                        error!("Object '{}' not found in refs but present in HEAD", request.object_name);
                        Err(ObjectsTreeError::SerializationError(
                            "Object reference not found".to_string()
                        ))
                    }
                }
            }
            None => {
                info!("Object '{}' not found in HEAD", request.object_name);
                error!("Object '{}' not found", request.object_name);
                Err(ObjectsTreeError::SledError(sled::Error::Unsupported(
                    format!("Object '{}' not found", request.object_name)
                )))
            }
        }
    }
}

impl Operation for ObjectGetOperation {
    fn name(&self) -> &'static str {
        "object/get"
    }
    
    fn description(&self) -> &'static str {
        "Retrieves a MOO object definition by name from the database"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/object/get".to_string(),
                method: Method::POST,
                is_json: true, // Expects JSON body with object_name
            },
            OperationRoute {
                path: "/api/object/get".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>) -> moor_var::Var {
        // For RPC calls, we expect the args to contain:
        // args[0] = object_name
        
        if args.is_empty() {
            error!("Object get operation requires object name");
            return moor_var::v_str("Error: Object name is required");
        }

        let object_name = args[0].clone();

        let request = ObjectGetRequest {
            object_name,
        };

        match self.process_object_get(request) {
            Ok(result) => {
                info!("Object get operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Object get operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
