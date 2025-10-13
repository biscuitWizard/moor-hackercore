use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use crate::providers::user::UserProvider;
use axum::http::Method;
use std::sync::Arc;
use tracing::{error, info};

use crate::types::{Permission, User};

/// Delete user operation
#[derive(Clone)]
pub struct UserDeleteOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserDeleteOperation {
    pub fn new(user_provider: Arc<dyn UserProvider>) -> Self {
        Self { user_provider }
    }
}

impl Operation for UserDeleteOperation {
    fn name(&self) -> &'static str {
        "user/delete"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Delete a user from the system"
    }

    fn philosophy(&self) -> &'static str {
        "Permanently deletes a user account from the system, removing all their data, permissions, \
        and API keys. This operation is irreversible and should be used with caution. System users \
        (like Wizard and Everyone) cannot be deleted to maintain system integrity. This operation \
        requires the DeleteUser permission and is intended for removing accounts that are no longer \
        needed or were created in error."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![OperationParameter {
            name: "user_id".to_string(),
            description: "ID of the user to delete".to_string(),
            required: true,
        }]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Delete a user".to_string(),
            moocode: r#"result = worker_request("vcs", {"user/delete", "alice"});
// Permanently deletes user 'alice' and all associated data"#
                .to_string(),
            http_curl: Some(
                r#"curl -X POST http://localhost:8081/api/user/delete \
  -H "Content-Type: application/json" \
  -d '{"operation": "user/delete", "args": ["alice"]}'
"#
                .to_string(),
            ),
        }]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/user/delete".to_string(),
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
        info!("Executing user/delete operation for user: {}", user.id);

        // Check permission
        if !user.has_permission(&Permission::DeleteUser) {
            error!("User {} does not have DeleteUser permission", user.id);
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: You do not have permission to delete users"),
            );
        }

        // Validate arguments
        if args.is_empty() {
            error!("Invalid arguments for user/delete: expected 1, got 0");
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: Expected 1 argument: user_id"),
            );
        }

        let target_user_id = &args[0];

        // Delete the user
        match self.user_provider.delete_user(target_user_id) {
            Ok(true) => {
                info!("Deleted user '{}'", target_user_id);
                moor_var::v_str(&format!("Successfully deleted user '{target_user_id}'"))
            }
            Ok(false) => {
                error!("User '{}' not found", target_user_id);
                moor_var::v_error(moor_var::E_INVARG.msg(format!(
                    "Error: User '{target_user_id}' not found"
                )))
            }
            Err(e) => {
                error!("Failed to delete user: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}

