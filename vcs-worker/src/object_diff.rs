use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use moor_var::{Var, v_map, v_str};
use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::objects::ObjectsProvider;
use crate::providers::refs::RefsProvider;
use crate::providers::index::IndexProvider;
use crate::types::{Change, VcsObjectType};
use moor_compiler::ObjectDefinition;

/// Represents a single object change with detailed verb and property modifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectChange {
    /// Object ID - either the OID as string (e.g., "#4") or object name (e.g., "Foobar") 
    /// if the name differs from the OID
    pub obj_id: String,
    /// Verbs that were modified (existing verbs with changes)
    pub verbs_modified: HashSet<String>,
    /// Verbs that were added (new verbs)
    pub verbs_added: HashSet<String>,
    /// Verbs that were renamed (old_name -> new_name mapping)
    pub verbs_renamed: HashMap<String, String>,
    /// Verbs that were deleted
    pub verbs_deleted: HashSet<String>,
    /// Properties that were modified (existing properties with changes)
    pub props_modified: HashSet<String>,
    /// Properties that were added (new properties)
    pub props_added: HashSet<String>,
    /// Properties that were renamed (old_name -> new_name mapping)
    pub props_renamed: HashMap<String, String>,
    /// Properties that were deleted
    pub props_deleted: HashSet<String>,
}

/// Represents a complete set of object changes/deltas for communication to MOO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDiffModel {
    /// Objects that were renamed (from_obj_id -> to_obj_id mapping)
    pub objects_renamed: HashMap<String, String>,
    /// Objects that were deleted
    pub objects_deleted: HashSet<String>,
    /// Objects that were added
    pub objects_added: HashSet<String>,
    /// Objects that were modified
    pub objects_modified: HashSet<String>,
    /// Detailed list of changes for each modified object
    pub changes: Vec<ObjectChange>,
}

impl ObjectChange {
    /// Create a new empty ObjectChange
    pub fn new(obj_id: String) -> Self {
        Self {
            obj_id,
            verbs_modified: HashSet::new(),
            verbs_added: HashSet::new(),
            verbs_renamed: HashMap::new(),
            verbs_deleted: HashSet::new(),
            props_modified: HashSet::new(),
            props_added: HashSet::new(),
            props_renamed: HashMap::new(),
            props_deleted: HashSet::new(),
        }
    }

    /// Convert this ObjectChange to a MOO v_map
    pub fn to_moo_var(&self) -> Var {
        let mut pairs = Vec::new();
        
        // obj_id
        pairs.push((v_str("obj_id"), v_str(&self.obj_id)));
        
        // verbs_modified
        let verbs_modified_list: Vec<Var> = self.verbs_modified.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("verbs_modified"), moor_var::v_list(&verbs_modified_list)));
        
        // verbs_added
        let verbs_added_list: Vec<Var> = self.verbs_added.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("verbs_added"), moor_var::v_list(&verbs_added_list)));
        
        // verbs_renamed
        let verbs_renamed_map: Vec<(Var, Var)> = self.verbs_renamed.iter()
            .map(|(k, v)| (v_str(k), v_str(v)))
            .collect();
        pairs.push((v_str("verbs_renamed"), v_map(&verbs_renamed_map)));
        
        // verbs_deleted
        let verbs_deleted_list: Vec<Var> = self.verbs_deleted.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("verbs_deleted"), moor_var::v_list(&verbs_deleted_list)));
        
        // props_modified
        let props_modified_list: Vec<Var> = self.props_modified.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("props_modified"), moor_var::v_list(&props_modified_list)));
        
        // props_added
        let props_added_list: Vec<Var> = self.props_added.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("props_added"), moor_var::v_list(&props_added_list)));
        
        // props_renamed
        let props_renamed_map: Vec<(Var, Var)> = self.props_renamed.iter()
            .map(|(k, v)| (v_str(k), v_str(v)))
            .collect();
        pairs.push((v_str("props_renamed"), v_map(&props_renamed_map)));
        
        // props_deleted
        let props_deleted_list: Vec<Var> = self.props_deleted.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("props_deleted"), moor_var::v_list(&props_deleted_list)));
        
        v_map(&pairs)
    }
}

impl ObjectDiffModel {
    /// Create a new empty ObjectDiffModel
    pub fn new() -> Self {
        Self {
            objects_renamed: HashMap::new(),
            objects_deleted: HashSet::new(),
            objects_added: HashSet::new(),
            objects_modified: HashSet::new(),
            changes: Vec::new(),
        }
    }

