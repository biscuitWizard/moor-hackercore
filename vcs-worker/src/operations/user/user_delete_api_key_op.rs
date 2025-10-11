use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use crate::providers::user::UserProvider;
use axum::http::Method;
use tracing::{error, info};
use std::sync::Arc;

use crate::types::{User, Permission};

/// Delete API key from user operation
#[derive(Clone)]
pub struct UserDeleteApiKeyOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserDeleteApiKeyOperation {
    pub fn new(user_provider: Arc<dyn UserProvider>) -> Self {
        Self { user_provider }
    }
}

impl Operation for UserDeleteApiKeyOperation {
    fn name(&self) -> &'static str {
        "user/delete_api_key"
    }
    
    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Delete an API key from a user"
    }
    
    fn philosophy(&self) -> &'static str {
        "Removes an API key from a user account, immediately revoking access for that key. Users can \
        delete their own API keys without special permissions (self-service). To delete an API key from \
        another user's account, the ManageApiKeys permission is required. Use this to revoke compromised \
        keys or remove access that is no longer needed."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "api_key".to_string(),
                description: "The API key to delete".to_string(),
                required: true,
            },
            OperationParameter {
                name: "target_user_id".to_string(),
                description: "Optional: ID of the user to delete key from. If not provided or empty, deletes from current user.".to_string(),
                required: false,
            }
        ]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Delete your own API key".to_string(),
                moocode: r#"result = worker_request("vcs", {"user/delete_api_key", "abc123..."});
// Deletes the specified API key from your account"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/api/user/delete_api_key \
  -H "Content-Type: application/json" \
  -d '{"operation": "user/delete_api_key", "args": ["abc123-your-api-key"]}'
"#.to_string()),
            },
            OperationExample {
                description: "Delete API key from another user (requires ManageApiKeys)".to_string(),
                moocode: r#"result = worker_request("vcs", {"user/delete_api_key", "abc123...", "alice"});
// Deletes the API key from user 'alice'"#.to_string(),
                http_curl: None,
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/user/delete_api_key".to_string(),
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
        info!("Executing user/delete_api_key operation for user: {}", user.id);
        
        // Validate arguments
        if args.is_empty() {
            error!("Invalid arguments for user/delete_api_key: expected at least 1 argument");
            return moor_var::v_error(moor_var::E_INVARG.msg("Error: Expected at least 1 argument: api_key"));
        }
        
        let api_key = &args[0];
        
        // Determine target user
        let target_user_id = if args.len() < 2 || args[1].is_empty() {
            // Self-service: delete from current user
            user.id.clone()
        } else {
            // Delete from another user
            let target = args[1].clone();
            
            // Check if deleting from self
            if target == user.id {
                target
            } else {
                // Deleting from another user requires ManageApiKeys permission
                if !user.has_permission(&Permission::ManageApiKeys) {
                    error!("User {} does not have ManageApiKeys permission", user.id);
                    return moor_var::v_error(moor_var::E_INVARG.msg("Error: You do not have permission to manage API keys for other users"));
                }
                target
            }
        };
        
        // Delete the API key
        match self.user_provider.delete_api_key(&target_user_id, api_key) {
            Ok(deleted) => {
                if deleted {
                    info!("Deleted API key from user '{}'", target_user_id);
                    moor_var::v_str(&format!("Successfully deleted API key from user '{target_user_id}'"))
                } else {
                    info!("API key not found for user '{}'", target_user_id);
                    moor_var::v_str(&format!("API key not found for user '{target_user_id}'"))
                }
            }
            Err(e) => {
                error!("Failed to delete API key: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}

