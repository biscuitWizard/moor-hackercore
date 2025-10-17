use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::providers::index::IndexProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::refs::RefsProvider;
use crate::types::{Change, VcsObjectType};
use moor_compiler::ObjectDefinition;
use moor_var::{Var, v_map, v_str, v_objid};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    /// Meta: Properties that became ignored in this change
    pub meta_ignored_properties: HashSet<String>,
    /// Meta: Verbs that became ignored in this change
    pub meta_ignored_verbs: HashSet<String>,
    /// Meta: Properties that were unignored in this change
    pub meta_unignored_properties: HashSet<String>,
    /// Meta: Verbs that were unignored in this change
    pub meta_unignored_verbs: HashSet<String>,
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
            meta_ignored_properties: HashSet::new(),
            meta_ignored_verbs: HashSet::new(),
            meta_unignored_properties: HashSet::new(),
            meta_unignored_verbs: HashSet::new(),
        }
    }

    /// Invert this ObjectChange to create the reverse operations needed to undo it
    /// 
    /// This is used when abandoning a change - we need to return the inverse operations
    /// so the MOO database can undo the changes. For example:
    /// - verbs_added becomes verbs_deleted (to undo an addition, we delete)
    /// - verbs_deleted becomes verbs_added (to undo a deletion, we add back)
    /// - verbs_renamed is reversed (old->new becomes new->old)
    /// - verbs_modified stays the same (modifications need to be reverted)
    pub fn invert(&self) -> ObjectChange {
        ObjectChange {
            obj_id: self.obj_id.clone(),
            // Modifications are symmetric - still need to modify to revert
            verbs_modified: self.verbs_modified.clone(),
            // Swap added ↔ deleted to invert the operation
            verbs_added: self.verbs_deleted.clone(),
            verbs_deleted: self.verbs_added.clone(),
            // Reverse rename direction (old->new becomes new->old)
            verbs_renamed: self
                .verbs_renamed
                .iter()
                .map(|(k, v)| (v.clone(), k.clone()))
                .collect(),
            // Properties follow the same pattern
            props_modified: self.props_modified.clone(),
            props_added: self.props_deleted.clone(),
            props_deleted: self.props_added.clone(),
            props_renamed: self
                .props_renamed
                .iter()
                .map(|(k, v)| (v.clone(), k.clone()))
                .collect(),
            // Meta changes invert: ignored ↔ unignored
            meta_ignored_properties: self.meta_unignored_properties.clone(),
            meta_ignored_verbs: self.meta_unignored_verbs.clone(),
            meta_unignored_properties: self.meta_ignored_properties.clone(),
            meta_unignored_verbs: self.meta_ignored_verbs.clone(),
        }
    }

    /// Convert this ObjectChange to a MOO v_map
    pub fn to_moo_var(&self) -> Var {
        let mut pairs = Vec::new();

        // obj_id - use helper to convert to v_obj if it's a numeric ID
        pairs.push((v_str("obj_id"), object_id_to_var(&self.obj_id)));

        // verbs_modified
        let verbs_modified_list: Vec<Var> = self.verbs_modified.iter().map(|v| v_str(v)).collect();
        pairs.push((
            v_str("verbs_modified"),
            moor_var::v_list(&verbs_modified_list),
        ));

        // verbs_added
        let verbs_added_list: Vec<Var> = self.verbs_added.iter().map(|v| v_str(v)).collect();
        pairs.push((v_str("verbs_added"), moor_var::v_list(&verbs_added_list)));

        // verbs_renamed
        let verbs_renamed_map: Vec<(Var, Var)> = self
            .verbs_renamed
            .iter()
            .map(|(k, v)| (v_str(k), v_str(v)))
            .collect();
        pairs.push((v_str("verbs_renamed"), v_map(&verbs_renamed_map)));

        // verbs_deleted
        let verbs_deleted_list: Vec<Var> = self.verbs_deleted.iter().map(|v| v_str(v)).collect();
        pairs.push((
            v_str("verbs_deleted"),
            moor_var::v_list(&verbs_deleted_list),
        ));

        // props_modified
        let props_modified_list: Vec<Var> = self.props_modified.iter().map(|v| v_str(v)).collect();
        pairs.push((
            v_str("props_modified"),
            moor_var::v_list(&props_modified_list),
        ));

        // props_added
        let props_added_list: Vec<Var> = self.props_added.iter().map(|v| v_str(v)).collect();
        pairs.push((v_str("props_added"), moor_var::v_list(&props_added_list)));

        // props_renamed
        let props_renamed_map: Vec<(Var, Var)> = self
            .props_renamed
            .iter()
            .map(|(k, v)| (v_str(k), v_str(v)))
            .collect();
        pairs.push((v_str("props_renamed"), v_map(&props_renamed_map)));

        // props_deleted
        let props_deleted_list: Vec<Var> = self.props_deleted.iter().map(|v| v_str(v)).collect();
        pairs.push((
            v_str("props_deleted"),
            moor_var::v_list(&props_deleted_list),
        ));

        // Build meta map if any meta changes exist
        if !self.meta_ignored_properties.is_empty()
            || !self.meta_ignored_verbs.is_empty()
            || !self.meta_unignored_properties.is_empty()
            || !self.meta_unignored_verbs.is_empty()
        {
            let mut meta_pairs = Vec::new();

            // meta_ignored_properties
            let meta_ignored_props_list: Vec<Var> = self
                .meta_ignored_properties
                .iter()
                .map(|v| v_str(v))
                .collect();
            meta_pairs.push((
                v_str("ignored_properties"),
                moor_var::v_list(&meta_ignored_props_list),
            ));

            // meta_ignored_verbs
            let meta_ignored_verbs_list: Vec<Var> = self
                .meta_ignored_verbs
                .iter()
                .map(|v| v_str(v))
                .collect();
            meta_pairs.push((
                v_str("ignored_verbs"),
                moor_var::v_list(&meta_ignored_verbs_list),
            ));

            // meta_unignored_properties
            let meta_unignored_props_list: Vec<Var> = self
                .meta_unignored_properties
                .iter()
                .map(|v| v_str(v))
                .collect();
            meta_pairs.push((
                v_str("unignored_properties"),
                moor_var::v_list(&meta_unignored_props_list),
            ));

            // meta_unignored_verbs
            let meta_unignored_verbs_list: Vec<Var> = self
                .meta_unignored_verbs
                .iter()
                .map(|v| v_str(v))
                .collect();
            meta_pairs.push((
                v_str("unignored_verbs"),
                moor_var::v_list(&meta_unignored_verbs_list),
            ));

            pairs.push((v_str("meta"), v_map(&meta_pairs)));
        }

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

        // objects_renamed - use helper to convert object IDs to v_obj if they're numeric
        let objects_renamed_map: Vec<(Var, Var)> = self
            .objects_renamed
            .iter()
            .map(|(k, v)| (object_id_to_var(k), object_id_to_var(v)))
            .collect();
        pairs.push((v_str("objects_renamed"), v_map(&objects_renamed_map)));

        // objects_deleted - use helper to convert object IDs to v_obj if they're numeric
        let objects_deleted_list: Vec<Var> =
            self.objects_deleted.iter().map(|v| object_id_to_var(v)).collect();
        pairs.push((
            v_str("objects_deleted"),
            moor_var::v_list(&objects_deleted_list),
        ));

        // objects_added - use helper to convert object IDs to v_obj if they're numeric
        let objects_added_list: Vec<Var> = self.objects_added.iter().map(|v| object_id_to_var(v)).collect();
        pairs.push((
            v_str("objects_added"),
            moor_var::v_list(&objects_added_list),
        ));

        // objects_modified - use helper to convert object IDs to v_obj if they're numeric
        let objects_modified_list: Vec<Var> =
            self.objects_modified.iter().map(|v| object_id_to_var(v)).collect();
        pairs.push((
            v_str("objects_modified"),
            moor_var::v_list(&objects_modified_list),
        ));

        // changes
        let changes_list: Vec<Var> = self
            .changes
            .iter()
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

/// Helper function to convert an object ID string to a MOO Var
/// If the string is in the format "#<number>", returns a v_obj (object reference)
/// Otherwise, returns a v_str (string)
pub fn object_id_to_var(obj_id: &str) -> Var {
    // Check if the string starts with '#' and the rest is a valid number
    if let Some(stripped) = obj_id.strip_prefix('#') {
        if let Ok(num) = stripped.parse::<i32>() {
            // This is a numeric object ID like "#73", return as v_obj
            return v_objid(num);
        }
    }
    // Otherwise, return as a string
    v_str(obj_id)
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
pub fn compare_object_versions(
    database: &DatabaseRef,
    obj_name: &str,
    local_version: u64,
    verb_hints: Option<&[crate::types::VerbRenameHint]>,
    prop_hints: Option<&[crate::types::PropertyRenameHint]>,
) -> Result<ObjectChange, ObjectsTreeError> {
    let mut object_change = ObjectChange::new(obj_name.to_string());

    // Get the local version content
    let local_sha256 = database
        .refs()
        .get_ref(VcsObjectType::MooObject, obj_name, Some(local_version))
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        .ok_or_else(|| {
            ObjectsTreeError::SerializationError(format!(
                "Local version {local_version} of object '{obj_name}' not found"
            ))
        })?;

    let local_content = database
        .objects()
        .get(&local_sha256)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
        .ok_or_else(|| {
            ObjectsTreeError::SerializationError(format!(
                "Object content for SHA256 '{local_sha256}' not found"
            ))
        })?;

    // Parse local object definition
    let local_def = database
        .objects()
        .parse_object_dump(&local_content)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

    // Get the baseline version (previous version)
    // For version 1 (new object), there is no baseline (version 0 doesn't exist)
    // For version 2+, the baseline is the previous version
    let baseline_version = local_version.saturating_sub(1);
    let baseline_sha256 = database
        .refs()
        .get_ref(VcsObjectType::MooObject, obj_name, Some(baseline_version))
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

    if let Some(baseline_sha256) = baseline_sha256 {
        // Get baseline content and parse it
        let baseline_content = database
            .objects()
            .get(&baseline_sha256)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Baseline object content for SHA256 '{baseline_sha256}' not found"
                ))
            })?;

        let baseline_def = database
            .objects()
            .parse_object_dump(&baseline_content)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        // Compare the two object definitions with meta filtering and hints
        compare_object_definitions_with_meta(
            &baseline_def,
            &local_def,
            &mut object_change,
            Some(database),
            Some(obj_name),
            verb_hints,
            prop_hints,
        );
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
            object_change
                .props_added
                .insert(prop_override.name.as_string());
        }
    }

    // Add meta changes to the object change
    compare_meta_versions(database, obj_name, &mut object_change)?;

    Ok(object_change)
}

