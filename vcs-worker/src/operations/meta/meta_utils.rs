use crate::database::DatabaseRef;
use crate::types::{ObjectsTreeError, VcsObjectType, MooMetaObject, ObjectInfo, Change};
use crate::providers::refs::RefsProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::index::IndexProvider;

/// Validates that a MOO object exists
pub fn validate_object_exists(
    database: &DatabaseRef,
    object_name: &str,
) -> Result<(), ObjectsTreeError> {
    database.refs().get_ref(VcsObjectType::MooObject, object_name, None)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        .ok_or_else(|| ObjectsTreeError::ObjectNotFound(format!("Object '{}' not found", object_name)))?;
    Ok(())
}

/// Loads an existing meta object or creates a default one
/// Returns (meta, meta_existed_before)
pub fn load_or_create_meta(
    database: &DatabaseRef,
    object_name: &str,
) -> Result<(MooMetaObject, bool), ObjectsTreeError> {
    let meta_existed_before = database.refs()
        .get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        .is_some();
    
    let meta = if let Some(meta_sha256) = database.refs()
        .get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
        // Meta exists, load it
        let yaml = database.objects().get(&meta_sha256)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError("Meta SHA256 exists but data not found".to_string()))?;
        database.objects().parse_meta_dump(&yaml)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
    } else {
        // No meta exists yet, create default
        MooMetaObject::default()
    };
    
    Ok((meta, meta_existed_before))
}

/// Loads an existing meta object, returns None if it doesn't exist
/// Used by clear operations that need to check if meta exists first
pub fn load_meta_if_exists(
    database: &DatabaseRef,
    object_name: &str,
) -> Result<Option<MooMetaObject>, ObjectsTreeError> {
    let meta_sha256 = match database.refs()
        .get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))? {
        Some(sha256) => sha256,
        None => return Ok(None),
    };
    
    let yaml = database.objects().get(&meta_sha256)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        .ok_or_else(|| ObjectsTreeError::SerializationError("Meta SHA256 exists but data not found".to_string()))?;
    let meta = database.objects().parse_meta_dump(&yaml)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    
    Ok(Some(meta))
}

/// Checks if a meta object is already tracked in the current change
fn is_meta_in_change(current_change: &Change, object_name: &str) -> bool {
    current_change.added_objects.iter()
        .filter(|obj| obj.object_type == VcsObjectType::MooMetaObject)
        .any(|obj| obj.name == object_name) ||
    current_change.modified_objects.iter()
        .filter(|obj| obj.object_type == VcsObjectType::MooMetaObject)
        .any(|obj| obj.name == object_name)
}

/// Saves a meta object and tracks it in the current change
/// This handles:
/// - Generating YAML dump and SHA256
/// - Storing the content
/// - Version management
/// - Updating refs
/// - Tracking in change (added vs modified)
pub fn save_and_track_meta(
    database: &DatabaseRef,
    meta: &MooMetaObject,
    object_name: &str,
    meta_existed_before: bool,
    current_change: &mut Change,
) -> Result<(), ObjectsTreeError> {
    // Generate YAML dump and SHA256
    let yaml_dump = database.objects().generate_meta_dump(meta)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    let sha256_key = database.objects().generate_sha256_hash(&yaml_dump);
    
    // Store the YAML content
    database.objects().store(&sha256_key, &yaml_dump)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    
    // Check if meta already exists in this change to determine versioning
    let is_already_in_change = is_meta_in_change(current_change, object_name);
    
    let version = if is_already_in_change {
        // Reuse the current version
        database.refs().get_current_version(VcsObjectType::MooMetaObject, object_name)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .unwrap_or(1)
    } else {
        // Increment version
        database.refs().get_next_version(VcsObjectType::MooMetaObject, object_name)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
    };
    
    // Update the ref
    database.refs().update_ref(VcsObjectType::MooMetaObject, object_name, version, &sha256_key)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    
    // Track in change if not already tracked
    if !is_already_in_change {
        let obj_info = ObjectInfo {
            object_type: VcsObjectType::MooMetaObject,
            name: object_name.to_string(),
            version,
        };
        
        if !meta_existed_before {
            current_change.added_objects.push(obj_info);
        } else {
            current_change.modified_objects.push(obj_info);
        }
    }
    
    // Always update the change
    database.index().update_change(current_change)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    
    Ok(())
}

