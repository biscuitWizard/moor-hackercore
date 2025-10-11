use super::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use crate::types::User;

/// Simple hello operation implementation
#[derive(Clone)]
pub struct HelloOperation;

impl Operation for HelloOperation {
    fn name(&self) -> &'static str {
        "hello"
    }
    
    fn description(&self) -> &'static str {
        "A simple greeting operation that returns goodbye"
    }
    
    fn philosophy(&self) -> &'static str {
        "This is a test operation used to verify that the VCS worker is running and accessible. \
        It demonstrates the basic request-response pattern for worker operations without making \
        any changes to the version control system."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Basic hello operation call".to_string(),
                moocode: r#"result = worker_request("vcs", {"hello"});
// Returns: "goodbye""#.to_string(),
                http_curl: Some(r#"curl -X GET http://localhost:8081/api/hello"#.to_string()),
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/hello".to_string(),
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
                r#""Error: Invalid operation arguments""#
            ),
            OperationResponse::new(
                404,
                "Not Found - Resource not found",
                r#""Error: Resource not found""#
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#""Error: Database error: operation failed""#
            ),
        ]
    }

    fn execute(&self, _args: Vec<String>, _user: &User) -> moor_var::Var {
        tracing::info!("Executing hello operation");
        moor_var::v_str("goodbye")
    }
}
