use crate::operations::{Operation, OperationRoute, OperationParameter, OperationExample};
use axum::http::Method;
use tracing::{error, info, debug};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;
use crate::types::{User, VcsObjectType};
use moor_objdef::dump_object;
use moor_var::{v_error, E_INVARG};

/// Request structure for object update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectUpdateRequest {
    pub object_name: String,
    pub vars: Vec<String>, // List of strings representing the MOO object dump
}

/// Object update operation that accepts a list of Vars and compiles them into an ObjectDefinition
#[derive(Clone)]
pub struct ObjectUpdateOperation {
    database: DatabaseRef,
}

impl ObjectUpdateOperation {
    /// Create a new object update operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }

    /// Parse and process the object update request
    fn process_object_update(&self, request: ObjectUpdateRequest, author: Option<String>) -> Result<String, ObjectsTreeError> {
        info!("Processing object update for '{}' with {} var(s)", request.object_name, request.vars.len());
        
        // Get or create a local change using the index
        let mut current_change = self.database.index().get_or_create_local_change(author)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // The index already manages the current change, so we don't need repository management
        // The change is already set as top change in index via get_or_create_local_change
    
        // Join all the var strings into a single MOO object dump
        let object_dump = request.vars.join("\n");
        
        debug!("Object dump for '{}':\n{}", request.object_name, object_dump);
        
        // Parse the object dump into an ObjectDefinition (validates the syntax)
        let mut object_def = self.database.objects().parse_object_dump(&object_dump)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Check if meta exists for this object
        let meta = match self.database.refs().get_ref(VcsObjectType::MooMetaObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
            Some(meta_sha256) => {
                // Meta exists, load it
                let yaml = self.database.objects().get(&meta_sha256)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                    .ok_or_else(|| ObjectsTreeError::SerializationError("Meta SHA256 exists but data not found".to_string()))?;
                Some(self.database.objects().parse_meta_dump(&yaml)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?)
            }
            None => None,
        };
        
        // Filter the object if meta exists with ignored items
        let final_dump = if let Some(meta) = meta {
            if !meta.ignored_properties.is_empty() || !meta.ignored_verbs.is_empty() {
                info!("Filtering object '{}' update - ignoring {} properties and {} verbs", 
                      request.object_name, meta.ignored_properties.len(), meta.ignored_verbs.len());
                
                // Filter out ignored properties from property_definitions
                object_def.property_definitions.retain(|prop| {
                    !meta.ignored_properties.contains(&prop.name.as_string())
                });
                
                // Filter out ignored properties from property_overrides
                object_def.property_overrides.retain(|prop| {
                    !meta.ignored_properties.contains(&prop.name.as_string())
                });
                
                // Filter out ignored verbs
                object_def.verbs.retain(|verb| {
                    // A verb can have multiple names, check all of them
                    !verb.names.iter().any(|name| meta.ignored_verbs.contains(&name.as_string()))
                });
                
                // Re-dump the filtered object
                let index_names = HashMap::new(); // Empty index for simple object names
                let lines = dump_object(&index_names, &object_def)
                    .map_err(|e| ObjectsTreeError::SerializationError(format!("Failed to dump filtered object: {e}")))?;
                
                let filtered = lines.join("\n");
                info!("Filtered object '{}', reduced from {} to {} lines", 
                      request.object_name, object_dump.lines().count(), lines.len());
                filtered
            } else {
                info!("No filtering needed for object '{}' (meta exists but is empty)", request.object_name);
                object_dump
            }
        } else {
            info!("No meta exists for object '{}', no filtering needed", request.object_name);
            object_dump
        };
        
        // Generate SHA256 hash for the (potentially filtered) object dump
        let sha256_key = self.database.objects().generate_sha256_hash(&final_dump);
        info!("Generated SHA256 key '{}' for object '{}'", sha256_key, request.object_name);
        
        // Check if this content already exists (exact same SHA256)
        let existing_sha256 = self.database.refs().get_ref(VcsObjectType::MooObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        let is_duplicate_content = existing_sha256.is_some_and(|existing| existing == sha256_key);
        
        if is_duplicate_content {
            info!("Object '{}' content is unchanged (same SHA256), skipping version increment", request.object_name);
            // Early return - no changes needed
            return Ok(format!("Object '{}' unchanged (no modifications)", request.object_name));
        }
        
        // Store the (potentially filtered) object definition in the database by SHA256 key
        self.database.objects().store(&sha256_key, &final_dump)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Check if this object exists in refs to determine adding vs modifying
        // Do this BEFORE updating the refs to avoid race condition
        let is_existing_object = self.database.refs().get_ref(VcsObjectType::MooObject, &request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .is_some();
        
        // Check if this object is in deleted_objects (resurrect it if so)
        let was_deleted = current_change.deleted_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooObject)
            .any(|obj| obj.name == request.object_name);
        
        if was_deleted {
            // Remove from deleted_objects (resurrect the object)
            current_change.deleted_objects.retain(|obj| !(obj.object_type == VcsObjectType::MooObject && obj.name == request.object_name));
            info!("Object '{}' was deleted, removing from deleted_objects (resurrecting)", request.object_name);
        }
        
        // Check if this object is already modified/added in the current change (filter to MooObject types)
        let is_already_in_change = current_change.added_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooObject)
            .any(|obj| obj.name == request.object_name) ||
            current_change.modified_objects.iter()
            .filter(|obj| obj.object_type == VcsObjectType::MooObject)
            .any(|obj| obj.name == request.object_name);
        
        let version;
        if is_already_in_change {
            // Object was already modified in this change, use the current version
            // instead of incrementing it
            version = self.database.refs().get_current_version(VcsObjectType::MooObject, &request.object_name)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .unwrap_or(1); // Default to 1 if no version exists yet
            
            // Get the old SHA256 for this version
            if let Some(old_sha256) = self.database.refs().get_ref(VcsObjectType::MooObject, &request.object_name, Some(version))
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
                
                // Check if the old SHA256 is referenced by any other ref (excluding this object:version)
                let is_referenced = self.database.refs().is_sha256_referenced_excluding(&old_sha256, VcsObjectType::MooObject, &request.object_name, version)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                
                if !is_referenced {
                    // The old SHA256 is orphaned, delete it
                    info!("Deleting orphaned SHA256 '{}' for object '{}' version {} (replaced in same change)", 
                          old_sha256, request.object_name, version);
                    self.database.objects().delete(&old_sha256)
                        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
                } else {
                    info!("Old SHA256 '{}' is still referenced by other refs, keeping it", old_sha256);
                }
            }
            
            info!("Updating object '{}' again in same change (keeping version {})", request.object_name, version);
        } else {
            // First time modifying this object in this change, increment version
            version = self.database.refs().get_next_version(VcsObjectType::MooObject, &request.object_name)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        }
        
