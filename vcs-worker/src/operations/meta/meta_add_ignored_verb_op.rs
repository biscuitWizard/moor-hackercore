use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, MetaAddIgnoredVerbRequest};
use crate::providers::index::IndexProvider;
use super::meta_utils;

/// Meta operation that adds an ignored verb to an object's meta
#[derive(Clone)]
pub struct MetaAddIgnoredVerbOperation {
    database: DatabaseRef,
}

impl MetaAddIgnoredVerbOperation {
    /// Create a new meta add ignored verb operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the meta add ignored verb request
    fn process_meta_add_ignored_verb(&self, request: MetaAddIgnoredVerbRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing meta add ignored verb for '{}', verb '{}'", request.object_name, request.verb_name);
        
        // Validate object exists
        meta_utils::validate_object_exists(&self.database, &request.object_name)?;
        
        // Get or create the local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Load existing meta or create default
        let (mut meta, meta_existed_before) = meta_utils::load_or_create_meta(&self.database, &request.object_name)?;
        
        // Add the verb to ignored_verbs
        let was_added = meta.ignored_verbs.insert(request.verb_name.clone());
        
        if !was_added {
            info!("Verb '{}' was already in ignored list for object '{}'", request.verb_name, request.object_name);
            return Ok(format!("Verb '{}' was already ignored for object '{}'", request.verb_name, request.object_name));
        }
        
        // Save and track the meta
        meta_utils::save_and_track_meta(&self.database, &meta, &request.object_name, meta_existed_before, &mut current_change)?;
        
        info!("Successfully added verb '{}' to ignored list for object '{}'", request.verb_name, request.object_name);
        Ok(format!("Verb '{}' added to ignored list for object '{}'", request.verb_name, request.object_name))
    }
}

impl Operation for MetaAddIgnoredVerbOperation {
    fn name(&self) -> &'static str {
        "meta/add_ignored_verb"
    }
    
    fn description(&self) -> &'static str {
        "Adds a verb to the ignored verbs list in the object's meta"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/meta/add_ignored_verb".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/meta/add_ignored_verb".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Meta add ignored verb operation received {} arguments: {:?}", args.len(), args);
        
        if args.len() < 2 {
            error!("Meta add ignored verb operation requires object name and verb name");
            return moor_var::v_str("Error: Object name and verb name are required");
        }

        let object_name = args[0].clone();
        let verb_name = args[1].clone();

        let request = MetaAddIgnoredVerbRequest {
            object_name,
            verb_name,
        };

        match self.process_meta_add_ignored_verb(request) {
            Ok(result) => {
                info!("Meta add ignored verb operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta add ignored verb operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}

