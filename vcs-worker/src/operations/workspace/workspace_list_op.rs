use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{Change, User, Permission, ChangeStatus};
use moor_var::{v_error, v_str, E_INVARG};

/// Workspace list operation that lists all changes in the workspace (stashed, in review, etc.)
#[derive(Clone)]
pub struct WorkspaceListOperation {
    database: DatabaseRef,
}

impl WorkspaceListOperation {
    /// Create a new workspace list operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the workspace list request
    fn process_workspace_list(&self, user: &User, status_filter: Option<ChangeStatus>) -> Result<String, ObjectsTreeError> {
        // Check if user has permission to view workspace changes (using SubmitChanges as it's the closest permission)
        if !user.has_permission(&Permission::SubmitChanges) {
            error!("User '{}' does not have permission to view workspace changes", user.id);
            return Err(ObjectsTreeError::SerializationError(
                format!("User '{}' does not have permission to view workspace changes", user.id)
            ));
        }

        info!("User '{}' requesting workspace changes list", user.id);

        // Get changes based on filter
        let changes = if let Some(status) = &status_filter {
            self.database.workspace().list_workspace_changes_by_status(status.clone())
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        } else {
            self.database.workspace().list_all_workspace_changes()
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        };

        info!("Found {} workspace changes for user '{}'", changes.len(), user.id);

        // Format the response
        let response = self.format_changes_response(&changes, status_filter);

        Ok(response)
    }

    /// Format the changes into a readable response
    fn format_changes_response(&self, changes: &[Change], status_filter: Option<ChangeStatus>) -> String {
        if changes.is_empty() {
            let filter_msg = if let Some(status) = status_filter {
                format!(" with status {:?}", status)
            } else {
                String::new()
            };
            return format!("No workspace changes found{}.", filter_msg);
        }

        let mut response = String::new();
        
        // Add header
        let filter_msg = if let Some(status) = status_filter {
            format!(" (status: {:?})", status)
        } else {
            String::new()
        };
        response.push_str(&format!("Workspace Changes{}\n", filter_msg));
        response.push_str(&"=".repeat(50));
        response.push('\n');

        // Group changes by status for better organization
        let mut changes_by_status: std::collections::HashMap<ChangeStatus, Vec<&Change>> = std::collections::HashMap::new();
        for change in changes {
            changes_by_status.entry(change.status.clone()).or_default().push(change);
        }

        // Sort statuses for consistent output
        let mut statuses: Vec<_> = changes_by_status.keys().collect();
        statuses.sort_by_key(|s| match s {
            ChangeStatus::Review => 0,
            ChangeStatus::Idle => 1,
            ChangeStatus::Merged => 2,
            ChangeStatus::Local => 3,
        });

        for status in statuses {
            let status_changes = &changes_by_status[status];
            response.push_str(&format!("\n{:?} Changes ({}):\n", status, status_changes.len()));
            response.push_str(&"-".repeat(30));
            response.push('\n');

            for change in status_changes {
                let short_id = crate::util::short_hash(&change.id);
                response.push_str(&format!("  ID: {} ({})\n", change.id, short_id));
                response.push_str(&format!("  Name: {}\n", change.name));
                if let Some(desc) = &change.description {
                    response.push_str(&format!("  Description: {}\n", desc));
                }
                response.push_str(&format!("  Author: {}\n", change.author));
                response.push_str(&format!("  Created: {}\n", 
                    chrono::DateTime::from_timestamp(change.timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                ));
                if let Some(index_id) = &change.index_change_id {
                    response.push_str(&format!("  Based on: {}\n", index_id));
                }
                
                // Show object counts
                let total_objects = change.added_objects.len() + change.modified_objects.len() + 
                                  change.deleted_objects.len() + change.renamed_objects.len();
                response.push_str(&format!("  Objects: {} total ({} added, {} modified, {} deleted, {} renamed)\n",
                    total_objects,
                    change.added_objects.len(),
                    change.modified_objects.len(),
                    change.deleted_objects.len(),
                    change.renamed_objects.len()
                ));
                response.push('\n');
            }
        }

        response.push_str(&format!("\nTotal: {} changes\n", changes.len()));
        response
    }
}

impl Operation for WorkspaceListOperation {
    fn name(&self) -> &'static str {
        "workspace/list"
    }
    
    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Lists all changes in the workspace. Optionally filter by status (Review, Idle). Usage: workspace/list [status]"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/workspace/list".to_string(),
                method: Method::GET,
                is_json: false,
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

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#""Operation completed successfully""#
            ),
            OperationResponse::new(
                400,
                "Bad Request - Invalid arguments",
                r#""Error: Invalid operation arguments""#
            ),
            OperationResponse::new(
                404,
                "Not Found - Resource not found",
                r#""Error: Resource not found""#
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#""Error: Database error: operation failed""#
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Workspace list operation received {} arguments for user: {}", args.len(), user.id);
        
        // Parse optional status filter
        let status_filter = if args.is_empty() {
            None
        } else {
            match args[0].to_lowercase().as_str() {
                "review" => Some(ChangeStatus::Review),
                "idle" => Some(ChangeStatus::Idle),
                _ => {
                    error!("Invalid status filter: {}. Valid options: review, idle", args[0]);
                    return v_error(E_INVARG.msg(&format!("Invalid status filter: {}. Valid options: review, idle", args[0])));
                }
            }
        };

        match self.process_workspace_list(user, status_filter) {
            Ok(message) => {
                info!("Workspace list operation completed successfully");
                v_str(&message)
            }
            Err(e) => {
                error!("Workspace list operation failed: {}", e);
                v_error(E_INVARG.msg(&format!("Error: {e}")))
            }
        }
    }
}