        // Update or create the object reference (latest version tracking)
        self.database.refs().update_ref(VcsObjectType::MooObject, &request.object_name, version, &sha256_key)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        
        // Check if this object has been renamed in the current change (filter to MooObject types)
        // If updating the renamed object (to.name), skip tracking (already tracked as rename)
        // If updating with the old name (from.name), treat as new object (the old one was renamed away)
        let is_renamed_to = current_change.renamed_objects.iter()
            .filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject)
            .any(|renamed| renamed.to.name == request.object_name);
        
        let was_renamed_from = current_change.renamed_objects.iter()
            .filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject)
            .any(|renamed| renamed.from.name == request.object_name);
        
        if is_renamed_to {
            // This is updating the renamed object itself - skip change tracking
            info!("Object '{}' is the target of a rename in this change, skipping change tracking", request.object_name);
        } else if !is_already_in_change {
            // Only add to tracking lists if this is the first time we're modifying this object in this change
            // If it's already in the change (added_objects or modified_objects), leave it where it is
            
            // If the object name was renamed away, treat this as a NEW object even if it exists in refs
            if was_renamed_from {
                // The object with this name was renamed away, so this is a new object
                let obj_info = crate::types::ObjectInfo { 
                    object_type: VcsObjectType::MooObject,
                    name: request.object_name.clone(), 
                    version 
                };
                current_change.added_objects.push(obj_info.clone());
                info!("Added object '{}' to added_objects (old name was renamed away)", request.object_name);
            } else if is_existing_object {
                // Object existed before this change started, add to modified_objects
                let obj_info = crate::types::ObjectInfo { 
                    object_type: VcsObjectType::MooObject,
                    name: request.object_name.clone(), 
                    version 
                };
                current_change.modified_objects.push(obj_info.clone());
                info!("Added object '{}' to modified_objects in change '{}'", request.object_name, current_change.name);
            } else {
                // Object is new in this change, add to added_objects
                let obj_info = crate::types::ObjectInfo { 
                    object_type: VcsObjectType::MooObject,
                    name: request.object_name.clone(), 
                    version 
                };
                current_change.added_objects.push(obj_info.clone());
                info!("Added object '{}' to added_objects in change '{}'", request.object_name, current_change.name);
            }
        } else {
            info!("Object '{}' is already tracked in change '{}', not updating tracking lists", request.object_name, current_change.name);
        }
        
        // Update the change in the database
        self.database.index().update_change(&current_change)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        info!("Successfully updated object '{}' (version {})", request.object_name, version);
        Ok(format!("Object '{}' updated successfully with version {}", request.object_name, version))
    }
}

