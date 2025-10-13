use crate::object_diff::compare_object_versions;
use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::types::{User, VcsObjectType};
use moor_var::{v_error, v_list, v_map, v_str, v_int, Var, E_INVARG};

/// Request structure for object history operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectHistoryRequest {
    pub object_name: String,
}

/// Object history operation that retrieves the history of changes for a specific object
#[derive(Clone)]
pub struct ObjectHistoryOperation {
    database: DatabaseRef,
}

impl ObjectHistoryOperation {
    /// Create a new object history operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the object history request
    fn process_object_history(
        &self,
        request: ObjectHistoryRequest,
    ) -> Result<Vec<HistoryEntry>, ObjectsTreeError> {
        info!(
            "Retrieving history for object '{}'",
            request.object_name
        );

        // Get all changes in chronological order
        let change_order = self
            .database
            .index()
            .get_change_order()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        let mut history = Vec::new();
        
        // Track all names this object has had (to handle renames)
        // First, build the complete set of names by walking through all changes
        // and finding all renames
        let mut tracked_names = std::collections::HashSet::new();
        tracked_names.insert(request.object_name.clone());
        
        // Pass 1: Build the complete set of tracked names by finding all renames
        for change_id in &change_order {
            let change = self
                .database
                .index()
                .get_change(change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .ok_or_else(|| {
                    ObjectsTreeError::SerializationError(format!(
                        "Change '{}' not found",
                        change_id
                    ))
                })?;
            
            // Check renamed objects and build the name chain
            for renamed in change.renamed_objects.iter() {
                if renamed.from.object_type == VcsObjectType::MooObject {
                    // If we track either the from or to name, add both to tracked names
                    if tracked_names.contains(&renamed.from.name)
                        || tracked_names.contains(&renamed.to.name)
                    {
                        tracked_names.insert(renamed.from.name.clone());
                        tracked_names.insert(renamed.to.name.clone());
                    }
                }
            }
        }
        
        info!(
            "Found {} name(s) for object '{}': {:?}",
            tracked_names.len(),
            request.object_name,
            tracked_names
        );

        // Pass 2: Walk through changes and find ones that affected this object
        for change_id in change_order {
            let change = self
                .database
                .index()
                .get_change(&change_id)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .ok_or_else(|| {
                    ObjectsTreeError::SerializationError(format!(
                        "Change '{}' not found",
                        change_id
                    ))
                })?;

            // Check if this change affected our object (added, modified, renamed, or deleted)
            let mut object_affected = false;
            let mut object_deleted = false;
            let mut object_added = false;
            let mut renamed_from = None;
            let mut renamed_to = None;
            let mut affected_name = None; // The name used in this change

            // Check added objects (check against all tracked names)
            for obj_info in change.added_objects.iter() {
                if obj_info.object_type == VcsObjectType::MooObject
                    && tracked_names.contains(&obj_info.name)
                {
                    object_affected = true;
                    object_added = true;
                    affected_name = Some(obj_info.name.clone());
                    break;
                }
            }

            // Check modified objects (check against all tracked names)
            for obj_info in change.modified_objects.iter() {
                if obj_info.object_type == VcsObjectType::MooObject
                    && tracked_names.contains(&obj_info.name)
                {
                    object_affected = true;
                    affected_name = Some(obj_info.name.clone());
                    break;
                }
            }

            // Check deleted objects (check against all tracked names)
            for obj_info in change.deleted_objects.iter() {
                if obj_info.object_type == VcsObjectType::MooObject
                    && tracked_names.contains(&obj_info.name)
                {
                    object_affected = true;
                    object_deleted = true;
                    affected_name = Some(obj_info.name.clone());
                    break;
                }
            }

            // Check renamed objects (check if any tracked name was involved)
            for renamed in change.renamed_objects.iter() {
                if renamed.from.object_type == VcsObjectType::MooObject
                    && (tracked_names.contains(&renamed.from.name)
                        || tracked_names.contains(&renamed.to.name))
                {
                    object_affected = true;
                    renamed_from = Some(renamed.from.name.clone());
                    renamed_to = Some(renamed.to.name.clone());
                    affected_name = Some(renamed.to.name.clone());
                    break;
                }
            }

            // If this change affected the object, add it to history
            if object_affected {
                // Get detailed object changes
                let object_change = if !object_deleted {
                    // Find the object version in this change (check against all tracked names)
                    let obj_info = change
                        .added_objects
                        .iter()
                        .chain(change.modified_objects.iter())
                        .find(|obj| {
                            obj.object_type == VcsObjectType::MooObject
                                && tracked_names.contains(&obj.name)
                        });

                    if let Some(obj_info) = obj_info {
                        // Get the object name to use for comparison
                        let comparison_name = affected_name.as_ref().unwrap_or(&request.object_name);

                        match compare_object_versions(
                            &self.database,
                            comparison_name,
                            obj_info.version,
                        ) {
                            Ok(change) => Some(change),
                            Err(e) => {
                                error!(
                                    "Failed to get detailed changes for '{}' at version {}: {}",
                                    comparison_name, obj_info.version, e
                                );
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                history.push(HistoryEntry {
                    change_id: change.id.clone(),
                    change_message: change.name.clone(),
                    change_description: change.description.clone(),
                    author: change.author.clone(),
                    timestamp: change.timestamp,
                    object_added,
                    object_deleted,
                    renamed_from,
                    renamed_to,
                    object_change,
                });
            }
        }

        info!(
            "Found {} history entries for object '{}'",
            history.len(),
            request.object_name
        );

        Ok(history)
    }
}

/// A single history entry for an object
#[derive(Debug, Clone)]
struct HistoryEntry {
    change_id: String,
    change_message: String,
    change_description: Option<String>,
    author: String,
    timestamp: u64,
    object_added: bool,
    object_deleted: bool,
    renamed_from: Option<String>,
    renamed_to: Option<String>,
    object_change: Option<crate::object_diff::ObjectChange>,
}

impl HistoryEntry {
    /// Convert this HistoryEntry to a MOO v_map
    fn to_moo_var(&self) -> Var {
        let mut pairs = Vec::new();

        // Basic change information
        pairs.push((v_str("change_id"), v_str(&self.change_id)));
        pairs.push((v_str("change_message"), v_str(&self.change_message)));

        if let Some(desc) = &self.change_description {
            pairs.push((v_str("change_description"), v_str(desc)));
        }

        pairs.push((v_str("author"), v_str(&self.author)));
        pairs.push((v_str("timestamp"), v_int(self.timestamp as i64)));

        // Operation type flags
        pairs.push((v_str("object_added"), v_int(if self.object_added { 1 } else { 0 })));
        pairs.push((v_str("object_deleted"), v_int(if self.object_deleted { 1 } else { 0 })));

        // Rename information
        if let Some(from) = &self.renamed_from {
            pairs.push((v_str("renamed_from"), v_str(from)));
        }
        if let Some(to) = &self.renamed_to {
            pairs.push((v_str("renamed_to"), v_str(to)));
        }

        // Object change details (if available)
        if let Some(obj_change) = &self.object_change {
            pairs.push((v_str("details"), obj_change.to_moo_var()));
        }

        v_map(&pairs)
    }
}

impl Operation for ObjectHistoryOperation {
    fn name(&self) -> &'static str {
        "object/history"
    }

    fn description(&self) -> &'static str {
        "Retrieves the complete change history for a specific MOO object, showing all modifications, renames, and state changes"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn philosophy(&self) -> &'static str {
        "Provides a chronological history of all changes that affected a specific MOO object. This \
        operation walks through the entire change history in order, identifying each point where \
        the object was created, modified, renamed, or deleted. For each change, it provides detailed \
        information about what specifically changed, including verb and property modifications. The \
        result is a MOO list of maps, with each map representing one change that affected the object. \
        This is useful for auditing object changes, understanding the evolution of code, tracking \
        down when specific modifications were made, and for debugging purposes."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![OperationParameter {
            name: "object_name".to_string(),
            description: "The name of the MOO object to retrieve history for (e.g., '$player', '#123')"
                .to_string(),
            required: true,
        }]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Get the complete history of an object".to_string(),
                moocode: r#"history = worker_request("vcs", {"object/history", "$player"});
// Returns a list of maps, each containing:
// - change_id: the change ID
// - change_message: the message of the change
// - author: who made the change
// - timestamp: when the change was made
// - details: map of detailed changes (verbs_added, verbs_modified, props_added, etc.)
for entry in (history)
    player:tell("Change: ", entry["change_message"], " by ", entry["author"]);
    if ("details" in entry)
        details = entry["details"];
        player:tell("  Verbs modified: ", toliteral(details["verbs_modified"]));
        player:tell("  Props modified: ", toliteral(details["props_modified"]));
    endif
endfor"#
                    .to_string(),
                http_curl: Some(
                    r#"curl -X POST http://localhost:8081/api/object/history \
  -H "Content-Type: application/json" \
  -d '{"operation": "object/history", "args": ["$player"]}'"#
                        .to_string(),
                ),
            },
            OperationExample {
                description: "Check if an object was recently modified".to_string(),
                moocode: r##"history = worker_request("vcs", {"object/history", "$room"});
if (length(history) > 0)
    last_change = history[$];
    player:tell("Last modified in change: ", last_change["change_message"]);
    player:tell("By: ", last_change["author"], " at ", ctime(last_change["timestamp"]));
endif"##
                    .to_string(),
                http_curl: None,
            },
            OperationExample {
                description: "Find when a specific verb was added".to_string(),
                moocode: r#"history = worker_request("vcs", {"object/history", "$player"});
for entry in (history)
    if ("details" in entry && "look" in entry["details"]["verbs_added"])
        player:tell("Verb 'look' was added in: ", entry["change_message"]);
        break;
    endif
endfor"#
                    .to_string(),
                http_curl: None,
            },
        ]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/object/history".to_string(),
            method: Method::POST,
            is_json: true,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully - returns list of change history entries",
                r#"[
  {
    "change_id": "abc123def456...",
    "change_message": "Add player object",
    "author": "wizard",
    "timestamp": 1234567890,
    "object_added": 1,
    "details": {
      "obj_id": "$player",
      "verbs_added": ["look", "say"],
      "props_added": ["name", "description"]
    }
  },
  {
    "change_id": "def456ghi789...",
    "change_message": "Update player verbs",
    "author": "developer",
    "timestamp": 1234567900,
    "details": {
      "obj_id": "$player",
      "verbs_modified": ["look"],
      "props_modified": ["description"]
    }
  }
]"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Object name is required",
                r#"E_INVARG("Object name is required")"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - Object has no history (never existed)",
                r#"[]"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, _user: &User) -> Var {
        // For RPC calls, we expect the args to contain:
        // args[0] = object_name

        if args.is_empty() {
            error!("Object history operation requires object name");
            return v_error(E_INVARG.msg("Object name is required"));
        }

        let object_name = args[0].clone();

        let request = ObjectHistoryRequest { object_name };

        match self.process_object_history(request) {
            Ok(history) => {
                info!(
                    "Object history operation completed successfully with {} entries",
                    history.len()
                );

                // Convert history entries to MOO list of maps
                let history_vars: Vec<Var> = history
                    .iter()
                    .map(|entry| entry.to_moo_var())
                    .collect();

                v_list(&history_vars)
            }
            Err(e) => {
                error!("Object history operation failed: {}", e);
                v_error(E_INVARG.msg(format!("{e}")))
            }
        }
    }
}

