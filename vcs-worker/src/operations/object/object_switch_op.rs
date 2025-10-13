use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::object_diff::{ObjectChange, ObjectDiffModel};
use crate::providers::index::IndexProvider;
use crate::providers::workspace::WorkspaceProvider;
use crate::types::{ChangeStatus, ObjectSwitchRequest};
use crate::types::{ObjectsTreeError, User, VcsObjectType};
use moor_var::{v_error, E_INVARG};

/// Object switch operation that moves an object from the local change to a target workspace change
#[derive(Clone)]
pub struct ObjectSwitchOperation {
    database: DatabaseRef,
}

impl ObjectSwitchOperation {
    /// Create a new object switch operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the object switch request
    fn process_object_switch(
        &self,
        request: ObjectSwitchRequest,
    ) -> Result<ObjectDiffModel, ObjectsTreeError> {
        info!(
            "Processing object switch for '{}' to change '{}'",
            request.object_name, request.change_id
        );

        let force = request.force.unwrap_or(false);

        // Step 1: Get current local change (error if none exists)
        let top_change_id = self
            .database
            .index()
            .get_top_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::InvalidOperation(
                    "No local change exists. Cannot switch object without an active change."
                        .to_string(),
                )
            })?;

        let mut current_change = self
            .database
            .index()
            .get_change(&top_change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Local change '{top_change_id}' not found"
                ))
            })?;

        // Verify it's a local change
        if current_change.status != ChangeStatus::Local {
            return Err(ObjectsTreeError::InvalidOperation(format!(
                "Current change is not Local (status: {:?})",
                current_change.status
            )));
        }

        // Step 2: Verify object is in added_objects or modified_objects (error if not)
        let obj_in_added = current_change
            .added_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .find(|o| o.name == request.object_name)
            .cloned();

        let obj_in_modified = current_change
            .modified_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .find(|o| o.name == request.object_name)
            .cloned();

        let (object_info, is_added) = if let Some(obj) = obj_in_added {
            (obj, true)
        } else if let Some(obj) = obj_in_modified {
            (obj, false)
        } else {
            return Err(ObjectsTreeError::InvalidOperation(format!(
                "Object '{}' not found in current change's added or modified objects",
                request.object_name
            )));
        };

        info!(
            "Found object '{}' in {} list (version: {})",
            request.object_name,
            if is_added { "added_objects" } else { "modified_objects" },
            object_info.version
        );

        // Also check for the meta object and determine if it's in added or modified
        let meta_info_and_is_added = {
            if let Some(meta) = current_change
                .added_objects
                .iter()
                .filter(|o| o.object_type == VcsObjectType::MooMetaObject)
                .find(|o| o.name == request.object_name)
                .cloned()
            {
                Some((meta, true))
            } else if let Some(meta) = current_change
                .modified_objects
                .iter()
                .filter(|o| o.object_type == VcsObjectType::MooMetaObject)
                .find(|o| o.name == request.object_name)
                .cloned()
            {
                Some((meta, false))
            } else {
                None
            }
        };

        // Step 3: Resolve target change_id (short or long)
        let target_change_id = self
            .database
            .resolve_change_id(&request.change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        // Step 4: Get target change from workspace
        let mut target_change = self
            .database
            .workspace()
            .get_workspace_change(&target_change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Target change '{target_change_id}' not found in workspace"
                ))
            })?;

        // Step 5: Verify target change status is not Merged
        if target_change.status == ChangeStatus::Merged {
            return Err(ObjectsTreeError::InvalidOperation(format!(
                "Cannot switch object to a Merged change (change '{}' has status: {:?})",
                target_change.name, target_change.status
            )));
        }

        info!(
            "Target change '{}' found in workspace with status {:?}",
            target_change.name, target_change.status
        );

        // Step 6: Check if object exists in target change (error unless force=true)
        let obj_exists_in_target_added = target_change
            .added_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == request.object_name);

        let obj_exists_in_target_modified = target_change
            .modified_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == request.object_name);

        if (obj_exists_in_target_added || obj_exists_in_target_modified) && !force {
            return Err(ObjectsTreeError::InvalidOperation(format!(
                "Object '{}' already exists in target change '{}'. Use force=true to overwrite.",
                request.object_name, target_change.name
            )));
        }

        // Step 7 & 8: Remove from target if force=true, then add object info to target change
        if force {
            target_change.added_objects.retain(|o| {
                !(o.object_type == VcsObjectType::MooObject && o.name == request.object_name)
            });
            target_change.modified_objects.retain(|o| {
                !(o.object_type == VcsObjectType::MooObject && o.name == request.object_name)
            });
            info!(
                "Force=true: Removed existing object '{}' from target change",
                request.object_name
            );
        }

        if is_added {
            target_change.added_objects.push(object_info.clone());
            info!(
                "Added object '{}' to target change's added_objects",
                request.object_name
            );
        } else {
            target_change.modified_objects.push(object_info.clone());
            info!(
                "Added object '{}' to target change's modified_objects",
                request.object_name
            );
        }

        // Step 9: Handle meta object similarly
        if let Some((meta_info, meta_is_added)) = meta_info_and_is_added {
            info!(
                "Found meta object for '{}', moving it as well",
                request.object_name
            );

            // Remove from target if force=true
            if force {
                target_change.added_objects.retain(|o| {
                    !(o.object_type == VcsObjectType::MooMetaObject
                        && o.name == request.object_name)
                });
                target_change.modified_objects.retain(|o| {
                    !(o.object_type == VcsObjectType::MooMetaObject
                        && o.name == request.object_name)
                });
            }

            // Add meta to target
            if meta_is_added {
                target_change.added_objects.push(meta_info);
            } else {
                target_change.modified_objects.push(meta_info);
            }

            info!(
                "Moved meta object for '{}' to target change",
                request.object_name
            );
        }

        // Step 10: Remove object and meta from local change
        current_change.added_objects.retain(|o| {
            !(o.object_type == VcsObjectType::MooObject && o.name == request.object_name)
        });
        current_change.modified_objects.retain(|o| {
            !(o.object_type == VcsObjectType::MooObject && o.name == request.object_name)
        });
        current_change.added_objects.retain(|o| {
            !(o.object_type == VcsObjectType::MooMetaObject && o.name == request.object_name)
        });
        current_change.modified_objects.retain(|o| {
            !(o.object_type == VcsObjectType::MooMetaObject && o.name == request.object_name)
        });

        info!(
            "Removed object '{}' and its meta from current change",
            request.object_name
        );

        // Step 11: Update both changes in database
        self.database
            .index()
            .update_change(&current_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        self.database
            .workspace()
            .store_workspace_change(&target_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        info!(
            "Successfully switched object '{}' from change '{}' to change '{}'",
            request.object_name, current_change.name, target_change.name
        );

        // Step 12: Build and return ObjectDiffModel for the removed object
        // This represents the "revert" operation - what needs to happen in MOO to reflect
        // the object being removed from the local change
        let mut diff_model = ObjectDiffModel::new();

        // Create an ObjectChange for the removed object
        let mut object_change = ObjectChange::new(request.object_name.clone());

        // If it was in added_objects, the revert is to delete it
        // If it was in modified_objects, the revert is to restore the previous version
        if is_added {
            diff_model.add_object_deleted(request.object_name.clone());
        } else {
            diff_model.add_object_modified(request.object_name.clone());
            // For modified objects, we need to indicate what changed
            // Since we're reverting, mark it as needing restoration
            object_change.props_modified.insert("content".to_string());
        }

        diff_model.add_object_change(object_change);

        Ok(diff_model)
    }
}

