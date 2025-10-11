use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::{User, CloneData, ObjectInfo};
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
        
        // Export refs directly as Vec
        let refs_map = self.database.refs().get_all_refs()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        let refs: Vec<(ObjectInfo, String)> = refs_map.into_iter().collect();
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
    
    /// Import repository state from a URL (async version)
    pub async fn import_from_url_async(&self, url: &str) -> Result<String, ObjectsTreeError> {
        info!("Importing repository state from URL: {}", url);
        
        // Make GET request to the URL using async client
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
        
        // Try to parse as OperationResponse first (HTTP API response)
        let clone_data: CloneData = if let Ok(op_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
            // Check if this is an operation response with a result field
            if let Some(result_field) = op_response.get("result") {
                if let Some(result_str) = result_field.as_str() {
                    // The result is a JSON string, parse it as CloneData
                    serde_json::from_str(result_str)
                        .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to parse CloneData from result: {}", e)))?
                } else {
                    // The result might be a direct object
                    serde_json::from_value(result_field.clone())
                        .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to parse CloneData from result object: {}", e)))?
                }
            } else {
                // No result field, try to parse the whole response as CloneData
                serde_json::from_value(op_response)
                    .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to parse CloneData: {}", e)))?
            }
        } else {
            // Not valid JSON, return error
            return Err(ObjectsTreeError::SerializationError(format!("Invalid JSON response: {}", response_text)));
        };
        
        // Import the data
        self.import_state(clone_data, url)?;
        
        Ok(format!("Successfully cloned from {}", url))
    }
    
    /// Import repository state from a URL (sync wrapper for use in execute())
    pub fn import_from_url(&self, url: &str) -> Result<String, ObjectsTreeError> {
        let url = url.to_string();
        let self_clone = self.clone();
        
        // Spawn an async task and wait for it
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        
        tokio::spawn(async move {
            let result = self_clone.import_from_url_async(&url).await;
            let _ = tx.send(result);
        });
        
        rx.recv()
            .map_err(|_| ObjectsTreeError::SerializationError("Channel closed during clone import".to_string()))?
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
        for (obj_info, sha256) in &data.refs {
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
        
        // Extract base URL from source_url (remove /api/clone or /clone suffix)
        let base_url = source_url
            .trim_end_matches("/api/clone")
            .trim_end_matches("/clone")
            .to_string();
        
        // Set the source URL (base URL only, for use with /rpc endpoint)
        self.database.index().set_source(&base_url)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully imported repository from {} (base: {})", source_url, base_url);
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
    
    fn philosophy(&self) -> &'static str {
        "Enables repository replication and synchronization. Export mode (no URL) serializes the entire \
        repository state including all objects, refs, and merged changes into a portable JSON format. Import \
        mode (with URL) fetches repository data from a remote source and loads it locally. This is essential \
        for setting up new repository clones, creating backups, or synchronizing between different MOO \
        instances. The operation preserves complete history and maintains referential integrity across the clone."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "url".to_string(),
                description: "Optional URL to import from. If not provided, exports current state.".to_string(),
                required: false,
            }
        ]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Export repository state".to_string(),
                moocode: r#"json_data = worker_request("vcs", {"clone"});
// Returns complete repository as JSON string
// Save this for backup or transfer to another system"#.to_string(),
                http_curl: Some(r#"curl -X GET http://localhost:8081/api/clone > backup.json"#.to_string()),
            },
            OperationExample {
                description: "Import from a URL".to_string(),
                moocode: r#"result = worker_request("vcs", {"clone", "http://source-server:8081/clone"});
// Imports complete repository from source server
// Returns success message"#.to_string(),
                http_curl: None,
            }
        ]
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
            
            // Call the synchronous import_from_url
            match self.import_from_url(&url) {
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
