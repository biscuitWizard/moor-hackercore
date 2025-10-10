//! Integration tests for object_rename_op and object_update_op interactions
//!
//! These tests verify complex scenarios involving renames and updates:
//! 1. Rename then rename back should delete the rename operation
//! 2. Update -> commit -> rename -> update with old name should show renamed + added
//! 3. Rename back after scenario 2 should delete the added object with cleanup

use crate::common::*;
use moor_vcs_worker::types::{ChangeStatus, VcsObjectType};

#[tokio::test]
async fn test_rename_and_rename_back_deletes_rename_operation() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Rename an object, then rename it back to original name");
    println!("Expected: Rename operation should be deleted from change\n");
    
    // Step 1: Create an object
    println!("Step 1: Creating object 'test_object'...");
    let object_name = "test_object";
    
    client.object_update_from_file(object_name, "test_object.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Object creation");
    
    println!("✅ Object created: {}", object_name);
    
    db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    
    let (top_change_id, change) = db.require_top_change();
    assert!(change.added_objects.iter().any(|obj| obj.name == object_name), "Object should be in added_objects");
    println!("✅ Object is in added_objects list");
    
    // Step 2: Rename the object
    println!("\nStep 2: Renaming '{}' to 'renamed_object'...", object_name);
    let new_name = "renamed_object";
    
    client.object_rename(object_name, new_name)
        .await
        .expect("Failed to rename object")
        .assert_success("Object rename");
    
    println!("✅ Object renamed to: {}", new_name);
    
    // Verify that NO rename operation was added (since object is in added_objects)
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(change.renamed_objects.len(), 0, "Should have NO rename operation for newly added object");
    println!("✅ No rename operation (name just updated in added_objects)");
    
    // Verify the object is in added_objects with the NEW name
    assert!(change.added_objects.iter().any(|obj| obj.name == new_name), "Object should be in added_objects with new name");
    assert!(!change.added_objects.iter().any(|obj| obj.name == object_name), "Object should NOT be in added_objects with old name");
    println!("✅ Object in added_objects updated to new name: {}", new_name);
    
    // Step 3: Rename the object back to its original name
    println!("\nStep 3: Renaming '{}' back to '{}'...", new_name, object_name);
    
    client.object_rename(new_name, object_name)
        .await
        .expect("Failed to rename object back")
        .assert_success("Rename back");
    
    println!("✅ Object renamed back to: {}", object_name);
    
    // Step 4: Verify the object is back to its original name in added_objects
    println!("\nStep 4: Verifying object restored to original name...");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(change.renamed_objects.len(), 0, "Should still have no rename operations");
    println!("✅ No rename operations (as expected)");
    
    // Verify object is back in added_objects with original name
    assert!(change.added_objects.iter().any(|obj| obj.name == object_name), "Object should be in added_objects with original name");
    assert!(!change.added_objects.iter().any(|obj| obj.name == new_name), "Object should NOT be in added_objects with renamed name");
    println!("✅ Object restored to original name in added_objects: {}", object_name);
    
    println!("\n✅ Test passed: Rename + rename back of added object results in no net change");
}

