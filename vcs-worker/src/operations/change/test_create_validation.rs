// Test to demonstrate the change create operation validation logic

use crate::types::{Change, ChangeStatus};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_create_validation_logic() {
        // Test case 1: No existing changes - should allow creation
        let existing_changes: Vec<Change> = vec![];
        
        let can_create = existing_changes.first().map_or(true, |change| {
            change.status != ChangeStatus::Local
        });
        
        assert!(can_create, "Should be able to create change when no existing changes");

        // Test case 2: Existing merged change at top - should allow creation
        let existing_changes = vec![
            Change {
                id: "merged_change_1".to_string(),
                name: "Merged Change".to_string(),
                description: Some("Already merged".to_string()),
                author: "user1".to_string(),
                timestamp: 1234567890,
                status: ChangeStatus::Merged,
                added_objects: vec![],
                modified_objects: vec![],
                deleted_objects: vec![],
                renamed_objects: vec![],
                index_change_id: None,
            }
        ];
        
        let can_create = existing_changes.first().map_or(true, |change| {
            change.status != ChangeStatus::Local
        });
        
        assert!(can_create, "Should be able to create change when top change is merged");

        // Test case 3: Existing local change at top - should NOT allow creation
        let existing_changes = vec![
            Change {
                id: "local_change_1".to_string(),
                name: "Current Local Change".to_string(),
                description: Some("Currently working on this".to_string()),
                author: "user1".to_string(),
                timestamp: 1234567890,
                status: ChangeStatus::Local,
                added_objects: vec![],
                modified_objects: vec![],
                deleted_objects: vec![],
                renamed_objects: vec![],
                index_change_id: None,
            }
        ];
        
        let can_create = existing_changes.first().map_or(true, |change| {
            change.status != ChangeStatus::Local
        });
        
        assert!(!can_create, "Should NOT be able to create change when already in a local change");

        // Test case 4: Multiple changes with local at top - should NOT allow creation
        let existing_changes = vec![
            Change {
                id: "local_change_1".to_string(),
                name: "Current Local Change".to_string(),
                description: Some("Currently working on this".to_string()),
                author: "user1".to_string(),
                timestamp: 1234567890,
                status: ChangeStatus::Local,
                added_objects: vec![],
                modified_objects: vec![],
                deleted_objects: vec![],
                renamed_objects: vec![],
                index_change_id: None,
            },
            Change {
                id: "merged_change_1".to_string(),
                name: "Previous Merged Change".to_string(),
                description: Some("Already merged".to_string()),
                author: "user1".to_string(),
                timestamp: 1234567880,
                status: ChangeStatus::Merged,
                added_objects: vec![],
                modified_objects: vec![],
                deleted_objects: vec![],
                renamed_objects: vec![],
                index_change_id: None,
            }
        ];
        
        let can_create = existing_changes.first().map_or(true, |change| {
            change.status != ChangeStatus::Local
        });
        
        assert!(!can_create, "Should NOT be able to create change when local change is at top");

        println!("All change create validation logic tests passed!");
    }
}
