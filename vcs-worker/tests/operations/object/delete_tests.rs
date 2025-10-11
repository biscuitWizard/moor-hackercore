//! Integration tests for object/delete operations
//!
//! These tests verify:
//! 1. Deleting non-existent objects fails appropriately
//! 2. Deleting objects in added_objects removes them correctly
//! 3. Deleting objects in modified_objects adds them to deleted
//! 4. Delete interactions with rename operations
//! 5. Delete with associated meta objects

use crate::common::*;
use moor_vcs_worker::types::{ChangeStatus, VcsObjectType};

#[tokio::test]
async fn test_delete_non_existent_object() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting a non-existent object should fail");
    
    // Step 1: Verify no objects exist
    println!("\nStep 1: Verifying no objects initially...");
    db.assert_no_top_change();
    println!("✅ No objects initially");
    
    // Step 2: Attempt to delete non-existent object
    println!("\nStep 2: Attempting to delete non-existent object...");
    let response = client.object_delete("non_existent_object")
        .await
        .expect("Request should complete");
    
    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate object not found, got: {}", result_str);
    println!("✅ Delete failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot delete non-existent object");
}

#[tokio::test]
async fn test_delete_object_in_added_objects() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting an object in added_objects should remove it from added, not add to deleted");
    
    // Step 1: Create an object (will be in added_objects)
    println!("\nStep 1: Creating object...");
    client.object_update_from_file("test_delete_added", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, change_before) = db.require_top_change();
    assert_eq!(change_before.added_objects.len(), 1, "Should have 1 added object");
    assert_eq!(change_before.deleted_objects.len(), 0, "Should have 0 deleted objects");
    println!("✅ Object in added_objects");
    
    // Step 2: Delete the object
    println!("\nStep 2: Deleting object...");
    client.object_delete("test_delete_added")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    println!("✅ Delete successful");
    
    // Step 3: Verify object removed from added_objects, not in deleted_objects
    println!("\nStep 3: Verifying object removed from added_objects...");
    let change_after = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(change_after.added_objects.len(), 0, "Should have 0 added objects");
    assert_eq!(change_after.deleted_objects.len(), 0, "Should have 0 deleted objects (not added to deleted since it was just added)");
    println!("✅ Object removed from added_objects correctly");
    
    println!("\n✅ Test passed: Delete removes from added_objects");
}

#[tokio::test]
async fn test_delete_object_in_modified_objects() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting an object in modified_objects should remove it from modified and add to deleted");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_delete_modified", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved (merged)");
    
    // Step 2: Modify the object in a new change
    println!("\nStep 2: Modifying object...");
    client.change_create("modify_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_delete_modified", "detailed_test_object.moo")
        .await
        .expect("Failed to modify object");
    
    let (change_id, change_before) = db.require_top_change();
    assert_eq!(change_before.modified_objects.len(), 1, "Should have 1 modified object");
    assert_eq!(change_before.deleted_objects.len(), 0, "Should have 0 deleted objects");
    println!("✅ Object in modified_objects");
    
    // Step 3: Delete the object
    println!("\nStep 3: Deleting object...");
    client.object_delete("test_delete_modified")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    println!("✅ Delete successful");
    
    // Step 4: Verify object removed from modified_objects and added to deleted_objects
    println!("\nStep 4: Verifying object moved to deleted_objects...");
    let change_after = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(change_after.modified_objects.len(), 0, "Should have 0 modified objects");
    assert_eq!(change_after.deleted_objects.len(), 1, "Should have 1 deleted object");
    assert_eq!(change_after.deleted_objects[0].name, "test_delete_modified", "Deleted object name should match");
    println!("✅ Object moved to deleted_objects correctly");
    
    println!("\n✅ Test passed: Delete moves from modified to deleted");
}

#[tokio::test]
async fn test_delete_then_update_fails() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting an object then trying to update it should fail");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_delete_update", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Delete the object
    println!("\nStep 2: Deleting object...");
    client.object_delete("test_delete_update")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    
    let (_, change_after_delete) = db.require_top_change();
    assert_eq!(change_after_delete.deleted_objects.len(), 1, "Should have 1 deleted object");
    println!("✅ Object deleted");
    
    // Step 3: Try to update the deleted object
    println!("\nStep 3: Attempting to update deleted object...");
    client.object_update_from_file("test_delete_update", "detailed_test_object.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update should succeed (resurrects object)");
    
    // The update should succeed and remove from deleted, add to modified
    let (_, change_after_update) = db.require_top_change();
    
    // After update, the object should be in modified_objects, not deleted_objects
    let in_deleted = change_after_update.deleted_objects.iter()
        .any(|obj| obj.name == "test_delete_update");
    let in_modified = change_after_update.modified_objects.iter()
        .any(|obj| obj.name == "test_delete_update");
    
    assert!(!in_deleted, "Object should not be in deleted after update");
    assert!(in_modified, "Object should be in modified after update");
    
    println!("✅ Update after delete resurrects the object");
    
    println!("\n✅ Test passed: Update after delete resurrects object");
}

#[tokio::test]
async fn test_delete_renamed_object_old_name_fails() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting a renamed object using old name should fail");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("old_name", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Rename the object
    println!("\nStep 2: Renaming object...");
    client.object_rename("old_name", "new_name")
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    
    let (_, change_after_rename) = db.require_top_change();
    assert_eq!(change_after_rename.renamed_objects.len(), 1, "Should have 1 renamed object");
    println!("✅ Object renamed");
    
    // Step 3: Try to delete using old name
    println!("\nStep 3: Attempting to delete using old name...");
    let response = client.object_delete("old_name")
        .await
        .expect("Request should complete");
    
    // Should fail since old name no longer exists
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate object not found, got: {}", result_str);
    println!("✅ Delete failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot delete renamed object by old name");
}

