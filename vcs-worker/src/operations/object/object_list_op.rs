use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::{ObjectInfo, User};
use crate::providers::index::IndexProvider;

/// Request structure for object list operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectListRequest {
    #[allow(dead_code)]
    pub include_deleted: bool, // For future use - whether to include deleted objects
}

/// Response structure containing the list of objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectListResponse {
    pub objects: Vec<ObjectInfo>  // ObjectInfo comes from crate::types
}

/// Object list operation that walks through the entire change history chronologically
/// and tracks object names, handling renames, additions, and deletions
#[derive(Clone)]
pub struct ObjectListOperation {
    database: DatabaseRef,
}

impl ObjectListOperation {
    /// Create a new object list operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the object list request by delegating to IndexProvider
    fn process_object_list(&self, _request: ObjectListRequest) -> Result<String, ObjectsTreeError> {
        info!("Requesting complete object list from IndexProvider");
        
        // Use the IndexProvider to compute the complete object list
        let object_list = self.database.index().compute_complete_object_list()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        let response = ObjectListResponse {
            objects: object_list,
        };
        
        // Return as JSON string
        serde_json::to_string(&response)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON serialization error: {}", e)))
    }
}

impl Operation for ObjectListOperation {
    fn name(&self) -> &'static str {
        "object/list"
    }
    
    fn description(&self) -> &'static str {
        "Lists all objects by walking through the entire change history chronologically, tracking names, renames, additions, and deletions"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/object/list".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/object/list".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Executing object list operation with {} args", args.len());
        
        let request = ObjectListRequest {
            include_deleted: false, // For now, don't include deleted objects
        };

        match self.process_object_list(request) {
            Ok(result) => {
                info!("Object list operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Object list operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}