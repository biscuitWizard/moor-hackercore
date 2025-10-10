use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info, warn};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{ChangeSubmitRequest, ChangeStatus, User, Permission};
use crate::object_diff::{ObjectDiffModel, build_abandon_diff_from_change, build_object_diff_from_change};
use moor_var::{v_error, E_INVARG};

/// Change submit operation that submits a local change for review
#[derive(Clone)]
pub struct ChangeSubmitOperation {
    database: DatabaseRef,
}

impl ChangeSubmitOperation {
    /// Create a new change submit operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change submit request
    fn process_change_submit(&self, request: ChangeSubmitRequest, user: &User) -> Result<ObjectDiffModel, ObjectsTreeError> {
        // Check if user has permission to submit changes
        if !user.has_permission(&Permission::SubmitChanges) {
            error!("User '{}' does not have permission to submit changes", user.id);
            return Err(ObjectsTreeError::SerializationError(
                format!("User '{}' does not have permission to submit changes", user.id)
            ));
        }

        // Get the top change from the index
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError("No change to submit".to_string()))?;

        // Get the change
        let mut change = self.database.index().get_change(&top_change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Change '{}' not found", top_change_id)))?;

        info!("User '{}' attempting to submit change: {} ({})", user.id, change.name, change.id);

        // Validate that author is set
        if change.author.is_empty() {
            error!("Cannot submit change '{}' - author is not set", change.name);
            return Err(ObjectsTreeError::SerializationError(
                "Cannot submit change - author is not set".to_string()
            ));
        }

        // Use provided message or existing description, both are optional
        let final_message = if let Some(request_message) = request.message {
            if request_message.trim().is_empty() {
                None
            } else {
                Some(request_message)
            }
        } else if let Some(existing_description) = &change.description {
            if existing_description.trim().is_empty() {
                None
            } else {
                Some(existing_description.clone())
            }
        } else {
            None
        };

        // Update the change description if a new message was provided
        if let Some(message) = &final_message {
            change.description = Some(message.clone());
        }

        // Check if the change is local
        if change.status != ChangeStatus::Local {
            error!("Cannot submit change '{}' ({}) - it is not local (status: {:?})", 
                   change.name, change.id, change.status);
            return Err(ObjectsTreeError::SerializationError(
                format!("Cannot submit change '{}' - it is not local (status: {:?})", change.name, change.status)
            ));
        }

        // Check if there's a source URL to determine the workflow
        let source_url = self.database.index().get_source()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        if let Some(url) = source_url {
            // REMOTE INDEX: Submit for review (existing behavior)
            info!("Source URL found: {}, submitting change for review", url);
            
            // Build the undo diff (like abandon does)
            let undo_diff = build_abandon_diff_from_change(&self.database, &change)?;

            // Change the status to Review (submitted, waiting for approval)
            change.status = ChangeStatus::Review;

            // Store the change in the workspace (where changes waiting for approval live)
            self.database.workspace().store_workspace_change(&change)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

            info!("Stored change '{}' in workspace with Review status", change.name);

            // Remove the change from the working index
            self.database.index().remove_from_index(&change.id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

            info!("Removed change '{}' from top of index", change.name);

            // Make a REST call to submit the change remotely
            match self.submit_to_remote(&url, &change, user) {
                Ok(_) => {
                    info!("Successfully submitted change '{}' to remote: {}", change.name, url);
                }
                Err(e) => {
                    warn!("Failed to submit change '{}' to remote {}: {}. Change still submitted locally.", 
                          change.name, url, e);
                    // Don't fail the whole operation if remote submission fails
                    // The local submission succeeded, remote is best-effort
                }
            }

            info!("Successfully submitted change '{}' ({}), moved to workspace for review", 
                  change.name, change.id);

            Ok(undo_diff)
        } else {
            // NON-REMOTE INDEX: Instantly approve the change
            info!("No source URL configured, instantly approving change");
            
            // Build the diff model showing what was approved
            let diff_model = build_object_diff_from_change(&self.database, &change)?;
            
            // Update the change status to Merged
            change.status = ChangeStatus::Merged;
            
            // Update the change in the database
            self.database.index().update_change(&change)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            
            // Clear the top_change pointer (change stays in history as merged)
            self.database.index().clear_top_change_if(&change.id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
            
            // Remove the change from workspace if it exists there (as a pending or stashed change)
            if self.database.workspace().get_workspace_change(&change.id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .is_some() {
                self.database.workspace().delete_workspace_change(&change.id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                info!("Removed change '{}' from workspace", change.name);
            }
            
            info!("Successfully approved change '{}' ({}), marked as merged and removed from index", 
                  change.name, change.id);
            
            Ok(diff_model)
        }
    }

    /// Submit the change to a remote server via REST API
    fn submit_to_remote(&self, source_url: &str, change: &crate::types::Change, _user: &User) -> Result<(), ObjectsTreeError> {
        // Build the URL for the remote workspace/submit endpoint
        let submit_url = if source_url.ends_with('/') {
            format!("{}workspace/submit", source_url)
        } else {
            format!("{}/workspace/submit", source_url)
        };

        info!("Submitting change '{}' to remote URL: {}", change.id, submit_url);

        // Serialize the change to a JSON string
        let serialized_change = serde_json::to_string(change)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to serialize change: {}", e)))?;

        // Prepare the payload for the workspace/submit operation
        // The operation expects args[0] to be the serialized change
        let payload = serde_json::json!({
            "operation": "workspace/submit",
            "args": [serialized_change]
        });

        // Clone URL and payload for the thread
        let url_clone = submit_url.clone();
        let payload_clone = payload.clone();

        // Run the blocking HTTP call in a thread pool to avoid runtime conflicts
        let result = std::thread::spawn(move || {
            // Create a blocking HTTP client
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

            // Make the PUT request to workspace/submit
            let response = client
                .put(&url_clone)
                .json(&payload_clone)
                .send()
                .map_err(|e| format!("HTTP request failed: {}", e))?;

            if response.status().is_success() {
                Ok(())
            } else {
                let status = response.status();
                let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
                Err(format!("Remote submission failed with status {}: {}", status, error_text))
            }
        })
        .join()
        .map_err(|_| ObjectsTreeError::SerializationError("Thread panicked during remote submission".to_string()))?;

        match result {
            Ok(_) => {
                info!("Remote submission successful for change '{}'", change.id);
                Ok(())
            }
            Err(e) => {
                error!("Remote submission failed: {}", e);
                Err(ObjectsTreeError::SerializationError(e))
            }
        }
    }
}

impl Operation for ChangeSubmitOperation {
    fn name(&self) -> &'static str {
        "change/submit"
    }
    
    fn description(&self) -> &'static str {
        "Submits the top local change. Requires author to be set. If a source URL is configured (remote index), moves it to workspace with Review status for remote approval. If no source URL is configured (non-remote index), instantly approves and merges the change. Returns an ObjectDiffModel. Optional message argument can be provided to set/override the commit message."
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/change/submit".to_string(),
                method: Method::POST,
                is_json: false, // No body needed
            },
            OperationRoute {
                path: "/api/change/submit".to_string(),
                method: Method::POST,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Change submit operation received {} arguments for user: {}", args.len(), user.id);
        
        // Parse optional message argument
        let message = if args.is_empty() {
            None
        } else {
            // Join all arguments as the message
            let message_text = args.join(" ").trim().to_string();
            if message_text.is_empty() {
                None
            } else {
                Some(message_text)
            }
        };
        
        let request = ChangeSubmitRequest { message };

        match self.process_change_submit(request, user) {
            Ok(undo_diff) => {
                info!("Change submit operation completed successfully, returning undo diff");
                // Return the ObjectDiffModel as a MOO variable showing what needs to be undone
                undo_diff.to_moo_var()
            }
            Err(e) => {
                error!("Change submit operation failed: {}", e);
                v_error(E_INVARG.msg(&format!("Error: {e}")))
            }
        }
    }
}
