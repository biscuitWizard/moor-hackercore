use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::{ObjectInfo, User};
use moor_var::{E_INVARG, Var, v_error, v_list, v_str};

/// Request structure for object list operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectListRequest {
    // Currently no parameters needed, but kept as a struct for future extensibility
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
    fn process_object_list(
        &self,
    ) -> Result<Vec<ObjectInfo>, ObjectsTreeError> {
        info!("Requesting complete object list from IndexProvider");

        // Use the IndexProvider to compute the complete object list
        let object_list = self
            .database
            .index()
            .compute_complete_object_list()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        Ok(object_list)
    }
}

impl Operation for ObjectListOperation {
    fn name(&self) -> &'static str {
        "object/list"
    }

    fn description(&self) -> &'static str {
        "Lists all objects by walking through the entire change history chronologically, tracking names, renames, additions, and deletions. Returns a MOO list of object names."
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn philosophy(&self) -> &'static str {
        "Provides a complete view of all MOO objects currently in the version control repository. This \
        operation walks through the entire change history chronologically, computing the current state by \
        applying all additions, modifications, renames, and deletions. The result is a MOO list of object \
        names reflecting what objects exist right now, taking into account all submitted changes and your \
        current working changelist. This is useful for getting an overview of your repository contents, \
        synchronizing with the MOO database, or building tools that need to operate on the full object set."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "List all objects in the repository".to_string(),
            moocode: r##"// Returns a list of object names
objects = worker_request("vcs", {"object/list"});
// objects is a list like: {"$player", "$room", "#123", "#124"}
for obj in (objects)
    player:tell("Object: ", obj);
endfor"##
                .to_string(),
            http_curl: Some(r##"curl -X POST http://localhost:8081/api/object/list"##.to_string()),
        }]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/object/list".to_string(),
            method: Method::POST,
            is_json: true,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r##"{"$player", "$room", "#123", "#124"}"##,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or computation error",
                r##"E_INVARG("Failed to compute object list: database operation failed")"##,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, _user: &User) -> Var {
        info!("Executing object list operation with {} args", args.len());

        let _request = ObjectListRequest {};

        match self.process_object_list() {
            Ok(object_list) => {
                info!(
                    "Object list operation completed successfully with {} objects",
                    object_list.len()
                );

                // Convert ObjectInfo list to MOO list of object names (strings)
                let object_names: Vec<Var> =
                    object_list.iter().map(|obj| v_str(&obj.name)).collect();

                v_list(&object_names)
            }
            Err(e) => {
                error!("Object list operation failed: {}", e);
                v_error(E_INVARG.msg(format!("{e}")))
            }
        }
    }
}
