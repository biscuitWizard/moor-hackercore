use super::{Operation, OperationRoute};
use axum::http::Method;

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
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/hello".to_string(),
                method: Method::GET,
                is_json: false, // No body needed
            },
            OperationRoute {
                path: "/api/hello".to_string(),
                method: Method::GET,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, _args: Vec<String>) -> moor_var::Var {
        tracing::info!("Executing hello operation");
        moor_var::v_str("goodbye")
    }
}
