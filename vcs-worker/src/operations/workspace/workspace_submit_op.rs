use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{Change, Permission, User};
use moor_var::{E_INVARG, v_error, v_str};

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
    fn process_workspace_submit(
        &self,
        serialized_change: &str,
        user: &User,
    ) -> Result<String, ObjectsTreeError> {
        // Check if user has permission to submit changes
        if !user.has_permission(&Permission::SubmitChanges) {
            error!(
                "User '{}' does not have permission to submit changes",
                user.id
            );
            return Err(ObjectsTreeError::SerializationError(format!(
                "User '{}' does not have permission to submit changes",
                user.id
            )));
        }

        // Deserialize the change from the provided string
        let change: Change = serde_json::from_str(serialized_change).map_err(|e| {
            ObjectsTreeError::SerializationError(format!("Failed to deserialize change: {e}"))
        })?;

        info!(
            "User '{}' submitting change for review: {} ({})",
            user.id, change.name, change.id
        );

        // Store the change in the workspace
        self.database
            .workspace()
            .store_workspace_change(&change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        info!(
            "Successfully stored change '{}' in workspace for review",
            change.name
        );

        Ok(format!(
            "Change '{}' ({}) successfully submitted for review",
            change.name, change.id
        ))
    }
}

impl Operation for WorkspaceSubmitOperation {
    fn name(&self) -> &'static str {
        "workspace/submit"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Submits a serialized change to the workspace for review. Takes a JSON-serialized Change object and stores it in the workspace."
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/workspace/submit".to_string(),
            method: Method::PUT,
            is_json: true,
        }]
    }

    fn philosophy(&self) -> &'static str {
        "Submits a serialized change to the workspace for review and approval. This operation accepts a JSON-serialized \
        Change object and stores it in the workspace where it can be reviewed by users with approval permissions. \
        This operation requires the SubmitChanges permission and is typically used after making local changes that need \
        to be shared with other developers or submitted to the main repository."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Submit a serialized change for review".to_string(),
            moocode: r#"// First, serialize your local change to JSON
change_json = worker_request("vcs", {"serialize_change", change_id});
// Then submit it to the workspace for review
result = worker_request("vcs", {"workspace/submit", change_json});
// Returns: "Change 'my-feature' (change-abc123) successfully submitted for review""#
                .to_string(),
            http_curl: Some(
                r#"curl -X PUT http://localhost:8081/api/workspace/submit \
  -H "Content-Type: application/json" \
  -d '{"id":"abc123...","name":"my-feature","description":"Added new login system",...}'"#
                    .to_string(),
            ),
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#""Change 'my-feature' (change-abc123) successfully submitted for review""#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - No serialized change argument provided",
                r#"E_INVARG("Workspace submit operation requires a serialized change argument")"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Failed to deserialize change",
                r#"E_INVARG("Failed to deserialize change: invalid JSON at line 1 column 5")"#,
            ),
            OperationResponse::new(
                403,
                "Forbidden - User lacks permission to submit changes",
                r#"E_INVARG("User 'player123' does not have permission to submit changes")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database error",
                r#"E_INVARG("Database error: failed to store workspace change")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!(
            "Workspace submit operation received {} arguments for user: {}",
            args.len(),
            user.id
        );

        if args.is_empty() {
            error!("Workspace submit operation requires a serialized change argument");
            return v_error(
                E_INVARG.msg("Workspace submit operation requires a serialized change argument"),
            );
        }

        let serialized_change = &args[0];

        match self.process_workspace_submit(serialized_change, user) {
            Ok(message) => {
                info!("Workspace submit operation completed successfully");
                v_str(&message)
            }
            Err(e) => {
                error!("Workspace submit operation failed: {}", e);
                v_error(E_INVARG.msg(format!("{e}")))
            }
        }
    }
}
