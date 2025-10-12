use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::{ChangeStatus, User};
use moor_var::{E_INVARG, v_error};

/// Request structure for index calc delta operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexCalcDeltaRequest {
    pub change_id: String,
}

/// Index calc delta operation that finds a change in the index and returns all merged changes
/// chronologically after it, including their change IDs, ref pairs, and objects added to the database
///
/// Usage:
/// - `index/calc_delta "{change_id}"`
/// - Returns a map containing:
///   - change_ids: List of change IDs that are merged and chronologically after the specified change
///   - ref_pairs: List of ref pairs from those changes
///   - objects_added: List of objects added to the database from those changes
///
/// Example: `index/calc_delta "abc123"` returns delta information for changes after abc123
#[derive(Clone)]
pub struct IndexCalcDeltaOperation {
    database: DatabaseRef,
}

impl IndexCalcDeltaOperation {
    /// Create a new index calc delta operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the index calc delta request
    fn process_calc_delta(
        &self,
        request: IndexCalcDeltaRequest,
    ) -> Result<moor_var::Var, ObjectsTreeError> {
        info!(
            "Processing index calc delta request for change_id: {}",
            request.change_id
        );

        // Resolve short or full hash to full hash
        let change_id = self.database.resolve_change_id(&request.change_id)?;

        // Get the ordered list of change IDs from index
        let change_order = self
            .database
            .index()
            .get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        // Find the position of the specified change in the chronological order
        let target_position = change_order.iter().position(|id| id == &change_id);

        let target_position = match target_position {
            Some(pos) => pos,
            None => {
                error!("Change '{}' not found in index order", change_id);
                return Err(ObjectsTreeError::SerializationError(format!(
                    "Change '{change_id}' does not exist in index"
                )));
            }
        };

        info!(
            "Found change '{}' at position {} in chronological order",
            change_id, target_position
        );

        // Get all changes chronologically after the target change
        // Note: change_order is oldest first, so we want changes with indices > target_position
        let subsequent_changes = &change_order[target_position + 1..];

        let mut change_ids = Vec::new();
        let mut ref_pairs = Vec::new();
        let mut objects_added = Vec::new();

        // Process each subsequent change
        for change_id in subsequent_changes {
            if let Some(change) = self
                .database
                .index()
                .get_change(change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            {
                // Only include merged changes
                if change.status == ChangeStatus::Merged {
                    info!("Processing merged change '{}' ({})", change.name, change.id);

                    // Add change ID
                    change_ids.push(moor_var::v_str(&change.id));

                    // Extract ref pairs from the change
                    // Note: The Change struct doesn't directly contain ref pairs, but we can infer them
                    // from the object operations. For now, we'll create ref pairs based on object names
                    // Filter to only MooObject types
                    for added_obj in change
                        .added_objects
                        .iter()
                        .filter(|o| o.object_type == crate::types::VcsObjectType::MooObject)
                    {
                        let ref_pair = moor_var::v_map(&[
                            (moor_var::v_str("from"), moor_var::v_str("")), // No source for new objects
                            (moor_var::v_str("to"), moor_var::v_str(&added_obj.name)),
                        ]);
                        ref_pairs.push(ref_pair);
                    }

                    for modified_obj in change
                        .modified_objects
                        .iter()
                        .filter(|o| o.object_type == crate::types::VcsObjectType::MooObject)
                    {
                        let ref_pair = moor_var::v_map(&[
                            (moor_var::v_str("from"), moor_var::v_str(&modified_obj.name)),
                            (moor_var::v_str("to"), moor_var::v_str(&modified_obj.name)),
                        ]);
                        ref_pairs.push(ref_pair);
                    }

                    for renamed_obj in change.renamed_objects.iter().filter(|r| {
                        r.from.object_type == crate::types::VcsObjectType::MooObject
                            && r.to.object_type == crate::types::VcsObjectType::MooObject
                    }) {
                        let ref_pair = moor_var::v_map(&[
                            (
                                moor_var::v_str("from"),
                                moor_var::v_str(&renamed_obj.from.name),
                            ),
                            (moor_var::v_str("to"), moor_var::v_str(&renamed_obj.to.name)),
                        ]);
                        ref_pairs.push(ref_pair);
                    }

                    // Extract objects added to the database - filter to only MooObject types
                    for added_obj in change
                        .added_objects
                        .iter()
                        .filter(|o| o.object_type == crate::types::VcsObjectType::MooObject)
                    {
                        let object_info = moor_var::v_map(&[
                            (moor_var::v_str("name"), moor_var::v_str(&added_obj.name)),
                            (
                                moor_var::v_str("version"),
                                moor_var::v_int(added_obj.version as i64),
                            ),
                        ]);
                        objects_added.push(object_info);
                    }

                    for modified_obj in change
                        .modified_objects
                        .iter()
                        .filter(|o| o.object_type == crate::types::VcsObjectType::MooObject)
                    {
                        let object_info = moor_var::v_map(&[
                            (moor_var::v_str("name"), moor_var::v_str(&modified_obj.name)),
                            (
                                moor_var::v_str("version"),
                                moor_var::v_int(modified_obj.version as i64),
                            ),
                        ]);
                        objects_added.push(object_info);
                    }

                    for renamed_obj in change.renamed_objects.iter().filter(|r| {
                        r.from.object_type == crate::types::VcsObjectType::MooObject
                            && r.to.object_type == crate::types::VcsObjectType::MooObject
                    }) {
                        let object_info = moor_var::v_map(&[
                            (
                                moor_var::v_str("name"),
                                moor_var::v_str(&renamed_obj.to.name),
                            ),
                            (
                                moor_var::v_str("version"),
                                moor_var::v_int(renamed_obj.to.version as i64),
                            ),
                        ]);
                        objects_added.push(object_info);
                    }
                } else {
                    info!(
                        "Skipping non-merged change '{}' (status: {:?})",
                        change.name, change.status
                    );
                }
            } else {
                warn!(
                    "Change {} was referenced in index but not found in changes storage",
                    change_id
                );
            }
        }

        info!(
            "Successfully processed {} merged changes after '{}'",
            change_ids.len(),
            change_id
        );

        // Return the result as a map
        Ok(moor_var::v_map(&[
            (moor_var::v_str("change_ids"), moor_var::v_list(&change_ids)),
            (moor_var::v_str("ref_pairs"), moor_var::v_list(&ref_pairs)),
            (
                moor_var::v_str("objects_added"),
                moor_var::v_list(&objects_added),
            ),
        ]))
    }
}

impl Operation for IndexCalcDeltaOperation {
    fn name(&self) -> &'static str {
        "index/calc_delta"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn description(&self) -> &'static str {
        "Calculates delta information for changes chronologically after a specified change ID, returning change IDs, ref pairs, and objects added to the database"
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/index/calc_delta".to_string(),
            method: Method::GET,
            is_json: false,
        }]
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
                "Operation executed successfully - Returns delta information",
                r#"["change_ids" -> {"abc123def", "def456ghi"}, "ref_pairs" -> {["from" -> "", "to" -> "obj1"], ["from" -> "obj2", "to" -> "obj2"], ["from" -> "obj3", "to" -> "obj3_renamed"]}, "objects_added" -> {["name" -> "obj1", "version" -> 1], ["name" -> "obj2", "version" -> 2], ["name" -> "obj3_renamed", "version" -> 1]}]"#,
            ),
            OperationResponse::success(
                "Operation executed successfully - No changes after specified change",
                r#"["change_ids" -> {}, "ref_pairs" -> {}, "objects_added" -> {}]"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Missing change_id argument",
                r#"E_INVARG("change_id argument is required")"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - Change does not exist in index",
                r#"E_INVARG("Change 'xyz789' does not exist in index")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#"E_INVARG("Database error: failed to retrieve change order")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!(
            "Index calc delta operation received {} arguments: {:?}",
            args.len(),
            args
        );

        // Parse change_id argument
        if args.is_empty() || args[0].is_empty() {
            error!("Index calc delta operation requires a change_id argument");
            return v_error(E_INVARG.msg("change_id argument is required"));
        }

        let change_id = args[0].clone();
        let request = IndexCalcDeltaRequest { change_id };

        match self.process_calc_delta(request) {
            Ok(result_var) => {
                info!("Index calc delta operation completed successfully");
                result_var
            }
            Err(e) => {
                error!("Index calc delta operation failed: {}", e);
                v_error(E_INVARG.msg(format!("{e}")))
            }
        }
    }
}
