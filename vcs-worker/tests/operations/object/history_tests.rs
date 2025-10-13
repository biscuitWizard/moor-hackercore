//! Integration tests for object/history operations
//!
//! These tests verify:
//! 1. Getting history for a non-existent object returns empty list
//! 2. Getting history for a newly created object shows the creation
//! 3. Getting history for a modified object shows all modifications
//! 4. Getting history for a renamed object tracks the rename
//! 5. Getting history for a deleted object shows the deletion
//! 6. History entries contain detailed change information
//! 7. History is returned in chronological order

use crate::common::*;

#[tokio::test]
async fn test_history_non_existent_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Getting history for a non-existent object should return empty list");

    // Get history for non-existent object
    println!("\nGetting history for non-existent object...");
    let response = client
        .object_history("non_existent_object")
        .await
        .expect("Request should complete");

    // Should succeed but return empty list
    response.assert_success("Get history");
    let history = response.require_result_list("Get history");
    assert_eq!(
        history.len(),
        0,
        "History should be empty for non-existent object"
    );
    println!("✅ Empty history returned");

    println!("\n✅ Test passed: Non-existent object returns empty history");
}

#[tokio::test]
async fn test_history_newly_created_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting history for a newly created object shows the creation");

    // Step 1: Create an object
    println!("\nStep 1: Creating object...");
    client
        .change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history_new", "object_with_property.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    println!("✅ Object created in change: {}", change_id);

    // Step 2: Approve the change
    println!("\nStep 2: Approving change...");
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Change approved");

    // Step 3: Get history
    println!("\nStep 3: Getting object history...");
    let response = client
        .object_history("test_history_new")
        .await
        .expect("Failed to get history");

    response.assert_success("Get history");
    let history = response.require_result_list("Get history");
    
    // Should have exactly 1 entry for the creation
    assert_eq!(history.len(), 1, "History should have 1 entry");
    
    let entry = history[0].as_object().expect("Entry should be a map");
    
    // Verify basic fields
    assert_eq!(
        entry["change_id"].as_str().unwrap(),
        change_id,
        "Change ID should match"
    );
    assert_eq!(
        entry["change_message"].as_str().unwrap(),
        "initial_change",
        "Change message should match"
    );
    assert_eq!(
        entry["author"].as_str().unwrap(),
        "test_author",
        "Author should match"
    );
    
    // Verify it's marked as added
    assert_eq!(
        entry["object_added"].as_i64().unwrap(),
        1,
        "object_added should be true"
    );
    
    // Verify details exist
    assert!(entry.contains_key("details"), "Should have details");
    let details = entry["details"].as_object().expect("Details should be a map");
    
    // Verify verbs and props were added (test_object.moo only has properties, no verbs)
    let verbs_added = details["verbs_added"].as_array().expect("verbs_added should be an array");
    let props_added = details["props_added"].as_array().expect("props_added should be an array");
    assert!(!props_added.is_empty(), "Should have added properties");
    
    println!("✅ History correctly shows object creation with {} verbs and {} properties",
        verbs_added.len(), props_added.len());

    println!("\n✅ Test passed: Newly created object has correct history");
}

