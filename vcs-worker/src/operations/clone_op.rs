use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::{User, CloneData};
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::index::IndexProvider;

/// Clone operation that exports or imports repository state
#[derive(Clone)]
pub struct CloneOperation {
    database: DatabaseRef,
}

impl CloneOperation {
    /// Create a new clone operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Export the current repository state
    fn export_state(&self) -> Result<CloneData, ObjectsTreeError> {
        info!("Exporting repository state");
        
        // Export refs directly
        let refs = self.database.refs().get_all_refs()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        info!("Exported {} refs", refs.len());
        
        // Export objects directly
        let objects = self.database.objects().get_all_objects()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        info!("Exported {} objects", objects.len());
        
        // Export only MERGED changes
        let all_changes = self.database.index().list_changes()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let changes: Vec<_> = all_changes.into_iter()
            .filter(|change| change.status == crate::types::ChangeStatus::Merged)
            .collect();
        info!("Exported {} merged changes (filtered from all changes)", changes.len());
        
        // Export change order, filtered to only include merged changes
        let all_change_order = self.database.index().get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let merged_change_ids: std::collections::HashSet<_> = changes.iter()
            .map(|c| c.id.clone())
            .collect();
        let change_order: Vec<_> = all_change_order.into_iter()
            .filter(|id| merged_change_ids.contains(id))
            .collect();
        info!("Exported change order with {} merged changes", change_order.len());
        
        // Get source URL
        let source = self.database.index().get_source()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        Ok(CloneData {
            refs,
            objects,
            changes,
            change_order,
            source,
        })
    }
    
    /// Import repository state from a URL
    pub async fn import_from_url(&self, url: &str) -> Result<String, ObjectsTreeError> {
        info!("Importing repository state from URL: {}", url);
        
        // Make GET request to the URL
        let client = reqwest::Client::new();
        let response = client.get(url)
            .send()
            .await
            .map_err(|e| ObjectsTreeError::SerializationError(format!("HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ObjectsTreeError::SerializationError(
                format!("HTTP request failed with status: {}", response.status())
            ));
        }
        
        let response_text = response.text()
            .await
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to read response: {}", e)))?;
        
        let clone_data: CloneData = serde_json::from_str(&response_text)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to parse JSON: {}", e)))?;
        
        // Import the data
        self.import_state(clone_data, url)?;
        
        Ok(format!("Successfully cloned from {}", url))
    }
    
    /// Import repository state from CloneData
    fn import_state(&self, data: CloneData, source_url: &str) -> Result<(), ObjectsTreeError> {
        let object_count = data.objects.len();
        let refs_count = data.refs.len();
        let changes_count = data.changes.len();
        
        info!("Importing {} refs, {} objects, {} changes", 
              refs_count, object_count, changes_count);
        
        // Clear existing state first
        info!("Clearing existing state...");
        self.database.objects().clear()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        self.database.refs().clear()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        self.database.index().clear()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        info!("Existing state cleared");
        
        // Import objects first
        for (sha256, object_data) in data.objects {
            self.database.objects().store(&sha256, &object_data)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        }
        info!("Imported {} objects", object_count);
        
        // Import refs
        for (obj_info, sha256) in data.refs {
            self.database.refs().update_ref(obj_info.object_type, &obj_info.name, obj_info.version, &sha256)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        }
        info!("Imported {} refs", refs_count);
        
        // Import changes
        for change in &data.changes {
            self.database.index().store_change(change)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        }
        info!("Imported {} changes", data.changes.len());
        
        // Set the change order directly
        self.database.index().set_change_order(data.change_order)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Set the source URL
        self.database.index().set_source(source_url)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully imported repository from {}", source_url);
        Ok(())
    }
}

impl Operation for CloneOperation {
    fn name(&self) -> &'static str {
        "clone"
    }
    
    fn description(&self) -> &'static str {
        "Export repository state (no URL) or import from a URL"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/clone".to_string(),
                method: Method::GET,
                is_json: false,
            },
            OperationRoute {
                path: "/clone".to_string(),
                method: Method::POST,
                is_json: true,
            },
            OperationRoute {
                path: "/api/clone".to_string(),
                method: Method::GET,
                is_json: false,
            },
            OperationRoute {
                path: "/api/clone".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!("Clone operation received {} arguments: {:?}", args.len(), args);
        
        // Check if user has Clone permission
        if !user.has_permission(&crate::types::Permission::Clone) {
            error!("User {} does not have Clone permission", user.id);
            return moor_var::v_str("Error: You do not have permission to clone repositories");
        }
        
        // If no URL provided, export state
        if args.is_empty() || args[0].is_empty() {
            match self.export_state() {
                Ok(clone_data) => {
                    match serde_json::to_string(&clone_data) {
                        Ok(json) => {
                            info!("Exported repository state as JSON ({} bytes)", json.len());
                            moor_var::v_str(&json)
                        }
                        Err(e) => {
                            error!("Failed to serialize clone data: {}", e);
                            moor_var::v_str(&format!("Error: Failed to serialize: {}", e))
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to export repository state: {}", e);
                    moor_var::v_str(&format!("Error: {}", e))
                }
            }
        } else {
            // URL provided, import from URL
            let url = args[0].clone();
            
            // We need to use async runtime here
            let rt = tokio::runtime::Handle::current();
            match rt.block_on(self.import_from_url(&url)) {
                Ok(result) => {
                    info!("Clone operation completed successfully");
                    moor_var::v_str(&result)
                }
                Err(e) => {
                    error!("Clone operation failed: {}", e);
                    moor_var::v_str(&format!("Error: {}", e))
                }
            }
        }
    }
}