    /// Convert this ObjectDiffModel to a MOO v_map
    pub fn to_moo_var(&self) -> Var {
        let mut pairs = Vec::new();
        
        // objects_renamed
        let objects_renamed_map: Vec<(Var, Var)> = self.objects_renamed.iter()
            .map(|(k, v)| (v_str(k), v_str(v)))
            .collect();
        pairs.push((v_str("objects_renamed"), v_map(&objects_renamed_map)));
        
        // objects_deleted
        let objects_deleted_list: Vec<Var> = self.objects_deleted.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("objects_deleted"), moor_var::v_list(&objects_deleted_list)));
        
        // objects_added
        let objects_added_list: Vec<Var> = self.objects_added.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("objects_added"), moor_var::v_list(&objects_added_list)));
        
        // objects_modified
        let objects_modified_list: Vec<Var> = self.objects_modified.iter()
            .map(|v| v_str(v))
            .collect();
        pairs.push((v_str("objects_modified"), moor_var::v_list(&objects_modified_list)));
        
        // changes
        let changes_list: Vec<Var> = self.changes.iter()
            .map(|change| change.to_moo_var())
            .collect();
        pairs.push((v_str("changes"), moor_var::v_list(&changes_list)));
        
        v_map(&pairs)
    }

    /// Add a renamed object to the model
    pub fn add_object_renamed(&mut self, from: String, to: String) {
        self.objects_renamed.insert(from, to);
    }

    /// Add a deleted object to the model
    pub fn add_object_deleted(&mut self, obj_id: String) {
        self.objects_deleted.insert(obj_id);
    }

    /// Add an added object to the model
    pub fn add_object_added(&mut self, obj_id: String) {
        self.objects_added.insert(obj_id);
    }

    /// Add a modified object to the model
    pub fn add_object_modified(&mut self, obj_id: String) {
        self.objects_modified.insert(obj_id);
    }

    /// Add or update an object change in the model
    pub fn add_object_change(&mut self, change: ObjectChange) {
        // Remove any existing change for this object
        self.changes.retain(|c| c.obj_id != change.obj_id);
        self.changes.push(change);
    }

    /// Merge another ObjectDiffModel into this one
    pub fn merge(&mut self, other: ObjectDiffModel) {
        // Merge renamed objects
        for (from, to) in other.objects_renamed {
            self.objects_renamed.insert(from, to);
        }
        
        // Merge deleted objects
        for obj_id in other.objects_deleted {
            self.objects_deleted.insert(obj_id);
        }
        
        // Merge added objects
        for obj_id in other.objects_added {
            self.objects_added.insert(obj_id);
        }
        
        // Merge modified objects
        for obj_id in other.objects_modified {
            self.objects_modified.insert(obj_id);
        }
        
        // Merge changes
        for change in other.changes {
            self.add_object_change(change);
        }
    }
}

impl Default for ObjectDiffModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to convert an object ID to an object name
/// Returns the object name if it's different from the OID, otherwise returns the OID
pub fn obj_id_to_object_name(obj_id: &str, object_name: Option<&str>) -> String {
    match object_name {
        Some(name) if name != obj_id => {
            // Capitalize first letter if it's a name
            if let Some(first_char) = name.chars().next() {
                let mut result = String::with_capacity(name.len());
                result.push(first_char.to_uppercase().next().unwrap_or(first_char));
                result.push_str(&name[1..]);
                result
            } else {
                name.to_string()
            }
        }
        _ => obj_id.to_string(),
    }
}

