use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, MetaClearIgnoredVerbsRequest};
use crate::providers::index::IndexProvider;
use super::meta_utils;

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
    fn process_meta_clear_ignored_verbs(&self, request: MetaClearIgnoredVerbsRequest, author: Option<String>) -> Result<String, ObjectsTreeError> {
        info!("Processing meta clear ignored verbs for '{}'", request.object_name);
        
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
                return Ok(format!("No meta exists for object '{}', cleared 0 ignored verbs", request.object_name));
            }
        };
        
        // Clear all ignored verbs
        let count = meta.ignored_verbs.len();
        meta.ignored_verbs.clear();
        
        // Save and track the meta (meta existed before since we loaded it)
        meta_utils::save_and_track_meta(&self.database, &meta, &request.object_name, true, &mut current_change)?;
        
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
                path: "/api/meta/clear_ignored_verbs".to_string(),
                method: Method::POST,
                is_json: true,
            },
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

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Meta clear ignored verbs operation received {} arguments: {:?}", args.len(), args);
        
        if args.is_empty() {
            error!("Meta clear ignored verbs operation requires object name");
            return moor_var::v_str("Error: Object name is required");
        }

        let object_name = args[0].clone();

        let request = MetaClearIgnoredVerbsRequest {
            object_name,
        };

        match self.process_meta_clear_ignored_verbs(request, Some(user.id.clone())) {
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
