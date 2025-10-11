use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use crate::providers::user::UserProvider;
use axum::http::Method;
use tracing::{error, info};
use std::sync::Arc;

use crate::types::{User, Permission};

/// Generate API key for user operation
#[derive(Clone)]
pub struct UserGenerateApiKeyOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserGenerateApiKeyOperation {
    pub fn new(user_provider: Arc<dyn UserProvider>) -> Self {
        Self { user_provider }
    }
}

impl Operation for UserGenerateApiKeyOperation {
    fn name(&self) -> &'static str {
        "user/generate_api_key"
    }
    
    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Generate a new API key for a user"
    }
    
    fn philosophy(&self) -> &'static str {
        "Generates a new UUID-based API key for user authentication. Users can generate API keys for \
        themselves without any special permissions (self-service). To generate an API key for another \
        user, the ManageApiKeys permission is required. The generated key is returned and should be \
        saved securely by the caller. API keys can be used with HTTP requests or for configuring \
        external VCS worker connections."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "target_user_id".to_string(),
                description: "Optional: ID of the user to generate key for. If not provided or empty, generates for current user.".to_string(),
                required: false,
            }
        ]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Generate API key for yourself".to_string(),
                moocode: r#"api_key = worker_request("vcs", {"user/generate_api_key"});
// Returns new API key for the current user
player:tell("Your new API key: ", api_key);"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/api/user/generate_api_key \
  -H "Content-Type: application/json" \
  -d '{"operation": "user/generate_api_key", "args": []}'
"#.to_string()),
            },
            OperationExample {
                description: "Generate API key for another user (requires ManageApiKeys)".to_string(),
                moocode: r#"api_key = worker_request("vcs", {"user/generate_api_key", "alice"});
// Generates new API key for user 'alice'
player:tell("New API key for alice: ", api_key);"#.to_string(),
                http_curl: None,
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/user/generate_api_key".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
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
        info!("Executing user/generate_api_key operation for user: {}", user.id);
        
        // Determine target user
        let target_user_id = if args.is_empty() || args[0].is_empty() {
            // Self-service: generate for current user
            user.id.clone()
        } else {
            // Generate for another user
            let target = args[0].clone();
            
            // Check if generating for self
            if target == user.id {
                target
            } else {
                // Generating for another user requires ManageApiKeys permission
                if !user.has_permission(&Permission::ManageApiKeys) {
                    error!("User {} does not have ManageApiKeys permission", user.id);
                    return moor_var::v_error(moor_var::E_INVARG.msg("Error: You do not have permission to manage API keys for other users"));
                }
                target
            }
        };
        
        // Generate the API key
        match self.user_provider.generate_api_key(&target_user_id) {
            Ok(api_key) => {
                info!("Generated new API key for user '{}'", target_user_id);
                // Return just the API key string
                moor_var::v_str(&api_key)
            }
            Err(e) => {
                error!("Failed to generate API key: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}

