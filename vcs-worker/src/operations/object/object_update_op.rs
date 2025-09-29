use crate::operations::{Operation, OperationRoute};
use axum::http::Method;
use tracing::{error, info, debug};
use serde::{Deserialize, Serialize};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;

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
    fn process_object_update(&self, request: ObjectUpdateRequest) -> Result<String, ObjectsTreeError> {
        info!("Processing object update for '{}' with {} var(s)", request.object_name, request.vars.len());
        
        // Get or create a local change using the index
        let mut current_change = self.database.index().get_or_create_local_change()
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // The index already manages the current change, so we don't need repository management
        // The change is already set as top change in index via get_or_create_local_change
    
        // Join all the var strings into a single MOO object dump
        let object_dump = request.vars.join("\n");
        
        debug!("Object dump for '{}':\n{}", request.object_name, object_dump);
        
        // Parse the object dump into an ObjectDefinition (validates the syntax)
        let _object_def = self.database.objects().parse_object_dump(&object_dump)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Generate SHA256 hash for the object dump
        let sha256_key = self.database.objects().generate_sha256_hash(&object_dump);
        info!("Generated SHA256 key '{}' for object '{}'", sha256_key, request.object_name);
        
        // Check if this content already exists (exact same SHA256)
        let existing_sha256 = self.database.refs().get_ref(&request.object_name, None)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        let is_duplicate_content = existing_sha256.map_or(false, |existing| existing == sha256_key);
        
        if is_duplicate_content {
            info!("Object '{}' content is unchanged (same SHA256), skipping version increment", request.object_name);
            // Early return - no changes needed
            return Ok(format!("Object '{}' unchanged (no modifications)", request.object_name));
        }
        
        // Store the object definition in the database by SHA256 key
        self.database.objects().store(&sha256_key, &object_dump)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Get the next version number for this object
        let version = self.database.refs().get_next_version(&request.object_name)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Update or create the object reference (latest version tracking)
        self.database.refs().update_ref(&request.object_name, version, &sha256_key)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        
        // Check if this object has been renamed in the current change
        let is_renamed_object = current_change.renamed_objects.iter()
            .any(|renamed| renamed.from == request.object_name || renamed.to == request.object_name);
        
        if is_renamed_object {
            info!("Object '{}' has been renamed in this change, skipping change tracking", request.object_name);
        } else {
            // Check if this object exists in refs to determine adding vs modifying
            let is_existing_object = self.database.refs().get_ref(&request.object_name, None)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                .is_some();
            
            if is_existing_object {
                // Update the modified_objects list if not already present
                if !current_change.modified_objects.contains(&request.object_name) {
                    current_change.modified_objects.push(request.object_name.clone());
                    info!("Added object '{}' to modified_objects in change '{}'", request.object_name, current_change.name);
                }
                
                // Remove from added_objects if it was previously added but now we know it's modified
                if let Some(pos) = current_change.added_objects.iter().position(|name| name == &request.object_name) {
                    current_change.added_objects.remove(pos);
                    info!("Moved object '{}' from added_objects to modified_objects in change '{}'", request.object_name, current_change.name);
                }
            } else {
                // Update the added_objects list if not already present
                if !current_change.added_objects.contains(&request.object_name) {
                    current_change.added_objects.push(request.object_name.clone());
                    info!("Added object '{}' to added_objects in change '{}'", request.object_name, current_change.name);
                }
                
                // Remove from modified_objects if it was previously modified but now we know it's new
                if let Some(pos) = current_change.modified_objects.iter().position(|name| name == &request.object_name) {
                    current_change.modified_objects.remove(pos);
                    info!("Moved object '{}' from modified_objects to added_objects in change '{}'", request.object_name, current_change.name);
                }
            }
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
    
    fn routes(&self) -> Vec<OperationRoute> {
        vec![
            OperationRoute {
                path: "/object/update".to_string(),
                method: Method::POST,
                is_json: true, // Expects JSON body with object_name and vars
            },
            OperationRoute {
                path: "/api/object/update".to_string(),
                method: Method::POST,
                is_json: true,
            }
        ]
    }
    
    fn execute(&self, args: Vec<String>) -> moor_var::Var {
        // For RPC calls, we expect the args to contain:
        // args[0] = object_name
        // args[1..] = the var strings (either JSON encoded or individual strings)
        
        info!("Object update operation received {} arguments: {:?}", args.len(), args);
        
        if args.is_empty() {
            error!("Object update operation requires at least object name");
            return moor_var::v_str("Error: Object name is required");
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
            return moor_var::v_str("Error: At least one var is required");
        }

        if vars.is_empty() {
            error!("Object update operation requires at least one var");
            return moor_var::v_str("Error: At least one var is required");
        }

        let request = ObjectUpdateRequest {
            object_name,
            vars,
        };

        match self.process_object_update(request) {
            Ok(result) => {
                info!("Object update operation completed successfully");
                moor_var::v_str(&result)
            }
            Err(e) => {
                error!("Object update operation failed: {}", e);
                moor_var::v_str(&format!("Error: {e}"))
            }
        }
    }
}
