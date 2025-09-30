use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::info;

use crate::types::User;

/// Stat operation that returns the current user's permissions and information
#[derive(Clone)]
pub struct StatOperation;

impl Operation for StatOperation {
    fn name(&self) -> &'static str {
        "user/stat"
    }
    
    fn description(&self) -> &'static str {
        "Returns the current user's permissions and information"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/user/stat".to_string(),
                method: Method::GET,
                is_json: false, // No body needed
            },
            OperationRoute {
                path: "/api/user/stat".to_string(),
                method: Method::GET,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, _args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Executing user/stat operation for user: {}", user.id);
        
        // Build a list representing the user information
        // Format: [id, email, v_obj, permissions]
        
        // Create permissions list
        let mut permissions_list = Vec::new();
        for permission in &user.permissions {
            permissions_list.push(moor_var::v_str(&permission.to_string()));
        }
        
        // Build the result as a list containing:
        // [user_id, email, v_obj, [permissions...]]
        let result = moor_var::v_list(&[
            moor_var::v_str(&user.id),
            moor_var::v_str(&user.email),
            moor_var::v_obj(user.v_obj),
            moor_var::v_list(&permissions_list),
        ]);
        
        info!("User stat operation completed successfully");
        result
    }
}
