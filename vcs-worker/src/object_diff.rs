use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use moor_var::{Var, v_map, v_str};
use crate::types::Change;

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
