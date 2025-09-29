use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError, Change};
use crate::providers::changes::ChangesProvider;
use crate::providers::repository::RepositoryProvider;

/// Request structure for change create operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub author: String,
}

/// Change create operation that creates a new change and sets it as current
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
        
        let change = Change {
            id: uuid::Uuid::new_v4().to_string(),
            name: request.name.clone(),
            description: request.description,
            author: request.author,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            added_objects: Vec::new(),
            modified_objects: Vec::new(),
            deleted_objects: Vec::new(),
            renamed_objects: Vec::new(),
            version_overrides: Vec::new(),
        };
        
        // Store the change
        self.database.changes().store_change(&change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Set it as the current change
        let mut repository = self.database.repository().get_repository()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        repository.current_change = Some(change.id.clone());
        self.database.repository().set_repository(&repository)
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
        "Creates a new change with the given name, description, and author"
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
