use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
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

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Returns the current user's permissions and information"
    }

    fn philosophy(&self) -> &'static str {
        "Provides information about the authenticated user making the request. This is useful for \
        verifying authentication, checking what permissions you have, and debugging authorization issues. \
        The operation returns user details including ID, email, associated MOO object, and a list of \
        granted permissions (e.g., SubmitChanges, ApproveChanges, Clone)."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Get current user information".to_string(),
            moocode: r#"user_info = worker_request("vcs", {"user/stat"});
// Returns: {user_id, email, v_obj, {permissions...}}
player:tell("User ID: ", user_info[1]);
player:tell("Email: ", user_info[2]);
player:tell("Permissions: ", tostr(user_info[4]));"#
                .to_string(),
            http_curl: Some(r#"curl -X GET http://localhost:8081/api/user/stat"#.to_string()),
        }]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/user/stat".to_string(),
            method: Method::GET,
            is_json: false,
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
