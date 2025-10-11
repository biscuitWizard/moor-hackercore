//! Integration tests for object/get operations
//!
//! These tests verify:
//! 1. Getting non-existent objects fails appropriately
//! 2. Getting deleted objects fails
//! 3. Getting renamed objects by old/new names
//! 4. Getting objects after updates
//! 5. Getting objects with invalid names

use crate::common::*;

#[tokio::test]
async fn test_get_non_existent_object() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Getting a non-existent object should fail");
    
    // Attempt to get non-existent object
    println!("\nAttempting to get non-existent object...");
    let response = client.object_get("non_existent_object")
        .await
        .expect("Request should complete");
    
    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate object not found, got: {}", result_str);
    println!("✅ Get failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot get non-existent object");
}

#[tokio::test]
async fn test_get_deleted_object() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Getting a deleted object in current change should fail");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_get_deleted", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (first_change_id, _) = db.require_top_change();
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved");
    
    // Step 2: Verify we can get the object
    println!("\nStep 2: Verifying we can get the object...");
    let get_response = client.object_get("test_get_deleted")
        .await
        .expect("Failed to get object");
    
    get_response.assert_success("Get object");
    let content = get_response.require_result_str("Get object");
    assert!(content.contains("Test Object"), "Should contain object content");
    println!("✅ Object retrieved successfully");
    
    // Step 3: Delete the object
    println!("\nStep 3: Deleting object...");
    client.object_delete("test_get_deleted")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    println!("✅ Object deleted");
    
    // Step 4: Try to get the deleted object
    println!("\nStep 4: Attempting to get deleted object...");
    let response = client.object_get("test_get_deleted")
        .await
        .expect("Request should complete");
    
    // Should fail since object is deleted
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate object not found, got: {}", result_str);
    println!("✅ Get failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot get deleted object");
}

#[tokio::test]
async fn test_get_renamed_object_by_old_name() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Getting a renamed object by old name should fail");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("old_get_name", "test_object.moo")
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
    client.object_rename("old_get_name", "new_get_name")
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    println!("✅ Object renamed");
    
    // Step 3: Try to get by old name
    println!("\nStep 3: Attempting to get by old name...");
    let response = client.object_get("old_get_name")
        .await
        .expect("Request should complete");
    
    // Should fail since old name no longer exists
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate object not found, got: {}", result_str);
    println!("✅ Get failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot get renamed object by old name");
}

#[tokio::test]
async fn test_get_renamed_object_by_new_name() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Getting a renamed object by new name should work");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("old_name_get2", "test_object.moo")
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
    client.object_rename("old_name_get2", "new_name_get2")
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    println!("✅ Object renamed");
    
    // Step 3: Get by new name
    println!("\nStep 3: Getting by new name...");
    let response = client.object_get("new_name_get2")
        .await
        .expect("Failed to get object");
    
    response.assert_success("Get object");
    let content = response.require_result_str("Get object");
    assert!(content.contains("Test Object"), "Should contain object content");
    println!("✅ Object retrieved successfully with new name");
    
    println!("\n✅ Test passed: Can get renamed object by new name");
}

#[tokio::test]
async fn test_get_object_after_update_in_current_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Getting an object after update in current change should return updated content");
    
    // Step 1: Create an object
    println!("\nStep 1: Creating object...");
    client.object_update_from_file("test_get_updated", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    db.require_top_change();
    println!("✅ Object created");
    
    // Step 2: Get the object (should return initial content)
    println!("\nStep 2: Getting initial object...");
    let response1 = client.object_get("test_get_updated")
        .await
        .expect("Failed to get object");
    
    response1.assert_success("Get object");
    let content1 = response1.require_result_str("Get object");
    assert!(content1.contains("Test Object"), "Should contain initial content");
    assert!(!content1.contains("Detailed Test Object"), "Should not contain updated content yet");
    println!("✅ Initial object retrieved");
    
    // Step 3: Update the object in the same change
    println!("\nStep 3: Updating object...");
    client.object_update_from_file("test_get_updated", "detailed_test_object.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update object");
    println!("✅ Object updated");
    
    // Step 4: Get the object again (should return updated content)
    println!("\nStep 4: Getting updated object...");
    let response2 = client.object_get("test_get_updated")
        .await
        .expect("Failed to get object");
    
    response2.assert_success("Get object");
    let content2 = response2.require_result_str("Get object");
    assert!(content2.contains("Detailed Test Object"), "Should contain updated content");
    println!("✅ Updated object retrieved");
    
    println!("\n✅ Test passed: Get returns updated content");
}

