use axum::{response::{Json, Redirect}, routing::{get, post}, Router};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use utoipa::OpenApi;
use serde_json;
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
        .description(Some(r#"A worker for handling version control of MOO entities.

## Overview

The VCS Worker provides version control capabilities for MOO (Multi-User Object Oriented) environments. Unlike traditional RESTful APIs, this service operates as an **RPC (Remote Procedure Call) system** where POST requests execute specific operations by name.

## Key Concepts

### What is a Change?

A **change** is a logical unit of work containing a set of modifications to MOO objects. Think of it as a commit or a feature branch in traditional version control systems. A change can include:

- **Added objects**: New MOO objects being introduced
- **Modified objects**: Existing MOO objects being updated
- **Deleted objects**: MOO objects being removed
- **Renamed objects**: MOO objects being given new names

Typically, a change represents a complete feature or fix, such as:
- A bug fix
- A new game system
- A rebalance
- A refactoring effort

### What is a Changelist (History)?

The **changelist** (also called the **working set** or **working index**) is the chronological order of changes that have been applied to the repository. It represents the complete history of modifications, showing how the repository evolved over time.

### What is an Index?

The **index** is a compilation of all changes in the repository, including:
- Changes that have been merged (permanent history)
- Changes that are currently being worked on (local changes)
- Changes awaiting review (submitted but not yet approved)

The index maintains the state of the repository and tracks which changes are active.

### Change States

A change can be in one of several states:

- **Local**: Currently being worked on. This is your active changelist where you're making modifications. Only one change can be Local at a time.

- **Idle**: Saved for later. The change has been stashed or set aside, preserving your work without it being active. Use `change/switch` to resume an Idle change.

- **Review**: Submitted for approval. The change has been submitted (typically to a remote repository) and is awaiting review and approval before being merged.

- **Merged**: Permanently committed to history. The change has been approved and is now part of the repository's permanent record. Merged changes cannot be modified or abandoned.

### Workspace

The **workspace** is where non-active changes are stored, including:
- Idle changes (stashed work)
- Changes in Review (awaiting approval)
- Recently merged changes (for reference)

Use workspace operations to manage these changes.

## API Architecture: RPC, Not REST

This API follows an **RPC (Remote Procedure Call)** pattern rather than traditional REST:

### RPC Request Structure

All operations use a consistent request format:

```json
{
  "operation": "operation/name",
  "args": ["arg1", "arg2", ...]
}
```

**Example:**
```json
{
  "operation": "object/update",
  "args": ["$player", ["obj $player", "parent #1", "..."]]
}
```

### From MOOCode

When calling from MOO, use the `worker_request` function:

```moo
result = worker_request("vcs", {"operation/name", arg1, arg2, ...});
```

**Example:**
```moo
result = worker_request("vcs", {"object/update", "$player", objdef_lines});
```

## Typical Workflow

1. **Create a change**: `change/create` - Start a new changelist for your feature
2. **Modify objects**: `object/update`, `object/rename`, `object/delete` - Make your changes
3. **Check status**: `change/status` - Review what you've changed
4. **Submit**: `change/submit` - Submit for review (remote) or merge (local)
5. **Switch contexts**: `change/switch` or `change/stash` - Work on multiple features

## Getting Started

1. **Check your permissions**: `user/stat` - Verify you have the necessary permissions
2. **Create your first change**: `change/create` with a descriptive name
3. **Start modifying objects**: Use `object/update` to save object definitions
4. **Review your work**: Use `change/status` to see pending changes
5. **Submit when ready**: Use `change/submit` to finalize

For more details on each operation, see the categorized endpoints below."#))
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
            
            // Build request body with parameter schema if operation has parameters
            let mut request_body_builder = None;
            
            if let Some(op) = operation_opt.as_ref() {
                let params = op.parameters();
                if !params.is_empty() && method != "GET" {
                    // Build a JSON schema example for the request body
                    let mut example_obj = serde_json::Map::new();
                    example_obj.insert("operation".to_string(), serde_json::json!(op_name));
                    
                    let mut args_example = Vec::new();
                    for param in &params {
                        if param.required {
                            // Add a placeholder for required params
                            args_example.push(format!("<{}>", param.name));
                        }
                    }
                    example_obj.insert("args".to_string(), serde_json::json!(args_example));
                    
                    let example_json = serde_json::to_string_pretty(&example_obj).unwrap_or_default();
                    
                    request_body_builder = Some(
                        RequestBodyBuilder::new()
                            .description(Some(format!(
                                "Request body with operation name and arguments.\n\nParameters:\n{}",
                                params.iter()
                                    .map(|p| format!("- **{}** {}: {}", 
                                        p.name,
                                        if p.required { "(required)" } else { "(optional)" },
                                        p.description))
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            )))
                            .content("application/json", ContentBuilder::new()
                                .example(Some(serde_json::from_str(&example_json).unwrap_or(serde_json::json!({}))))
                                .schema(Some(RefOr::Ref(utoipa::openapi::Ref::from_schema_name("HttpRequest"))))
                                .build())
                            .required(Some(utoipa::openapi::Required::True))
                            .build()
                    );
                }
            }
            
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
            
            let mut operation_builder = OperationBuilder::new()
                .tag(tag)
                .operation_id(Some(op_name.replace("/", "_")))
                .summary(Some(op_name.clone()))
                .description(Some(full_description))
                .response("200", ResponseBuilder::new()
                    .description("Operation executed successfully")
                    .content("application/json", ContentBuilder::new()
                        .schema(Some(RefOr::Ref(utoipa::openapi::Ref::from_schema_name("OperationResponse"))))
                        .build())
                    .build());
            
            // Add request body if we have one
            if let Some(request_body) = request_body_builder {
                operation_builder = operation_builder.request_body(Some(request_body));
            }
            
            let operation = operation_builder.build();
            
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
        .route("/", get(|| async { Redirect::permanent("/swagger-ui") }))
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