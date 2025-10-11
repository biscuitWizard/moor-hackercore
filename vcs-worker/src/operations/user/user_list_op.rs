use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use crate::providers::user::UserProvider;
use axum::http::Method;
use tracing::{error, info};
use std::sync::Arc;

use crate::types::{User, Permission};

/// List users operation
#[derive(Clone)]
pub struct UserListOperation {
    user_provider: Arc<dyn UserProvider>,
}

impl UserListOperation {
    pub fn new(user_provider: Arc<dyn UserProvider>) -> Self {
        Self { user_provider }
    }
}

impl Operation for UserListOperation {
    fn name(&self) -> &'static str {
        "user/list"
    }
    
    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "List all users in the system with their permissions"
    }
    
    fn philosophy(&self) -> &'static str {
        "Lists all user accounts in the system, including their IDs, emails, MOO objects, permissions, \
        and status (enabled/disabled, system user). This is useful for auditing user access and managing \
        permissions across the system. This operation requires the ManagePermissions permission as it \
        exposes sensitive user information."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "List all users".to_string(),
                moocode: r#"users = worker_request("vcs", {"user/list"});
// Returns list of users: {user_id, email, v_obj, {permissions}, is_disabled, is_system_user}
for user in (users)
    player:tell("User: ", user[1], " Email: ", user[2], " Permissions: ", tostr(user[4]));
endfor"#.to_string(),
                http_curl: Some(r#"curl -X GET http://localhost:8081/api/user/list"#.to_string()),
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/user/list".to_string(),
                method: Method::GET,
                is_json: false,
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

    fn execute(&self, _args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Executing user/list operation for user: {}", user.id);
        
        // Check permission
        if !user.has_permission(&Permission::ManagePermissions) {
            error!("User {} does not have ManagePermissions permission", user.id);
            return moor_var::v_error(moor_var::E_INVARG.msg("Error: You do not have permission to list users"));
        }
        
        // Get all users
        match self.user_provider.list_users() {
            Ok(users) => {
                info!("Retrieved {} users", users.len());
                
                // Build list of user information
                let mut user_list = Vec::new();
                for u in users {
                    // Create permissions list
                    let mut permissions_list = Vec::new();
                    for permission in &u.permissions {
                        permissions_list.push(moor_var::v_str(&permission.to_string()));
                    }
                    
                    // Build user entry: [user_id, email, v_obj, [permissions], is_disabled, is_system_user]
                    let user_entry = moor_var::v_list(&[
                        moor_var::v_str(&u.id),
                        moor_var::v_str(&u.email),
                        moor_var::v_obj(u.v_obj),
                        moor_var::v_list(&permissions_list),
                        moor_var::v_int(if u.is_disabled { 1 } else { 0 }),
                        moor_var::v_int(if u.is_system_user { 1 } else { 0 }),
                    ]);
                    
                    user_list.push(user_entry);
                }
                
                // Return list of users
                moor_var::v_list(&user_list)
            }
            Err(e) => {
                error!("Failed to list users: {}", e);
                moor_var::v_error(moor_var::E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}

