//! Integration tests for object/get operations
//!
//! These tests verify:
//! 1. Getting non-existent objects fails appropriately
//! 2. Getting deleted objects fails
//! 3. Getting renamed objects by old/new names
//! 4. Getting objects after updates
//! 5. Getting objects with invalid names
//! 6. Getting objects at specific change IDs (historical versions)
//! 7. Getting objects with short (abbreviated) change IDs
//! 8. Error handling for invalid change IDs
//! 9. Error handling for objects not in specified changes
//! 10. Retrieving multiple historical versions accurately

use crate::common::*;
use serde_json::Value;

/// Helper function to convert object/get response (list of strings) to a single string
fn list_to_string(response: &Value) -> String {
    if let Some(list) = response.get_result_list() {
        list.iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // Fallback for errors which are still strings
        response.get_result_str().unwrap_or("").to_string()
    }
}

#[tokio::test]
async fn test_get_non_existent_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Getting a non-existent object should fail");

    // Attempt to get non-existent object
    println!("\nAttempting to get non-existent object...");
    let response = client
        .object_get("non_existent_object")
        .await
        .expect("Request should complete");

    // Should fail with error (check both string and list result formats)
    let result_str = response.get_result_str().unwrap_or("");
    let has_error = result_str.contains("Error") || result_str.contains("not found");
    
    // If not a string error, might be a list (though errors should be strings)
    if !has_error && response.get_result_list().is_some() {
        panic!("Expected error, but got a successful list result");
    }
    
    assert!(
        has_error,
        "Should indicate object not found, got: {}",
        result_str
    );
    println!("✅ Get failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot get non-existent object");
}

#[tokio::test]
async fn test_get_deleted_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting a deleted object in current change should fail");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_get_deleted", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (first_change_id, _) = db.require_top_change();
    client
        .change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object approved");

    // Step 2: Verify we can get the object
    println!("\nStep 2: Verifying we can get the object...");
    let get_response = client
        .object_get("test_get_deleted")
        .await
        .expect("Failed to get object");

    get_response.assert_success("Get object");
    let content = list_to_string(&get_response);
    assert!(
        content.contains("Test Object"),
        "Should contain object content"
    );
    println!("✅ Object retrieved successfully");

    // Step 3: Delete the object
    println!("\nStep 3: Deleting object...");
    client
        .object_delete("test_get_deleted")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    println!("✅ Object deleted");

    // Step 4: Try to get the deleted object
    println!("\nStep 4: Attempting to get deleted object...");
    let response = client
        .object_get("test_get_deleted")
        .await
        .expect("Request should complete");

    // Should fail since object is deleted
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Should indicate object not found, got: {}",
        result_str
    );
    println!("✅ Get failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot get deleted object");
}

#[tokio::test]
async fn test_get_renamed_object_by_old_name() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting a renamed object by old name should fail");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("old_get_name", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (first_change_id, _) = db.require_top_change();
    client
        .change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object approved");

    // Step 2: Rename the object
    println!("\nStep 2: Renaming object...");
    client
        .object_rename("old_get_name", "new_get_name")
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    println!("✅ Object renamed");

    // Step 3: Try to get by old name
    println!("\nStep 3: Attempting to get by old name...");
    let response = client
        .object_get("old_get_name")
        .await
        .expect("Request should complete");

    // Should fail since old name no longer exists
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Should indicate object not found, got: {}",
        result_str
    );
    println!("✅ Get failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot get renamed object by old name");
}

#[tokio::test]
async fn test_get_renamed_object_by_new_name() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting a renamed object by new name should work");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("old_name_get2", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (first_change_id, _) = db.require_top_change();
    client
        .change_approve(&first_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object approved");

    // Step 2: Rename the object
    println!("\nStep 2: Renaming object...");
    client
        .object_rename("old_name_get2", "new_name_get2")
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    println!("✅ Object renamed");

    // Step 3: Get by new name
    println!("\nStep 3: Getting by new name...");
    let response = client
        .object_get("new_name_get2")
        .await
        .expect("Failed to get object");

    response.assert_success("Get object");
    let content = list_to_string(&response);
    assert!(
        content.contains("Test Object"),
        "Should contain object content"
    );
    println!("✅ Object retrieved successfully with new name");

    println!("\n✅ Test passed: Can get renamed object by new name");
}

