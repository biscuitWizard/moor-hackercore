use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::types::User;
use crate::providers::index::IndexProvider;
use crate::types::{ChangeAbandonRequest, ChangeStatus};
use crate::object_diff::{ObjectDiffModel, build_abandon_diff_from_change};

/// Change abandon operation that abandons the top change in the index
#[derive(Clone)]
pub struct ChangeAbandonOperation {
    database: DatabaseRef,
}

impl ChangeAbandonOperation {
    /// Create a new change abandon operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the change abandon request and return an ObjectDiffModel showing what needs to be undone
    fn process_change_abandon(&self, _request: ChangeAbandonRequest) -> Result<ObjectDiffModel, ObjectsTreeError> {
        // Get the current change from the top of the index
        let top_change_id = self.database.index().get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        if let Some(change_id) = top_change_id {
            let change = self.database.index().get_change(&change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Top change '{}' not found", change_id)))?;
            
            info!("Attempting to abandon current change: {}", change.id);
            
            if change.status == ChangeStatus::Merged {
                error!("Cannot abandon change '{}' ({}) - it has already been merged", change.name, change.id);
                return Err(ObjectsTreeError::SerializationError(
                    format!("Cannot abandon merged change '{}'", change.name)
                ));
            }
            
            // Build the abandon diff using shared logic
            let undo_delta = build_abandon_diff_from_change(&self.database, &change)?;
            
            // Remove from working index if it's LOCAL
            if change.status == ChangeStatus::Local {
                self.database.index().remove_from_index(&change.id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                info!("Removed change '{}' from working index", change.name);
                
                // Delete the change from history storage (abandoned changes are not kept)
                self.database.index().delete_change(&change.id)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                info!("Deleted abandoned change '{}' from history storage", change.name);
            }
            
            info!("Successfully abandoned change '{}' ({}), created undo delta", change.name, change.id);
            Ok(undo_delta)
        } else {
            error!("No current change to abandon");
            return Err(ObjectsTreeError::SerializationError(
                "Error: No change to abandon".to_string()
            ));
        }
    }

}

impl Operation for ChangeAbandonOperation {
    fn name(&self) -> &'static str {
        "change/abandon"
    }
    
    fn description(&self) -> &'static str {
        "Abandons the top local change in the index, removing it from index. Returns an ObjectDiffModel showing what changes need to be undone. Cannot abandon merged changes."
    }
    
    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }
    
    fn philosophy(&self) -> &'static str {
        "Discards your current local changelist completely, removing all tracked changes without submitting them. \
        Use this when you've made changes you don't want to keep - perhaps experimental work that didn't pan out, \
        or changes made in error. The operation returns a diff showing what needs to be undone in your MOO database \
        to revert to the previous state. Abandoned changes are permanently deleted and cannot be recovered. You \
        cannot abandon changes that have already been merged into the repository; this operation only works on \
        local (unsubmitted) changes."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Abandon the current change".to_string(),
                moocode: r#"diff = worker_request("vcs", {"change/abandon"});
// Returns an ObjectDiffModel showing what to undo
// Apply this diff to revert your MOO database to previous state
player:tell("Change abandoned. You need to revert ", length(diff["modified_objects"]), " objects");"#.to_string(),
                http_curl: Some(r#"curl -X POST http://localhost:8081/api/change/abandon"#.to_string()),
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/change/abandon".to_string(),
                method: Method::POST,
                is_json: false,
            }
        ]
    }
    
    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r#"["objects_renamed" -> [], "objects_deleted" -> {}, "objects_added" -> {}, "objects_modified" -> {"obj1"}, "changes" -> {["obj_id" -> "obj1", "verbs_modified" -> {}, "verbs_added" -> {}, "verbs_renamed" -> [], "verbs_deleted" -> {}, "props_modified" -> {}, "props_added" -> {}, "props_renamed" -> [], "props_deleted" -> {}]}]"#
            ),
            OperationResponse::new(
                400,
                "Bad Request - Cannot abandon merged change",
                r#""Error: Cannot abandon merged change 'my-change'""#
            ),
            OperationResponse::new(
                404,
                "Not Found - No change to abandon",
                r#""Error: No change to abandon""#
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#""Error: Database error: failed to abandon change""#
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, _user: &User) -> moor_var::Var {
        info!("Change abandon operation received {} arguments", args.len());
        
        let request = ChangeAbandonRequest {};

        match self.process_change_abandon(request) {
            Ok(delta_model) => {
                info!("Change abandon operation completed successfully, returning undo delta");
                // Return the ObjectDiffModel as a MOO variable showing what needs to be undone
                delta_model.to_moo_var()
            }
            Err(e) => {
                error!("Change abandon operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
