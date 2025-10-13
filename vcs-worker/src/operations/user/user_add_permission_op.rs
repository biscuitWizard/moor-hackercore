use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use crate::providers::user::UserProvider;
use axum::http::Method;
use std::sync::Arc;
use tracing::{error, info};

use crate::types::{Permission, User};

/// Add permission to user operation
#[derive(Clone)]
pub struct UserAddPermissionOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserAddPermissionOperation {
    pub fn new(user_provider: Arc<dyn UserProvider>) -> Self {
        Self { user_provider }
    }

    fn parse_permission(perm_str: &str) -> Result<Permission, String> {
        match perm_str {
            "ApproveChanges" | "Approve_Changes" => Ok(Permission::ApproveChanges),
            "SubmitChanges" | "Submit_Changes" => Ok(Permission::SubmitChanges),
            "Clone" => Ok(Permission::Clone),
            "CreateUser" | "Create_User" => Ok(Permission::CreateUser),
            "DisableUser" | "Disable_User" => Ok(Permission::DisableUser),
            "DeleteUser" | "Delete_User" => Ok(Permission::DeleteUser),
            "ManagePermissions" | "Manage_Permissions" => Ok(Permission::ManagePermissions),
            "ManageApiKeys" | "Manage_Api_Keys" => Ok(Permission::ManageApiKeys),
            _ => Err(format!("Unknown permission: {perm_str}")),
        }
    }
}

impl Operation for UserAddPermissionOperation {
    fn name(&self) -> &'static str {
        "user/add_permission"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Add a permission to a user"
    }

    fn philosophy(&self) -> &'static str {
        "Grants a specific permission to a user account. Permissions control what operations a user \
        can perform in the system. Available permissions include: ApproveChanges (approve code reviews), \
        SubmitChanges (submit changes for review), Clone (clone repositories), CreateUser (create new users), \
        DisableUser (disable/enable users), DeleteUser (delete users), ManagePermissions (grant/revoke permissions), \
        and ManageApiKeys (manage API keys for other users). This operation requires the ManagePermissions permission."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "user_id".to_string(),
                description: "ID of the user to grant permission to".to_string(),
                required: true,
            },
            OperationParameter {
                name: "permission".to_string(),
                description:
                    "Permission to grant (e.g., 'ApproveChanges', 'Clone', 'ManagePermissions')"
                        .to_string(),
                required: true,
            },
        ]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Grant ApproveChanges permission".to_string(),
                moocode: r#"result = worker_request("vcs", {"user/add_permission", "alice", "ApproveChanges"});
// User 'alice' can now approve code reviews"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/api/user/add_permission \
  -H "Content-Type: application/json" \
  -d '{"operation": "user/add_permission", "args": ["alice", "ApproveChanges"]}'
"#.to_string()),
            },
            OperationExample {
                description: "Grant Clone permission".to_string(),
                moocode: r#"result = worker_request("vcs", {"user/add_permission", "bob", "Clone"});
// User 'bob' can now clone repositories"#.to_string(),
                http_curl: None,
            }
        ]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/user/add_permission".to_string(),
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
        info!(
            "Executing user/add_permission operation for user: {}",
            user.id
        );

        // Check permission
        if !user.has_permission(&Permission::ManagePermissions) {
            error!(
                "User {} does not have ManagePermissions permission",
                user.id
            );
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: You do not have permission to manage permissions"),
            );
        }

        // Validate arguments
        if args.len() < 2 {
            error!(
                "Invalid arguments for user/add_permission: expected 2, got {}",
                args.len()
            );
            return moor_var::v_error(
                moor_var::E_INVARG.msg("Error: Expected 2 arguments: user_id, permission"),
            );
        }

        let target_user_id = &args[0];
        let permission_str = &args[1];

        // Parse permission
        let permission = match Self::parse_permission(permission_str) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid permission: {}", e);
                return moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")));
            }
        };

        // Add the permission
        match self
            .user_provider
            .add_permission(target_user_id, permission.clone())
        {
            Ok(()) => {
                info!(
                    "Added permission {:?} to user '{}'",
                    permission, target_user_id
                );
                moor_var::v_str(&format!(
                    "Successfully added permission '{permission_str}' to user '{target_user_id}'"
                ))
            }
            Err(e) => {
                error!("Failed to add permission: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}
