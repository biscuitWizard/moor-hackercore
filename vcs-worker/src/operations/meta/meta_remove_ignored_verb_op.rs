use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use super::meta_utils;
use crate::database::DatabaseRef;
use crate::providers::index::IndexProvider;
use crate::types::{MetaRemoveIgnoredVerbRequest, ObjectsTreeError, User};

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
    fn process_meta_remove_ignored_verb(
        &self,
        request: MetaRemoveIgnoredVerbRequest,
        author: Option<String>,
    ) -> Result<String, ObjectsTreeError> {
        info!(
            "Processing meta remove ignored verb for '{}', verb '{}'",
            request.object_name, request.verb_name
        );

        // Validate object exists
        meta_utils::validate_object_exists(&self.database, &request.object_name)?;

        // Get or create the local change
        let mut current_change = self
            .database
            .index()
            .get_or_create_local_change(author)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        // Load existing meta or create default
        let (mut meta, meta_existed_before) =
            meta_utils::load_or_create_meta(&self.database, &request.object_name)?;

        // Remove the verb from ignored_verbs
        let was_removed = meta.ignored_verbs.remove(&request.verb_name);

        if !was_removed {
            info!(
                "Verb '{}' was not in ignored list for object '{}'",
                request.verb_name, request.object_name
            );
            return Ok(format!(
                "Verb '{}' was not in ignored list for object '{}'",
                request.verb_name, request.object_name
            ));
        }

        // Save and track the meta
        meta_utils::save_and_track_meta(
            &self.database,
            &meta,
            &request.object_name,
            meta_existed_before,
            &mut current_change,
        )?;

        info!(
            "Successfully removed verb '{}' from ignored list for object '{}'",
            request.verb_name, request.object_name
        );
        Ok(format!(
            "Verb '{}' removed from ignored list for object '{}'",
            request.verb_name, request.object_name
        ))
    }
}

impl Operation for MetaRemoveIgnoredVerbOperation {
    fn name(&self) -> &'static str {
        "meta/remove_ignored_verb"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Removes a verb from the ignored verbs list in the object's meta"
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/meta/remove_ignored_verb".to_string(),
            method: Method::POST,
            is_json: true,
        }]
    }

    fn philosophy(&self) -> &'static str {
        "Removes a verb from the object's meta ignored verbs list, causing the VCS to resume tracking that verb in \
        version control. This is useful when a previously ignored verb becomes important to track (e.g., a debug verb \
        becomes a permanent feature), or when cleaning up ignore lists after development is complete. The operation \
        modifies the object's meta file and adds it to the current local change."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Remove a verb from the ignored list".to_string(),
            moocode: r#"// Resume tracking a verb that was previously ignored
result = worker_request("vcs", {"meta/remove_ignored_verb", "obj123", "debug_state"});
// Returns: "Verb 'debug_state' removed from ignored list for object 'obj123'"
// Future changes to this verb will now appear in diffs"#
                .to_string(),
            http_curl: Some(
                r#"curl -X POST http://localhost:8081/api/meta/remove_ignored_verb \
  -H "Content-Type: application/json" \
  -d '{"object_name":"obj123","verb_name":"debug_state"}'"#
                    .to_string(),
            ),
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#""Operation completed successfully""#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Invalid arguments",
                r#"E_INVARG("Error: Invalid operation arguments")"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - Resource not found",
                r#"E_INVARG("Error: Resource not found")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#"E_INVARG("Error: Database error: operation failed")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!(
            "Meta remove ignored verb operation received {} arguments: {:?}",
            args.len(),
            args
        );

        if args.len() < 2 {
            error!("Meta remove ignored verb operation requires object name and verb name");
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: Object name and verb name are required"),
            );
        }

        let object_name = args[0].clone();
        let verb_name = args[1].clone();

        let request = MetaRemoveIgnoredVerbRequest {
            object_name,
            verb_name,
        };

        match self.process_meta_remove_ignored_verb(request, Some(user.id.clone())) {
            Ok(result) => {
                info!("Meta remove ignored verb operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta remove ignored verb operation failed: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}
