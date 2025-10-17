use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::User;
use moor_var::{E_INVARG, v_error};

/// Request structure for index list operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexListRequest {
    pub limit: Option<usize>,
    pub page: Option<usize>,
}

/// Index list operation that returns a paginated list of merged changes in reverse chronological order (newest first, oldest last)
///
/// Usage:
/// - `index/list` or `index/list "{limit}"` or `index/list "{limit}" "{page}"`
/// - Returns a v_list of maps containing change information
/// - Each map contains: change_id, message, name, timestamp, author, status
/// - Default limit is 20, default page is 0
/// - Page 0 is the first page (newest merged changes)
/// - Only shows merged changes by default
///
/// Example: `index/list "10" "1"` returns page 1 with up to 10 merged changes per page
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
    fn process_index_list(
        &self,
        request: IndexListRequest,
    ) -> Result<moor_var::Var, ObjectsTreeError> {
        // Get the ordered list of change IDs from index
        let change_order = self
            .database
            .index()
            .get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        // Calculate pagination parameters
        let limit = request.limit.unwrap_or(20); // Default to 20
        let page = request.page.unwrap_or(0); // Page 0 is first page
        let offset = page * limit;

        info!(
            "Processing index list request with limit: {:?}, page: {:?}",
            request.limit, request.page
        );

        // Filter to only merged changes and reverse order (newest first)
        let mut merged_changes = Vec::new();
        
        for change_id in &change_order {
            if let Some(change) = self
                .database
                .index()
                .get_change(change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            {
                if change.status == crate::types::ChangeStatus::Merged {
                    merged_changes.push(change);
                }
            } else {
                warn!(
                    "Change {} was referenced in index but not found in changes storage",
                    change_id
                );
            }
        }
        
        // Reverse to get newest first
        merged_changes.reverse();
        
        let total_merged_changes = merged_changes.len();
        
        // Calculate pagination for merged changes
        if offset >= total_merged_changes {
            info!(
                "Requested page {} is beyond available merged changes, returning empty list",
                page
            );
            return Ok(moor_var::v_list(&[]));
        }
        
        let end_index = std::cmp::min(offset + limit, total_merged_changes);
        let page_changes = &merged_changes[offset..end_index];
        
        // Convert changes to maps
        let mut changes_list = Vec::new();
        
        for change in page_changes {
            let short_id = crate::util::short_hash(&change.id);
            let change_map = moor_var::v_map(&[
                (moor_var::v_str("change_id"), moor_var::v_str(&change.id)),
                (moor_var::v_str("short_id"), moor_var::v_str(&short_id)),
                (
                    moor_var::v_str("message"),
                    moor_var::v_str(change.description.as_deref().unwrap_or("")),
                ),
                (moor_var::v_str("name"), moor_var::v_str(&change.name)),
                (
                    moor_var::v_str("timestamp"),
                    moor_var::v_int(change.timestamp as i64),
                ),
                (moor_var::v_str("author"), moor_var::v_str(&change.author)),
                (
                    moor_var::v_str("status"),
                    moor_var::v_str(match change.status {
                        crate::types::ChangeStatus::Local => "local",
                        crate::types::ChangeStatus::Merged => "merged",
                        crate::types::ChangeStatus::Review => "review",
                        crate::types::ChangeStatus::Idle => "idle",
                    }),
                ),
            ]);
            
            changes_list.push(change_map);
        }

        info!(
            "Successfully retrieved {} merged changes for page {} (total merged: {})",
            changes_list.len(),
            page,
            total_merged_changes
        );
        Ok(moor_var::v_list(&changes_list))
    }
}

impl Operation for IndexListOperation {
    fn name(&self) -> &'static str {
        "index/list"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Lists merged changes in reverse chronological order (newest first, oldest last) with optional pagination (limit/page)"
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/index/list".to_string(),
            method: Method::GET,
            is_json: false,
        }]
    }

    fn philosophy(&self) -> &'static str {
        "Lists merged changes in the index in reverse chronological order (newest first, oldest last) with optional pagination support. \
        This operation provides a way to browse through the repository's merged history, showing change metadata including IDs, \
        authors, timestamps, messages, and status. Only merged changes are shown by default, filtering out local, review, and idle changes. \
        Pagination allows efficient handling of large repositories by limiting results per query. Default limit is 20 merged changes, starting from page 0."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "List first 20 merged changes (default)".to_string(),
                moocode: r#"changes = worker_request("vcs", {"index/list"});
// Returns up to 20 merged changes with full metadata (newest first)
for change in (changes)
    player:tell(change["short_id"], ": ", change["message"], " by ", change["author"]);
endfor"#
                    .to_string(),
                http_curl: Some(r#"curl -X GET http://localhost:8081/api/index/list"#.to_string()),
            },
            OperationExample {
                description: "List merged changes with custom pagination".to_string(),
                moocode: r#"// Get 10 merged changes per page, starting at page 2 (skip first 20)
changes = worker_request("vcs", {"index/list", "10", "2"});
player:tell("Found ", length(changes), " merged changes on page 2");"#
                    .to_string(),
                http_curl: Some(
                    r#"curl -X GET "http://localhost:8081/api/index/list?limit=10&page=2""#
                        .to_string(),
                ),
            },
        ]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully - returns list of merged changes",
                r#"{[change_id -> "abc123def456...", short_id -> "abc123", message -> "Fixed login bug", name -> "fix-login", timestamp -> 1697020800, author -> "developer", status -> "merged"], [change_id -> "def456ghi789...", short_id -> "def456", message -> "Added new feature", name -> "new-feature", timestamp -> 1697107200, author -> "developer", status -> "merged"]}"#,
            ),
            OperationResponse::success("Empty result - page beyond available merged changes", r#"{}"#),
            OperationResponse::new(
                500,
                "Internal Server Error - Database error retrieving merged changes",
                r#"E_INVARG("Database error: failed to retrieve change order")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!(
            "Index list operation received {} arguments: {:?}",
            args.len(),
            args
        );

        // Parse pagination arguments
        let mut limit = None;
        let mut page = None;

        // Parse optional limit parameter
        if !args.is_empty() && !args[0].is_empty() {
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
                v_error(E_INVARG.msg(format!("{e}")))
            }
        }
    }
}
