use axum::{response::Json, routing::{get, post}, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use utoipa::OpenApi;
use utoipa::openapi::{
    PathsBuilder, InfoBuilder, ResponseBuilder, ContentBuilder, RefOr,
    path::{PathItemBuilder, OperationBuilder},
    request_body::RequestBodyBuilder,
    HttpMethod,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::operations::{OperationRegistry, OperationRequest};
use crate::types::{HttpRequest, OperationResponse};

// Import moor types for RPC
use moor_var::{Obj, Symbol, Var, v_str};
use moor_common::tasks::WorkerError;
use rpc_common::WorkerToken;
use uuid::Uuid;

/// Type alias for RPC handler function signature to reduce complexity
type RpcHandler = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Var, WorkerError>> + Send + Sync + 'static>>;

/// Base OpenAPI documentation for VCS Worker API
#[derive(OpenApi)]
#[openapi(
    components(
        schemas(HttpRequest, OperationResponse)
    )
)]
struct BaseApiDoc;

/// Generate OpenAPI spec dynamically from registered operations
fn generate_openapi_spec(registry: &OperationRegistry) -> utoipa::openapi::OpenApi {
    let mut openapi = BaseApiDoc::openapi();
    
    // Set API info
    openapi.info = InfoBuilder::new()
        .title("VCS Worker API")
        .version("0.9.0-alpha")
        .description(Some("A worker for handling version control of MOO entities"))
        .build();
    
    // Add tags with descriptions for each category
    openapi.tags = Some(vec![
        utoipa::openapi::tag::TagBuilder::new()
            .name("object")
            .description(Some("Object operations for managing MOO object definitions in version control. These operations handle retrieving, updating, renaming, deleting, and listing objects."))
            .build(),
        utoipa::openapi::tag::TagBuilder::new()
            .name("change")
            .description(Some("Change operations for managing changelists in the VCS workflow. Changes are the fundamental unit of work organization, similar to branches in git but lighter weight. Use these to create, switch, submit, approve, and manage your work."))
            .build(),
        utoipa::openapi::tag::TagBuilder::new()
            .name("index")
            .description(Some("Index operations for managing the version control index. The index tracks the current state of the repository and maintains the history of changes."))
            .build(),
        utoipa::openapi::tag::TagBuilder::new()
            .name("workspace")
            .description(Some("Workspace operations for managing saved changes. The workspace stores changes that are not currently active, including stashed changes, changes awaiting review, and idle changes."))
            .build(),
        utoipa::openapi::tag::TagBuilder::new()
            .name("meta")
            .description(Some("Meta operations for configuring object filtering rules. Use these to specify which properties and verbs should be ignored when storing objects in version control."))
            .build(),
        utoipa::openapi::tag::TagBuilder::new()
            .name("user")
            .description(Some("User operations for authentication and permission management. Use these to check current user status and permissions."))
            .build(),
        utoipa::openapi::tag::TagBuilder::new()
            .name("system")
            .description(Some("System-level operations including repository cloning and basic connectivity tests."))
            .build(),
    ]);
    
    let mut paths = PathsBuilder::new();
    
    // Add the generic RPC endpoint
    let rpc_op = OperationBuilder::new()
        .tag("vcs-worker")
        .summary(Some("Generic RPC endpoint"))
        .description(Some("Execute any registered operation by name"))
        .request_body(Some(
            RequestBodyBuilder::new()
                .content("application/json", ContentBuilder::new()
                    .schema(Some(RefOr::Ref(utoipa::openapi::Ref::from_schema_name("HttpRequest"))))
                    .build())
                .required(Some(utoipa::openapi::Required::True))
                .build()
        ))
        .response("200", ResponseBuilder::new()
            .description("Operation executed successfully")
            .content("application/json", ContentBuilder::new()
                .schema(Some(RefOr::Ref(utoipa::openapi::Ref::from_schema_name("OperationResponse"))))
                .build())
            .build())
        .build();
    
    paths = paths.path("/rpc", PathItemBuilder::new()
        .operation(HttpMethod::Post, rpc_op)
        .build());
    
    // Add dynamic routes from operations
    let mut operation_routes: std::collections::HashMap<String, Vec<(String, axum::http::Method, String)>> = std::collections::HashMap::new();
    
    for (route, op_name) in registry.get_all_routes() {
        let operations_list = registry.list_operations();
        if let Some(operation) = operations_list.iter().find(|&name| name == &op_name) {
            if let Some(desc) = registry.get_operation_description(operation) {
                operation_routes.entry(route.path.clone())
                    .or_insert_with(Vec::new)
                    .push((op_name.clone(), route.method.clone(), desc.to_string()));
            }
        }
    }
    
    for (path, ops) in operation_routes {
        let mut path_item = PathItemBuilder::new();
        
        for (op_name, method, description) in ops {
            // Get the full operation to extract detailed documentation
            let operation_opt = registry.get_operation(&op_name);
            
            // Build comprehensive description with philosophy, parameters, and examples
            let mut full_description = description.clone();
            
            // Determine tag/category from operation name (e.g., "object/get" -> "object")
            let tag = if op_name.contains('/') {
                op_name.split('/').next().unwrap_or("system")
            } else {
                "system"
            };
            
            if let Some(op) = operation_opt {
                // Add philosophy section
                let philosophy = op.philosophy();
                if !philosophy.is_empty() {
                    full_description.push_str("\n\n## Philosophy\n\n");
                    full_description.push_str(philosophy);
                }
                
                // Add parameters section
                let params = op.parameters();
                if !params.is_empty() {
                    full_description.push_str("\n\n## Parameters\n\n");
                    for param in params {
                        full_description.push_str(&format!(
                            "- **{}** {}: {}\n",
                            param.name,
                            if param.required { "(required)" } else { "(optional)" },
                            param.description
                        ));
                    }
                }
                
                // Add examples section
                let examples = op.examples();
                if !examples.is_empty() {
                    full_description.push_str("\n\n## Examples\n\n");
                    for example in examples {
                        full_description.push_str(&format!("### {}\n\n", example.description));
                        full_description.push_str("**MOOCode:**\n```moo\n");
                        full_description.push_str(&example.moocode);
                        full_description.push_str("\n```\n\n");
                        
                        if let Some(ref curl) = example.http_curl {
                            full_description.push_str("**HTTP (curl):**\n```bash\n");
                            full_description.push_str(curl);
                            full_description.push_str("\n```\n\n");
                        }
                    }
                }
            }
            
            let operation = OperationBuilder::new()
                .tag(tag)
                .operation_id(Some(op_name.replace("/", "_")))
                .summary(Some(op_name.clone()))
                .description(Some(full_description))
                .response("200", ResponseBuilder::new()
                    .description("Operation executed successfully")
                    .content("application/json", ContentBuilder::new()
                        .schema(Some(RefOr::Ref(utoipa::openapi::Ref::from_schema_name("OperationResponse"))))
                        .build())
                    .build())
                .build();
            
            let http_method = match method.as_str() {
                "GET" => HttpMethod::Get,
                "POST" => HttpMethod::Post,
                "PUT" => HttpMethod::Put,
                "DELETE" => HttpMethod::Delete,
                _ => continue,
            };
            
            path_item = path_item.operation(http_method, operation);
        }
        
        paths = paths.path(path, path_item.build());
    }
    
    openapi.paths = paths.build();
    openapi
}

