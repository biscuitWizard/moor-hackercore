//! Integration tests for object/rename edge cases
//!
//! These tests verify:
//! 1. Renaming non-existent objects fails
//! 2. Renaming to existing names fails
//! 3. Renaming to same name fails
//! 4. Rename interactions with delete operations
//! 5. Invalid name handling

use crate::common::*;

#[tokio::test]
async fn test_rename_non_existent_object() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Renaming a non-existent object should fail");
    
    // Attempt to rename non-existent object
    println!("\nAttempting to rename non-existent object...");
    let response = client.object_rename("non_existent", "new_name")
        .await
        .expect("Request should complete");
    
    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate object not found, got: {}", result_str);
    println!("✅ Rename failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot rename non-existent object");
}

#[tokio::test]
async fn test_rename_to_existing_name() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Renaming to an existing object name should fail");
    
    // Step 1: Create and approve two objects
    println!("\nStep 1: Creating two objects...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("obj1", "test_object.moo")
        .await
        .expect("Failed to create obj1");
    
    client.object_update_from_file("obj2", "detailed_test_object.moo")
        .await
        .expect("Failed to create obj2");
    
    let (change_id, _) = db.require_top_change();
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Two objects created");
    
    // Step 2: Try to rename obj1 to obj2 (should fail)
    println!("\nStep 2: Attempting to rename obj1 to obj2...");
    let response = client.object_rename("obj1", "obj2")
        .await
        .expect("Request should complete");
    
    // Should fail since obj2 already exists
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("already exists"), 
            "Should indicate name already exists, got: {}", result_str);
    println!("✅ Rename failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot rename to existing name");
}

#[tokio::test]
async fn test_rename_to_same_name() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Renaming to the same name should fail");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("same_name_obj", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, _) = db.require_top_change();
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object created");
    
    // Step 2: Try to rename to same name
    println!("\nStep 2: Attempting to rename to same name...");
    let response = client.object_rename("same_name_obj", "same_name_obj")
        .await
        .expect("Request should complete");
    
    // Should fail
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("same name"), 
            "Should indicate cannot use same name, got: {}", result_str);
    println!("✅ Rename failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot rename to same name");
}

#[tokio::test]
async fn test_rename_deleted_object() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Renaming a deleted object should fail");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("to_delete", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, _) = db.require_top_change();
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Delete the object
    println!("\nStep 2: Deleting object...");
    client.object_delete("to_delete")
        .await
        .expect("Failed to delete")
        .assert_success("Delete");
    println!("✅ Object deleted");
    
    // Step 3: Try to rename deleted object
    println!("\nStep 3: Attempting to rename deleted object...");
    let response = client.object_rename("to_delete", "renamed_deleted")
        .await
        .expect("Request should complete");
    
    // Should fail
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate object not found, got: {}", result_str);
    println!("✅ Rename failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot rename deleted object");
}

#[tokio::test]
async fn test_rename_with_empty_names() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Renaming with empty names should fail");
    
    // Step 1: Create an object
    println!("\nStep 1: Creating object...");
    client.object_update_from_file("valid_obj", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    db.require_top_change();
    println!("✅ Object created");
    
    // Step 2: Try to rename from empty name
    println!("\nStep 2: Attempting to rename from empty name...");
    let response1 = client.object_rename("", "new_name")
        .await
        .expect("Request should complete");
    
    let result_str1 = response1.get_result_str().unwrap_or("");
    assert!(result_str1.contains("Error") || result_str1.contains("required") || result_str1.contains("not found"), 
            "Should indicate error, got: {}", result_str1);
    println!("✅ Rename from empty failed: {}", result_str1);
    
    // Step 3: Try to rename to empty name
    println!("\nStep 3: Attempting to rename to empty name...");
    let response2 = client.object_rename("valid_obj", "")
        .await
        .expect("Request should complete");
    
    let result_str2 = response2.get_result_str().unwrap_or("");
    assert!(result_str2.contains("Error") || result_str2.contains("required") || result_str2.contains("same name"), 
            "Should indicate error, got: {}", result_str2);
    println!("✅ Rename to empty failed: {}", result_str2);
    
    println!("\n✅ Test passed: Cannot rename with empty names");
}

#[tokio::test]
async fn test_rename_back_and_forth_cancellation() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Renaming back and forth should cancel rename operations");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("original_name", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, _) = db.require_top_change();
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Rename to new name
    println!("\nStep 2: Renaming to new name...");
    client.object_rename("original_name", "renamed_name")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");
    
    let (change_id, change_after_rename) = db.require_top_change();
    assert_eq!(change_after_rename.renamed_objects.len(), 1, "Should have 1 rename");
    println!("✅ Renamed to renamed_name");
    
    // Step 3: Rename back to original name
    println!("\nStep 3: Renaming back to original name...");
    client.object_rename("renamed_name", "original_name")
        .await
        .expect("Failed to rename back")
        .assert_success("Rename back");
    
    let change_after_rename_back = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Rename should be cancelled (removed)
    assert_eq!(change_after_rename_back.renamed_objects.len(), 0, "Rename should be cancelled");
    println!("✅ Rename cancelled");
    
    println!("\n✅ Test passed: Rename back and forth cancels operation");
}

