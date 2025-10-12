use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use crate::providers::user::UserProvider;
use axum::http::Method;
use std::sync::Arc;
use tracing::{error, info};

use crate::types::{Permission, User};

/// Disable user operation
#[derive(Clone)]
pub struct UserDisableOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserDisableOperation {
    pub fn new(user_provider: Arc<dyn UserProvider>) -> Self {
        Self { user_provider }
    }
}

impl Operation for UserDisableOperation {
    fn name(&self) -> &'static str {
        "user/disable"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Disable a user to prevent authentication"
    }

    fn philosophy(&self) -> &'static str {
        "Disables a user account, preventing them from authenticating and using any operations. \
        The user's data, permissions, and API keys are preserved but cannot be used until the account \
        is re-enabled. System users (like Wizard and Everyone) cannot be disabled. This operation \
        requires the DisableUser permission and is useful for temporarily suspending access without \
        deleting the account."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![OperationParameter {
            name: "user_id".to_string(),
            description: "ID of the user to disable".to_string(),
            required: true,
        }]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Disable a user".to_string(),
            moocode: r#"result = worker_request("vcs", {"user/disable", "alice"});
// Disables user 'alice' preventing authentication"#
                .to_string(),
            http_curl: Some(
                r#"curl -X POST http://localhost:8081/api/user/disable \
  -H "Content-Type: application/json" \
  -d '{"operation": "user/disable", "args": ["alice"]}'
"#
                .to_string(),
            ),
        }]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/user/disable".to_string(),
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
        info!("Executing user/disable operation for user: {}", user.id);

        // Check permission
        if !user.has_permission(&Permission::DisableUser) {
            error!("User {} does not have DisableUser permission", user.id);
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: You do not have permission to disable users"),
            );
        }

        // Validate arguments
        if args.is_empty() {
            error!("Invalid arguments for user/disable: expected 1, got 0");
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: Expected 1 argument: user_id"),
            );
        }

        let target_user_id = &args[0];

        // Disable the user
        match self.user_provider.disable_user(target_user_id) {
            Ok(()) => {
                info!("Disabled user '{}'", target_user_id);
                moor_var::v_str(&format!("Successfully disabled user '{target_user_id}'"))
            }
            Err(e) => {
                error!("Failed to disable user: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}
