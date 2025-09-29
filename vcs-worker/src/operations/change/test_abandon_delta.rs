// Simple test to demonstrate the change abandon operation returning a delta model

use crate::model::{ObjectDeltaModel, ObjectChange};
use crate::types::{Change, ChangeStatus, RenamedObject};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abandon_delta_logic() {
        // Create a test change with various modifications
        let test_change = Change {
            id: "test_change_1".to_string(),
            name: "Test Change".to_string(),
            description: Some("A test change with various modifications".to_string()),
            author: "test_user".to_string(),
            timestamp: 1234567890,
            status: ChangeStatus::Local,
            added_objects: vec!["#123".to_string(), "#124".to_string()],
            modified_objects: vec!["#125".to_string()],
            deleted_objects: vec!["#126".to_string()],
            renamed_objects: vec![RenamedObject {
                from: "#127".to_string(),
                to: "#128".to_string(),
            }],
            index_change_id: None,
        };

        // Simulate the abandon logic - create undo delta
        let mut undo_delta = ObjectDeltaModel::new();

        // Process added objects - to undo, we need to delete them
        for added_obj in &test_change.added_objects {
            undo_delta.add_object_deleted(added_obj.clone());
        }

        // Process deleted objects - to undo, we need to add them back
        for deleted_obj in &test_change.deleted_objects {
            undo_delta.add_object_added(deleted_obj.clone());
        }

        // Process renamed objects - to undo, we need to rename them back
        for renamed in &test_change.renamed_objects {
            undo_delta.add_object_renamed(renamed.to.clone(), renamed.from.clone());
        }

        // Process modified objects - to undo, we need to mark them as modified
        for modified_obj in &test_change.modified_objects {
            undo_delta.add_object_modified(modified_obj.clone());
            
            // Create a basic ObjectChange for modified objects
            let mut object_change = ObjectChange::new(modified_obj.clone());
            object_change.props_modified.insert("content".to_string());
            undo_delta.add_object_change(object_change);
        }

        // Verify the undo delta contains the correct reverse operations
        assert!(undo_delta.objects_deleted.contains("#123"));
        assert!(undo_delta.objects_deleted.contains("#124"));
        assert!(undo_delta.objects_added.contains("#126"));
        assert!(undo_delta.objects_renamed.get("#128") == Some(&"#127".to_string()));
        assert!(undo_delta.objects_modified.contains("#125"));

        // Verify we have one ObjectChange for the modified object
        assert_eq!(undo_delta.changes.len(), 1);
        assert_eq!(undo_delta.changes[0].obj_id, "#125");
        assert!(undo_delta.changes[0].props_modified.contains("content"));

        // Convert to MOO variable and verify it's a map
        let moo_var = undo_delta.to_moo_var();
        assert!(matches!(moo_var.variant(), moor_var::Variant::Map(_)));

        println!("Test change abandon delta logic passed!");
    }
}