/// Compare object versions to determine detailed changes
pub fn compare_object_versions(database: &DatabaseRef, obj_name: &str, local_version: u64) -> Result<ObjectChange, ObjectsTreeError> {
    let mut object_change = ObjectChange::new(obj_name.to_string());
    
    // Get the local version content
    let local_sha256 = database.refs().get_ref(VcsObjectType::MooObject, obj_name, Some(local_version))
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Local version {} of object '{}' not found", local_version, obj_name)))?;
    
    let local_content = database.objects().get(&local_sha256)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Object content for SHA256 '{}' not found", local_sha256)))?;
    
    // Parse local object definition
    let local_def = database.objects().parse_object_dump(&local_content)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    
    // Get the baseline version (previous version)
    // For version 1 (new object), there is no baseline (version 0 doesn't exist)
    // For version 2+, the baseline is the previous version
    let baseline_version = if local_version > 1 { local_version - 1 } else { 0 };
    let baseline_sha256 = database.refs().get_ref(VcsObjectType::MooObject, obj_name, Some(baseline_version))
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    
    if let Some(baseline_sha256) = baseline_sha256 {
        // Get baseline content and parse it
        let baseline_content = database.objects().get(&baseline_sha256)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| ObjectsTreeError::SerializationError(format!("Baseline object content for SHA256 '{}' not found", baseline_sha256)))?;
        
        let baseline_def = database.objects().parse_object_dump(&baseline_content)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
        
        // Compare the two object definitions
        compare_object_definitions(&baseline_def, &local_def, &mut object_change);
    } else {
        // No baseline version - this is a new object, mark all as added
        for verb in &local_def.verbs {
            for verb_name in &verb.names {
                object_change.verbs_added.insert(verb_name.as_string());
            }
        }
        for prop_def in &local_def.property_definitions {
            object_change.props_added.insert(prop_def.name.as_string());
        }
        for prop_override in &local_def.property_overrides {
            object_change.props_added.insert(prop_override.name.as_string());
        }
    }
    
    Ok(object_change)
}

/// Compare two ObjectDefinitions and populate the ObjectChange with detailed differences
pub fn compare_object_definitions(baseline: &ObjectDefinition, local: &ObjectDefinition, object_change: &mut ObjectChange) {
    // Compare verbs
    let baseline_verbs: HashMap<String, &moor_compiler::ObjVerbDef> = baseline.verbs.iter()
        .flat_map(|v| v.names.iter().map(move |name| (name.as_string(), v)))
        .collect();
    
    let local_verbs: HashMap<String, &moor_compiler::ObjVerbDef> = local.verbs.iter()
        .flat_map(|v| v.names.iter().map(move |name| (name.as_string(), v)))
        .collect();
    
    // Find added, modified, and deleted verbs
    for (verb_name, local_verb) in &local_verbs {
        if let Some(baseline_verb) = baseline_verbs.get(verb_name) {
            // Verb exists in both - check if it's modified
            if verbs_differ(baseline_verb, local_verb) {
                object_change.verbs_modified.insert(verb_name.clone());
            }
        } else {
            // Verb is new
            object_change.verbs_added.insert(verb_name.clone());
        }
    }
    
    for verb_name in baseline_verbs.keys() {
        if !local_verbs.contains_key(verb_name) {
            // Verb was deleted
            object_change.verbs_deleted.insert(verb_name.clone());
        }
    }
    
    // Compare property definitions
    let baseline_props: HashMap<String, &moor_compiler::ObjPropDef> = baseline.property_definitions.iter()
        .map(|p| (p.name.as_string(), p))
        .collect();
    
    let local_props: HashMap<String, &moor_compiler::ObjPropDef> = local.property_definitions.iter()
        .map(|p| (p.name.as_string(), p))
        .collect();
    
    // Find added, modified, and deleted property definitions
    for (prop_name, local_prop) in &local_props {
        if let Some(baseline_prop) = baseline_props.get(prop_name) {
            // Property exists in both - check if it's modified
            if property_definitions_differ(baseline_prop, local_prop) {
                object_change.props_modified.insert(prop_name.clone());
            }
        } else {
            // Property is new
            object_change.props_added.insert(prop_name.clone());
        }
    }
    
    for prop_name in baseline_props.keys() {
        if !local_props.contains_key(prop_name) {
            // Property was deleted
            object_change.props_deleted.insert(prop_name.clone());
        }
    }
    
    // Compare property overrides
    let baseline_overrides: HashMap<String, &moor_compiler::ObjPropOverride> = baseline.property_overrides.iter()
        .map(|p| (p.name.as_string(), p))
        .collect();
    
    let local_overrides: HashMap<String, &moor_compiler::ObjPropOverride> = local.property_overrides.iter()
        .map(|p| (p.name.as_string(), p))
        .collect();
    
    // Find added, modified, and deleted property overrides
    for (prop_name, local_override) in &local_overrides {
        if let Some(baseline_override) = baseline_overrides.get(prop_name) {
            // Override exists in both - check if it's modified
            if property_overrides_differ(baseline_override, local_override) {
                object_change.props_modified.insert(prop_name.clone());
            }
        } else {
            // Override is new
            object_change.props_added.insert(prop_name.clone());
        }
    }
    
    for prop_name in baseline_overrides.keys() {
        if !local_overrides.contains_key(prop_name) {
            // Override was deleted
            object_change.props_deleted.insert(prop_name.clone());
        }
    }
}