#[tokio::test]
async fn test_rename_then_delete() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Renaming then deleting should work correctly");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("rename_then_del", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, _) = db.require_top_change();
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Rename the object
    println!("\nStep 2: Renaming object...");
    client.object_rename("rename_then_del", "renamed_obj")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");
    
    let (change_id, change_after_rename) = db.require_top_change();
    assert_eq!(change_after_rename.renamed_objects.len(), 1, "Should have 1 rename");
    println!("✅ Object renamed");
    
    // Step 3: Delete using new name
    println!("\nStep 3: Deleting using new name...");
    client.object_delete("renamed_obj")
        .await
        .expect("Failed to delete")
        .assert_success("Delete");
    
    let change_after_delete = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Rename should be removed, object should be in deleted with original name
    assert_eq!(change_after_delete.renamed_objects.len(), 0, "Rename should be removed");
    assert_eq!(change_after_delete.deleted_objects.len(), 1, "Should have 1 deleted object");
    assert_eq!(change_after_delete.deleted_objects[0].name, "rename_then_del", "Should use original name");
    
    println!("✅ Delete handled rename correctly");
    
    println!("\n✅ Test passed: Rename then delete works");
}

#[tokio::test]
async fn test_rename_multiple_times() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Renaming multiple times should track correctly");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("name1", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, _) = db.require_top_change();
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved with name1");
    
    // Step 2: Rename to name2
    println!("\nStep 2: Renaming to name2...");
    client.object_rename("name1", "name2")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");
    println!("✅ Renamed to name2");
    
    // Step 3: Rename to name3
    println!("\nStep 3: Renaming to name3...");
    client.object_rename("name2", "name3")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");
    
    let (_change_id, change_after) = db.require_top_change();
    
    // Should still have only 1 rename entry (from name1 to name3)
    assert_eq!(change_after.renamed_objects.len(), 1, "Should have 1 rename");
    assert_eq!(change_after.renamed_objects[0].from.name, "name1", "Should be from original name");
    assert_eq!(change_after.renamed_objects[0].to.name, "name3", "Should be to final name");
    
    println!("✅ Multiple renames tracked as single rename from original to final");
    
    // Step 4: Verify object accessible by name3
    println!("\nStep 4: Verifying object accessible by name3...");
    let get_response = client.object_get("name3")
        .await
        .expect("Failed to get object");
    
    get_response.assert_success("Get object");
    println!("✅ Object accessible by name3");
    
    // Step 5: Verify not accessible by name1 or name2
    println!("\nStep 5: Verifying not accessible by old names...");
    let get_name1 = client.object_get("name1")
        .await
        .expect("Request should complete");
    let result1 = get_name1.get_result_str().unwrap_or("");
    assert!(result1.contains("Error") || result1.contains("not found"), "name1 should not exist");
    
    let get_name2 = client.object_get("name2")
        .await
        .expect("Request should complete");
    let result2 = get_name2.get_result_str().unwrap_or("");
    assert!(result2.contains("Error") || result2.contains("not found"), "name2 should not exist");
    
    println!("✅ Old names not accessible");
    
    println!("\n✅ Test passed: Multiple renames work correctly");
}

#[tokio::test]
async fn test_rename_in_added_objects() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Renaming an object in added_objects should just update the name");
    
    // Step 1: Create an object (not approved)
    println!("\nStep 1: Creating object...");
    client.object_update_from_file("added_obj", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, change_before) = db.require_top_change();
    assert_eq!(change_before.added_objects.len(), 1, "Should have 1 added object");
    assert_eq!(change_before.renamed_objects.len(), 0, "Should have 0 renames");
    println!("✅ Object in added_objects");
    
    // Step 2: Rename the object
    println!("\nStep 2: Renaming object...");
    client.object_rename("added_obj", "renamed_added")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");
    
    let change_after = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Should still have 1 added object with new name, no rename entries
    assert_eq!(change_after.added_objects.len(), 1, "Should still have 1 added object");
    assert_eq!(change_after.added_objects[0].name, "renamed_added", "Should have new name");
    assert_eq!(change_after.renamed_objects.len(), 0, "Should have 0 renames (just name change)");
    
    println!("✅ Name updated in added_objects, no rename entry");
    
    println!("\n✅ Test passed: Rename in added_objects updates name only");
}