/// Generic RPC endpoint handler
async fn rpc_handler(
    registry: Arc<OperationRegistry>,
    Json(payload): Json<HttpRequest>
) -> Json<OperationResponse> {
    let request = OperationRequest {
        operation: payload.operation,
        args: payload.args,
    };
    Json(registry.execute_http(request))
}

/// Create the HTTP router from registered operations.
/// Automatically generates routes from the operation definitions in the registry.
pub fn create_http_router(registry: Arc<OperationRegistry>) -> Router {
    let mut api_router = Router::new()
        .route("/rpc", post({
            let registry = registry.clone();
            move |payload| rpc_handler(registry.clone(), payload)
        }));
    
    // Dynamically add routes from registered operations
    for (route, op_name) in registry.get_all_routes() {
        let registry_for_route = registry.clone();
        let operation_name = op_name.clone();
        
        match route.method {
            axum::http::Method::GET => {
                api_router = api_router.route(&route.path, get({
                    let registry = registry_for_route.clone();
                    let op_name = operation_name.clone();
                    move || {
                        let registry = registry.clone();
                        let op_name = op_name.clone();
                        async move {
                            let request = OperationRequest {
                                operation: op_name,
                                args: vec![],
                            };
                            Json(registry.execute_http(request))
                        }
                    }
                }));
            }
            axum::http::Method::POST => {
                if route.is_json {
                    // For JSON POST requests, use the generic RPC handler
                    api_router = api_router.route(&route.path, post({
                        let registry = registry_for_route.clone();
                        let op_name = operation_name.clone();
                        move |Json(payload): Json<HttpRequest>| {
                            let registry = registry.clone();
                            let op_name = op_name.clone();
                            async move {
                                let request = OperationRequest {
                                    operation: op_name,
                                    args: payload.args,
                                };
                                Json(registry.execute_http(request))
                            }
                        }
                    }));
                }
            }
            _ => {
                tracing::warn!("Unsupported HTTP method for route: {} {:?}", route.path, route.method);
            }
        }
    }
    
    // Generate dynamic OpenAPI spec from operations
    let openapi_spec = generate_openapi_spec(&registry);
    
    // Apply state to API router first, then merge with Swagger UI
    let api_router = api_router.with_state(registry);
    
    // Create the final router with Swagger UI (no state needed for Swagger routes)
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui")
            .url("/api-docs/openapi.json", openapi_spec))
        .merge(api_router)
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