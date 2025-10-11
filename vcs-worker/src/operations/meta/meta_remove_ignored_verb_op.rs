use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, MetaRemoveIgnoredVerbRequest};
use crate::providers::index::IndexProvider;
use super::meta_utils;

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
    fn process_meta_remove_ignored_verb(&self, request: MetaRemoveIgnoredVerbRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing meta remove ignored verb for '{}', verb '{}'", request.object_name, request.verb_name);
        
        // Validate object exists
        meta_utils::validate_object_exists(&self.database, &request.object_name)?;
        
        // Get or create the local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Load existing meta or create default
        let (mut meta, meta_existed_before) = meta_utils::load_or_create_meta(&self.database, &request.object_name)?;
        
        // Remove the verb from ignored_verbs
        let was_removed = meta.ignored_verbs.remove(&request.verb_name);
        
        if !was_removed {
            info!("Verb '{}' was not in ignored list for object '{}'", request.verb_name, request.object_name);
            return Ok(format!("Verb '{}' was not in ignored list for object '{}'", request.verb_name, request.object_name));
        }
        
        // Save and track the meta
        meta_utils::save_and_track_meta(&self.database, &meta, &request.object_name, meta_existed_before, &mut current_change)?;
        
        info!("Successfully removed verb '{}' from ignored list for object '{}'", request.verb_name, request.object_name);
        Ok(format!("Verb '{}' removed from ignored list for object '{}'", request.verb_name, request.object_name))
    }
}

impl Operation for MetaRemoveIgnoredVerbOperation {
    fn name(&self) -> &'static str {
        "meta/remove_ignored_verb"
    }
    
    fn description(&self) -> &'static str {
        "Removes a verb from the ignored verbs list in the object's meta"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/meta/remove_ignored_verb".to_string(),
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

    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Meta remove ignored verb operation received {} arguments: {:?}", args.len(), args);
        
        if args.len() < 2 {
            error!("Meta remove ignored verb operation requires object name and verb name");
            return moor_var::v_str("Error: Object name and verb name are required");
        }

        let object_name = args[0].clone();
        let verb_name = args[1].clone();

        let request = MetaRemoveIgnoredVerbRequest {
            object_name,
            verb_name,
        };

        match self.process_meta_remove_ignored_verb(request) {
            Ok(result) => {
                info!("Meta remove ignored verb operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta remove ignored verb operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
