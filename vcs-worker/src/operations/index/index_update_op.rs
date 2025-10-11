use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::User;
use crate::providers::index::IndexProvider;
use crate::object_diff::{ObjectDiffModel, build_object_diff_from_change};

/// Request structure for index update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexUpdateRequest {
    // No parameters needed - uses source URL from index
}

/// Index update operation that fetches deltas from the source URL and applies them to the local index
/// 
/// Usage:
/// - `index/update`
/// - Requires a source URL to be set in the index
/// - Calculates delta from the last known change and applies it to index, refs, and objects
/// - Returns an object diff containing all changes that were applied
/// 
/// Example: `index/update` updates the local repository with changes from the remote source and returns the diff
#[derive(Clone)]
pub struct IndexUpdateOperation {
    database: DatabaseRef,
}

impl IndexUpdateOperation {
    /// Create a new index update operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }
    
    /// Public async method for testing and direct async use
    pub async fn update_async(&self) -> Result<moor_var::Var, ObjectsTreeError> {
        let request = IndexUpdateRequest {};
        self.process_update_async(request).await
    }

    /// Process the index update request (async version)
    async fn process_update_async(&self, _request: IndexUpdateRequest) -> Result<moor_var::Var, ObjectsTreeError> {
        info!("Processing index update request");
        
        // Check if there's a source URL in the index
        let source_url = match self.database.index().get_source()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            Some(url) => {
                info!("Found source URL: {}", url);
                url
            }
            None => {
                error!("No source URL found in index - nothing to update");
                return Ok(moor_var::v_str("Error: No source URL configured. This repository was not cloned from a remote source."));
            }
        };
        
        // Get the current change order to find the last known change
        let change_order = self.database.index().get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if change_order.is_empty() {
            info!("No changes in index - performing full clone");
            return self.perform_full_clone_async(&source_url).await;
        }
        
        // Get the most recent change ID (last in the list since ordering is oldest first, newest last)
        let last_change_id = change_order.last().unwrap(); // Safe because we checked is_empty above
        info!("Last known change ID: {}", last_change_id);
        
        // Calculate delta using the index_calc_delta operation
        let delta_result = self.calculate_delta_from_remote_async(&source_url, last_change_id).await?;
        
        // Check if there are new changes
        let delta_str = delta_result.as_string()
            .ok_or_else(|| ObjectsTreeError::SerializationError("Expected string delta result".to_string()))?;
        let delta_data: serde_json::Value = serde_json::from_str(delta_str)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to parse delta: {}", e)))?;
        
