use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use super::meta_utils;
use crate::database::DatabaseRef;
use crate::providers::index::IndexProvider;
use crate::types::{MetaAddIgnoredPropertyRequest, ObjectsTreeError, User};

/// Meta operation that adds an ignored property to an object's meta
#[derive(Clone)]
pub struct MetaAddIgnoredPropertyOperation {
    database: DatabaseRef,
}

impl MetaAddIgnoredPropertyOperation {
    /// Create a new meta add ignored property operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the meta add ignored property request
    fn process_meta_add_ignored_property(
        &self,
        request: MetaAddIgnoredPropertyRequest,
        author: Option<String>,
    ) -> Result<String, ObjectsTreeError> {
        info!(
            "Processing meta add ignored property for '{}', property '{}'",
            request.object_name, request.property_name
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

        // Add the property to ignored_properties
        let was_added = meta
            .ignored_properties
            .insert(request.property_name.clone());

        if !was_added {
            info!(
                "Property '{}' was already in ignored list for object '{}'",
                request.property_name, request.object_name
            );
            return Ok(format!(
                "Property '{}' was already ignored for object '{}'",
                request.property_name, request.object_name
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
            "Successfully added property '{}' to ignored list for object '{}'",
            request.property_name, request.object_name
        );
        Ok(format!(
            "Property '{}' added to ignored list for object '{}'",
            request.property_name, request.object_name
        ))
    }
}

impl Operation for MetaAddIgnoredPropertyOperation {
    fn name(&self) -> &'static str {
        "meta/add_ignored_property"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Adds a property to the ignored properties list in the object's meta"
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/meta/add_ignored_property".to_string(),
            method: Method::POST,
            is_json: true,
        }]
    }

    fn philosophy(&self) -> &'static str {
        "Adds a property to the object's meta ignored properties list, causing the VCS to exclude that property from \
        version control tracking (similar to .gitignore). This is useful for properties that change frequently but don't \
        need to be tracked, such as temporary state, cache values, or runtime statistics. The operation modifies the \
        object's meta file and adds it to the current local change."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Add a property to the ignored list".to_string(),
            moocode: r#"// Ignore a property that changes frequently but doesn't need tracking
result = worker_request("vcs", {"meta/add_ignored_property", "obj123", "last_access_time"});
// Returns: "Property 'last_access_time' added to ignored list for object 'obj123'"
// Future changes to this property will not appear in diffs"#
                .to_string(),
            http_curl: Some(
                r#"curl -X POST http://localhost:8081/api/meta/add_ignored_property \
  -H "Content-Type: application/json" \
  -d '{"object_name":"obj123","property_name":"last_access_time"}'"#
                    .to_string(),
            ),
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Property successfully added to ignored list",
                r#""Property 'property_name' added to ignored list for object 'object_name'""#,
            ),
            OperationResponse::success(
                "Property was already in ignored list",
                r#""Property 'property_name' was already ignored for object 'object_name'""#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Missing required arguments",
                r#"E_INVARG("Error: Object name and property name are required")"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - Object does not exist",
                r#"E_INVARG("Error: Object not found")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Serialization or database error",
                r#"E_INVARG("Error: SerializationError: failed to serialize data")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!(
            "Meta add ignored property operation received {} arguments: {:?}",
            args.len(),
            args
        );

        if args.len() < 2 {
            error!("Meta add ignored property operation requires object name and property name");
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: Object name and property name are required"),
            );
        }

        let object_name = args[0].clone();
        let property_name = args[1].clone();

        let request = MetaAddIgnoredPropertyRequest {
            object_name,
            property_name,
        };

        match self.process_meta_add_ignored_property(request, Some(user.id.clone())) {
            Ok(result) => {
                info!("Meta add ignored property operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta add ignored property operation failed: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}
