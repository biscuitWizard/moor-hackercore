use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, MetaClearIgnoredPropertiesRequest};
use crate::providers::index::IndexProvider;
use super::meta_utils;

/// Meta operation that clears all ignored properties from an object's meta
#[derive(Clone)]
pub struct MetaClearIgnoredPropertiesOperation {
    database: DatabaseRef,
}

impl MetaClearIgnoredPropertiesOperation {
    /// Create a new meta clear ignored properties operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the meta clear ignored properties request
    fn process_meta_clear_ignored_properties(&self, request: MetaClearIgnoredPropertiesRequest, author: Option<String>) -> Result<String, ObjectsTreeError> {
        info!("Processing meta clear ignored properties for '{}'", request.object_name);
        
        // Validate object exists
        meta_utils::validate_object_exists(&self.database, &request.object_name)?;
        
        // Get or create the local change
        let mut current_change = self.database.index().get_or_create_local_change(author)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Check if meta exists - if not, nothing to clear
        let mut meta = match meta_utils::load_meta_if_exists(&self.database, &request.object_name)? {
            Some(m) => m,
            None => {
                info!("No meta exists for object '{}', nothing to clear", request.object_name);
                return Ok(format!("No meta exists for object '{}', cleared 0 ignored properties", request.object_name));
            }
        };
        
        // Clear all ignored properties
        let count = meta.ignored_properties.len();
        meta.ignored_properties.clear();
        
        // Save and track the meta (meta existed before since we loaded it)
        meta_utils::save_and_track_meta(&self.database, &meta, &request.object_name, true, &mut current_change)?;
        
        info!("Successfully cleared {} ignored properties for object '{}'", count, request.object_name);
        Ok(format!("Cleared {} ignored properties for object '{}'", count, request.object_name))
    }
}

impl Operation for MetaClearIgnoredPropertiesOperation {
    fn name(&self) -> &'static str {
        "meta/clear_ignored_properties"
    }
    
    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Clears all ignored properties from the object's meta"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/meta/clear_ignored_properties".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn philosophy(&self) -> &'static str {
        "Documentation for this operation is being prepared."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#""Operation completed successfully""#
            ),
            OperationResponse::new(
                400,
                "Bad Request - Invalid arguments",
                r#"E_INVARG("Error: Invalid operation arguments")"#
            ),
            OperationResponse::new(
                404,
                "Not Found - Resource not found",
                r#"E_INVARG("Error: Resource not found")"#
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#"E_INVARG("Error: Database error: operation failed")"#
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Meta clear ignored properties operation received {} arguments: {:?}", args.len(), args);
        
        if args.is_empty() {
            error!("Meta clear ignored properties operation requires object name");
            return moor_var::v_error(moor_var::E_INVARG.msg("Error: Object name is required"));
        }

        let object_name = args[0].clone();

        let request = MetaClearIgnoredPropertiesRequest {
            object_name,
        };

        match self.process_meta_clear_ignored_properties(request, Some(user.id.clone())) {
            Ok(result) => {
                info!("Meta clear ignored properties operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta clear ignored properties operation failed: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(&format!("Error: {e}")))
            }
        }
    }
}