impl Operation for ObjectSwitchOperation {
    fn name(&self) -> &'static str {
        "object/switch"
    }

    fn description(&self) -> &'static str {
        "Moves an object from the current local change to a target workspace change"
    }

    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }

    fn philosophy(&self) -> &'static str {
        "Enables moving individual objects between changes for better change organization. When working on \
        multiple features simultaneously, you may realize that an object belongs in a different change. \
        This operation moves the object (and its meta) from your current local change to a target change \
        in the workspace. Only objects in added_objects or modified_objects can be switched (not deleted \
        objects). The target change must not be Merged. If the object already exists in the target change, \
        the operation fails unless force=true is specified. The operation returns an ObjectDiffModel \
        showing what needs to be reverted in your MOO to reflect the object being removed from the local change."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "object_name".to_string(),
                description: "The name of the object to switch (e.g., '$player', '#123')".to_string(),
                required: true,
            },
            OperationParameter {
                name: "change_id".to_string(),
                description: "The ID of the target workspace change (short or long hash)".to_string(),
                required: true,
            },
            OperationParameter {
                name: "force".to_string(),
                description: "Optional. Set to 'true' to overwrite if object exists in target change (default: false)".to_string(),
                required: false,
            },
        ]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Switch an object to a different change".to_string(),
                moocode: r#"// First, list workspace changes to get the target change ID