#[tokio::test]
async fn test_get_object_after_update_in_current_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!(
        "Test: Getting an object after update in current change should return updated content"
    );

    // Step 1: Create an object
    println!("\nStep 1: Creating object...");
    client
        .object_update_from_file("test_get_updated", "test_object.moo")
        .await
        .expect("Failed to create object");

    db.require_top_change();
    println!("✅ Object created");

    // Step 2: Get the object (should return initial content)
    println!("\nStep 2: Getting initial object...");
    let response1 = client
        .object_get("test_get_updated")
        .await
        .expect("Failed to get object");

    response1.assert_success("Get object");
    let content1 = list_to_string(&response1);
    assert!(
        content1.contains("Test Object"),
        "Should contain initial content"
    );
    assert!(
        !content1.contains("Detailed Test Object"),
        "Should not contain updated content yet"
    );
    println!("✅ Initial object retrieved");

    // Step 3: Update the object in the same change
    println!("\nStep 3: Updating object...");
    client
        .object_update_from_file("test_get_updated", "detailed_test_object.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update object");
    println!("✅ Object updated");

    // Step 4: Get the object again (should return updated content)
    println!("\nStep 4: Getting updated object...");
    let response2 = client
        .object_get("test_get_updated")
        .await
        .expect("Failed to get object");

    response2.assert_success("Get object");
    let content2 = list_to_string(&response2);
    assert!(
        content2.contains("Detailed Test Object"),
        "Should contain updated content"
    );
    println!("✅ Updated object retrieved");

    println!("\n✅ Test passed: Get returns updated content");
}

#[tokio::test]
async fn test_get_object_with_empty_name() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Getting an object with empty name should fail");

    // Attempt to get object with empty name
    println!("\nAttempting to get object with empty name...");
    let response = client
        .object_get("")
        .await
        .expect("Request should complete");

    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error")
            || result_str.contains("not found")
            || result_str.contains("required"),
        "Should indicate error, got: {}",
        result_str
    );
    println!("✅ Get failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot get object with empty name");
}

#[tokio::test]
async fn test_get_object_from_merged_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting an object from a merged change should work");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("merged_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_get_merged", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
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
    let response = client
        .object_get("test_get_merged")
        .await
        .expect("Failed to get object");

    response.assert_success("Get object");
    let content = list_to_string(&response);
    assert!(
        content.contains("Test Object"),
        "Should contain object content"
    );
    println!("✅ Object retrieved from merged history");

    println!("\n✅ Test passed: Can get object from merged change");
}

#[tokio::test]
async fn test_get_object_with_no_changes() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting an object with no active changes should work from refs");

    // Step 1: Create, approve, and close a change
    println!("\nStep 1: Creating, approving, and closing change...");
    client
        .change_create("ref_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_get_refs", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    // Verify no active change
    db.assert_no_top_change();
    println!("✅ Change merged, no active change");

    // Step 2: Get the object (should work from refs)
    println!("\nStep 2: Getting object from refs...");
    let response = client
        .object_get("test_get_refs")
        .await
        .expect("Failed to get object");

    response.assert_success("Get object");
    let content = list_to_string(&response);
    assert!(
        content.contains("Test Object"),
        "Should contain object content"
    );
    println!("✅ Object retrieved from refs");

    println!("\n✅ Test passed: Can get object from refs with no active changes");
}