#[tokio::test]
async fn test_delete_renamed_object_new_name_succeeds() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting a renamed object using new name should work");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("old_name_del", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Rename the object
    println!("\nStep 2: Renaming object...");
    client.object_rename("old_name_del", "new_name_del")
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    
    let (change_id, change_after_rename) = db.require_top_change();
    assert_eq!(change_after_rename.renamed_objects.len(), 1, "Should have 1 renamed object");
    println!("✅ Object renamed");
    
    // Step 3: Delete using new name
    println!("\nStep 3: Deleting using new name...");
    client.object_delete("new_name_del")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    println!("✅ Delete successful");
    
    // Step 4: Verify rename entry removed and object in deleted
    println!("\nStep 4: Verifying delete handled rename correctly...");
    let change_after_delete = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // The rename should be removed and object should be in deleted
    assert_eq!(change_after_delete.renamed_objects.len(), 0, "Should have 0 renamed objects");
    assert_eq!(change_after_delete.deleted_objects.len(), 1, "Should have 1 deleted object");
    
    // The deleted object should use the original name (old_name_del)
    assert_eq!(change_after_delete.deleted_objects[0].name, "old_name_del", "Deleted object should use original name");
    
    println!("✅ Delete handled rename correctly");
    
    println!("\n✅ Test passed: Delete renamed object by new name works");
}

#[tokio::test]
async fn test_delete_with_no_active_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting with no active change should auto-create a change");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_auto_change", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Verify no active change
    println!("\nStep 2: Verifying no active change...");
    db.assert_no_top_change();
    println!("✅ No active change");
    
    // Step 3: Delete the object (should auto-create change)
    println!("\nStep 3: Deleting object (should auto-create change)...");
    client.object_delete("test_auto_change")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    println!("✅ Delete successful");
    
    // Step 4: Verify change was auto-created
    println!("\nStep 4: Verifying change was auto-created...");
    let (_, change) = db.require_top_change();
    assert_eq!(change.status, ChangeStatus::Local, "Should be Local");
    assert_eq!(change.deleted_objects.len(), 1, "Should have 1 deleted object");
    assert_eq!(change.deleted_objects[0].name, "test_auto_change", "Deleted object name should match");
    println!("✅ Change auto-created with deleted object");
    
    println!("\n✅ Test passed: Delete auto-creates change when needed");
}

#[tokio::test]
async fn test_delete_already_deleted_is_idempotent() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting an already deleted object should be idempotent");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_idempotent_delete", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Delete the object
    println!("\nStep 2: Deleting object...");
    client.object_delete("test_idempotent_delete")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    
    let (change_id, change_after_first_delete) = db.require_top_change();
    assert_eq!(change_after_first_delete.deleted_objects.len(), 1, "Should have 1 deleted object");
    println!("✅ First delete successful");
    
    // Step 3: Delete again (should be idempotent)
    println!("\nStep 3: Deleting again (should be idempotent)...");
    client.object_delete("test_idempotent_delete")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object again");
    
    let change_after_second_delete = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Should still have exactly 1 deleted object (not duplicated)
    assert_eq!(change_after_second_delete.deleted_objects.len(), 1, "Should still have 1 deleted object");
    
    println!("✅ Second delete was idempotent");
    
    println!("\n✅ Test passed: Delete is idempotent");
}

#[tokio::test]
async fn test_delete_object_with_meta() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting an object with associated meta should delete both");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_with_meta", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Add meta (ignored property)
    println!("\nStep 2: Adding meta...");
    client.meta_add_ignored_property("test_with_meta", "test_prop")
        .await
        .expect("Failed to add ignored property")
        .assert_success("Add ignored property");
    
    // Verify meta exists
    let meta_exists = server.database().refs().get_ref(VcsObjectType::MooMetaObject, "test_with_meta", None)
        .expect("Failed to check meta")
        .is_some();
    assert!(meta_exists, "Meta should exist");
    println!("✅ Meta created");
    
    // Step 3: Approve the meta change
    let (meta_change_id, _) = db.require_top_change();
    client.change_approve(&meta_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve meta");
    
    println!("✅ Meta approved");
    
    // Step 4: Delete the object
    println!("\nStep 4: Deleting object (should delete meta too)...");
    client.object_delete("test_with_meta")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    
    let (_, change_after_delete) = db.require_top_change();
    
    // Should have both object and meta in deleted_objects
    let deleted_object_count = change_after_delete.deleted_objects.iter()
        .filter(|obj| obj.object_type == VcsObjectType::MooObject && obj.name == "test_with_meta")
        .count();
    let deleted_meta_count = change_after_delete.deleted_objects.iter()
        .filter(|obj| obj.object_type == VcsObjectType::MooMetaObject && obj.name == "test_with_meta")
        .count();
    
    assert_eq!(deleted_object_count, 1, "Should have 1 deleted object");
    assert_eq!(deleted_meta_count, 1, "Should have 1 deleted meta");
    
    println!("✅ Both object and meta deleted");
    
    println!("\n✅ Test passed: Delete removes object and meta");
}

#[tokio::test]
async fn test_delete_then_rename_fails() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Deleting an object then trying to rename it should fail");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_delete_rename", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Delete the object
    println!("\nStep 2: Deleting object...");
    client.object_delete("test_delete_rename")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    println!("✅ Object deleted");
    
    // Step 3: Try to rename the deleted object
    println!("\nStep 3: Attempting to rename deleted object...");
    let response = client.object_rename("test_delete_rename", "renamed_deleted")
        .await
        .expect("Request should complete");
    
    // Should fail since object is deleted
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate object not found, got: {}", result_str);
    println!("✅ Rename failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot rename deleted object");
}

