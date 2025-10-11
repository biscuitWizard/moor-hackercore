use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{Change, User, Permission};
use moor_var::{v_error, v_str, E_INVARG};

/// Workspace submit operation that accepts a serialized change and stores it for review
#[derive(Clone)]
pub struct WorkspaceSubmitOperation {
    database: DatabaseRef,
}

impl WorkspaceSubmitOperation {
    /// Create a new workspace submit operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the workspace submit request with a serialized change
    fn process_workspace_submit(&self, serialized_change: &str, user: &User) -> Result<String, ObjectsTreeError> {
        // Check if user has permission to submit changes
        if !user.has_permission(&Permission::SubmitChanges) {
            error!("User '{}' does not have permission to submit changes", user.id);
            return Err(ObjectsTreeError::SerializationError(
                format!("User '{}' does not have permission to submit changes", user.id)
            ));
        }

        // Deserialize the change from the provided string
        let change: Change = serde_json::from_str(serialized_change)
            .map_err(|e| ObjectsTreeError::SerializationError(
                format!("Failed to deserialize change: {}", e)
            ))?;

        info!("User '{}' submitting change for review: {} ({})", user.id, change.name, change.id);

        // Store the change in the workspace
        self.database.workspace().store_workspace_change(&change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        info!("Successfully stored change '{}' in workspace for review", change.name);

        Ok(format!("Change '{}' ({}) successfully submitted for review", change.name, change.id))
    }
}

impl Operation for WorkspaceSubmitOperation {
    fn name(&self) -> &'static str {
        "workspace/submit"
    }
    
    fn description(&self) -> &'static str {
        "Submits a serialized change to the workspace for review. Takes a JSON-serialized Change object and stores it in the workspace."
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/workspace/submit".to_string(),
                method: Method::PUT,
                is_json: true,
            },
            OperationRoute {
                path: "/api/workspace/submit".to_string(),
                method: Method::PUT,
                is_json: true,
            }
        ]
    }
    
    fn philosophy(&self) -> &'static str {
        "Documentation for this operation is being prepared."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Workspace submit operation received {} arguments for user: {}", args.len(), user.id);
        
        if args.is_empty() {
            error!("Workspace submit operation requires a serialized change argument");
            return v_error(E_INVARG.msg("Workspace submit operation requires a serialized change argument"));
        }
        
        let serialized_change = &args[0];

        match self.process_workspace_submit(serialized_change, user) {
            Ok(message) => {
                info!("Workspace submit operation completed successfully");
                v_str(&message)
            }
            Err(e) => {
                error!("Workspace submit operation failed: {}", e);
                v_error(E_INVARG.msg(&format!("Error: {e}")))
            }
        }
    }
}