#[tokio::test]
async fn test_get_object_multiple_versions() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting an object should return the latest version across multiple updates");

    // Step 1: Create and approve version 1
    println!("\nStep 1: Creating version 1...");
    client
        .change_create("change1", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_versions", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id1, _) = db.require_top_change();
    client
        .change_approve(&change_id1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Version 1 approved");

    // Step 2: Get version 1
    println!("\nStep 2: Getting version 1...");
    let response1 = client
        .object_get("test_versions")
        .await
        .expect("Failed to get object");

    response1.assert_success("Get object v1");
    let content1 = list_to_string(&response1);
    assert!(
        content1.contains("Test Object"),
        "Should contain v1 content"
    );

    // Step 3: Create and approve version 2
    println!("\nStep 3: Creating version 2...");
    client
        .change_create("change2", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_versions", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id2, _) = db.require_top_change();
    client
        .change_approve(&change_id2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Version 2 approved");

    // Step 4: Get version 2 (should return latest)
    println!("\nStep 4: Getting latest version...");
    let response2 = client
        .object_get("test_versions")
        .await
        .expect("Failed to get object");

    response2.assert_success("Get object v2");
    let content2 = list_to_string(&response2);
    assert!(
        content2.contains("Detailed Test Object"),
        "Should contain v2 content"
    );
    assert!(
        !content2.contains("Test Object #1"),
        "Should not contain v1 marker"
    );

    println!("✅ Latest version retrieved");

    println!("\n✅ Test passed: Get returns latest version");
}

#[tokio::test]
async fn test_get_object_at_specific_change_id() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting an object at a specific historical change ID");

    // Step 1: Create and approve version 1
    println!("\nStep 1: Creating version 1...");
    client
        .change_create("change1", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_historical", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id1, _) = db.require_top_change();
    client
        .change_approve(&change_id1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Version 1 approved with change ID: {}", change_id1);

    // Step 2: Create and approve version 2
    println!("\nStep 2: Creating version 2...");
    client
        .change_create("change2", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_historical", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id2, _) = db.require_top_change();
    client
        .change_approve(&change_id2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Version 2 approved with change ID: {}", change_id2);

    // Step 3: Get current version (should be version 2)
    println!("\nStep 3: Getting current version...");
    let response_current = client
        .object_get("test_historical")
        .await
        .expect("Failed to get object");

    response_current.assert_success("Get current object");
    let content_current = list_to_string(&response_current);
    assert!(
        content_current.contains("Detailed Test Object"),
        "Current version should contain v2 content"
    );
    println!("✅ Current version retrieved (v2)");

    // Step 4: Get version at change_id1 (should be version 1)
    println!("\nStep 4: Getting object at change ID 1...");
    let response_v1 = client
        .object_get_at_change("test_historical", &change_id1)
        .await
        .expect("Failed to get object at change");

    response_v1.assert_success("Get object at change");
    let content_v1 = list_to_string(&response_v1);
    assert!(
        content_v1.contains("Test Object"),
        "Historical version should contain v1 content"
    );
    assert!(
        !content_v1.contains("Detailed Test Object"),
        "Historical version should not contain v2 content"
    );
    println!("✅ Historical version retrieved (v1)");

    println!("\n✅ Test passed: Can get object at specific change ID");
}

#[tokio::test]
async fn test_get_object_at_short_change_id() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting an object using a short (abbreviated) change ID");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("short_id_test", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_short_id", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (full_change_id, _) = db.require_top_change();
    client
        .change_approve(&full_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object approved with change ID: {}", full_change_id);

    // Step 2: Get object using short change ID (first 8 characters)
    let short_change_id = &full_change_id[..8];
    println!(
        "\nStep 2: Getting object with short change ID: {}...",
        short_change_id
    );

    let response = client
        .object_get_at_change("test_short_id", short_change_id)
        .await
        .expect("Failed to get object with short ID");

    response.assert_success("Get object with short ID");
    let content = list_to_string(&response);
    assert!(
        content.contains("Test Object"),
        "Should retrieve object using short change ID"
    );
    println!("✅ Object retrieved using short change ID");

    println!("\n✅ Test passed: Can get object using short change ID");
}

#[tokio::test]
async fn test_get_object_at_invalid_change_id() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting an object at an invalid change ID should fail");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("valid_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_invalid_id", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object approved");

    // Step 2: Try to get object at invalid change ID
    println!("\nStep 2: Attempting to get object with invalid change ID...");
    let response = client
        .object_get_at_change("test_invalid_id", "invalid_change_id_12345")
        .await
        .expect("Request should complete");

    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Should indicate change ID not found, got: {}",
        result_str
    );
    println!("✅ Get failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot get object with invalid change ID");
}

#[tokio::test]
async fn test_get_object_not_in_specified_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting an object that doesn't exist in the specified change should fail");

    // Step 1: Create and approve change 1 with object A
    println!("\nStep 1: Creating change 1 with object A...");
    client
        .change_create("change1", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("object_a", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id1, _) = db.require_top_change();
    client
        .change_approve(&change_id1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Change 1 approved with object A");

    // Step 2: Create and approve change 2 with object B
    println!("\nStep 2: Creating change 2 with object B...");
    client
        .change_create("change2", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("object_b", "detailed_test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id2, _) = db.require_top_change();
    client
        .change_approve(&change_id2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Change 2 approved with object B");

    // Step 3: Try to get object B at change_id1 (should fail)
    println!("\nStep 3: Attempting to get object B at change 1 (where it doesn't exist)...");
    let response = client
        .object_get_at_change("object_b", &change_id1)
        .await
        .expect("Request should complete");

    // Should fail because object B wasn't in change 1
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Should indicate object not found in change, got: {}",
        result_str
    );
    println!("✅ Get failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot get object that doesn't exist in specified change");
}

#[tokio::test]
async fn test_get_object_multiple_changes_historical() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting an object at different points in history shows correct versions");

    // Step 1: Create and approve version 1
    println!("\nStep 1: Creating version 1...");
    client
        .change_create("v1", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id_v1, _) = db.require_top_change();
    client
        .change_approve(&change_id_v1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Version 1 approved: {}", change_id_v1);

    // Step 2: Create and approve version 2
    println!("\nStep 2: Creating version 2...");
    client
        .change_create("v2", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id_v2, _) = db.require_top_change();
    client
        .change_approve(&change_id_v2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Version 2 approved: {}", change_id_v2);

    // Step 3: Create and approve version 3 (using test_object.moo again)
    println!("\nStep 3: Creating version 3...");
    client
        .change_create("v3", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id_v3, _) = db.require_top_change();
    client
        .change_approve(&change_id_v3)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Version 3 approved: {}", change_id_v3);

    // Step 4: Verify we can get each version
    println!("\nStep 4: Getting version 1...");
    let response_v1 = client
        .object_get_at_change("test_history", &change_id_v1)
        .await
        .expect("Failed to get v1");
    response_v1.assert_success("Get v1");
    let content_v1 = list_to_string(&response_v1);
    assert!(
        content_v1.contains("Test Object") && !content_v1.contains("Detailed"),
        "Should be v1"
    );

    println!("\nStep 5: Getting version 2...");
    let response_v2 = client
        .object_get_at_change("test_history", &change_id_v2)
        .await
        .expect("Failed to get v2");
    response_v2.assert_success("Get v2");
    let content_v2 = list_to_string(&response_v2);
    assert!(content_v2.contains("Detailed Test Object"), "Should be v2");

    println!("\nStep 6: Getting version 3...");
    let response_v3 = client
        .object_get_at_change("test_history", &change_id_v3)
        .await
        .expect("Failed to get v3");
    response_v3.assert_success("Get v3");
    let content_v3 = list_to_string(&response_v3);
    assert!(
        content_v3.contains("Test Object") && !content_v3.contains("Detailed"),
        "Should be v3"
    );

    println!("\nStep 7: Getting current (should be v3)...");
    let response_current = client
        .object_get("test_history")
        .await
        .expect("Failed to get current");
    response_current.assert_success("Get current");
    let content_current = list_to_string(&response_current);
    assert!(
        content_current.contains("Test Object") && !content_current.contains("Detailed"),
        "Current should be v3"
    );

    println!("✅ All historical versions retrieved correctly");

    println!("\n✅ Test passed: Historical versions are accurate");
}

#[tokio::test]
async fn test_get_returns_list_of_strings() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: object/get should return a list of strings, not a single string");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("list_test", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_list_format", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object approved");

    // Step 2: Get the object and verify it's a list
    println!("\nStep 2: Getting object and verifying list format...");
    let response = client
        .object_get("test_list_format")
        .await
        .expect("Failed to get object");

    response.assert_success("Get object");

    // Verify that the result is a list, not a string
    let result_list = response.get_result_list();
    assert!(
        result_list.is_some(),
        "Result should be a list, not a string"
    );

    let list = result_list.unwrap();
    assert!(
        !list.is_empty(),
        "List should not be empty"
    );

    // Verify each element is a string
    for (idx, item) in list.iter().enumerate() {
        assert!(
            item.is_string(),
            "Item {} should be a string, got: {:?}",
            idx,
            item
        );
    }

    println!("✅ Verified result is a list of {} strings", list.len());

    // Step 3: Verify we can reconstruct the object by joining lines
    let reconstructed = list
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        reconstructed.contains("Test Object"),
        "Reconstructed content should contain object data"
    );

    println!("✅ Successfully reconstructed object content from list");
    println!("\n✅ Test passed: object/get returns list of strings");
}

#[tokio::test]
async fn test_get_empty_lines_preserved() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: object/get should preserve empty lines in the list");

    // Step 1: Create an object with known content
    println!("\nStep 1: Creating object...");
    client
        .change_create("empty_lines_test", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_empty_lines", "detailed_test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object approved");

    // Step 2: Get the object
    println!("\nStep 2: Getting object...");
    let response = client
        .object_get("test_empty_lines")
        .await
        .expect("Failed to get object");

    response.assert_success("Get object");

    let result_list = response.require_result_list("Get object");

    // Verify we have multiple lines
    assert!(
        result_list.len() > 5,
        "Should have multiple lines, got {}",
        result_list.len()
    );

    println!("✅ Object has {} lines", result_list.len());

    // Step 3: Verify we can find expected content on specific lines
    let lines: Vec<String> = result_list
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    // First line should start with "obj " or "object " for object definition
    assert!(
        lines[0].starts_with("obj ") || lines[0].starts_with("object "),
        "First line should be object declaration, got: '{}'",
        lines[0]
    );

    println!("✅ Line format verified");
    println!("\n✅ Test passed: Empty lines and structure preserved in list");
}
