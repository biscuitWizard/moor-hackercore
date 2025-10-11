use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, User, MetaRemoveIgnoredPropertyRequest};
use crate::providers::index::IndexProvider;
use super::meta_utils;

/// Meta operation that removes an ignored property from an object's meta
#[derive(Clone)]
pub struct MetaRemoveIgnoredPropertyOperation {
    database: DatabaseRef,
}

impl MetaRemoveIgnoredPropertyOperation {
    /// Create a new meta remove ignored property operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the meta remove ignored property request
    fn process_meta_remove_ignored_property(&self, request: MetaRemoveIgnoredPropertyRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing meta remove ignored property for '{}', property '{}'", request.object_name, request.property_name);
        
        // Validate object exists
        meta_utils::validate_object_exists(&self.database, &request.object_name)?;
        
        // Get or create the local change
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Load existing meta or create default
        let (mut meta, meta_existed_before) = meta_utils::load_or_create_meta(&self.database, &request.object_name)?;
        
        // Remove the property from ignored_properties
        let was_removed = meta.ignored_properties.remove(&request.property_name);
        
        if !was_removed {
            info!("Property '{}' was not in ignored list for object '{}'", request.property_name, request.object_name);
            return Ok(format!("Property '{}' was not in ignored list for object '{}'", request.property_name, request.object_name));
        }
        
        // Save and track the meta
        meta_utils::save_and_track_meta(&self.database, &meta, &request.object_name, meta_existed_before, &mut current_change)?;
        
        info!("Successfully removed property '{}' from ignored list for object '{}'", request.property_name, request.object_name);
        Ok(format!("Property '{}' removed from ignored list for object '{}'", request.property_name, request.object_name))
    }
}

impl Operation for MetaRemoveIgnoredPropertyOperation {
    fn name(&self) -> &'static str {
        "meta/remove_ignored_property"
    }
    
    fn description(&self) -> &'static str {
        "Removes a property from the ignored properties list in the object's meta"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/meta/remove_ignored_property".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/meta/remove_ignored_property".to_string(),
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
        info!("Meta remove ignored property operation received {} arguments: {:?}", args.len(), args);
        
        if args.len() < 2 {
            error!("Meta remove ignored property operation requires object name and property name");
            return moor_var::v_str("Error: Object name and property name are required");
        }

        let object_name = args[0].clone();
        let property_name = args[1].clone();

        let request = MetaRemoveIgnoredPropertyRequest {
            object_name,
            property_name,
        };

        match self.process_meta_remove_ignored_property(request) {
            Ok(result) => {
                info!("Meta remove ignored property operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Meta remove ignored property operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
