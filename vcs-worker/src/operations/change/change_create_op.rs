use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::User;
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
        if let Some(top_change_id) = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            if let Some(existing_change) = self.database.index().get_change(&top_change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
                if existing_change.status == ChangeStatus::Local {
                    error!("Cannot create new change '{}' - already in a local change '{}' ({})", 
                           request.name, existing_change.name, existing_change.id);
                    return Err(ObjectsTreeError::SerializationError(
                        format!("Already in a local change '{}' ({}). Abandon the current change before creating a new one.", 
                                existing_change.name, existing_change.id)
                    ));
                }
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
        self.database.index().push_change(&change.id)
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
    
    fn philosophy(&self) -> &'static str {
        "Creates a new changelist for organizing your work. In the VCS workflow, changes are the fundamental \
        unit of work organization - similar to branches in git, but lighter weight. When you create a change, \
        you're starting a new workspace where you can add, modify, rename, or delete objects. All operations \
        are tracked within this change until you submit it. You can only have one local (active) change at a \
        time; if you want to work on multiple features simultaneously, use 'change/switch' to move between \
        workspace changes. The change name and description help you and your team understand what work is being done."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "name".to_string(),
                description: "A short name for the change (e.g., 'fix-parser-bug', 'add-new-command')".to_string(),
                required: true,
            },
            OperationParameter {
                name: "author".to_string(),
                description: "The name of the person creating the change (e.g., player name or user id)".to_string(),
                required: true,
            },
            OperationParameter {
                name: "description".to_string(),
                description: "A detailed description of what this change accomplishes (optional)".to_string(),
                required: false,
            }
        ]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Create a new change with a name and author".to_string(),
                moocode: r#"result = worker_request("vcs", {"change/create", "fix-login-bug", player.name});
// Returns: "Created change 'fix-login-bug' with ID: abc-123..."
// Now you can start modifying objects in this change"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/api/change/create \
  -H "Content-Type: application/json" \
  -d '{"operation": "change/create", "args": ["fix-login-bug", "Wizard"]}'"#.to_string()),
            },
            OperationExample {
                description: "Create a change with a description".to_string(),
                moocode: r#"result = worker_request("vcs", {"change/create", "new-feature", player.name, "Adding support for new command syntax"});
// The description helps document the purpose of the change"#.to_string(),
                http_curl: None,
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/change/create".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
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
