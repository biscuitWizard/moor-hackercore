use axum::{response::Json, routing::{get, post}, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use crate::operations::{OperationRegistry, OperationRequest};
use crate::types::{ObjectUpdateRequest, ObjectGetRequest};

// Import moor types for RPC
use moor_var::{Obj, Symbol, Var, v_str};
use moor_common::tasks::WorkerError;
use rpc_common::WorkerToken;
use uuid::Uuid;

/// Type alias for RPC handler function signature to reduce complexity
type RpcHandler = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Var, WorkerError>> + Send + Sync + 'static>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub operation: String,
    pub args: Vec<String>,
}

/// Create the HTTP router from registered operations.
/// Both endpoints access the same operation registry:
/// - GET /hello calls the "hello" operation directly 
/// - POST /rpc accepts any operation name via JSON body
pub fn create_http_router(registry: Arc<OperationRegistry>) -> Router {
    let registry = registry.clone();
    
    Router::new()
        .route("/rpc", post({
            let registry = registry.clone();
            // Generic endpoint - accepts any operation name from registered operations
            move |Json(payload): Json<HttpRequest>| async move {
                let request = OperationRequest {
                    operation: payload.operation,
                    args: payload.args,
                };
                Json(registry.execute_http(request))
            }
        }))
        .route("/object/update", post({
            let registry = registry.clone();
            // Specific endpoint for object update operations
            move |Json(payload): Json<ObjectUpdateRequest>| async move {
                let request = OperationRequest {
                    operation: "object/update".to_string(),
                    args: vec![payload.object_name.clone()].into_iter().chain(payload.vars).collect(),
                };
                Json(registry.execute_http(request))
            }
        }))
        .route("/api/object/update", post({
            let registry = registry.clone();
            // API endpoint for object update operations
            move |Json(payload): Json<ObjectUpdateRequest>| async move {
                let request = OperationRequest {
                    operation: "object/update".to_string(),
                    args: vec![payload.object_name.clone()].into_iter().chain(payload.vars).collect(),
                };
                Json(registry.execute_http(request))
            }
        }))
        .route("/object/get", post({
            let registry = registry.clone();
            // Specific endpoint for object get operations
            move |Json(payload): Json<ObjectGetRequest>| async move {
                let request = OperationRequest {
                    operation: "object/get".to_string(),
                    args: vec![payload.object_name],
                };
                Json(registry.execute_http(request))
            }
        }))
        .route("/api/object/get", post({
            let registry = registry.clone();
            // API endpoint for object get operations
            move |Json(payload): Json<ObjectGetRequest>| async move {
                let request = OperationRequest {
                    operation: "object/get".to_string(),
                    args: vec![payload.object_name],
                };
                Json(registry.execute_http(request))
            }
        }))
        .route("/hello", get({
            let registry = registry.clone();
            // Direct call to hello operation
            move || async move {
                let request = OperationRequest {
                    operation: "hello".to_string(),
                    args: vec![],
                };
                Json(registry.execute_http(request))
            }
        }))
        .with_state(registry)
}

/// Start the HTTP server
pub async fn start_http_server(
    address: &str,
    registry: Arc<OperationRegistry>,
) -> Result<(), eyre::Error> {
    let router = create_http_router(registry);
    
    let listener = TcpListener::bind(address).await?;
    info!("HTTP server listening on {}", address);
    
    axum::serve(listener, router.into_make_service())
        .await
        .map_err(|e| eyre::eyre!(e))?;
        
    Ok(())
}

/// RPC handler that converts RPC calls to operation requests
pub async fn process_rpc_request(
    registry: Arc<OperationRegistry>,
    _token: WorkerToken,
    _request_id: Uuid,
    _worker_type: Symbol,
    _perms: Obj,
    arguments: Vec<Var>,
    _timeout: Option<std::time::Duration>,
) -> Result<Var, WorkerError> {
    if arguments.is_empty() {
        return Ok(v_str("No arguments provided"));
    }

    // First argument should be the operation name
    let operation_name = match arguments[0].as_string() {
        Some(name) => name.to_string(),
        None => {
            return Ok(v_str("First argument must be a string (operation name)"));
        }
    };

    // Convert remaining arguments to strings
    let mut args = Vec::new();
    for (i, arg) in arguments[1..].iter().enumerate() {
        if let Some(s) = arg.as_string() {
            args.push(s.to_string());
            info!("RPC arg {}: string = '{}'", i + 1, s);
        } else if let Some(list) = arg.as_list() {
            // Convert list elements to strings and then to JSON array
            let mut string_list = Vec::new();
            for item in list.iter() {
                if let Some(s) = item.as_string() {
                    string_list.push(s.to_string());
                } else {
                    // Convert non-string items to string representation
                    string_list.push(format!("{item:?}"));
                }
            }
            let json_str = serde_json::to_string(&string_list).unwrap_or_else(|_| format!("{list:?}"));
            args.push(json_str.clone());
            info!("RPC arg {}: list with {} items converted to JSON = '{}'", i + 1, string_list.len(), json_str);
        } else {
            // Convert other types to string representation
            let repr = format!("{arg:?}");
            args.push(repr.clone());
            info!("RPC arg {}: other type = '{}'", i + 1, repr);
        }
    }

    // Create operation request
    let request = OperationRequest {
        operation: operation_name,
        args,
    };

    // Execute the operation and return the result as a Var
    Ok(registry.execute_var(request))
}

/// Create a handler closure that can be used with the RPC worker loop
pub fn create_rpc_handler(
    registry: Arc<OperationRegistry>,
) -> impl Fn(
        WorkerToken,
        Uuid,
        Symbol,
        Obj,
        Vec<Var>,
        Option<std::time::Duration>,
    ) -> RpcHandler
           + Clone {
    move |token, request_id, worker_type, perms, arguments, timeout| {
        let registry = registry.clone();
        Box::pin(process_rpc_request(
            registry,
            token,
            request_id,
            worker_type,
            perms,
            arguments,
            timeout,
        ))
    }
}