/// Compare meta object versions to determine what meta changes occurred
/// Updates the object_change with meta_ignored_* and meta_unignored_* fields
pub fn compare_meta_versions(
    database: &DatabaseRef,
    obj_name: &str,
    object_change: &mut ObjectChange,
) -> Result<(), ObjectsTreeError> {
    // Get current meta version
    let current_meta = if let Some(meta_sha256) = database
        .refs()
        .get_ref(VcsObjectType::MooMetaObject, obj_name, None)
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
    {
        let yaml = database
            .objects()
            .get(&meta_sha256)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Meta SHA256 '{meta_sha256}' not found"
                ))
            })?;
        Some(
            database
                .objects()
                .parse_meta_dump(&yaml)
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?,
        )
    } else {
        None
    };

    // Get baseline meta version by finding the previous meta version
    // We need to find what the meta looked like before the local change
    let baseline_meta = {
        // Get the current version of the meta
        let current_meta_version = database
            .refs()
            .get_current_version(VcsObjectType::MooMetaObject, obj_name)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        if let Some(current_version) = current_meta_version {
            // Get the previous version
            let baseline_version = current_version.saturating_sub(1);
            if baseline_version > 0 {
                if let Some(baseline_sha256) = database
                    .refs()
                    .get_ref(
                        VcsObjectType::MooMetaObject,
                        obj_name,
                        Some(baseline_version),
                    )
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                {
                    let yaml = database
                        .objects()
                        .get(&baseline_sha256)
                        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                        .ok_or_else(|| {
                            ObjectsTreeError::SerializationError(format!(
                                "Baseline meta SHA256 '{baseline_sha256}' not found"
                            ))
                        })?;
                    Some(
                        database
                            .objects()
                            .parse_meta_dump(&yaml)
                            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?,
                    )
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    // Compare the two meta objects
    let current_meta = current_meta.unwrap_or_default();
    let baseline_meta = baseline_meta.unwrap_or_default();

    // Find properties that became ignored
    for prop in &current_meta.ignored_properties {
        if !baseline_meta.ignored_properties.contains(prop) {
            object_change.meta_ignored_properties.insert(prop.clone());
        }
    }

    // Find properties that became unignored
    for prop in &baseline_meta.ignored_properties {
        if !current_meta.ignored_properties.contains(prop) {
            object_change
                .meta_unignored_properties
                .insert(prop.clone());
        }
    }

    // Find verbs that became ignored
    for verb in &current_meta.ignored_verbs {
        if !baseline_meta.ignored_verbs.contains(verb) {
            object_change.meta_ignored_verbs.insert(verb.clone());
        }
    }

    // Find verbs that became unignored
    for verb in &baseline_meta.ignored_verbs {
        if !current_meta.ignored_verbs.contains(verb) {
            object_change.meta_unignored_verbs.insert(verb.clone());
        }
    }

    Ok(())
}

/// Apply hints to convert added/deleted verbs/properties to renames
fn apply_hints_to_object_change(
    object_change: &mut ObjectChange,
    obj_name: &str,
    verb_hints: &[crate::types::VerbRenameHint],
    prop_hints: &[crate::types::PropertyRenameHint],
) {
    // Apply verb rename hints
    for hint in verb_hints {
        if hint.object_name != obj_name {
            continue; // Skip hints for other objects
        }

        // Check if both from_verb and to_verb are in the expected sets
        let from_in_deleted = object_change.verbs_deleted.contains(&hint.from_verb);
        let to_in_added = object_change.verbs_added.contains(&hint.to_verb);

        if from_in_deleted && to_in_added {
            // This is a valid rename hint - apply it
            object_change.verbs_deleted.remove(&hint.from_verb);
            object_change.verbs_added.remove(&hint.to_verb);
            object_change.verbs_renamed.insert(hint.from_verb.clone(), hint.to_verb.clone());
            
            tracing::debug!(
                "Applied verb rename hint for object '{}': '{}' -> '{}'",
                obj_name, hint.from_verb, hint.to_verb
            );
        }
    }

    // Apply property rename hints
    for hint in prop_hints {
        if hint.object_name != obj_name {
            continue; // Skip hints for other objects
        }

        // Check if both from_prop and to_prop are in the expected sets
        let from_in_deleted = object_change.props_deleted.contains(&hint.from_prop);
        let to_in_added = object_change.props_added.contains(&hint.to_prop);

        if from_in_deleted && to_in_added {
            // This is a valid rename hint - apply it
            object_change.props_deleted.remove(&hint.from_prop);
            object_change.props_added.remove(&hint.to_prop);
            object_change.props_renamed.insert(hint.from_prop.clone(), hint.to_prop.clone());
            
            tracing::debug!(
                "Applied property rename hint for object '{}': '{}' -> '{}'",
                obj_name, hint.from_prop, hint.to_prop
            );
        }
    }
}

/// Compare two ObjectDefinitions with optional meta filtering and rename hints
pub fn compare_object_definitions_with_meta(
    baseline: &ObjectDefinition,
    local: &ObjectDefinition,
    object_change: &mut ObjectChange,
    database: Option<&DatabaseRef>,
    obj_name: Option<&str>,
    verb_hints: Option<&[crate::types::VerbRenameHint]>,
    prop_hints: Option<&[crate::types::PropertyRenameHint]>,
) {
    // Load meta if database and obj_name are provided
    let meta = if let (Some(db), Some(name)) = (database, obj_name) {
        match db.refs().get_ref(VcsObjectType::MooMetaObject, name, None) {
            Ok(Some(meta_sha256)) => match db.objects().get(&meta_sha256) {
                Ok(Some(yaml)) => db.objects().parse_meta_dump(&yaml).ok(),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    };
    // Compare verbs
    // Use a map from first verb name to the verb definition to track each verb uniquely
    let baseline_verbs: HashMap<String, &moor_compiler::ObjVerbDef> = baseline
        .verbs
        .iter()
        .filter_map(|v| {
            // Use the first name as the identifier for this verb
            v.names.first().map(|name| (name.as_string(), v))
        })
        .collect();

    let local_verbs: HashMap<String, &moor_compiler::ObjVerbDef> = local
        .verbs
        .iter()
        .filter_map(|v| {
            // Use the first name as the identifier for this verb
            v.names.first().map(|name| (name.as_string(), v))
        })
        .collect();

    // Find added, modified, and deleted verbs
    // We track which baseline verbs have been matched to avoid marking them as deleted
    let mut matched_baseline_verbs: HashSet<String> = HashSet::new();

    for (first_name, local_verb) in &local_verbs {
        // Check if this verb exists in baseline by looking for any matching name
        let baseline_match = baseline_verbs.iter().find(|(_, baseline_verb)| {
            // Check if any name from baseline_verb matches any name from local_verb
            baseline_verb.names.iter().any(|bn| 
                local_verb.names.iter().any(|ln| bn.as_string() == ln.as_string())
            )
        });

        if let Some((baseline_first_name, baseline_verb)) = baseline_match {
            matched_baseline_verbs.insert(baseline_first_name.clone());
            
            // Verb exists in both - check if it's modified
            if verbs_differ(baseline_verb, local_verb) {
                object_change.verbs_modified.insert(first_name.clone());
            }
        } else {
            // Verb is new (no matching names in baseline)
            object_change.verbs_added.insert(first_name.clone());
        }
    }

    // Check for deleted verbs (those in baseline but not matched in local)
    for (baseline_first_name, baseline_verb) in &baseline_verbs {
        if !matched_baseline_verbs.contains(baseline_first_name) {
            // Check if any name from this verb is ignored
            let is_ignored = meta.as_ref().map(|m| {
                baseline_verb.names.iter().any(|name| m.ignored_verbs.contains(&name.as_string()))
            }).unwrap_or(false);

            if !is_ignored {
                // Verb was actually deleted (not just ignored)
                object_change.verbs_deleted.insert(baseline_first_name.clone());
            } else {
                tracing::debug!(
                    "Verb '{}' is missing but ignored in meta, not marking as deleted",
                    baseline_first_name
                );
            }
        }
    }

    // Compare property definitions
    let baseline_props: HashMap<String, &moor_compiler::ObjPropDef> = baseline
        .property_definitions
        .iter()
        .map(|p| (p.name.as_string(), p))
        .collect();

    let local_props: HashMap<String, &moor_compiler::ObjPropDef> = local
        .property_definitions
        .iter()
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
            // Property is missing - check if it's ignored before marking as deleted
            let is_ignored = meta
                .as_ref()
                .map(|m| m.ignored_properties.contains(prop_name))
                .unwrap_or(false);

            if !is_ignored {
                // Property was actually deleted (not just ignored)
                object_change.props_deleted.insert(prop_name.clone());
            } else {
                tracing::debug!(
                    "Property '{}' is missing but ignored in meta, not marking as deleted",
                    prop_name
                );
            }
        }
    }

    // Compare property overrides
    let baseline_overrides: HashMap<String, &moor_compiler::ObjPropOverride> = baseline
        .property_overrides
        .iter()
        .map(|p| (p.name.as_string(), p))
        .collect();

    let local_overrides: HashMap<String, &moor_compiler::ObjPropOverride> = local
        .property_overrides
        .iter()
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
            // Override is missing - check if it's ignored before marking as deleted
            let is_ignored = meta
                .as_ref()
                .map(|m| m.ignored_properties.contains(prop_name))
                .unwrap_or(false);

            if !is_ignored {
                // Override was actually deleted (not just ignored)
                object_change.props_deleted.insert(prop_name.clone());
            } else {
                tracing::debug!(
                    "Property override '{}' is missing but ignored in meta, not marking as deleted",
                    prop_name
                );
            }
        }
    }

    // Apply hints if provided
    if let Some(name) = obj_name {
        if let (Some(v_hints), Some(p_hints)) = (verb_hints, prop_hints) {
            apply_hints_to_object_change(object_change, name, v_hints, p_hints);
        }
    }
}

/// Check if two verb definitions differ
pub fn verbs_differ(
    baseline: &moor_compiler::ObjVerbDef,
    local: &moor_compiler::ObjVerbDef,
) -> bool {
    baseline.argspec != local.argspec
        || baseline.owner != local.owner
        || baseline.flags != local.flags
        || baseline.program != local.program
}

/// Check if two property definitions differ
pub fn property_definitions_differ(
    baseline: &moor_compiler::ObjPropDef,
    local: &moor_compiler::ObjPropDef,
) -> bool {
    baseline.perms != local.perms || baseline.value != local.value
}

/// Check if two property overrides differ
pub fn property_overrides_differ(
    baseline: &moor_compiler::ObjPropOverride,
    local: &moor_compiler::ObjPropOverride,
) -> bool {
    baseline.value != local.value || baseline.perms_update != local.perms_update
}

/// Build an ObjectDiffModel by comparing a change against the compiled state
/// This is the shared logic used by approve and status operations
pub fn build_object_diff_from_change(
    database: &DatabaseRef,
    change: &Change,
) -> Result<ObjectDiffModel, ObjectsTreeError> {
    let mut diff_model = ObjectDiffModel::new();

    // Get the complete object list from the index state (excluding the local change)
    let complete_object_list = database
        .index()
        .compute_complete_object_list()
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

    tracing::info!(
        "Using complete object list with {} objects as baseline for change '{}'",
        complete_object_list.len(),
        change.name
    );

    // Process the change to build the diff
    process_change_for_diff(database, &mut diff_model, change)?;

    Ok(diff_model)
}

/// Process a single change and add its modifications to the diff model
/// This is the shared logic used by approve and status operations
pub fn process_change_for_diff(
    database: &DatabaseRef,
    diff_model: &mut ObjectDiffModel,
    change: &Change,
) -> Result<(), ObjectsTreeError> {
    // Use hints from the change (they're kept permanently now)
    let verb_hints_ref = Some(change.verb_rename_hints.as_slice());
    let prop_hints_ref = Some(change.property_rename_hints.as_slice());

    // Process added objects (filter to only MooObject types)
    for obj_info in change
        .added_objects
        .iter()
        .filter(|o| o.object_type == VcsObjectType::MooObject)
    {
        let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
        diff_model.add_object_added(obj_name.clone());

        // Get detailed object changes by comparing local vs baseline (which will be empty for new objects)
        let object_change = compare_object_versions(
            database,
            &obj_name,
            obj_info.version,
            verb_hints_ref,
            prop_hints_ref,
        )?;
        diff_model.add_object_change(object_change);
    }

    // Process deleted objects (filter to only MooObject types)
    for obj_info in change
        .deleted_objects
        .iter()
        .filter(|o| o.object_type == VcsObjectType::MooObject)
    {
        let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
        diff_model.add_object_deleted(obj_name);
    }

    // Process renamed objects (filter to only MooObject types)
    for renamed in change.renamed_objects.iter().filter(|r| {
        r.from.object_type == VcsObjectType::MooObject
            && r.to.object_type == VcsObjectType::MooObject
    }) {
        let from_name = obj_id_to_object_name(&renamed.from.name, Some(&renamed.from.name));
        let to_name = obj_id_to_object_name(&renamed.to.name, Some(&renamed.to.name));
        diff_model.add_object_renamed(from_name, to_name);
    }

    // Process modified objects with detailed comparison (filter to only MooObject types)
    for obj_info in change
        .modified_objects
        .iter()
        .filter(|o| o.object_type == VcsObjectType::MooObject)
    {
        let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
        diff_model.add_object_modified(obj_name.clone());

        // Get detailed object changes by comparing local vs baseline
        let object_change = compare_object_versions(
            database,
            &obj_name,
            obj_info.version,
            verb_hints_ref,
            prop_hints_ref,
        )?;
        diff_model.add_object_change(object_change);
    }

    // Process meta-only changes (meta objects that were added/modified without MOO object changes)
    // We need to check for meta objects that have changes but their corresponding MOO object
    // wasn't in the added/modified lists
    let mut processed_objects = std::collections::HashSet::new();
    
    // Collect all MOO object names we've already processed
    for obj_info in change.added_objects.iter()
        .chain(change.modified_objects.iter())
        .filter(|o| o.object_type == VcsObjectType::MooObject)
    {
        processed_objects.insert(obj_info.name.clone());
    }

    // Now look for meta objects whose MOO objects weren't processed
    for obj_info in change
        .added_objects
        .iter()
        .chain(change.modified_objects.iter())
        .filter(|o| o.object_type == VcsObjectType::MooMetaObject)
    {
        // Only process if the corresponding MOO object wasn't already processed
        if !processed_objects.contains(&obj_info.name) {
            let obj_name = obj_id_to_object_name(&obj_info.name, Some(&obj_info.name));
            
            // Create an ObjectChange with just meta tracking
            let mut object_change = ObjectChange::new(obj_name);
            
            // Compare meta versions to populate meta fields
            if let Err(e) = compare_meta_versions(database, &obj_info.name, &mut object_change) {
                tracing::warn!(
                    "Failed to compare meta versions for '{}': {}",
                    obj_info.name,
                    e
                );
            }
            
            // Only add if there are actual meta changes
            if !object_change.meta_ignored_properties.is_empty()
                || !object_change.meta_ignored_verbs.is_empty()
                || !object_change.meta_unignored_properties.is_empty()
                || !object_change.meta_unignored_verbs.is_empty()
            {
                diff_model.add_object_change(object_change);
            }
        }
    }

    Ok(())
}

/// Build an ObjectDiffModel for abandoning a change (undo operations)
/// This creates the reverse operations needed to undo the change
pub fn build_abandon_diff_from_change(
    database: &DatabaseRef,
    change: &Change,
) -> Result<ObjectDiffModel, ObjectsTreeError> {
    // Get the complete object list from the index state for comparison
    let complete_object_list = database
        .index()
        .compute_complete_object_list()
        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

    tracing::info!(
        "Using complete object list with {} objects as baseline for abandoning change '{}'",
        complete_object_list.len(),
        change.name
    );

    // Create a delta model showing what needs to be undone
    let mut undo_delta = ObjectDiffModel::new();

    // Get object name mappings for better display names
    let object_names = get_object_names_for_change(change);

    // Use hints from the change for proper rename tracking
    let verb_hints_ref = Some(change.verb_rename_hints.as_slice());
    let prop_hints_ref = Some(change.property_rename_hints.as_slice());

    // Process added objects - to undo, we need to delete them (filter to only MooObject types)
    for added_obj in change
        .added_objects
        .iter()
        .filter(|o| o.object_type == VcsObjectType::MooObject)
    {
        let object_name = obj_id_to_object_name(
            &added_obj.name,
            object_names.get(&added_obj.name).map(|s| s.as_str()),
        );
        undo_delta.add_object_deleted(object_name.clone());

        // Get the detailed changes for this added object, then invert them
        // This gives us the verb/property details for the deletion
        let object_change = compare_object_versions(
            database,
            &object_name,
            added_obj.version,
            verb_hints_ref,
            prop_hints_ref,
        )?;
        
        // Invert the change to show what needs to be deleted/undone
        let inverted_change = object_change.invert();
        undo_delta.add_object_change(inverted_change);
    }

    // Process deleted objects - to undo, we need to add them back (filter to only MooObject types)
    for deleted_obj in change
        .deleted_objects
        .iter()
        .filter(|o| o.object_type == VcsObjectType::MooObject)
    {
        let object_name = obj_id_to_object_name(
            &deleted_obj.name,
            object_names.get(&deleted_obj.name).map(|s| s.as_str()),
        );
        undo_delta.add_object_added(object_name.clone());

        // For deleted objects, we need to get the baseline version (the version before deletion)
        // and show what needs to be added back
        // The deleted_obj.version represents the last version before deletion
        let baseline_version = deleted_obj.version;
        
        // Get the baseline object to see what needs to be re-added
        if let Ok(Some(baseline_sha256)) = database.refs().get_ref(
            VcsObjectType::MooObject,
            &deleted_obj.name,
            Some(baseline_version),
        ) {
            if let Ok(Some(baseline_content)) = database.objects().get(&baseline_sha256) {
                if let Ok(baseline_def) = database.objects().parse_object_dump(&baseline_content) {
                    // Create an ObjectChange showing what needs to be added back
                    let mut object_change = ObjectChange::new(object_name);
                    
                    // Mark all verbs as needing to be added back
                    for verb in &baseline_def.verbs {
                        for verb_name in &verb.names {
                            object_change.verbs_added.insert(verb_name.as_string());
                        }
                    }
                    
                    // Mark all properties as needing to be added back
                    for prop_def in &baseline_def.property_definitions {
                        object_change.props_added.insert(prop_def.name.as_string());
                    }
                    for prop_override in &baseline_def.property_overrides {
                        object_change.props_added.insert(prop_override.name.as_string());
                    }
                    
                    undo_delta.add_object_change(object_change);
                }
            }
        }
    }

    // Process renamed objects - to undo, we need to rename them back (filter to only MooObject types)
    for renamed in change.renamed_objects.iter().filter(|r| {
        r.from.object_type == VcsObjectType::MooObject
            && r.to.object_type == VcsObjectType::MooObject
    }) {
        let from_name = obj_id_to_object_name(
            &renamed.from.name,
            object_names.get(&renamed.from.name).map(|s| s.as_str()),
        );
        let to_name = obj_id_to_object_name(
            &renamed.to.name,
            object_names.get(&renamed.to.name).map(|s| s.as_str()),
        );
        // Reverse the rename direction for undo
        undo_delta.add_object_renamed(to_name, from_name);
    }

    // Process modified objects - get detailed changes and invert them (filter to only MooObject types)
    for modified_obj in change
        .modified_objects
        .iter()
        .filter(|o| o.object_type == VcsObjectType::MooObject)
    {
        let object_name = obj_id_to_object_name(
            &modified_obj.name,
            object_names.get(&modified_obj.name).map(|s| s.as_str()),
        );
        undo_delta.add_object_modified(object_name.clone());

        // Get the detailed changes by comparing versions
        let object_change = compare_object_versions(
            database,
            &object_name,
            modified_obj.version,
            verb_hints_ref,
            prop_hints_ref,
        )?;
        
        // INVERT the change to get the undo operations
        // If a verb was added in the change, we need to delete it to undo
        // If a verb was deleted in the change, we need to add it back to undo
        let inverted_change = object_change.invert();
        undo_delta.add_object_change(inverted_change);
    }

    Ok(undo_delta)
}

/// Get object names for the change objects to improve display names
/// This is a simplified implementation - in practice you'd want to
/// query the actual object names from the MOO database
pub fn get_object_names_for_change(change: &Change) -> HashMap<String, String> {
    let mut object_names = HashMap::new();

    // Try to get object names from workspace provider (filter to only MooObject types)
    for obj_info in change
        .added_objects
        .iter()
        .chain(change.modified_objects.iter())
        .chain(change.deleted_objects.iter())
        .filter(|o| o.object_type == VcsObjectType::MooObject)
    {
        // For now, we'll just use the object name as the name
        // In a real implementation, you'd query the actual object names
        object_names.insert(obj_info.name.clone(), obj_info.name.clone());
    }

    for renamed in change.renamed_objects.iter().filter(|r| {
        r.from.object_type == VcsObjectType::MooObject
            && r.to.object_type == VcsObjectType::MooObject
    }) {
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
        assert_eq!(
            obj_id_to_object_name("TestObject", Some("TestObject")),
            "TestObject"
        );
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