/// Check if two verb definitions differ
pub fn verbs_differ(baseline: &moor_compiler::ObjVerbDef, local: &moor_compiler::ObjVerbDef) -> bool {
    baseline.argspec != local.argspec ||
    baseline.owner != local.owner ||
    baseline.flags != local.flags ||
    baseline.program != local.program
}

/// Check if two property definitions differ
pub fn property_definitions_differ(baseline: &moor_compiler::ObjPropDef, local: &moor_compiler::ObjPropDef) -> bool {
    baseline.perms != local.perms ||
    baseline.value != local.value
}

/// Check if two property overrides differ
pub fn property_overrides_differ(baseline: &moor_compiler::ObjPropOverride, local: &moor_compiler::ObjPropOverride) -> bool {
    baseline.value != local.value ||
    baseline.perms_update != local.perms_update
}

/// Build an ObjectDiffModel by comparing a change against the compiled state
/// This is the shared logic used by approve and status operations
pub fn build_object_diff_from_change(database: &DatabaseRef, change: &Change) -> Result<ObjectDiffModel, ObjectsTreeError> {
    let mut diff_model = ObjectDiffModel::new();
    
    // Get the complete object list from the index state (excluding the local change)
    let complete_object_list = database.index().compute_complete_object_list()
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    
    tracing::info!("Using complete object list with {} objects as baseline for change '{}'", 
          complete_object_list.len(), change.name);
    
    // Process the change to build the diff
    process_change_for_diff(database, &mut diff_model, change)?;
    
    Ok(diff_model)
}

/// Process a single change and add its modifications to the diff model
/// This is the shared logic used by approve and status operations
pub fn process_change_for_diff(database: &DatabaseRef, diff_model: &mut ObjectDiffModel, change: &Change) -> Result<(), ObjectsTreeError> {
    // Process added objects (filter to only MooObject types)
    for obj_info in change.added_objects.iter().filter(|o| o.object_type == VcsObjectType::MooObject) {
        let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
        diff_model.add_object_added(obj_name.clone());
        
        // Get detailed object changes by comparing local vs baseline (which will be empty for new objects)
        let object_change = compare_object_versions(database, &obj_name, obj_info.version)?;
        diff_model.add_object_change(object_change);
    }
    
    // Process deleted objects (filter to only MooObject types)
    for obj_info in change.deleted_objects.iter().filter(|o| o.object_type == VcsObjectType::MooObject) {
        let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
        diff_model.add_object_deleted(obj_name);
    }
    
    // Process renamed objects (filter to only MooObject types)
    for renamed in change.renamed_objects.iter().filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject) {
        let from_name = obj_id_to_object_name(&renamed.from.name, Some(&renamed.from.name));
        let to_name = obj_id_to_object_name(&renamed.to.name, Some(&renamed.to.name));
        diff_model.add_object_renamed(from_name, to_name);
    }
    
    // Process modified objects with detailed comparison (filter to only MooObject types)
    for obj_info in change.modified_objects.iter().filter(|o| o.object_type == VcsObjectType::MooObject) {
        let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
        diff_model.add_object_modified(obj_name.clone());
        
        // Get detailed object changes by comparing local vs baseline
        let object_change = compare_object_versions(database, &obj_name, obj_info.version)?;
        diff_model.add_object_change(object_change);
    }
    
    Ok(())
}

