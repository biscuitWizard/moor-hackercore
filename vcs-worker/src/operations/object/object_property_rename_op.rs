use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use tracing::{error, info};

use crate::database::DatabaseRef;
use crate::providers::index::IndexProvider;
use crate::providers::refs::RefsProvider;
use crate::types::{ObjectsTreeError, PropertyRenameHint, User, VcsObjectType};
use moor_var::{v_err, v_str, E_INVARG, Var};

/// Object property rename operation that adds a hint for a property rename
#[derive(Clone)]
pub struct ObjectPropertyRenameOperation {
    database: DatabaseRef,
}

impl ObjectPropertyRenameOperation {
    /// Create a new object property rename operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Process the property rename hint request
    fn process_property_rename(
        &self,
        object_name: &str,
        from_prop: &str,
        to_prop: &str,
        author: Option<String>,
    ) -> Result<String, ObjectsTreeError> {
        info!(
            "Adding property rename hint for object '{}': '{}' -> '{}'",
            object_name, from_prop, to_prop
        );

        // Validate that names are not empty
        if object_name.is_empty() || from_prop.is_empty() || to_prop.is_empty() {
            error!("Cannot create property rename hint with empty names");
            return Err(ObjectsTreeError::InvalidOperation(
                "Object name and property names cannot be empty".to_string(),
            ));
        }

        // Check that we're not using the same name
        if from_prop == to_prop {
            error!("Cannot rename property to the same name");
            return Err(ObjectsTreeError::InvalidOperation(
                "Cannot rename property to the same name".to_string(),
            ));
        }

        // Get or create a local change
        let mut current_change = self
            .database
            .index()
            .get_or_create_local_change(author)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

    // Check if the object was added in this change - rename hints don't make sense for new objects
    let is_added_in_change = current_change
        .added_objects
        .iter()
        .any(|obj| obj.object_type == VcsObjectType::MooObject && obj.name == object_name);

    if is_added_in_change {
        error!(
            "Cannot create property rename hint for object '{}' - it was added in this change (no baseline to rename from)",
            object_name
        );
        return Err(ObjectsTreeError::InvalidOperation(format!(
            "Cannot create property rename hint for object '{}' - it was added in this change (no previous version exists)",
            object_name
        )));
    }

    // Check if the object exists (either modified in change or in the index)
    let object_exists = current_change
        .modified_objects
        .iter()
        .any(|obj| obj.object_type == VcsObjectType::MooObject && obj.name == object_name)
        || self
            .database
            .refs()
            .get_ref(VcsObjectType::MooObject, object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .is_some();

    if !object_exists {
        error!("Object '{}' not found in previous changes", object_name);
        return Err(ObjectsTreeError::ObjectNotFound(format!(
            "Object '{}' not found in previous changes",
            object_name
        )));
    }

        // Check if there's already a hint for this object and from_prop
        // Remove it if it exists (we're updating it)
        current_change
            .property_rename_hints
            .retain(|hint| !(hint.object_name == object_name && hint.from_prop == from_prop));

        // Check if we're renaming back (creating a no-op rename chain)
        // For example: A->B (existing hint) + B->A (new hint) = no-op
        let is_rename_back = current_change
            .property_rename_hints
            .iter()
            .any(|hint| hint.object_name == object_name && hint.to_prop == from_prop && hint.from_prop == to_prop);

        if is_rename_back {
            // Remove the existing hint that this would cancel out
            current_change
                .property_rename_hints
                .retain(|hint| !(hint.object_name == object_name && hint.to_prop == from_prop));
            
            info!(
                "Removed rename-back hint for object '{}' property '{}' -> '{}'",
                object_name, to_prop, from_prop
            );
        } else {
            // Check if there's a hint chain: A->B exists, and we're adding B->C
            // We should update the existing hint to A->C
            if let Some(existing_hint) = current_change
                .property_rename_hints
                .iter_mut()
                .find(|hint| hint.object_name == object_name && hint.to_prop == from_prop)
            {
                info!(
                    "Updating hint chain for object '{}': '{}' -> '{}' -> '{}'",
                    object_name, existing_hint.from_prop, from_prop, to_prop
                );
                existing_hint.to_prop = to_prop.to_string();
            } else {
                // Add the new hint
                let hint = PropertyRenameHint {
                    object_name: object_name.to_string(),
                    from_prop: from_prop.to_string(),
                    to_prop: to_prop.to_string(),
                };
                current_change.property_rename_hints.push(hint);
                
                info!(
                    "Added property rename hint for object '{}': '{}' -> '{}'",
                    object_name, from_prop, to_prop
                );
            }
        }

        // Store the updated change
        self.database
            .index()
            .store_change(&current_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        Ok(format!(
            "Property rename hint added for object '{}': '{}' -> '{}'",
            object_name, from_prop, to_prop
        ))
    }
}

impl Operation for ObjectPropertyRenameOperation {
    fn name(&self) -> &'static str {
        "object/property/rename"
    }

    fn description(&self) -> &'static str {
        "Add a hint that a property has been renamed on an object"
    }

    fn philosophy(&self) -> &'static str {
        "Adds a rename hint to the current local change indicating that a property has been renamed. \
        This hint is used by the diff tool to properly track property renames instead of showing them as \
        delete+add. Hints are kept permanently with the change for historical tracking. \
        Note: Hints can only be created for objects that existed in previous merged changes, not for \
        objects that were added in the current local change."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "object_name".to_string(),
                description: "Name of the object containing the property".to_string(),
                required: true,
            },
            OperationParameter {
                name: "from_property".to_string(),
                description: "Original property name".to_string(),
                required: true,
            },
            OperationParameter {
                name: "to_property".to_string(),
                description: "New property name".to_string(),
                required: true,
            },
        ]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![OperationExample {
            description: "Add a hint that the 'name' property was renamed to 'title'".to_string(),
            moocode: r#"result = worker_request("vcs", {"object/property/rename", "$player", "name", "title"});"#.to_string(),
            http_curl: None,
        }]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            method: Method::POST,
            path: "/api/object/property/rename".to_string(),
            is_json: true,
        }]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> Var {
        if args.len() != 3 {
            return v_err(E_INVARG);
        }

        let object_name = &args[0];
        let from_prop = &args[1];
        let to_prop = &args[2];
        let author = Some(user.id.clone());

        match self.process_property_rename(object_name, from_prop, to_prop, author) {
            Ok(message) => v_str(&message),
            Err(e) => {
                error!("Failed to add property rename hint: {}", e);
                v_err(E_INVARG)
            }
        }
    }
}