        let change_ids = delta_data.get("change_ids")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        
        if change_ids > 0 {
            info!("Delta contains {} new changes - performing full clone for consistency", change_ids);
            // Since incremental update isn't implemented yet, do a full clone
            self.perform_full_clone_async(&source_url).await
        } else {
            info!("No new changes in delta - index is up to date");
            Ok(moor_var::v_str("Index is up to date"))
        }
    }
    
    /// Process the index update request (sync wrapper)
    fn process_update(&self, request: IndexUpdateRequest) -> Result<moor_var::Var, ObjectsTreeError> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let self_clone = self.clone();
        
        tokio::spawn(async move {
            let result = self_clone.process_update_async(request).await;
            let _ = tx.send(result);
        });
        
        rx.recv()
            .map_err(|_| ObjectsTreeError::SerializationError("Channel closed during update".to_string()))?
    }
    
    /// Perform a full clone if no changes exist (async version)
    async fn perform_full_clone_async(&self, source_url: &str) -> Result<moor_var::Var, ObjectsTreeError> {
        info!("Performing full clone from source URL: {}", source_url);
        
        // Use the existing clone operation logic
        let clone_op = crate::operations::clone_op::CloneOperation::new(self.database.clone());
        
        // Construct the full clone endpoint URL (source_url is the base URL)
        let clone_url = if source_url.ends_with('/') {
            format!("{}api/clone", source_url)
        } else {
            format!("{}/api/clone", source_url)
        };
        
        info!("Cloning from: {}", clone_url);
        
        // Import from URL (this will clear existing state and import everything)
        match clone_op.import_from_url_async(&clone_url, None).await {
            Ok(result) => {
                info!("Full clone completed successfully");
                Ok(moor_var::v_str(&result))
            }
            Err(e) => {
                error!("Full clone failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Calculate delta from remote source (async version)
    async fn calculate_delta_from_remote_async(&self, source_url: &str, last_change_id: &str) -> Result<moor_var::Var, ObjectsTreeError> {
        info!("Calculating delta from remote source: {} since change: {}", source_url, last_change_id);
        
        // Construct the RPC URL
        let rpc_url = if source_url.ends_with('/') {
            format!("{}rpc", source_url.trim_end_matches('/'))
        } else {
            format!("{}/rpc", source_url)
        };
        
        info!("Fetching delta from: {}", rpc_url);
        
        // Make async HTTP request to RPC endpoint
        let client = reqwest::Client::new();
        let request_body = serde_json::json!({
            "operation": "index/calc_delta",
            "args": [last_change_id]
        });
        
        let response = client.post(&rpc_url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ObjectsTreeError::SerializationError(format!("HTTP request failed: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(ObjectsTreeError::SerializationError(
                format!("HTTP request failed with status: {}", response.status())
            ));
        }
        
        let response_json: serde_json::Value = response.json()
            .await
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to read response: {}", e)))?;
        
        // Extract result from RPC response
        let result = response_json.get("result")
            .ok_or_else(|| ObjectsTreeError::SerializationError("No result in RPC response".to_string()))?;
        
        // Convert result to JSON string for compatibility
        let result_str = serde_json::to_string(result)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to serialize result: {}", e)))?;
        
        info!("Successfully fetched delta from remote");
        Ok(moor_var::v_str(&result_str))
    }
    
    /// Apply delta to local index, refs, and objects
    fn apply_delta(&self, delta_result: moor_var::Var) -> Result<ObjectDiffModel, ObjectsTreeError> {
        info!("Applying delta to local index, refs, and objects");
        
        // Parse the delta result - convert to string
        let delta_str = match delta_result.as_string() {
            Some(s) => s.to_string(),
            None => return Err(ObjectsTreeError::SerializationError("Expected string delta result".to_string())),
        };
        
        let _delta_data: serde_json::Value = serde_json::from_str(&delta_str)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to parse delta: {}", e)))?;
        
        // Extract change IDs, ref pairs, and objects from delta
        let empty_vec = vec![];
        let change_ids = _delta_data.get("change_ids")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        
        let _ref_pairs = _delta_data.get("ref_pairs")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        
        let _objects_added = _delta_data.get("objects_added")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);
        
        info!("Delta contains {} change IDs, {} ref pairs, {} objects", 
              change_ids.len(), _ref_pairs.len(), _objects_added.len());
        
        // Collect commit IDs for object diff
        let mut commit_ids = Vec::new();
        for change_id in change_ids {
            if let Some(id_str) = change_id.as_str() {
                commit_ids.push(id_str.to_string());
            }
        }
        
        // For now, we'll implement a basic version that fetches the full state
        // In a more sophisticated implementation, we would:
        // 1. Fetch only the new changes from the remote
        // 2. Apply ref updates
        // 3. Fetch and store new objects
        // 4. Update the change order
        
        if !change_ids.is_empty() {
            info!("Delta contains {} new changes - building diff", change_ids.len());
            // TODO: Implement incremental update logic
            // For now, the full clone happens at the higher level in process_update_async
            
            // Build object diff from the commit IDs
            self.build_object_diff_from_commit_ids(&commit_ids)
        } else {
            info!("No new changes in delta - index is up to date");
            Ok(ObjectDiffModel::new())
        }
    }
    
    /// Build object diff from commit IDs
    fn build_object_diff_from_commit_ids(&self, commit_ids: &[String]) -> Result<ObjectDiffModel, ObjectsTreeError> {
        info!("Building object diff from {} commit IDs", commit_ids.len());
        
        let mut merged_diff = ObjectDiffModel::new();
        
        for commit_id in commit_ids {
            // Get the change from the database
            if let Some(change) = self.database.index().get_change(commit_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
                
                info!("Building diff for change '{}' ({})", change.name, change.id);
                
                // Build diff for this change
                let change_diff = build_object_diff_from_change(&self.database, &change)?;
                
                // Merge with the overall diff
                merged_diff.merge(change_diff);
            } else {
                warn!("Change '{}' not found in database", commit_id);
            }
        }
        
        info!("Merged object diff contains {} added, {} modified, {} deleted, {} renamed objects", 
              merged_diff.objects_added.len(), 
              merged_diff.objects_modified.len(), 
              merged_diff.objects_deleted.len(), 
              merged_diff.objects_renamed.len());
        
        Ok(merged_diff)
    }
}

impl Operation for IndexUpdateOperation {
    fn name(&self) -> &'static str {
        "index/update"
    }
    
    fn description(&self) -> &'static str {
        "Updates the local index with changes from the remote source URL and returns an object diff of the changes"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/index/update".to_string(),
                method: Method::POST,
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

    fn execute(&self, _args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Index update operation received");
        
        let request = IndexUpdateRequest {};

        match self.process_update(request) {
            Ok(result_var) => {
                info!("Index update operation completed successfully");
                result_var
            }
            Err(e) => {
                error!("Index update operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
