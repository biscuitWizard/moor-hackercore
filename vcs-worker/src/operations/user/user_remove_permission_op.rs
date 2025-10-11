use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use crate::providers::user::UserProvider;
use axum::http::Method;
use tracing::{error, info};
use std::sync::Arc;

use crate::types::{User, Permission};

/// Remove permission from user operation
#[derive(Clone)]
pub struct UserRemovePermissionOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserRemovePermissionOperation {
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
            "ManagePermissions" | "Manage_Permissions" => Ok(Permission::ManagePermissions),
            "ManageApiKeys" | "Manage_Api_Keys" => Ok(Permission::ManageApiKeys),
            _ => Err(format!("Unknown permission: {}", perm_str)),
        }
    }
}

impl Operation for UserRemovePermissionOperation {
    fn name(&self) -> &'static str {
        "user/remove_permission"
    }
    
    fn description(&self) -> &'static str {
        "Remove a permission from a user"
    }
    
    fn philosophy(&self) -> &'static str {
        "Revokes a specific permission from a user account. This removes the user's ability to perform \
        operations that require that permission. Use this to reduce a user's access level or remove \
        permissions that are no longer needed. This operation requires the ManagePermissions permission."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "user_id".to_string(),
                description: "ID of the user to revoke permission from".to_string(),
                required: true,
            },
            OperationParameter {
                name: "permission".to_string(),
                description: "Permission to revoke (e.g., 'ApproveChanges', 'Clone', 'ManagePermissions')".to_string(),
                required: true,
            }
        ]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Revoke ApproveChanges permission".to_string(),
                moocode: r#"result = worker_request("vcs", {"user/remove_permission", "alice", "ApproveChanges"});
// User 'alice' can no longer approve code reviews"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/api/user/remove_permission \
  -H "Content-Type: application/json" \
  -d '{"operation": "user/remove_permission", "args": ["alice", "ApproveChanges"]}'
"#.to_string()),
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/user/remove_permission".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Executing user/remove_permission operation for user: {}", user.id);
        
        // Check permission
        if !user.has_permission(&Permission::ManagePermissions) {
            error!("User {} does not have ManagePermissions permission", user.id);
            return moor_var::v_str("Error: You do not have permission to manage permissions");
        }
        
        // Validate arguments
        if args.len() < 2 {
            error!("Invalid arguments for user/remove_permission: expected 2, got {}", args.len());
            return moor_var::v_str("Error: Expected 2 arguments: user_id, permission");
        }
        
        let target_user_id = &args[0];
        let permission_str = &args[1];
        
        // Parse permission
        let permission = match Self::parse_permission(permission_str) {
            Ok(p) => p,
            Err(e) => {
                error!("Invalid permission: {}", e);
                return moor_var::v_str(&format!("Error: {}", e));
            }
        };
        
        // Remove the permission
        match self.user_provider.remove_permission(target_user_id, &permission) {
            Ok(removed) => {
                if removed {
                    info!("Removed permission {:?} from user '{}'", permission, target_user_id);
                    moor_var::v_str(&format!("Successfully removed permission '{}' from user '{}'", 
                                            permission_str, target_user_id))
                } else {
                    info!("User '{}' did not have permission {:?}", target_user_id, permission);
                    moor_var::v_str(&format!("User '{}' did not have permission '{}'", 
                                            target_user_id, permission_str))
                }
            }
            Err(e) => {
                error!("Failed to remove permission: {}", e);
                moor_var::v_str(&format!("Error: {}", e))
            }
        }
    }
}