impl Operation for ObjectUpdateOperation {
    fn name(&self) -> &'static str {
        "object/update"
    }
    
    fn description(&self) -> &'static str {
        "Updates a MOO object definition by parsing a list of vars and compiling them into an ObjectDefinition"
    }
    
    fn response_content_type(&self) -> &'static str {
        "text/x-moo"
    }
    
    fn philosophy(&self) -> &'static str {
        "Updates or adds a MOO object definition to the current changelist. This operation is central to \
        the VCS workflow - when you modify an object in your MOO database and want to track that change, \
        you use this operation to commit it to version control. The operation automatically tracks whether \
        this is a new object (adding it to added_objects) or a modification of an existing object (adding \
        it to modified_objects). Changes are staged in your current changelist and won't be permanently \
        committed until you submit the change. If meta filtering is configured for the object, ignored \
        properties and verbs are automatically filtered out before storage."
    }
    
    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "object_name".to_string(),
                description: "The name of the MOO object to update (e.g., '$player', '#123')".to_string(),
                required: true,
            },
            OperationParameter {
                name: "vars".to_string(),
                description: "List of strings representing the MOO object dump in objdef format. \
                             Each string is a line of the object definition.".to_string(),
                required: true,
            }
        ]
    }
    
    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Update an object definition".to_string(),
                moocode: "// First, get the object definition lines\nobjdef_lines = {\"obj number-123\", \"parent number-1\", \"name \\\"My Object\\\"\", \"owner number-2\"};\nresult = worker_request(\"vcs\", {\"object/update\", \"number-123\", objdef_lines});\n// Returns: \"Object 'number-123' updated successfully with version 2\"".to_string(),
                http_curl: Some("curl -X POST http://localhost:8081/api/object/update \\\n  -H \"Content-Type: application/json\" \\\n  -d '{\"operation\": \"object/update\", \"args\": [\"number-123\", [\"obj number-123\", \"parent number-1\"]]}'".to_string()),
            },
            OperationExample {
                description: "Create a new object in version control".to_string(),
                moocode: r#"// Define a new object
new_obj = {"obj $my_new_object", "parent $container", "name \"New Container\""};
result = worker_request("vcs", {"object/update", "$my_new_object", new_obj});
// The object is now tracked in your current changelist"#.to_string(),
                http_curl: None,
            }
        ]
    }
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/api/object/update".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully - object updated with new version",
                r#""Object '$player' updated successfully with version 2""#
            ),
            OperationResponse::success(
                "Operation executed successfully - object unchanged",
                r#""Object '$player' unchanged (no modifications)""#
            ),
            OperationResponse::new(
                400,
                "Bad Request - Missing object name",
                r#"E_INVARG("Object name is required")"#
            ),
            OperationResponse::new(
                400,
                "Bad Request - Missing object vars",
                r#"E_INVARG("At least one var is required")"#
            ),
            OperationResponse::new(
                400,
                "Bad Request - Failed to parse object dump",
                r#"E_INVARG("Failed to parse object: invalid syntax")"#
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Meta SHA256 exists but data not found",
                r#"E_INVARG("Meta SHA256 exists but data not found")"#
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Failed to dump filtered object",
                r#"E_INVARG("Failed to dump filtered object: serialization error")"#
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Database error",
                r#"E_INVARG("Database error: failed to update object")"#
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, user: &User) -> moor_var::Var {
        // For RPC calls, we expect the args to contain:
        // args[0] = object_name
        // args[1..] = the var strings (either JSON encoded or individual strings)
        
        info!("Object update operation received {} arguments: {:?}", args.len(), args);
        
        if args.is_empty() {
            error!("Object update operation requires at least object name");
            return v_error(E_INVARG.msg("Object name is required"));
        }

        let object_name = args[0].clone();
        let mut vars = Vec::new();

        // Handle the case where args[1] might be a JSON-encoded list of strings
        if args.len() == 2 {
            // Try to parse the second argument as JSON array of strings
            if let Ok(json_vars) = serde_json::from_str::<Vec<String>>(&args[1]) {
                vars = json_vars;
            } else {
                // If not JSON, treat it as a single string
                vars.push(args[1].clone());
            }
        } else if args.len() > 2 {
            // Multiple arguments - use them as individual strings
            vars = args[1..].to_vec();
        } else {
            error!("Object update operation requires at least one var");
            return v_error(E_INVARG.msg("At least one var is required"));
        }

        if vars.is_empty() {
            error!("Object update operation requires at least one var");
            return v_error(E_INVARG.msg("At least one var is required"));
        }

        let request = ObjectUpdateRequest {
            object_name,
            vars,
        };

        match self.process_object_update(request, Some(user.id.clone())) {
            Ok(result) => {
                info!("Object update operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Object update operation failed: {}", e);
                v_error(E_INVARG.msg(e.to_string()))
            }
        }
    }
}