#[tokio::test]
async fn test_history_modified_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting history for a modified object shows all modifications");

    // Step 1: Create initial version
    println!("\nStep 1: Creating initial version...");
    client
        .change_create("change1", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history_mod", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id1, _) = db.require_top_change();
    client
        .change_approve(&change_id1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Initial version created: {}", change_id1);

    // Step 2: Modify the object
    println!("\nStep 2: Modifying object...");
    client
        .change_create("change2", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history_mod", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id2, _) = db.require_top_change();
    client
        .change_approve(&change_id2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Modified version created: {}", change_id2);

    // Step 3: Get history
    println!("\nStep 3: Getting object history...");
    let response = client
        .object_history("test_history_mod")
        .await
        .expect("Failed to get history");

    response.assert_success("Get history");
    let history = response.require_result_list("Get history");
    
    // Should have exactly 2 entries (creation and modification)
    assert_eq!(history.len(), 2, "History should have 2 entries");
    
    // Verify chronological order (oldest first)
    let entry1 = history[0].as_object().expect("Entry 1 should be a map");
    let entry2 = history[1].as_object().expect("Entry 2 should be a map");
    
    assert_eq!(
        entry1["change_id"].as_str().unwrap(),
        change_id1,
        "First entry should be initial change"
    );
    assert_eq!(
        entry1["object_added"].as_i64().unwrap(),
        1,
        "First entry should be marked as added"
    );
    
    assert_eq!(
        entry2["change_id"].as_str().unwrap(),
        change_id2,
        "Second entry should be modification change"
    );
    assert_eq!(
        entry2["object_added"].as_i64().unwrap_or(0),
        0,
        "Second entry should not be marked as added"
    );
    
    // Verify second entry has modification details
    assert!(entry2.contains_key("details"), "Second entry should have details");
    
    println!("✅ History correctly shows 2 entries in chronological order");

    println!("\n✅ Test passed: Modified object has correct history");
}

#[tokio::test]
async fn test_history_renamed_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting history for a renamed object tracks the rename");

    // Step 1: Create initial version
    println!("\nStep 1: Creating initial version...");
    client
        .change_create("change1", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history_old", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id1, _) = db.require_top_change();
    client
        .change_approve(&change_id1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Initial version created");

    // Step 2: Rename the object
    println!("\nStep 2: Renaming object...");
    client
        .object_rename("test_history_old", "test_history_new")
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    
    let (change_id2, _) = db.require_top_change();
    client
        .change_approve(&change_id2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Object renamed");

    // Step 3: Get history using new name
    println!("\nStep 3: Getting history using new name...");
    let response = client
        .object_history("test_history_new")
        .await
        .expect("Failed to get history");

    response.assert_success("Get history");
    let history = response.require_result_list("Get history");
    
    // Should have exactly 2 entries (creation and rename)
    assert_eq!(history.len(), 2, "History should have 2 entries");
    
    // Verify the rename entry
    let rename_entry = history[1].as_object().expect("Rename entry should be a map");
    
    assert!(
        rename_entry.contains_key("renamed_from"),
        "Rename entry should have renamed_from"
    );
    assert!(
        rename_entry.contains_key("renamed_to"),
        "Rename entry should have renamed_to"
    );
    
    assert_eq!(
        rename_entry["renamed_from"].as_str().unwrap(),
        "test_history_old",
        "renamed_from should match old name"
    );
    assert_eq!(
        rename_entry["renamed_to"].as_str().unwrap(),
        "test_history_new",
        "renamed_to should match new name"
    );
    
    println!("✅ History correctly shows rename from 'test_history_old' to 'test_history_new'");

    println!("\n✅ Test passed: Renamed object has correct history");
}

#[tokio::test]
async fn test_history_deleted_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting history for a deleted object shows the deletion");

    // Step 1: Create initial version
    println!("\nStep 1: Creating initial version...");
    client
        .change_create("change1", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history_del", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id1, _) = db.require_top_change();
    client
        .change_approve(&change_id1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Initial version created");

    // Step 2: Delete the object
    println!("\nStep 2: Deleting object...");
    client
        .object_delete("test_history_del")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    
    let (change_id2, _) = db.require_top_change();
    client
        .change_approve(&change_id2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Object deleted");

    // Step 3: Get history
    println!("\nStep 3: Getting history for deleted object...");
    let response = client
        .object_history("test_history_del")
        .await
        .expect("Failed to get history");

    response.assert_success("Get history");
    let history = response.require_result_list("Get history");
    
    // Should have exactly 2 entries (creation and deletion)
    assert_eq!(history.len(), 2, "History should have 2 entries");
    
    // Verify the deletion entry
    let delete_entry = history[1].as_object().expect("Delete entry should be a map");
    
    assert_eq!(
        delete_entry["object_deleted"].as_i64().unwrap(),
        1,
        "object_deleted should be true"
    );
    
    // Deletion entry should not have details (object no longer exists)
    assert!(
        !delete_entry.contains_key("details"),
        "Deletion entry should not have details"
    );
    
    println!("✅ History correctly shows deletion");

    println!("\n✅ Test passed: Deleted object has correct history");
}

#[tokio::test]
async fn test_history_multiple_changes() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Getting history with multiple changes shows all in chronological order");

    // Step 1: Create initial version
    println!("\nStep 1: Creating initial version...");
    client
        .change_create("change1", "author1", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history_multi", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id1, _) = db.require_top_change();
    client
        .change_approve(&change_id1)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Change 1 approved");

    // Step 2: First modification
    println!("\nStep 2: First modification...");
    client
        .change_create("change2", "author2", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history_multi", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id2, _) = db.require_top_change();
    client
        .change_approve(&change_id2)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Change 2 approved");

    // Step 3: Second modification
    println!("\nStep 3: Second modification...");
    client
        .change_create("change3", "author3", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_history_multi", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id3, _) = db.require_top_change();
    client
        .change_approve(&change_id3)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    println!("✅ Change 3 approved");

    // Step 4: Get history
    println!("\nStep 4: Getting object history...");
    let response = client
        .object_history("test_history_multi")
        .await
        .expect("Failed to get history");

    response.assert_success("Get history");
    let history = response.require_result_list("Get history");
    
    // Should have exactly 3 entries
    assert_eq!(history.len(), 3, "History should have 3 entries");
    
    // Verify chronological order and change IDs
    let entry1 = history[0].as_object().expect("Entry 1 should be a map");
    let entry2 = history[1].as_object().expect("Entry 2 should be a map");
    let entry3 = history[2].as_object().expect("Entry 3 should be a map");
    
    assert_eq!(entry1["change_id"].as_str().unwrap(), change_id1);
    assert_eq!(entry1["change_message"].as_str().unwrap(), "change1");
    assert_eq!(entry1["author"].as_str().unwrap(), "author1");
    
    assert_eq!(entry2["change_id"].as_str().unwrap(), change_id2);
    assert_eq!(entry2["change_message"].as_str().unwrap(), "change2");
    assert_eq!(entry2["author"].as_str().unwrap(), "author2");
    
    assert_eq!(entry3["change_id"].as_str().unwrap(), change_id3);
    assert_eq!(entry3["change_message"].as_str().unwrap(), "change3");
    assert_eq!(entry3["author"].as_str().unwrap(), "author3");
    
    // Verify timestamps are in order (each should be >= previous)
    let ts1 = entry1["timestamp"].as_i64().unwrap();
    let ts2 = entry2["timestamp"].as_i64().unwrap();
    let ts3 = entry3["timestamp"].as_i64().unwrap();
    
    assert!(ts2 >= ts1, "Timestamp 2 should be >= timestamp 1");
    assert!(ts3 >= ts2, "Timestamp 3 should be >= timestamp 2");
    
    println!("✅ History correctly shows 3 entries in chronological order");

    println!("\n✅ Test passed: Multiple changes shown in correct order");
}

#[tokio::test]
async fn test_history_with_empty_object_name() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Getting history with empty object name should fail appropriately");

    // Get history with empty name
    println!("\nAttempting to get history with empty name...");
    let response = client
        .object_history("")
        .await
        .expect("Request should complete");

    // Should return empty list (no object with empty name exists)
    response.assert_success("Get history");
    let history = response.require_result_list("Get history");
    assert_eq!(history.len(), 0, "History should be empty");
    
    println!("✅ Empty history returned for empty name");

    println!("\n✅ Test passed: Empty object name handled correctly");
}