workspace_list = worker_request("vcs", {"workspace/list"});
// Assuming you know the target change ID:
target_id = "abc-123-def...";

// Switch the object - returns an ObjectDiffModel as a MOO map
diff = worker_request("vcs", {"object/switch", "$my_object", target_id});
// diff shows what needs to be reverted in MOO
player:tell("Object switched. Revert operation: ", toliteral(diff));"#
                    .to_string(),
                http_curl: Some(
                    r#"curl -X POST http://localhost:8081/api/object/switch \
  -H "Content-Type: application/json" \
  -d '{"operation": "object/switch", "args": ["$my_object", "abc-123-def..."]}'"#
                        .to_string(),
                ),
            },
            OperationExample {
                description: "Force switch an object that exists in both changes".to_string(),
                moocode: r#"// Force overwrite if object exists in target
diff = worker_request("vcs", {"object/switch", "$my_object", target_id, "true"});
player:tell("Object forcibly switched");"#
                    .to_string(),
                http_curl: Some(
                    r#"curl -X POST http://localhost:8081/api/object/switch \
  -H "Content-Type: application/json" \
  -d '{"operation": "object/switch", "args": ["$my_object", "abc-123", "true"]}'"#
                        .to_string(),
                ),
            },
        ]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/object/switch".to_string(),
            method: Method::POST,
            is_json: true,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully - Object switched to target change",
                r#"["objects_renamed" -> [], "objects_deleted" -> {"$my_object"}, "objects_added" -> {}, "objects_modified" -> {}, "changes" -> {["obj_id" -> "$my_object", "verbs_modified" -> {}, "verbs_added" -> {}, "verbs_renamed" -> [], "verbs_deleted" -> {}, "props_modified" -> {}, "props_added" -> {}, "props_renamed" -> [], "props_deleted" -> {}]}]"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Object not in local change or already in target",
                r#"E_INVARG("Error: Object '$my_object' not found in current change's added or modified objects")"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - No local change exists",
                r#"E_INVARG("Error: No local change exists. Cannot switch object without an active change.")"#,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Target change is merged",
                r#"E_INVARG("Error: Cannot switch object to a Merged change")"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - Target change not found",
                r#"E_INVARG("Error: Target change 'abc-123-def...' not found in workspace")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database or system error",
                r#"E_INVARG("Error: Database error: failed to update change")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        info!(
            "Object switch operation received {} arguments for user: {}",
            args.len(),
            user.id
        );

        if args.len() < 2 {
            error!("Object switch operation requires object_name and change_id arguments");
            return v_error(E_INVARG.msg(
                "Object switch operation requires object_name and change_id arguments",
            ));
        }

        let object_name = args[0].clone();
        let change_id = args[1].clone();
        let force = args.get(2).and_then(|s| s.parse::<bool>().ok());

        let request = ObjectSwitchRequest {
            object_name,
            change_id,
            force,
        };

        match self.process_object_switch(request) {
            Ok(diff_model) => {
                info!("Object switch operation completed successfully, returning diff");
                diff_model.to_moo_var()
            }
            Err(e) => {
                error!("Object switch operation failed: {}", e);
                v_error(E_INVARG.msg(format!("Error: {e}")))
            }
        }
    }
}