#[tokio::test]
async fn test_update_commit_rename_update_shows_renamed_and_added() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Update object, commit, rename it, then update with new object using old name");
    println!("Expected: Old object shown as renamed, new object shown as added\n");
    
    // Step 1: Create an object
    println!("Step 1: Creating object 'original_object'...");
    let original_name = "original_object";
    
    client.object_update_from_file(original_name, "test_object.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Object creation");
    
    println!("✅ Object created: {}", original_name);
    
    // Get the change ID before committing
    let change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert!(
        change.added_objects.iter().any(|obj| obj.name == original_name),
        "Object should be in added_objects"
    );
    println!("✅ Object is in added_objects list");
    
    // Step 2: Approve/commit the change to move it to Merged status
    println!("\nStep 2: Committing the change (marking as Merged)...");
    
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");
    
    println!("✅ Change committed (status: Merged) using change/approve API");
    
    // Verify no local change exists now
    let top_change = server.database().index().get_top_change()
        .expect("Failed to get top change");
    assert!(top_change.is_none(), "Should have no local change after commit");
    println!("✅ No local change exists after commit");
    
    // Step 3: Rename the object
    println!("\nStep 3: Renaming '{}' to 'renamed_object'...", original_name);
    let renamed_name = "renamed_object";
    
    client.object_rename(original_name, renamed_name)
        .await
        .expect("Failed to rename object")
        .assert_success("Object rename");
    println!("✅ Object renamed to: {}", renamed_name);
    
    // Verify a new local change was created
    let new_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a new local change");
    
    let new_change = server.database().index().get_change(&new_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(new_change.status, ChangeStatus::Local);
    assert_eq!(new_change.renamed_objects.len(), 1);
    println!("✅ New local change created with rename operation");
    
    // Step 4: Update with a new object using the old object's name
    println!("\nStep 4: Creating new object with original name '{}'...", original_name);
    
    client.object_update_from_file(original_name, "detailed_test_object.moo")
        .await
        .expect("Failed to create new object")
        .assert_success("New object creation");
    println!("✅ New object created with name: {}", original_name);
    
    // Step 5: Verify the internal program state
    println!("\nStep 5: Verifying internal program state...");
    
    let change = server.database().index().get_change(&new_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Verify renamed_objects still contains the rename
    assert_eq!(
        change.renamed_objects.len(),
        1,
        "Should have exactly one rename operation"
    );
    assert_eq!(change.renamed_objects[0].from.name, original_name);
    assert_eq!(change.renamed_objects[0].to.name, renamed_name);
    println!("✅ Old object '{}' is shown as renamed to '{}'", original_name, renamed_name);
    
    // Verify added_objects contains the new object
    assert_eq!(
        change.added_objects.len(),
        1,
        "Should have exactly one added object"
    );
    assert_eq!(change.added_objects[0].name, original_name);
    println!("✅ New object '{}' is shown as added (not modified)", original_name);
    
    // Verify modified_objects is empty
    assert_eq!(
        change.modified_objects.len(),
        0,
        "Should have no modified objects"
    );
    println!("✅ No objects shown as modified");
    
    // Verify refs state
    // Note: refs don't update during rename operations - they only update when changes are committed
    // So renamed_object won't have a ref yet, only original_object (the new one) will
    let original_ref = server.database().refs().get_ref(VcsObjectType::MooObject, original_name, None)
        .expect("Failed to get original ref");
    assert!(original_ref.is_some(), "Original name should have a ref (new object)");
    
    let renamed_ref = server.database().refs().get_ref(VcsObjectType::MooObject, renamed_name, None)
        .expect("Failed to get renamed ref");
    assert!(renamed_ref.is_none(), "Renamed name should NOT have a ref yet (rename not committed)");
    
    println!("✅ New object has ref, renamed object doesn't have ref yet (rename not committed)");
    
    println!("\n✅ Test passed: Update -> commit -> rename -> update shows correct state");
}

#[tokio::test]
async fn test_rename_back_after_rename_and_add_deletes_added_object_with_cleanup() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Update, commit, rename, update with old name, then rename back");
    println!("Expected: Added object should be deleted with SHA256 and ref cleanup\n");
    
    // Step 1-4: Same as previous test - create, commit, rename, create new with old name
    println!("Step 1: Creating object 'original_object'...");
    let original_name = "original_object";
    let original_content = moo_to_lines(&load_moo_file("test_object.moo"));
    let original_content_str = original_content.join("\n");
    let original_sha256 = TestServer::calculate_sha256(&original_content_str);
    
    client.object_update(original_name, original_content)
        .await
        .expect("Failed to create object")
        .assert_success("Create object");
    
    println!("✅ Object created: {} (SHA256: {})", original_name, original_sha256);
    
    // Commit the change
    println!("\nStep 2: Committing the change...");
    let change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");
    
    println!("✅ Change committed using change/approve API");
    
    // Rename the object
    println!("\nStep 3: Renaming '{}' to 'renamed_object'...", original_name);
    let renamed_name = "renamed_object";
    
    client.object_rename(original_name, renamed_name)
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    
    println!("✅ Object renamed to: {}", renamed_name);
    
    // Create new object with old name
    println!("\nStep 4: Creating new object with original name '{}'...", original_name);
    let new_content = moo_to_lines(&load_moo_file("detailed_test_object.moo"));
    let new_content_str = new_content.join("\n");
    let new_sha256 = TestServer::calculate_sha256(&new_content_str);
    
    client.object_update(original_name, new_content)
        .await
        .expect("Failed to create new object")
        .assert_success("Create new object");
    
    println!("✅ New object created: {} (SHA256: {})", original_name, new_sha256);
    
    // Verify the new SHA256 exists
    let new_sha256_exists_before = server.database().objects().get(&new_sha256)
        .expect("Failed to check new SHA256")
        .is_some();
    assert!(new_sha256_exists_before, "New SHA256 should exist before rename back");
    println!("✅ New SHA256 exists in objects provider");
    
    // Get the version number for the new object
    let new_version = server.database().refs().get_current_version(VcsObjectType::MooObject, original_name)
        .expect("Failed to get version")
        .expect("Version should exist");
    println!("✅ New object version: {}", new_version);
    
    // Verify current state
    let change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a local change");
    
    let change = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(change.renamed_objects.len(), 1, "Should have one rename");
    assert_eq!(change.added_objects.len(), 1, "Should have one added object");
    println!("✅ State before rename back: 1 renamed, 1 added");
    
    // Step 5: Rename the new object back to the old object's name
    println!("\nStep 5: Renaming new object '{}' back to '{}'...", original_name, renamed_name);
    
    client.object_rename(original_name, renamed_name)
        .await
        .expect("Failed to rename back")
        .assert_success("Rename back");
    println!("✅ Rename back executed");
    
    // Step 6: Verify the "added" object is deleted
    println!("\nStep 6: Verifying added object was deleted...");
    
    let change = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(
        change.added_objects.len(),
        0,
        "Added object should be deleted (no net add)"
    );
    println!("✅ Added object deleted from change");
    
    // Verify the rename operation is also deleted (no net change)
    assert_eq!(
        change.renamed_objects.len(),
        0,
        "Rename operation should be deleted (no net change)"
    );
    println!("✅ Rename operation deleted (no net change)");
    
    // Step 7: Verify SHA256 cleanup
    println!("\nStep 7: Verifying SHA256 cleanup...");
    
    let new_sha256_exists_after = server.database().objects().get(&new_sha256)
        .expect("Failed to check new SHA256")
        .is_some();
    
    assert!(
        !new_sha256_exists_after,
        "New SHA256 should be deleted (orphaned, not in history)"
    );
    println!("✅ New object's SHA256 was cleaned up");
    
    // Step 8: Verify ref cleanup
    println!("\nStep 8: Verifying ref cleanup...");
    
    // The ref for original_name version 2 (the new object) should be deleted
    let ref_with_version = server.database().refs().get_ref(VcsObjectType::MooObject, original_name, Some(new_version))
        .expect("Failed to get ref with version");
    
    assert!(
        ref_with_version.is_none(),
        "Ref for new object version {} should be deleted", new_version
    );
    println!("✅ Ref for new object version {} was cleaned up", new_version);
    
    // The latest ref for original_name should now point to version 1 (the original committed object)
    // NOT version 2 (the deleted one)
    let latest_version_after = server.database().refs().get_current_version(VcsObjectType::MooObject, original_name)
        .expect("Failed to get current version");
    
    assert!(
        latest_version_after.is_some(),
        "Should still have a ref for original name (version 1 from history)"
    );
    assert_eq!(
        latest_version_after.unwrap(),
        1,
        "Latest version should be 1 (the original), not 2 (deleted)"
    );
    println!("✅ Latest ref for '{}' points to version 1 (original committed object)", original_name);
    
    // Step 9: Verify we're back to the original state
    println!("\nStep 9: Verifying we're back to original state...");
    
    // Since the rename was canceled (never committed), renamed_object should NOT exist
    let renamed_ref = server.database().refs().get_ref(VcsObjectType::MooObject, renamed_name, None)
        .expect("Failed to get renamed ref");
    
    assert!(
        renamed_ref.is_none(),
        "Renamed object should NOT exist (rename was canceled)"
    );
    println!("✅ Renamed object doesn't exist (rename was canceled)");
    
    // The change should be empty (no changes)
    let change = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(change.renamed_objects.len(), 0, "Should have no renames");
    assert_eq!(change.added_objects.len(), 0, "Should have no added objects");
    assert_eq!(change.modified_objects.len(), 0, "Should have no modified objects");
    println!("✅ Change is empty (all operations canceled out)");
    
    // Verify original SHA256 still exists (it's in committed history)
    let original_sha256_exists = server.database().objects().get(&original_sha256)
        .expect("Failed to check original SHA256")
        .is_some();
    
    assert!(
        original_sha256_exists,
        "Original SHA256 should still exist (in history)"
    );
    println!("✅ Original SHA256 still exists (in committed history)");
    
    // Verify original_object still exists with version 1 (the original)
    let original_ref = server.database().refs().get_ref(VcsObjectType::MooObject, original_name, Some(1))
        .expect("Failed to get original ref v1");
    assert!(original_ref.is_some(), "Original object version 1 should exist");
    assert_eq!(original_ref.unwrap(), original_sha256, "Should point to original SHA256");
    println!("✅ Original object version 1 still exists with original content");
    
    println!("\n✅ Test passed: Rename back deletes added object with full cleanup, returning to original state");
}

