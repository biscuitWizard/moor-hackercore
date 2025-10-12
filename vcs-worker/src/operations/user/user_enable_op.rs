use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use crate::providers::user::UserProvider;
use axum::http::Method;
use std::sync::Arc;
use tracing::{error, info};

use crate::types::{Permission, User};

/// Enable user operation
#[derive(Clone)]
pub struct UserEnableOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserEnableOperation {
    pub fn new(user_provider: Arc<dyn UserProvider>) -> Self {
        Self { user_provider }
    }
}

impl Operation for UserEnableOperation {
    fn name(&self) -> &'static str {
        "user/enable"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Enable a previously disabled user"
    }

    fn philosophy(&self) -> &'static str {
        "Re-enables a disabled user account, allowing them to authenticate and use operations again. \
        All of the user's permissions and API keys are restored to their previous state. This operation \
        requires the DisableUser permission (the same permission used to disable users)."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![OperationParameter {
            name: "user_id".to_string(),
            description: "ID of the user to enable".to_string(),
            required: true,
        }]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Enable a user".to_string(),
            moocode: r#"result = worker_request("vcs", {"user/enable", "alice"});
// Re-enables user 'alice' allowing authentication"#
                .to_string(),
            http_curl: Some(
                r#"curl -X POST http://localhost:8081/api/user/enable \
  -H "Content-Type: application/json" \
  -d '{"operation": "user/enable", "args": ["alice"]}'
"#
                .to_string(),
            ),
        }]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/user/enable".to_string(),
            method: Method::POST,
            is_json: true,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#""Operation completed successfully""#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Invalid arguments",
                r#"E_INVARG("Error: Invalid operation arguments")"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - Resource not found",
                r#"E_INVARG("Error: Resource not found")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#"E_INVARG("Error: Database error: operation failed")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Executing user/enable operation for user: {}", user.id);

        // Check permission
        if !user.has_permission(&Permission::DisableUser) {
            error!("User {} does not have DisableUser permission", user.id);
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: You do not have permission to enable users"),
            );
        }

        // Validate arguments
        if args.is_empty() {
            error!("Invalid arguments for user/enable: expected 1, got 0");
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: Expected 1 argument: user_id"),
            );
        }

        let target_user_id = &args[0];

        // Enable the user
        match self.user_provider.enable_user(target_user_id) {
            Ok(()) => {
                info!("Enabled user '{}'", target_user_id);
                moor_var::v_str(&format!("Successfully enabled user '{target_user_id}'"))
            }
            Err(e) => {
                error!("Failed to enable user: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}
