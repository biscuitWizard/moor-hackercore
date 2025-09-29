mod registry;
mod hello_op;
mod change;
mod object;
mod index;

pub use registry::OperationRegistry;
pub use hello_op::HelloOperation;
pub use change::{ChangeCreateOperation, ChangeAbandonOperation, ChangeStatusOperation};
pub use object::{ObjectGetOperation, ObjectUpdateOperation, ObjectRenameOperation, ObjectDeleteOperation, ObjectListOperation};
pub use index::IndexListOperation;

// Re-export common types from crate::types
pub use crate::types::{OperationRequest, OperationResponse};

use axum::http::Method;
use std::sync::Arc;

use crate::config::Config;
use crate::database::{Database, DatabaseRef, ObjectsTreeError};

#[derive(Debug, Clone)]
pub struct OperationRoute {
    #[allow(dead_code)]
    pub path: String,
    #[allow(dead_code)]
    pub method: Method,
    #[allow(dead_code)]
    pub is_json: bool, // Whether to expect JSON body vs query params
}

pub trait Operation: Send + Sync {
    /// The name of the operation (used for RPC)
    fn name(&self) -> &'static str;
    
    /// Description of what the operation does
    #[allow(dead_code)]
    fn description(&self) -> &'static str;
    
    /// HTTP routing information for this operation
    #[allow(dead_code)]
    fn routes(&self) -> Vec<OperationRoute>;
    
    /// Execute the operation with the given arguments, returning a moor Var
    fn execute(&self, args: Vec<String>) -> moor_var::Var;
}

/// Create the default registry with built-in operations
pub fn create_default_registry() -> Result<(OperationRegistry, DatabaseRef), ObjectsTreeError> {
    let mut registry = OperationRegistry::new();
    
    // Initialize config and database
    let config = Config::new();
    let database = Arc::new(Database::new(&config)?);
    
    // Register built-in operations
    registry.register(HelloOperation);
    registry.register(ObjectUpdateOperation::new(database.clone()));
    registry.register(ObjectGetOperation::new(database.clone()));
    registry.register(ObjectRenameOperation::new(database.clone()));
    registry.register(ObjectDeleteOperation::new(database.clone()));
    registry.register(ObjectListOperation::new(database.clone()));
    registry.register(ChangeCreateOperation::new(database.clone()));
    registry.register(ChangeAbandonOperation::new(database.clone()));
    registry.register(ChangeStatusOperation::new(database.clone()));
    registry.register(IndexListOperation::new(database.clone()));
    
    Ok((registry, database))
}
