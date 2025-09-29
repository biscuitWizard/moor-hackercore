use crate::types::{Change, RenamedObject};
use crate::model::object_delta::{ObjectDeltaModel, ObjectChange, obj_id_to_object_name};
use std::collections::HashMap;

/// Compiles a vector of commits into a complete ObjectDeltaModel
/// This function processes multiple changes and merges them into a single
/// comprehensive delta model that can be sent to MOO
pub fn compile_commits_to_delta_model(
    commits: Vec<Change>,
    object_name_mapping: Option<HashMap<String, String>>, // obj_id -> object_name mapping
) -> ObjectDeltaModel {
    let mut delta_model = ObjectDeltaModel::new();
    let name_mapping = object_name_mapping.unwrap_or_default();

    for commit in commits {
        // Process renamed objects
        for renamed in commit.renamed_objects {
            let from_display = obj_id_to_object_name(&renamed.from, name_mapping.get(&renamed.from).map(|s| s.as_str()));
            let to_display = obj_id_to_object_name(&renamed.to, name_mapping.get(&renamed.to).map(|s| s.as_str()));
            delta_model.add_object_renamed(from_display, to_display);
        }

        // Process deleted objects
        for deleted_obj in commit.deleted_objects {
            let display_name = obj_id_to_object_name(&deleted_obj, name_mapping.get(&deleted_obj).map(|s| s.as_str()));
            delta_model.add_object_deleted(display_name);
        }

        // Process added objects
        for added_obj in commit.added_objects {
            let display_name = obj_id_to_object_name(&added_obj, name_mapping.get(&added_obj).map(|s| s.as_str()));
            delta_model.add_object_added(display_name);
        }

        // Process modified objects
        for modified_obj in commit.modified_objects {
            let display_name = obj_id_to_object_name(&modified_obj, name_mapping.get(&modified_obj).map(|s| s.as_str()));
            delta_model.add_object_modified(display_name.clone());
            
            // Create a basic ObjectChange entry for modified objects
            // Note: This is a simplified version - in a real implementation,
            // you'd want to track specific verb/property changes per commit
            let change = ObjectChange::new(display_name);
            // You could add specific verb/property changes here based on commit details
            delta_model.add_object_change(change);
        }
    }

    delta_model
}

/// Compiles multiple ObjectDeltaModels into a single comprehensive model
/// This is useful when you have changes from different sources that need to be combined
pub fn compile_delta_models(models: Vec<ObjectDeltaModel>) -> ObjectDeltaModel {
    let mut result = ObjectDeltaModel::new();
    
    for model in models {
        result.merge(model);
    }
    
    result
}

/// Creates an ObjectDeltaModel from a single Change
/// This is a convenience function for processing individual commits
pub fn change_to_delta_model(
    change: Change,
    object_name_mapping: Option<HashMap<String, String>>,
) -> ObjectDeltaModel {
    compile_commits_to_delta_model(vec![change], object_name_mapping)
}

/// Advanced compilation that can track detailed verb and property changes
/// This version allows you to provide detailed change information beyond
/// just the high-level object modifications
pub fn compile_commits_with_detailed_changes(
    commits: Vec<Change>,
    detailed_changes: HashMap<String, ObjectChange>, // obj_id -> detailed changes
    object_name_mapping: Option<HashMap<String, String>>,
) -> ObjectDeltaModel {
    let mut delta_model = compile_commits_to_delta_model(commits, object_name_mapping.clone());
    let name_mapping = object_name_mapping.unwrap_or_default();

    // Add detailed changes for each object
    for (obj_id, detailed_change) in detailed_changes {
        let display_name = obj_id_to_object_name(&obj_id, name_mapping.get(&obj_id).map(|s| s.as_str()));
        let mut change_with_display_name = detailed_change;
        change_with_display_name.obj_id = display_name;
        delta_model.add_object_change(change_with_display_name);
    }

    delta_model
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ChangeStatus};

    fn create_test_change(id: &str, name: &str) -> Change {
        Change {
            id: id.to_string(),
            name: name.to_string(),
            description: Some("Test change".to_string()),
            author: "test_user".to_string(),
            timestamp: 1234567890,
            status: ChangeStatus::Merged,
            added_objects: vec![],
            modified_objects: vec![],
            deleted_objects: vec![],
            renamed_objects: vec![],
            index_change_id: None,
        }
    }

    #[test]
    fn test_compile_empty_commits() {
        let result = compile_commits_to_delta_model(vec![], None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compile_single_commit() {
        let mut change = create_test_change("change1", "Test Change");
        change.added_objects = vec!["#123".to_string()];
        change.deleted_objects = vec!["#456".to_string()];
        change.modified_objects = vec!["#789".to_string()];
        change.renamed_objects = vec![RenamedObject {
            from: "#111".to_string(),
            to: "#222".to_string(),
        }];

        let result = compile_commits_to_delta_model(vec![change], None);

        assert!(result.objects_added.contains("#123"));
        assert!(result.objects_deleted.contains("#456"));
        assert!(result.objects_modified.contains("#789"));
        assert_eq!(result.objects_renamed.get("#111"), Some(&"#222".to_string()));
    }

    #[test]
    fn test_compile_with_name_mapping() {
        let mut change = create_test_change("change1", "Test Change");
        change.added_objects = vec!["#123".to_string()];

        let mut name_mapping = HashMap::new();
        name_mapping.insert("#123".to_string(), "foobar".to_string());

        let result = compile_commits_to_delta_model(vec![change], Some(name_mapping));

        assert!(result.objects_added.contains("Foobar"));
        assert!(!result.objects_added.contains("#123"));
    }

    #[test]
    fn test_compile_multiple_commits() {
        let mut change1 = create_test_change("change1", "First Change");
        change1.added_objects = vec!["#123".to_string()];

        let mut change2 = create_test_change("change2", "Second Change");
        change2.deleted_objects = vec!["#456".to_string()];

        let result = compile_commits_to_delta_model(vec![change1, change2], None);

        assert!(result.objects_added.contains("#123"));
        assert!(result.objects_deleted.contains("#456"));
    }

    #[test]
    fn test_compile_delta_models() {
        let mut model1 = ObjectDeltaModel::new();
        model1.add_object_added("Object1".to_string());

        let mut model2 = ObjectDeltaModel::new();
        model2.add_object_deleted("Object2".to_string());

        let result = compile_delta_models(vec![model1, model2]);

        assert!(result.objects_added.contains("Object1"));
        assert!(result.objects_deleted.contains("Object2"));
    }

    #[test]
    fn test_change_to_delta_model() {
        let mut change = create_test_change("change1", "Test Change");
        change.modified_objects = vec!["#123".to_string()];

        let result = change_to_delta_model(change, None);

        assert!(result.objects_modified.contains("#123"));
    }

    #[test]
    fn test_compile_with_detailed_changes() {
        let mut change = create_test_change("change1", "Test Change");
        change.modified_objects = vec!["#123".to_string()];

        let mut detailed_change = ObjectChange::new("#123".to_string());
        detailed_change.verbs_added.insert("new_verb".to_string());
        detailed_change.props_modified.insert("existing_prop".to_string());

        let mut detailed_changes = HashMap::new();
        detailed_changes.insert("#123".to_string(), detailed_change);

        let result = compile_commits_with_detailed_changes(
            vec![change],
            detailed_changes,
            None,
        );

        assert!(result.objects_modified.contains("#123"));
        assert_eq!(result.changes.len(), 1);
        assert!(result.changes[0].verbs_added.contains("new_verb"));
        assert!(result.changes[0].props_modified.contains("existing_prop"));
    }
}
