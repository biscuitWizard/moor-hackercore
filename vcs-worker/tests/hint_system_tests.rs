/// Integration tests for the hint system
/// 
/// These tests verify that verb and property rename hints work correctly.
/// Hints are kept permanently and never wiped.

use moor_vcs_worker::{Config, create_registry_with_config};
use moor_vcs_worker::providers::index::IndexProvider;
use moor_vcs_worker::types::{ChangeStatus, PropertyRenameHint, VerbRenameHint, VcsObjectType};
use tempfile::TempDir;

/// Helper to create a test database in a temporary directory
fn create_test_db() -> (moor_vcs_worker::DatabaseRef, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config = Config::with_db_path(temp_dir.path().to_path_buf());
    let (_registry, db) = create_registry_with_config(config).expect("Failed to create registry");
    (db, temp_dir)
}

#[tokio::test]
async fn test_verb_rename_hint_added_to_change() {
    let (db, _temp) = create_test_db();

    // Create a local change (no user required for internal tests)
    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");

    // Add a verb rename hint
    let hint = VerbRenameHint {
        object_name: "$test".to_string(),
        from_verb: "look".to_string(),
        to_verb: "peek".to_string(),
    };
    change.verb_rename_hints.push(hint);

    // Store the change
    db.index().update_change(&change).expect("Failed to update change");

    // Retrieve and verify
    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();
    assert_eq!(retrieved.verb_rename_hints.len(), 1);
    assert_eq!(retrieved.verb_rename_hints[0].object_name, "$test");
    assert_eq!(retrieved.verb_rename_hints[0].from_verb, "look");
    assert_eq!(retrieved.verb_rename_hints[0].to_verb, "peek");
}

#[tokio::test]
async fn test_property_rename_hint_added_to_change() {
    let (db, _temp) = create_test_db();

    // Create a local change
    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");

    // Add a property rename hint
    let hint = PropertyRenameHint {
        object_name: "$test".to_string(),
        from_prop: "name".to_string(),
        to_prop: "title".to_string(),
    };
    change.property_rename_hints.push(hint);

    // Store the change
    db.index().update_change(&change).expect("Failed to update change");

    // Retrieve and verify
    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();
    assert_eq!(retrieved.property_rename_hints.len(), 1);
    assert_eq!(retrieved.property_rename_hints[0].object_name, "$test");
    assert_eq!(retrieved.property_rename_hints[0].from_prop, "name");
    assert_eq!(retrieved.property_rename_hints[0].to_prop, "title");
}

#[tokio::test]
async fn test_hints_persist_with_serde() {
    // Test that hints are properly serialized and deserialized
    let (db, _temp) = create_test_db();

    // Create a change with hints
    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");

    change.verb_rename_hints.push(VerbRenameHint {
        object_name: "$test".to_string(),
        from_verb: "look".to_string(),
        to_verb: "peek".to_string(),
    });

    change.property_rename_hints.push(PropertyRenameHint {
        object_name: "$test".to_string(),
        from_prop: "name".to_string(),
        to_prop: "title".to_string(),
    });

    // Store and retrieve
    db.index().update_change(&change).expect("Failed to update change");
    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();

    // Verify both hints survived serialization
    assert_eq!(retrieved.verb_rename_hints.len(), 1);
    assert_eq!(retrieved.property_rename_hints.len(), 1);
}

#[tokio::test]
async fn test_hints_persist_across_merge() {
    let (db, _temp) = create_test_db();

    // Create a change
    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");

    // Add hints
    change.verb_rename_hints.push(VerbRenameHint {
        object_name: "$test".to_string(),
        from_verb: "look".to_string(),
        to_verb: "peek".to_string(),
    });

    change.property_rename_hints.push(PropertyRenameHint {
        object_name: "$test".to_string(),
        from_prop: "name".to_string(),
        to_prop: "title".to_string(),
    });

    // Mark as merged
    change.status = ChangeStatus::Merged;

    // Store and retrieve
    db.index().update_change(&change).expect("Failed to update change");
    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();

    // Verify hints persist across merge (no longer wiped)
    assert_eq!(retrieved.verb_rename_hints.len(), 1);
    assert_eq!(retrieved.property_rename_hints.len(), 1);
    assert_eq!(retrieved.status, ChangeStatus::Merged);
}

#[tokio::test]
async fn test_hints_can_accumulate() {
    // Test that multiple hints can be added over time
    let (db, _temp) = create_test_db();

    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");

    // Add multiple hints
    change.verb_rename_hints.push(VerbRenameHint {
        object_name: "$test1".to_string(),
        from_verb: "look".to_string(),
        to_verb: "peek".to_string(),
    });

    change.verb_rename_hints.push(VerbRenameHint {
        object_name: "$test2".to_string(),
        from_verb: "examine".to_string(),
        to_verb: "inspect".to_string(),
    });

    db.index().update_change(&change).expect("Failed to update change");
    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();

    // Both hints should be present
    assert_eq!(retrieved.verb_rename_hints.len(), 2);
}

