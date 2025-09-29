use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::{ChangeCreateRequest, Change, ChangeStatus};

/// Change create operation that creates a new change
#[derive(Clone)]
pub struct ChangeCreateOperation {
    database: DatabaseRef,
}

impl ChangeCreateOperation {
    /// Create a new change create operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change create request
    fn process_change_create(&self, request: ChangeCreateRequest) -> Result<String, ObjectsTreeError> {
        info!("Creating new change '{}' with author '{}'", request.name, request.author);
        
        // Check if there's already a local change at the top of the index
        let changes = self.database.index().list_changes()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(existing_change) = changes.first() {
            if existing_change.status == ChangeStatus::Local {
                error!("Cannot create new change '{}' - already in a local change '{}' ({})", 
                       request.name, existing_change.name, existing_change.id);
                return Err(ObjectsTreeError::SerializationError(
                    format!("Already in a local change '{}' ({}). Abandon the current change before creating a new one.", 
                            existing_change.name, existing_change.id)
                ));
            }
        }
        
        let change = Change {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name.clone(),
            description: request.description,
            author: request.author,
            timestamp: crate::util::current_unix_timestamp(),
            status: ChangeStatus::Local,
            added_objects: Vec::new(),
            modified_objects: Vec::new(),
            deleted_objects: Vec::new(),
            renamed_objects: Vec::new(),
            index_change_id: None,
        };
        
        // Store the change
        self.database.index().store_change(&change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Add change to the top of the index (since it's local)
        self.database.index().prepend_change(&change.id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully created change '{}' ({})", change.name, change.id);
        Ok(format!("Created change '{}' with ID: {}", change.name, change.id))
    }
}

impl Operation for ChangeCreateOperation {
    fn name(&self) -> &'static str {
        "change/create"
    }
    
    fn description(&self) -> &'static str {
        "Creates a new change with the given name, description, and author. Fails if already in a local change."
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/change/create".to_string(),
                method: Method::POST,
                is_json: true, // Expects JSON body with name, description, author
            },
            OperationRoute {
                path: "/api/change/create".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>) -> moor_var::Var {
        info!("Change create operation received {} arguments: {:?}", args.len(), args);
        
        if args.len() < 2 {
            error!("Change create operation requires at least name and author");
            return moor_var::v_str("Error: Name and author are required");
        }

        let name = args[0].clone();
        let author = args[1].clone();
        let description = if args.len() > 2 && !args[2].is_empty() {
            Some(args[2].clone())
        } else {
            None
        };

        let request = ChangeCreateRequest {
            name,
            description,
            author,
        };

        match self.process_change_create(request) {
            Ok(result) => {
                info!("Change create operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Change create operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