/// Build an ObjectDiffModel for abandoning a change (undo operations)
/// This creates the reverse operations needed to undo the change
pub fn build_abandon_diff_from_change(database: &DatabaseRef, change: &Change) -> Result<ObjectDiffModel, ObjectsTreeError> {
    // Get the complete object list from the index state for comparison
    let complete_object_list = database.index().compute_complete_object_list()
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;
    
    tracing::info!("Using complete object list with {} objects as baseline for abandoning change '{}'", 
          complete_object_list.len(), change.name);
    
    // Create a delta model showing what needs to be undone
    let mut undo_delta = ObjectDiffModel::new();
    
    // Get object name mappings for better display names
    let object_names = get_object_names_for_change(change);
    
    // Process added objects - to undo, we need to delete them (filter to only MooObject types)
    for added_obj in change.added_objects.iter().filter(|o| o.object_type == VcsObjectType::MooObject) {
        let object_name = obj_id_to_object_name(&added_obj.name, object_names.get(&added_obj.name).map(|s| s.as_str()));
        undo_delta.add_object_deleted(object_name);
    }
    
    // Process deleted objects - to undo, we need to add them back (filter to only MooObject types)
    for deleted_obj in change.deleted_objects.iter().filter(|o| o.object_type == VcsObjectType::MooObject) {
        let object_name = obj_id_to_object_name(&deleted_obj.name, object_names.get(&deleted_obj.name).map(|s| s.as_str()));
        undo_delta.add_object_added(object_name);
    }
    
    // Process renamed objects - to undo, we need to rename them back (filter to only MooObject types)
    for renamed in change.renamed_objects.iter().filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject) {
        let from_name = obj_id_to_object_name(&renamed.from.name, object_names.get(&renamed.from.name).map(|s| s.as_str()));
        let to_name = obj_id_to_object_name(&renamed.to.name, object_names.get(&renamed.to.name).map(|s| s.as_str()));
        undo_delta.add_object_renamed(to_name, from_name);
    }
    
    // Process modified objects - to undo, we need to mark them as modified
    // and create basic ObjectChange entries (filter to only MooObject types)
    for modified_obj in change.modified_objects.iter().filter(|o| o.object_type == VcsObjectType::MooObject) {
        let object_name = obj_id_to_object_name(&modified_obj.name, object_names.get(&modified_obj.name).map(|s| s.as_str()));
        undo_delta.add_object_modified(object_name.clone());
        
        // Create a basic ObjectChange for modified objects
        // In a real implementation, you'd want to track what specifically changed
        let mut object_change = ObjectChange::new(object_name);
        object_change.props_modified.insert("content".to_string());
        undo_delta.add_object_change(object_change);
    }
    
    Ok(undo_delta)
}

/// Get object names for the change objects to improve display names
/// This is a simplified implementation - in practice you'd want to
/// query the actual object names from the MOO database
pub fn get_object_names_for_change(change: &Change) -> HashMap<String, String> {
    let mut object_names = HashMap::new();
    
    // Try to get object names from workspace provider (filter to only MooObject types)
    for obj_info in change.added_objects.iter()
        .chain(change.modified_objects.iter())
        .chain(change.deleted_objects.iter())
        .filter(|o| o.object_type == VcsObjectType::MooObject) {
        
        // For now, we'll just use the object name as the name
        // In a real implementation, you'd query the actual object names
        object_names.insert(obj_info.name.clone(), obj_info.name.clone());
    }
    
    for renamed in change.renamed_objects.iter()
        .filter(|r| r.from.object_type == VcsObjectType::MooObject && r.to.object_type == VcsObjectType::MooObject) {
        object_names.insert(renamed.from.name.clone(), renamed.from.name.clone());
        object_names.insert(renamed.to.name.clone(), renamed.to.name.clone());
    }
    
    object_names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_change_to_moo_var() {
        let mut change = ObjectChange::new("TestObject".to_string());
        change.verbs_added.insert("new_verb".to_string());
        change.props_modified.insert("existing_prop".to_string());
        
        let moo_var = change.to_moo_var();
        
        // Verify it's a map
        assert!(matches!(moo_var.variant(), moor_var::Variant::Map(_)));
    }

    #[test]
    fn test_object_diff_model_to_moo_var() {
        let mut model = ObjectDiffModel::new();
        model.add_object_added("NewObject".to_string());
        model.add_object_deleted("OldObject".to_string());
        
        let moo_var = model.to_moo_var();
        
        // Verify it's a map
        assert!(matches!(moo_var.variant(), moor_var::Variant::Map(_)));
    }

    #[test]
    fn test_obj_id_to_object_name() {
        assert_eq!(obj_id_to_object_name("#4", Some("foobar")), "Foobar");
        assert_eq!(obj_id_to_object_name("#4", Some("#4")), "#4");
        assert_eq!(obj_id_to_object_name("#4", None), "#4");
        assert_eq!(obj_id_to_object_name("TestObject", Some("TestObject")), "TestObject");
    }

    #[test]
    fn test_merge_object_diff_models() {
        let mut model1 = ObjectDiffModel::new();
        model1.add_object_added("Object1".to_string());
        
        let mut model2 = ObjectDiffModel::new();
        model2.add_object_added("Object2".to_string());
        model2.add_object_deleted("Object3".to_string());
        
        model1.merge(model2);
        
        assert!(model1.objects_added.contains("Object1"));
        assert!(model1.objects_added.contains("Object2"));
        assert!(model1.objects_deleted.contains("Object3"));
    }
}
