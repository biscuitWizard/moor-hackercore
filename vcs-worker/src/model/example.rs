//! Example usage of the ObjectDeltaModel for communicating changes to MOO

use crate::model::{ObjectDeltaModel, ObjectChange, compile_commits_to_delta_model};
use crate::types::{Change, ChangeStatus, RenamedObject};
use std::collections::HashMap;

/// Example function showing how to create and use ObjectDeltaModel
pub fn example_usage() -> moor_var::Var {
    // Create a new ObjectDeltaModel
    let mut delta_model = ObjectDeltaModel::new();
    
    // Add some object changes
    delta_model.add_object_added("NewSword".to_string());
    delta_model.add_object_deleted("OldShield".to_string());
    delta_model.add_object_renamed("TempObject".to_string(), "PermanentObject".to_string());
    
    // Create a detailed change for a modified object
    let mut detailed_change = ObjectChange::new("ModifiedRoom".to_string());
    detailed_change.verbs_added.insert("new_command".to_string());
    detailed_change.props_modified.insert("description".to_string());
    detailed_change.props_renamed.insert("old_prop".to_string(), "new_prop".to_string());
    
    delta_model.add_object_change(detailed_change);
    
    // Convert to MOO variable format
    delta_model.to_moo_var()
}

/// Example function showing how to compile commits into a delta model
pub fn example_compile_commits() -> moor_var::Var {
    // Create some example commits
    let mut commit1 = Change {
        id: "commit1".to_string(),
        name: "Add new objects".to_string(),
        description: Some("Added sword and shield objects".to_string()),
        author: "player1".to_string(),
        timestamp: 1234567890,
        status: ChangeStatus::Merged,
        added_objects: vec!["#123".to_string(), "#124".to_string()],
        modified_objects: vec!["#125".to_string()],
        deleted_objects: vec![],
        renamed_objects: vec![],
        index_change_id: None,
    };
    
    let mut commit2 = Change {
        id: "commit2".to_string(),
        name: "Remove old objects".to_string(),
        description: Some("Removed obsolete items".to_string()),
        author: "player2".to_string(),
        timestamp: 1234567891,
        status: ChangeStatus::Merged,
        added_objects: vec![],
        modified_objects: vec!["#126".to_string()],
        deleted_objects: vec!["#127".to_string()],
        renamed_objects: vec![RenamedObject {
            from: "#128".to_string(),
            to: "#129".to_string(),
        }],
        index_change_id: None,
    };
    
    // Create object name mapping (optional)
    let mut name_mapping = HashMap::new();
    name_mapping.insert("#123".to_string(), "sword".to_string());
    name_mapping.insert("#124".to_string(), "shield".to_string());
    name_mapping.insert("#125".to_string(), "room".to_string());
    name_mapping.insert("#126".to_string(), "player".to_string());
    name_mapping.insert("#127".to_string(), "old_item".to_string());
    name_mapping.insert("#128".to_string(), "temp_obj".to_string());
    name_mapping.insert("#129".to_string(), "final_obj".to_string());
    
    // Compile commits into delta model
    let delta_model = compile_commits_to_delta_model(
        vec![commit1, commit2],
        Some(name_mapping),
    );
    
    // Convert to MOO variable format
    delta_model.to_moo_var()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_usage() {
        let moo_var = example_usage();
        // Verify it's a map
        assert!(matches!(moo_var.variant(), moor_var::Variant::Map(_)));
    }

    #[test]
    fn test_example_compile_commits() {
        let moo_var = example_compile_commits();
        // Verify it's a map
        assert!(matches!(moo_var.variant(), moor_var::Variant::Map(_)));
    }
}
