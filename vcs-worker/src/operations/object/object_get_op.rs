use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::objects::ObjectsProvider;
use crate::providers::index::IndexProvider;
use crate::providers::refs::RefsProvider;
use crate::types::User;

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
        
        // Use the index provider to resolve the current state of the object
        match self.database.index().resolve_object_current_state(
            &request.object_name,
            |obj_name| self.database.refs().get_ref(obj_name, None).map_err(|e| crate::providers::ProviderError::SerializationError(e.to_string()))
        ).map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            Some(sha256_key) => {
                // Object exists - get its content
                self.database.objects().get(&sha256_key)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))
                    .and_then(|opt| opt.ok_or_else(|| ObjectsTreeError::SerializationError(
                        format!("Object '{}' content not found", request.object_name)
                    )))
            }
            None => {
                // Object is deleted or doesn't exist
                error!("Object '{}' not found or has been deleted", request.object_name);
                Err(ObjectsTreeError::SerializationError(
                    format!("Object '{}' not found", request.object_name)
                ))
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
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
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
