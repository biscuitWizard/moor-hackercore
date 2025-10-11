use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use crate::providers::user::UserProvider;
use axum::http::Method;
use tracing::{error, info};
use std::sync::Arc;

use crate::types::{User, Permission};

/// Create user operation
#[derive(Clone)]
pub struct UserCreateOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserCreateOperation {
    pub fn new(user_provider: Arc<dyn UserProvider>) -> Self {
        Self { user_provider }
    }
}

impl Operation for UserCreateOperation {
    fn name(&self) -> &'static str {
        "user/create"
    }
    
    fn description(&self) -> &'static str {
        "Create a new user in the system"
    }
    
    fn philosophy(&self) -> &'static str {
        "Creates a new user account with the specified ID, email, and MOO object reference. The new user \
        starts with no permissions or API keys - these must be granted separately. This operation requires \
        the CreateUser permission. System administrators can use this to provision new user accounts for \
        team members or automated systems."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "user_id".to_string(),
                description: "Unique identifier for the new user".to_string(),
                required: true,
            },
            OperationParameter {
                name: "email".to_string(),
                description: "Email address for the new user".to_string(),
                required: true,
            },
            OperationParameter {
                name: "v_obj".to_string(),
                description: "MOO object reference as an integer (e.g., 123 for #123)".to_string(),
                required: true,
            }
        ]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Create a new user".to_string(),
                moocode: r#"result = worker_request("vcs", {"user/create", "alice", "alice@example.com", "100"});
// Creates user 'alice' with email alice@example.com and object #100"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/api/user/create \
  -H "Content-Type: application/json" \
  -d '{"operation": "user/create", "args": ["alice", "alice@example.com", "100"]}'
"#.to_string()),
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/user/create".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Executing user/create operation for user: {}", user.id);
        
        // Check permission
        if !user.has_permission(&Permission::CreateUser) {
            error!("User {} does not have CreateUser permission", user.id);
            return moor_var::v_str("Error: You do not have permission to create users");
        }
        
        // Validate arguments
        if args.len() < 3 {
            error!("Invalid arguments for user/create: expected 3, got {}", args.len());
            return moor_var::v_str("Error: Expected 3 arguments: user_id, email, v_obj");
        }
        
        let user_id = &args[0];
        let email = &args[1];
        let v_obj_str = &args[2];
        
        // Parse v_obj as integer
        let v_obj_num = match v_obj_str.parse::<i32>() {
            Ok(n) => n,
            Err(_) => {
                error!("Invalid v_obj: {}", v_obj_str);
                return moor_var::v_str(&format!("Error: Invalid v_obj '{}', must be an integer", v_obj_str));
            }
        };
        
        let v_obj = moor_var::Obj::mk_id(v_obj_num);
        
        // Create the user
        match self.user_provider.create_user(user_id.clone(), email.clone(), v_obj) {
            Ok(created_user) => {
                info!("Created user '{}' with email '{}' and v_obj {:?}", 
                      created_user.id, created_user.email, created_user.v_obj);
                moor_var::v_str(&format!("Successfully created user '{}'", user_id))
            }
            Err(e) => {
                error!("Failed to create user: {}", e);
                moor_var::v_str(&format!("Error: {}", e))
            }
        }
    }
}

