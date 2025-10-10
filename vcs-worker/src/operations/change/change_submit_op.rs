use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info, warn};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{ChangeSubmitRequest, ChangeStatus, User, Permission};
use crate::object_diff::{ObjectDiffModel, build_abandon_diff_from_change};
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

        // Check if there's a source URL for remote submission
        if let Some(source_url) = self.database.index().get_source()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            
            info!("Source URL found: {}, attempting remote submission", source_url);
            
            // Make a REST call to submit the change remotely
            match self.submit_to_remote(&source_url, &change, user) {
                Ok(_) => {
                    info!("Successfully submitted change '{}' to remote: {}", change.name, source_url);
                }
                Err(e) => {
                    warn!("Failed to submit change '{}' to remote {}: {}. Change still submitted locally.", 
                          change.name, source_url, e);
                    // Don't fail the whole operation if remote submission fails
                    // The local submission succeeded, remote is best-effort
                }
            }
        } else {
            info!("No source URL configured, skipping remote submission");
        }

        info!("Successfully submitted change '{}' ({}), moved to workspace for review", 
              change.name, change.id);

        Ok(undo_diff)
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

        // Create a blocking HTTP client
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to create HTTP client: {}", e)))?;

        // Serialize the change to a JSON string
        let serialized_change = serde_json::to_string(change)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to serialize change: {}", e)))?;

        // Prepare the payload for the workspace/submit operation
        // The operation expects args[0] to be the serialized change
        let payload = serde_json::json!({
            "operation": "workspace/submit",
            "args": [serialized_change]
        });

        // Make the PUT request to workspace/submit
        let response = client
            .put(&submit_url)
            .json(&payload)
            .send()
            .map_err(|e| ObjectsTreeError::SerializationError(format!("HTTP request failed: {}", e)))?;

        if response.status().is_success() {
            info!("Remote submission successful for change '{}'", change.id);
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().unwrap_or_else(|_| "Unknown error".to_string());
            error!("Remote submission failed with status {}: {}", status, error_text);
            Err(ObjectsTreeError::SerializationError(
                format!("Remote submission failed with status {}: {}", status, error_text)
            ))
        }
    }
}

impl Operation for ChangeSubmitOperation {
    fn name(&self) -> &'static str {
        "change/submit"
    }
    
    fn description(&self) -> &'static str {
        "Submits the top local change for review. Requires author to be set. Moves it to workspace with Review status, removes from index, and optionally submits to remote if source URL is configured. Returns an ObjectDiffModel showing what changes need to be undone. Optional message argument can be provided to set/override the commit message."
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
