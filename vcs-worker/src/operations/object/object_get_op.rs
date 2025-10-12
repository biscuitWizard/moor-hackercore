use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::refs::RefsProvider;
use crate::types::{User, VcsObjectType};
use moor_compiler::{CompileOptions, ObjFileContext, compile_object_definitions};
use moor_objdef::dump_object;
use moor_var::{E_INVARG, v_error};

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
        let sha256_key = match self
            .database
            .index()
            .resolve_object_current_state(&request.object_name, |obj_name| {
                self.database
                    .refs()
                    .get_ref(VcsObjectType::MooObject, obj_name, None)
                    .map_err(|e| crate::providers::ProviderError::SerializationError(e.to_string()))
            })
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        {
            Some(key) => key,
            None => {
                // Object is deleted or doesn't exist
                error!(
                    "Object '{}' not found or has been deleted",
                    request.object_name
                );
                return Err(ObjectsTreeError::SerializationError(format!(
                    "Object '{}' not found",
                    request.object_name
                )));
            }
        };

        // Object exists - get its content
        let object_dump = self
            .database
            .objects()
            .get(&sha256_key)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Object '{}' content not found",
                    request.object_name
                ))
            })?;

        // Check if meta exists for this object
        let meta = match self
            .database
            .refs()
            .get_ref(VcsObjectType::MooMetaObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        {
            Some(meta_sha256) => {
                // Meta exists, load it
                let yaml = self
                    .database
                    .objects()
                    .get(&meta_sha256)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                    .ok_or_else(|| {
                        ObjectsTreeError::SerializationError(
                            "Meta SHA256 exists but data not found".to_string(),
                        )
                    })?;
                Some(
                    self.database
                        .objects()
                        .parse_meta_dump(&yaml)
                        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?,
                )
            }
            None => None,
        };

        // If there's no meta or meta has no ignored items, return as-is
        if meta.is_none()
            || (meta.as_ref().unwrap().ignored_properties.is_empty()
                && meta.as_ref().unwrap().ignored_verbs.is_empty())
        {
            info!("No filtering needed for object '{}'", request.object_name);
            return Ok(object_dump);
        }

        let meta = meta.unwrap();
        info!(
            "Filtering object '{}' - ignoring {} properties and {} verbs",
            request.object_name,
            meta.ignored_properties.len(),
            meta.ignored_verbs.len()
        );

        // Parse the object dump into an ObjectDefinition
        let mut context = ObjFileContext::new();
        let mut compiled_defs =
            compile_object_definitions(&object_dump, &CompileOptions::default(), &mut context)
                .map_err(|e| {
                    ObjectsTreeError::SerializationError(format!("Failed to parse object: {e}"))
                })?;

        if compiled_defs.len() != 1 {
            return Err(ObjectsTreeError::SerializationError(format!(
                "Expected exactly 1 object definition, got {}",
                compiled_defs.len()
            )));
        }

        let mut obj_def = compiled_defs.remove(0);

        // Filter out ignored properties from property_definitions
        obj_def
            .property_definitions
            .retain(|prop| !meta.ignored_properties.contains(&prop.name.as_string()));

        // Filter out ignored properties from property_overrides
        obj_def
            .property_overrides
            .retain(|prop| !meta.ignored_properties.contains(&prop.name.as_string()));

        // Filter out ignored verbs
        obj_def.verbs.retain(|verb| {
            // A verb can have multiple names, check all of them
            !verb
                .names
                .iter()
                .any(|name| meta.ignored_verbs.contains(&name.as_string()))
        });

        // Re-dump the filtered object
        let index_names = HashMap::new(); // Empty index for simple object names
        let lines = dump_object(&index_names, &obj_def).map_err(|e| {
            ObjectsTreeError::SerializationError(format!("Failed to dump object: {e}"))
        })?;

        let filtered_dump = lines.join("\n");
        info!(
            "Successfully filtered object '{}', returning {} lines",
            request.object_name,
            lines.len()
        );

        Ok(filtered_dump)
    }
}

impl Operation for ObjectGetOperation {
    fn name(&self) -> &'static str {
        "object/get"
    }

    fn description(&self) -> &'static str {
        "Retrieves a MOO object definition by name from the database"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn philosophy(&self) -> &'static str {
        "Retrieves the current state of a MOO object definition from the version control system. \
        This operation returns the object dump (in objdef format) after applying any meta filtering \
        rules that may be configured for the object. This is useful for examining object definitions, \
        downloading them for local editing, or synchronizing with the MOO database. The returned \
        definition reflects the most recent version of the object in the repository."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![OperationParameter {
            name: "object_name".to_string(),
            description: "The name of the MOO object to retrieve (e.g., '$player', '#123')"
                .to_string(),
            required: true,
        }]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Retrieve an object definition by name".to_string(),
                moocode: r#"objdef = worker_request("vcs", {"object/get", "$player"});
// Returns the object definition as a string"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/api/object/get \
  -H "Content-Type: application/json" \
  -d '{"operation": "object/get", "args": ["$player"]}'"#.to_string()),
            },
            OperationExample {
                description: "Retrieve an object by object number".to_string(),
                moocode: "objdef = worker_request(\"vcs\", {\"object/get\", \"#123\"});\n// Returns the object definition for object #123".to_string(),
                http_curl: None,
            }
        ]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/object/get".to_string(),
            method: Method::POST,
            is_json: true,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#""obj $player
parent #1
name \"Player Object\"
owner #2
property description \"A player object\"
verb \"look\" this none this
  player:tell(\"You look at \", this.name);
end""#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Object name is required",
                r#"E_INVARG("Object name is required")"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - Object not found or has been deleted",
                r#"E_INVARG("Object '$player' not found")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Object content not found",
                r#"E_INVARG("Object '$player' content not found")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Failed to parse object",
                r#"E_INVARG("Failed to parse object: compilation error")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        // For RPC calls, we expect the args to contain:
        // args[0] = object_name

        if args.is_empty() {
            error!("Object get operation requires object name");
            return v_error(E_INVARG.msg("Object name is required"));
        }

        let object_name = args[0].clone();

        let request = ObjectGetRequest { object_name };

        match self.process_object_get(request) {
            Ok(result) => {
                info!("Object get operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Object get operation failed: {}", e);
                v_error(E_INVARG.msg(format!("{e}")))
            }
        }
    }
}
