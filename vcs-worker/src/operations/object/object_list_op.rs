use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
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
    
    fn philosophy(&self) -> &'static str {
        "Provides a complete view of all MOO objects currently in the version control repository. This \
        operation walks through the entire change history chronologically, computing the current state by \
        applying all additions, modifications, renames, and deletions. The result reflects what objects \
        exist right now, taking into account all submitted changes and your current working changelist. \
        This is useful for getting an overview of your repository contents, synchronizing with the MOO \
        database, or building tools that need to operate on the full object set."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "List all objects in the repository".to_string(),
                moocode: r#"json_str = worker_request("vcs", {"object/list"});
// Returns JSON with list of objects: {"objects": [{"object_type": "MooObject", "name": "$player", "version": 3}, ...]}
obj_list = parse_json(json_str)["objects"];
for obj in (obj_list)
  player:tell("Object: ", obj["name"], " (v", obj["version"], ")");
endfor"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/object/list \
  -H "Content-Type: application/json" \
  -d '{"operation": "object/list", "args": []}'"#.to_string()),
            }
        ]
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