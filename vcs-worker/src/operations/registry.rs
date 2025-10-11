use std::collections::HashMap;
use tracing::{error, info};

use super::{Operation, OperationRoute};
use crate::types::{OperationRequest, OperationResponse};
use crate::providers::user::UserProvider;

/// Registry that holds all registered operations
#[derive(Default)]
pub struct OperationRegistry {
    operations: HashMap<String, Box<dyn Operation>>,
    user_provider: Option<std::sync::Arc<dyn UserProvider>>,
}

impl OperationRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set the user provider for the registry
    pub fn set_user_provider(&mut self, user_provider: std::sync::Arc<dyn UserProvider>) {
        self.user_provider = Some(user_provider);
    }
    
    /// Register a new operation
    pub fn register<O: Operation + 'static>(&mut self, operation: O) {
        let name = operation.name().to_string();
        info!("Registering operation: {}", name);
        self.operations.insert(name, Box::new(operation));
    }
    
    /// Check if an operation is registered
    #[allow(dead_code)]
    pub fn is_registered(&self, name: &str) -> bool {
        self.operations.contains_key(name)
    }
    
    /// List all registered operation names
    pub fn list_operations(&self) -> Vec<String> {
        self.operations.keys().cloned().collect()
    }
    
    /// Execute an operation by name and return a moor Var
    pub fn execute_var(&self, request: OperationRequest) -> moor_var::Var {
        let op_name = request.operation.clone();
        
        // Get the Wizard user for operations (has all permissions by default)
        let user = match &self.user_provider {
            Some(provider) => {
                match provider.get_wizard_user() {
                    Ok(user) => user,
                    Err(e) => {
                        error!("Failed to get Wizard user: {}", e);
                        return moor_var::v_str("Internal error: Unable to get user context");
                    }
                }
            }
            None => {
                error!("No user provider configured");
                return moor_var::v_str("Internal error: No user provider configured");
            }
        };
        
        match self.operations.get(&op_name) {
            Some(operation) => {
                info!("Executing operation: {} with {} args for user: {}", op_name, request.args.len(), user.id);
                operation.execute(request.args, &user)
            }
            None => {
                error!("Operation '{}' not found", op_name);
                moor_var::v_str(&format!("Operation '{op_name}' not found"))
            }
        }
    }
    
    /// Execute an operation by name and return an HTTP response with JSON
    pub fn execute_http(&self, request: OperationRequest) -> OperationResponse {
        let var_result = self.execute_var(request.clone());
        let operation_name = request.operation;
        
        // Convert moor Var to JSON Value
        let result_json = var_to_json_value(var_result.clone());
        
        OperationResponse {
            result: result_json,
            success: true, // For simplicity, we assume operations succeed
            operation: operation_name,
        }
    }
    
    /// Get all HTTP routes from registered operations
    pub fn get_all_routes(&self) -> Vec<(OperationRoute, String)> {
        let mut routes = Vec::new();
        for op_name in self.operations.keys() {
            if let Some(operation) = self.operations.get(op_name) {
                for route in operation.routes() {
                    routes.push((route, op_name.clone()));
                }
            }
        }
        routes
    }
    
    /// Get the description of an operation by name
    pub fn get_operation_description(&self, name: &str) -> Option<&'static str> {
        self.operations.get(name).map(|op| op.description())
    }
    
    /// Get an operation by name (returns a reference to the boxed trait object)
    pub fn get_operation(&self, name: &str) -> Option<&dyn Operation> {
        self.operations.get(name).map(|b| &**b)
    }
}

/// Convert a moor Var to a JSON Value for HTTP responses
pub fn var_to_json_value(var: moor_var::Var) -> serde_json::Value {
    use serde_json::{json, Value};
    
    // Handle different Var types properly
    if let Some(str_val) = var.as_string() {
        Value::String(str_val.to_string())
    } else if let Some(int_val) = var.as_integer() {
        json!(int_val)
    } else if let Some(float_val) = var.as_float() {
        json!(float_val)
    } else if let Some(bool_val) = var.as_bool() {
        Value::Bool(bool_val)
    } else if let Some(obj_val) = var.as_object() {
        Value::String(format!("#{}", obj_val.id()))
    } else if let Some(list) = var.as_list() {
        // Recursively convert list elements
        let items: Vec<Value> = list.iter().map(|v| var_to_json_value(v.clone())).collect();
        Value::Array(items)
    } else if let Some(map) = var.as_map() {
        // Convert map to JSON object with string keys
        // If keys are not strings, convert them to strings
        let mut obj = serde_json::Map::new();
        for (k, v) in map.iter() {
            let key_str = if let Some(s) = k.as_string() {
                s.to_string()
            } else {
                // Convert non-string keys to their debug representation
                format!("{k:?}")
            };
            obj.insert(key_str, var_to_json_value(v.clone()));
        }
        Value::Object(obj)
    } else if let Some(err) = var.as_error() {
        Value::String(format!("Error: {}", err.message()))
    } else if var.is_none() {
        Value::Null
    } else {
        // For other types (Symbol, Binary, Lambda, etc.), convert to debug string
        Value::String(format!("{var:?}"))
    }
}