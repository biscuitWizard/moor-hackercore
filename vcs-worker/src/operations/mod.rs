mod registry;
mod hello_op;
mod change;
mod object;
mod index;
mod clone_op;
mod user;
mod workspace;
mod meta;
mod system;

pub use registry::OperationRegistry;
pub use hello_op::HelloOperation;
pub use change::{ChangeCreateOperation, ChangeAbandonOperation, ChangeStatusOperation, ChangeApproveOperation, ChangeSubmitOperation, ChangeStashOperation, ChangeSwitchOperation};
pub use object::{ObjectGetOperation, ObjectUpdateOperation, ObjectRenameOperation, ObjectDeleteOperation, ObjectListOperation};
pub use index::{IndexListOperation, IndexCalcDeltaOperation, IndexUpdateOperation};
pub use clone_op::CloneOperation;
pub use user::StatOperation;
pub use workspace::{WorkspaceSubmitOperation, WorkspaceListOperation};
pub use meta::{MetaAddIgnoredPropertyOperation, MetaAddIgnoredVerbOperation, MetaRemoveIgnoredPropertyOperation, MetaRemoveIgnoredVerbOperation, MetaClearIgnoredPropertiesOperation, MetaClearIgnoredVerbsOperation};
pub use system::StatusOperation;

// Re-export common types from crate::types
pub use crate::types::OperationRequest;

use axum::http::Method;
use std::sync::Arc;

use crate::config::Config;
use crate::database::{Database, DatabaseRef, ObjectsTreeError};
use crate::types::User;
use crate::providers::user::UserProvider;

#[derive(Debug, Clone)]
pub struct OperationRoute {
    pub path: String,
    pub method: Method,
    pub is_json: bool, // Whether to expect JSON body vs query params
}

/// Describes a parameter for an operation
#[derive(Debug, Clone)]
pub struct OperationParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
}

/// Provides an example of how to use an operation
#[derive(Debug, Clone)]
pub struct OperationExample {
    pub description: String,
    pub moocode: String,
    pub http_curl: Option<String>,
}

pub trait Operation: Send + Sync {
    /// The name of the operation (used for RPC)
    fn name(&self) -> &'static str;
    
    /// Description of what the operation does
    fn description(&self) -> &'static str;
    
    /// Philosophy/intent of the operation and how it fits into the VCS workflow
    fn philosophy(&self) -> &'static str;
    
    /// Detailed parameter descriptions
    fn parameters(&self) -> Vec<OperationParameter>;
    
    /// Examples showing how to use this operation
    fn examples(&self) -> Vec<OperationExample>;
    
    /// HTTP routing information for this operation
    fn routes(&self) -> Vec<OperationRoute>;
    
    /// Execute the operation with the given arguments and user context, returning a moor Var
    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var;
}

/// Create the default registry with built-in operations
pub fn create_default_registry() -> Result<(OperationRegistry, DatabaseRef), ObjectsTreeError> {
    let config = Config::new();
    create_registry_with_config(config)
}

/// Create the default registry with a specific config (useful for testing)
pub fn create_registry_with_config(config: Config) -> Result<(OperationRegistry, DatabaseRef), ObjectsTreeError> {
    let mut registry = OperationRegistry::new();
    
    // Initialize database with provided config
    let database = Arc::new(Database::new(&config)?);
    
    // Set the user provider in the registry
    registry.set_user_provider(database.users().clone());
    
    // Ensure the Everyone user exists
    if let Err(e) = database.users().ensure_everyone_user() {
        tracing::warn!("Failed to ensure Everyone user exists: {}", e);
    }
    
    // Ensure the Wizard user exists with the configured API key
    if let Err(e) = database.users().ensure_wizard_user(config.wizard_api_key.clone()) {
        tracing::warn!("Failed to ensure Wizard user exists: {}", e);
    }
    
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
    registry.register(ChangeApproveOperation::new(database.clone()));
    registry.register(ChangeSubmitOperation::new(database.clone()));
    registry.register(ChangeStashOperation::new(database.clone()));
    registry.register(IndexListOperation::new(database.clone()));
    registry.register(IndexCalcDeltaOperation::new(database.clone()));
    registry.register(IndexUpdateOperation::new(database.clone()));
    registry.register(CloneOperation::new(database.clone()));
    registry.register(StatOperation);
    registry.register(WorkspaceSubmitOperation::new(database.clone()));
    registry.register(WorkspaceListOperation::new(database.clone()));
    registry.register(ChangeSwitchOperation::new(database.clone()));
    registry.register(MetaAddIgnoredPropertyOperation::new(database.clone()));
    registry.register(MetaAddIgnoredVerbOperation::new(database.clone()));
    registry.register(MetaRemoveIgnoredPropertyOperation::new(database.clone()));
    registry.register(MetaRemoveIgnoredVerbOperation::new(database.clone()));
    registry.register(MetaClearIgnoredPropertiesOperation::new(database.clone()));
    registry.register(MetaClearIgnoredVerbsOperation::new(database.clone()));
    registry.register(StatusOperation::new(database.clone()));
    
    Ok((registry, database))
}
