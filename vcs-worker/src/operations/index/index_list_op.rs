use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;

/// Request structure for index list operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexListRequest {
    pub limit: Option<usize>,
    pub page: Option<usize>,
}

/// Index list operation that returns a paginated list of changes in chronological order (newest first)
/// 
/// Usage:
/// - `index/list` or `index/list "{limit}"` or `index/list "{limit}" "{page}"`
/// - Returns a v_list of maps containing change information
/// - Each map contains: change_id, message, name, timestamp, author, status
/// - Default limit is 5, default page is 0
/// - Page 0 is the first page (newest changes)
/// 
/// Example: `index/list "10" "1"` returns page 1 with up to 10 changes per page
#[derive(Clone)]
pub struct IndexListOperation {
    database: DatabaseRef,
}

impl IndexListOperation {
    /// Create a new index list operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the index list request with pagination
    fn process_index_list(&self, request: IndexListRequest) -> Result<moor_var::Var, ObjectsTreeError> {
        info!("Processing index list request with limit: {:?}, page: {:?}", request.limit, request.page);
        
        // Get the ordered list of change IDs from index
        let change_order = self.database.index().get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        let total_changes = change_order.len();
        
        // Calculate pagination parameters
        let limit = request.limit.unwrap_or(5); // Default to 5
        let page = request.page.unwrap_or(0); // Page 0 is first page
        let offset = page * limit;
        
        info!("Pagination: total={}, limit={}, page={}, offset={}", total_changes, limit, page, offset);
        
        if offset >= total_changes {
            info!("Requested page {} is beyond available changes, returning empty list", page);
            return Ok(moor_var::v_list(&[]));
        }
        
        // Get the subset of changes for this page
        let end_index = std::cmp::min(offset + limit, total_changes);
        let page_change_ids = &change_order[offset..end_index];
        
        // Convert change IDs to change details using the existing Change struct directly
        let mut changes_list = Vec::new();
        
        for change_id in page_change_ids {
            if let Some(change) = self.database.index().get_change(change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
                
                // Create map representing the Change struct directly
                let change_map = moor_var::v_map(&[
                    (moor_var::v_str("change_id"), moor_var::v_str(&change.id)),
                    (moor_var::v_str("message"), moor_var::v_str(change.description.as_deref().unwrap_or(""))),
                    (moor_var::v_str("name"), moor_var::v_str(&change.name)),
                    (moor_var::v_str("timestamp"), moor_var::v_int(change.timestamp as i64)),
                    (moor_var::v_str("author"), moor_var::v_str(&change.author)),
                    (moor_var::v_str("status"), moor_var::v_str(match change.status {
                        crate::types::ChangeStatus::Local => "local",
                        crate::types::ChangeStatus::Merged => "merged",
                    })),
                ]);
                
                changes_list.push(change_map);
            } else {
                warn!("Change {} was referenced in index but not found in changes storage", change_id);
            }
        }
        
        info!("Successfully retrieved {} changes for page {}", changes_list.len(), page);
        Ok(moor_var::v_list(&changes_list))
    }
}

impl Operation for IndexListOperation {
    fn name(&self) -> &'static str {
        "index/list"
    }
    
    fn description(&self) -> &'static str {
        "Lists changes in chronological order (newest first) with optional pagination (limit/page)"
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/index/list".to_string(),
                method: Method::GET,
                is_json: false, // Uses query parameters for pagination
            },
            OperationRoute {
                path: "/api/index/list".to_string(),
                method: Method::GET,
                is_json: false,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>) -> moor_var::Var {
        info!("Index list operation received {} arguments: {:?}", args.len(), args);
        
        // Parse pagination arguments
        let mut limit = None;
        let mut page = None;
        
        // Parse optional limit parameter
        if args.len() > 0 && !args[0].is_empty() {
            if let Ok(parsed_limit) = args[0].parse::<usize>() {
                limit = Some(parsed_limit);
            } else {
                warn!("Invalid limit parameter '{}', using default", args[0]);
            }
        }
        
        // Parse optional page parameter
        if args.len() > 1 && !args[1].is_empty() {
            if let Ok(parsed_page) = args[1].parse::<usize>() {
                page = Some(parsed_page);
            } else {
                warn!("Invalid page parameter '{}', using default", args[1]);
            }
        }
        
        let request = IndexListRequest { limit, page };

        match self.process_index_list(request) {
            Ok(result_var) => {
                info!("Index list operation completed successfully");
                result_var
            }
            Err(e) => {
                error!("Index list operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