#[tokio::test]
async fn test_get_object_with_empty_name() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Getting an object with empty name should fail");
    
    // Attempt to get object with empty name
    println!("\nAttempting to get object with empty name...");
    let response = client.object_get("")
        .await
        .expect("Request should complete");
    
    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found") || result_str.contains("required"), 
            "Should indicate error, got: {}", result_str);
    println!("✅ Get failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot get object with empty name");
}

#[tokio::test]
async fn test_get_object_from_merged_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Getting an object from a merged change should work");
    
    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client.change_create("merged_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_get_merged", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, _) = db.require_top_change();
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Object approved (merged)");
    
    // Step 2: Verify no active change
    println!("\nStep 2: Verifying no active change...");
    db.assert_no_top_change();
    println!("✅ No active change");
    
    // Step 3: Get the object (should work from merged history)
    println!("\nStep 3: Getting object from merged history...");
    let response = client.object_get("test_get_merged")
        .await
        .expect("Failed to get object");
    
    response.assert_success("Get object");
    let content = response.require_result_str("Get object");
    assert!(content.contains("Test Object"), "Should contain object content");
    println!("✅ Object retrieved from merged history");
    
    println!("\n✅ Test passed: Can get object from merged change");
}

#[tokio::test]
async fn test_get_object_with_no_changes() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Getting an object with no active changes should work from refs");
    
    // Step 1: Create, approve, and close a change
    println!("\nStep 1: Creating, approving, and closing change...");
    client.change_create("ref_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_get_refs", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id, _) = db.require_top_change();
    client.change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    // Verify no active change
    db.assert_no_top_change();
    println!("✅ Change merged, no active change");
    
    // Step 2: Get the object (should work from refs)
    println!("\nStep 2: Getting object from refs...");
    let response = client.object_get("test_get_refs")
        .await
        .expect("Failed to get object");
    
    response.assert_success("Get object");
    let content = response.require_result_str("Get object");
    assert!(content.contains("Test Object"), "Should contain object content");
    println!("✅ Object retrieved from refs");
    
    println!("\n✅ Test passed: Can get object from refs with no active changes");
}

#[tokio::test]
async fn test_get_object_multiple_versions() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Getting an object should return the latest version across multiple updates");
    
    // Step 1: Create and approve version 1
    println!("\nStep 1: Creating version 1...");
    client.change_create("change1", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_versions", "test_object.moo")
        .await
        .expect("Failed to create object");
    
    let (change_id1, _) = db.require_top_change();
    client.change_approve(&change_id1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Version 1 approved");
    
    // Step 2: Get version 1
    println!("\nStep 2: Getting version 1...");
    let response1 = client.object_get("test_versions")
        .await
        .expect("Failed to get object");
    
    response1.assert_success("Get object v1");
    let content1 = response1.require_result_str("Get object v1");
    assert!(content1.contains("Test Object"), "Should contain v1 content");
    
    // Step 3: Create and approve version 2
    println!("\nStep 3: Creating version 2...");
    client.change_create("change2", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_versions", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_id2, _) = db.require_top_change();
    client.change_approve(&change_id2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    println!("✅ Version 2 approved");
    
    // Step 4: Get version 2 (should return latest)
    println!("\nStep 4: Getting latest version...");
    let response2 = client.object_get("test_versions")
        .await
        .expect("Failed to get object");
    
    response2.assert_success("Get object v2");
    let content2 = response2.require_result_str("Get object v2");
    assert!(content2.contains("Detailed Test Object"), "Should contain v2 content");
    assert!(!content2.contains("Test Object #1"), "Should not contain v1 marker");
    
    println!("✅ Latest version retrieved");
    
    println!("\n✅ Test passed: Get returns latest version");
}