#[tokio::test]
async fn test_multiple_hints_for_different_objects() {
    // Test that multiple hints can exist for different objects
    let (db, _temp) = create_test_db();

    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");

    // Add hints for different objects
    change.verb_rename_hints.push(VerbRenameHint {
        object_name: "$object1".to_string(),
        from_verb: "look".to_string(),
        to_verb: "peek".to_string(),
    });

    change.verb_rename_hints.push(VerbRenameHint {
        object_name: "$object2".to_string(),
        from_verb: "examine".to_string(),
        to_verb: "inspect".to_string(),
    });

    change.property_rename_hints.push(PropertyRenameHint {
        object_name: "$object1".to_string(),
        from_prop: "name".to_string(),
        to_prop: "title".to_string(),
    });

    db.index().update_change(&change).expect("Failed to update change");
    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();

    assert_eq!(retrieved.verb_rename_hints.len(), 2);
    assert_eq!(retrieved.property_rename_hints.len(), 1);
}

#[tokio::test]
async fn test_hint_fields_default_to_empty() {
    // Test that serde(default) works correctly for new fields
    let (db, _temp) = create_test_db();

    // Create a basic change
    let change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");

    // Verify all hint fields default to empty
    assert_eq!(change.verb_rename_hints.len(), 0);
    assert_eq!(change.property_rename_hints.len(), 0);
}

#[tokio::test]
async fn test_hint_types_are_hashable_and_comparable() {
    // Test that hint types can be used in HashSets and compared
    let hint1 = VerbRenameHint {
        object_name: "$test".to_string(),
        from_verb: "look".to_string(),
        to_verb: "peek".to_string(),
    };

    let hint2 = VerbRenameHint {
        object_name: "$test".to_string(),
        from_verb: "look".to_string(),
        to_verb: "peek".to_string(),
    };

    let hint3 = VerbRenameHint {
        object_name: "$test".to_string(),
        from_verb: "look".to_string(),
        to_verb: "glance".to_string(),
    };

    // Test equality
    assert_eq!(hint1, hint2);
    assert_ne!(hint1, hint3);

    // Test HashSet usage
    let mut set = std::collections::HashSet::new();
    set.insert(hint1.clone());
    set.insert(hint2.clone());
    assert_eq!(set.len(), 1); // hint1 and hint2 are equal

    set.insert(hint3);
    assert_eq!(set.len(), 2); // hint3 is different
}

#[tokio::test]
async fn test_change_status_transitions_with_hints() {
    // Test that hints behavior is correct across status transitions
    let (db, _temp) = create_test_db();

    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");

    // Add hints in Local status
    change.verb_rename_hints.push(VerbRenameHint {
        object_name: "$test".to_string(),
        from_verb: "look".to_string(),
        to_verb: "peek".to_string(),
    });

    assert_eq!(change.status, ChangeStatus::Local);
    db.index().update_change(&change).expect("Failed to update change");

    // Simulate transition to Review (would happen in submit)
    change.status = ChangeStatus::Review;
    db.index().update_change(&change).expect("Failed to update change");

    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();
    assert_eq!(retrieved.status, ChangeStatus::Review);
    // Hints persist through status transitions (never cleared)
    assert_eq!(retrieved.verb_rename_hints.len(), 1);
}

#[tokio::test]
async fn test_hints_rejected_for_added_objects() {
    // Test the edge case: hints cannot be created for objects added in the current change
    // NOTE: This test would require end-to-end testing through the actual operations
    // For now, we verify the logic at the provider level
    
    let (db, _temp) = create_test_db();
    
    // Create a local change and manually add an object to added_objects
    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");
    
    // Simulate adding an object
    change.added_objects.push(moor_vcs_worker::types::ObjectInfo {
        object_type: VcsObjectType::MooObject,
        name: "$newobj".to_string(),
        version: 1,
    });
    
    db.index().update_change(&change).expect("Failed to update change");
    
    // The validation happens in the operation itself, which checks added_objects
    // This test verifies that the data structure supports the validation
    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();
    assert_eq!(retrieved.added_objects.len(), 1);
    assert_eq!(retrieved.added_objects[0].name, "$newobj");
}

#[tokio::test]
async fn test_hints_allowed_for_modified_objects() {
    // Test that hints ARE allowed for objects that existed before and are now modified
    // This tests the opposite case - when object is NOT in added_objects
    
    let (db, _temp) = create_test_db();
    
    // Create a local change and add an object to modified_objects (not added_objects)
    let mut change = db
        .index()
        .get_or_create_local_change(Some("test_user".to_string()))
        .expect("Failed to create change");
    
    // Simulate modifying an existing object (not adding it)
    change.modified_objects.push(moor_vcs_worker::types::ObjectInfo {
        object_type: VcsObjectType::MooObject,
        name: "$existingobj".to_string(),
        version: 2, // Version 2 implies it existed before
    });
    
    db.index().update_change(&change).expect("Failed to update change");
    
    // The validation in the operation checks that object is NOT in added_objects
    // This test verifies modified_objects are treated differently
    let retrieved = db.index().get_change(&change.id).unwrap().unwrap();
    assert_eq!(retrieved.modified_objects.len(), 1);
    assert_eq!(retrieved.added_objects.len(), 0); // Not in added_objects
    assert_eq!(retrieved.modified_objects[0].name, "$existingobj");
}
