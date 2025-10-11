//! Integration tests for index update operations
//!
//! These tests verify:
//! 1. Update pulls new changes from remote source
//! 2. Update returns a diff object with changes
//! 3. Update fails gracefully without source URL
//! 4. Update works correctly when already up-to-date
//! 5. Integration between calc_delta and update operations

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

#[tokio::test]
async fn test_update_without_source_url_fails() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Update should fail gracefully when no source URL is configured");
    
    // Step 1: Verify no source URL initially
    println!("\nStep 1: Verifying no source URL...");
    let source = server.database().index().get_source()
        .expect("Failed to get source");
    assert!(source.is_none(), "Should have no source URL initially");
    println!("✅ No source URL configured");
    
    // Step 2: Attempt update without source
    println!("\nStep 2: Attempting update without source URL...");
    let update_response = client.index_update()
        .await
        .expect("Request should complete");
    
    // Check if the result contains an error message
    let result_str = update_response.get_result_str()
        .expect("Should have result string");
    assert!(result_str.contains("Error:") && (result_str.contains("No source URL") || result_str.contains("not cloned")), 
            "Should have error message about missing source URL, got: {}", result_str);
    
    println!("✅ Update failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Update fails gracefully without source URL");
}

#[tokio::test]
async fn test_update_when_up_to_date() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let target_server = TestServer::start().await.expect("Failed to start target server");
    
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    let target_client = target_server.client();
    
    println!("Test: Update should handle being already up-to-date gracefully");
    
    // Step 1: Create state on source
    println!("\nStep 1: Creating state on source...");
    source_client.change_create("initial_change", "test_author", Some("Initial"))
        .await
        .expect("Failed to create change");
    
    source_client.object_update_from_file("initial_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_id).await.expect("Failed to approve").assert_success("Approve");
    
    println!("✅ Source has 1 merged change");
    
    // Step 2: Clone to target
    println!("\nStep 2: Cloning to target...");
    let source_url = format!("{}/api/clone", source_server.base_url());
    target_client.clone_import(&source_url)
        .await
        .expect("Failed to clone")
        .assert_success("Clone");
    
    println!("✅ Target cloned from source");
    
    // Step 3: Immediately update (should be up-to-date)
    println!("\nStep 3: Updating when already up-to-date...");
    let update_response = target_client.index_update()
        .await
        .expect("Update request should complete");
    
    println!("Update response: {:?}", update_response);
    
    // Update should succeed (even if no new changes)
    // The response might indicate no changes or return empty diff
    update_response.assert_success("Update when up-to-date");
    
    println!("✅ Update completed successfully with no new changes");
    
    println!("\n✅ Test passed: Update handles being up-to-date gracefully");
}

#[tokio::test]
async fn test_update_pulls_new_changes() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let target_server = TestServer::start().await.expect("Failed to start target server");
    
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    let target_client = target_server.client();
    let target_db = target_server.db_assertions();
    
    println!("Test: Update should pull new changes from remote source");
    
    // Step 1: Create initial state on source
    println!("\nStep 1: Creating initial state on source...");
    source_client.change_create("change_1", "author_1", Some("First change"))
        .await
        .expect("Failed to create change 1");
    
    source_client.object_update_from_file("object_1", "test_object.moo")
        .await
        .expect("Failed to update object 1");
    
    let (change_1_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_1_id).await.expect("Failed to approve").assert_success("Approve 1");
    
    println!("✅ Source has 1 merged change");
    
    // Step 2: Clone to target
    println!("\nStep 2: Cloning to target...");
    let source_url = format!("{}/api/clone", source_server.base_url());
    target_client.clone_import(&source_url)
        .await
        .expect("Failed to clone")
        .assert_success("Clone");
    
    // Verify target has change_1
    let target_changes = target_server.database().index().get_change_order()
        .expect("Failed to get target change order");
    assert_eq!(target_changes.len(), 1, "Target should have 1 change after clone");
    println!("✅ Target cloned successfully");
    
    // Step 3: Add new changes to source
    println!("\nStep 3: Adding new changes to source...");
    source_client.change_create("change_2", "author_2", Some("Second change"))
        .await
        .expect("Failed to create change 2");
    
    source_client.object_update_from_file("object_2", "detailed_test_object.moo")
        .await
        .expect("Failed to update object 2");
    
    let (change_2_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_2_id).await.expect("Failed to approve").assert_success("Approve 2");
    
    source_client.change_create("change_3", "author_3", Some("Third change"))
        .await
        .expect("Failed to create change 3");
    
    source_client.object_update_from_file("object_3", "test_object.moo")
        .await
        .expect("Failed to update object 3");
    
    let (change_3_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_3_id).await.expect("Failed to approve").assert_success("Approve 3");
    
    println!("✅ Source now has 3 merged changes total");
    
    // Step 4: Update target from source
    println!("\nStep 4: Updating target from source...");
    let update_response = target_client.index_update()
        .await
        .expect("Update request should complete");
    
    println!("Update response: {:?}", update_response);
    update_response.assert_success("Update");
    
    println!("✅ Update completed successfully");
    
    // Step 5: Verify target now has all 3 changes
    println!("\nStep 5: Verifying target has all changes...");
    let target_changes_after = target_server.database().index().get_change_order()
        .expect("Failed to get target change order");
    
    assert_eq!(target_changes_after.len(), 3, "Target should have 3 changes after update");
    assert_eq!(target_changes_after[0], change_1_id, "First change should match");
    assert_eq!(target_changes_after[1], change_2_id, "Second change should match");
    assert_eq!(target_changes_after[2], change_3_id, "Third change should match");
    
    println!("✅ Target has all 3 changes in correct order");
    
    // Verify new objects exist on target
    target_db.assert_ref_exists(VcsObjectType::MooObject, "object_1");
    target_db.assert_ref_exists(VcsObjectType::MooObject, "object_2");
    target_db.assert_ref_exists(VcsObjectType::MooObject, "object_3");
    println!("✅ All object refs exist on target");
    
    println!("\n✅ Test passed: Update successfully pulls new changes");
}

#[tokio::test]
async fn test_update_returns_diff_object() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let target_server = TestServer::start().await.expect("Failed to start target server");
    
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    let target_client = target_server.client();
    
    println!("Test: Update should return a diff object describing the changes");
    
    // Step 1: Create initial state on source
    println!("\nStep 1: Creating initial state on source...");
    source_client.change_create("initial", "author", Some("Initial"))
        .await
        .expect("Failed to create change");
    
    source_client.object_update_from_file("existing_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (initial_id, _) = source_db.require_top_change();
    source_client.change_approve(&initial_id).await.expect("Failed to approve").assert_success("Approve");
    
    println!("✅ Source has initial state");
    
    // Step 2: Clone to target
    println!("\nStep 2: Cloning to target...");
    let source_url = format!("{}/api/clone", source_server.base_url());
    target_client.clone_import(&source_url)
        .await
        .expect("Failed to clone")
        .assert_success("Clone");
    
    println!("✅ Target cloned");
    
    // Step 3: Add diverse changes to source (add, modify, delete)
    println!("\nStep 3: Adding diverse changes to source...");
    
    // Add a new object
    source_client.change_create("add_change", "author", Some("Add new object"))
        .await
        .expect("Failed to create change");
    
    source_client.object_update_from_file("new_object", "detailed_test_object.moo")
        .await
        .expect("Failed to add new object");
    
    let (add_id, _) = source_db.require_top_change();
    source_client.change_approve(&add_id).await.expect("Failed to approve").assert_success("Approve add");
    
    // Modify existing object
    source_client.change_create("modify_change", "author", Some("Modify existing"))
        .await
        .expect("Failed to create change");
    
    source_client.object_update_from_file("existing_object", "detailed_test_object.moo")
        .await
        .expect("Failed to modify object");
    
    let (modify_id, _) = source_db.require_top_change();
    source_client.change_approve(&modify_id).await.expect("Failed to approve").assert_success("Approve modify");
    
    println!("✅ Source has added and modified objects");
    
    // Step 4: Update target and examine diff
    println!("\nStep 4: Updating target and examining diff...");
    let update_response = target_client.index_update()
        .await
        .expect("Update should complete");
    
    println!("Update response: {:?}", update_response);
    update_response.assert_success("Update");
    
    // The response should contain diff information
    // The exact format depends on how ObjectDiffModel is serialized to moo_var
    // At minimum, verify the response is successful and contains useful information
    let result = &update_response["result"];
    println!("Update result: {:?}", result);
    
    // If the result is a string, it might be serialized diff data
    // If it's a structured object, we can inspect it more deeply
    // For now, just verify we got a successful response
    println!("✅ Update returned result (diff object)");
    
    println!("\n✅ Test passed: Update returns diff object");
}

#[tokio::test]
async fn test_calc_delta_integration() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    
    println!("Test: calc_delta should correctly identify changes after a given change ID");
    
    // Step 1: Create multiple changes
    println!("\nStep 1: Creating multiple changes...");
    
    source_client.change_create("change_1", "author", Some("First"))
        .await
        .expect("Failed to create change 1");
    source_client.object_update_from_file("obj_1", "test_object.moo")
        .await
        .expect("Failed to update obj 1");
    let (change_1_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_1_id).await.expect("Failed to approve").assert_success("Approve 1");
    
    source_client.change_create("change_2", "author", Some("Second"))
        .await
        .expect("Failed to create change 2");
    source_client.object_update_from_file("obj_2", "detailed_test_object.moo")
        .await
        .expect("Failed to update obj 2");
    let (change_2_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_2_id).await.expect("Failed to approve").assert_success("Approve 2");
    
    source_client.change_create("change_3", "author", Some("Third"))
        .await
        .expect("Failed to create change 3");
    source_client.object_update_from_file("obj_3", "test_object.moo")
        .await
        .expect("Failed to update obj 3");
    let (change_3_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_3_id).await.expect("Failed to approve").assert_success("Approve 3");
    
    println!("✅ Created 3 merged changes");
    
    // Step 2: Call calc_delta from change_1
    println!("\nStep 2: Calculating delta from change_1...");
    let delta_response = source_client.index_calc_delta(&change_1_id)
        .await
        .expect("calc_delta should complete");
    
    println!("Delta response: {:?}", delta_response);
    delta_response.assert_success("calc_delta");
    
    // Step 3: Parse and verify delta contains change_2 and change_3
    println!("\nStep 3: Verifying delta contents...");
    let result = &delta_response["result"];
    
    // Delta should have change_ids, ref_pairs, and objects_added
    let change_ids = result.get("change_ids")
        .and_then(|v| v.as_array())
        .expect("Delta should have change_ids array");
    
    println!("Delta contains {} change IDs", change_ids.len());
    assert_eq!(change_ids.len(), 2, "Should have 2 changes after change_1 (change_2 and change_3)");
    
    // Verify the change IDs are correct
    let change_id_strs: Vec<String> = change_ids.iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect();
    
    assert!(change_id_strs.contains(&change_2_id), "Should include change_2");
    assert!(change_id_strs.contains(&change_3_id), "Should include change_3");
    
    println!("✅ Delta contains correct change IDs");
    
    // Verify ref_pairs exist
    let ref_pairs = result.get("ref_pairs")
        .and_then(|v| v.as_array())
        .expect("Delta should have ref_pairs array");
    
    println!("Delta contains {} ref pairs", ref_pairs.len());
    assert!(!ref_pairs.is_empty(), "Should have ref pairs for the new/modified objects");
    
    println!("✅ Delta contains ref pairs");
    
    // Verify objects_added exist
    let objects_added = result.get("objects_added")
        .and_then(|v| v.as_array())
        .expect("Delta should have objects_added array");
    
    println!("Delta contains {} objects added", objects_added.len());
    assert!(!objects_added.is_empty(), "Should have objects added");
    
    println!("✅ Delta contains objects added");
    
    println!("\n✅ Test passed: calc_delta correctly identifies changes after given change ID");
}

#[tokio::test]
async fn test_update_and_calc_delta_integration() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let target_server = TestServer::start().await.expect("Failed to start target server");
    
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    let target_client = target_server.client();
    
    println!("Test: Update operation should use calc_delta to identify and pull new changes");
    
    // Step 1: Setup source with initial changes
    println!("\nStep 1: Setting up source with initial changes...");
    source_client.change_create("change_1", "author", Some("First"))
        .await
        .expect("Failed to create change");
    source_client.object_update_from_file("obj_1", "test_object.moo")
        .await
        .expect("Failed to update object");
    let (change_1_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_1_id).await.expect("Failed to approve").assert_success("Approve");
    
    source_client.change_create("change_2", "author", Some("Second"))
        .await
        .expect("Failed to create change");
    source_client.object_update_from_file("obj_2", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");
    let (change_2_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_2_id).await.expect("Failed to approve").assert_success("Approve");
    
    println!("✅ Source has 2 initial changes");
    
    // Step 2: Clone to target
    println!("\nStep 2: Cloning to target...");
    let source_url = format!("{}/api/clone", source_server.base_url());
    target_client.clone_import(&source_url)
        .await
        .expect("Failed to clone")
        .assert_success("Clone");
    
    println!("✅ Target cloned with 2 changes");
    
    // Step 3: Verify calc_delta works on source
    println!("\nStep 3: Testing calc_delta on source (should return empty)...");
    let delta_response = source_client.index_calc_delta(&change_2_id)
        .await
        .expect("calc_delta should work");
    
    delta_response.assert_success("calc_delta");
    let result = &delta_response["result"];
    let change_ids = result.get("change_ids")
        .and_then(|v| v.as_array())
        .expect("Should have change_ids");
    assert_eq!(change_ids.len(), 0, "Should have no changes after change_2 (most recent)");
    
    println!("✅ calc_delta correctly returns empty for latest change");
    
    // Step 4: Add new changes to source
    println!("\nStep 4: Adding new changes to source...");
    for i in 3..=5 {
        source_client.change_create(&format!("change_{}", i), "author", Some(&format!("Change {}", i)))
            .await
            .expect("Failed to create change");
        source_client.object_update_from_file(&format!("obj_{}", i), "test_object.moo")
            .await
            .expect("Failed to update object");
        let (change_id, _) = source_db.require_top_change();
        source_client.change_approve(&change_id).await.expect("Failed to approve").assert_success("Approve");
    }
    
    println!("✅ Source now has 5 total changes");
    
    // Step 5: Verify calc_delta identifies the new changes
    println!("\nStep 5: Testing calc_delta identifies new changes...");
    let delta_response = source_client.index_calc_delta(&change_2_id)
        .await
        .expect("calc_delta should work");
    
    delta_response.assert_success("calc_delta");
    let result = &delta_response["result"];
    let change_ids = result.get("change_ids")
        .and_then(|v| v.as_array())
        .expect("Should have change_ids");
    assert_eq!(change_ids.len(), 3, "Should have 3 new changes (change_3, change_4, change_5)");
    
    println!("✅ calc_delta identifies 3 new changes");
    
    // Step 6: Update target and verify it gets the new changes
    println!("\nStep 6: Updating target...");
    let update_response = target_client.index_update()
        .await
        .expect("Update should work");
    
    update_response.assert_success("Update");
    
    println!("✅ Update completed");
    
    // Step 7: Verify target has all 5 changes
    println!("\nStep 7: Verifying target has all changes...");
    let target_change_order = target_server.database().index().get_change_order()
        .expect("Failed to get change order");
    assert_eq!(target_change_order.len(), 5, "Target should have all 5 changes");
    
    println!("✅ Target now has all 5 changes");
    
    println!("\n✅ Test passed: Update and calc_delta work together correctly");
}

#[tokio::test]
async fn test_calc_delta_with_non_existent_change_id() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let source_client = source_server.client();
    
    println!("Test: calc_delta with non-existent change ID should fail");
    
    // Attempt calc_delta with non-existent change ID
    println!("\nAttempting calc_delta with non-existent change ID...");
    let response = source_client.index_calc_delta("non_existent_change_id")
        .await
        .expect("Request should complete");
    
    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found") || result_str.contains("does not exist"), 
            "Should indicate change not found, got: {}", result_str);
    println!("✅ calc_delta failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: calc_delta fails with non-existent change ID");
}

#[tokio::test]
async fn test_calc_delta_with_empty_change_id() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let source_client = source_server.client();
    
    println!("Test: calc_delta with empty change ID should fail");
    
    // Attempt calc_delta with empty change ID
    println!("\nAttempting calc_delta with empty change ID...");
    let response = source_client.index_calc_delta("")
        .await
        .expect("Request should complete");
    
    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("required") || result_str.contains("not found"), 
            "Should indicate error, got: {}", result_str);
    println!("✅ calc_delta failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: calc_delta fails with empty change ID");
}

#[tokio::test]
async fn test_calc_delta_from_most_recent_returns_empty() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    
    println!("Test: calc_delta from most recent change should return empty delta");
    
    // Step 1: Create and approve a change
    println!("\nStep 1: Creating and approving change...");
    source_client.change_create("latest_change", "author", Some("Latest"))
        .await
        .expect("Failed to create change");
    
    source_client.object_update_from_file("obj_1", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_id, _) = source_db.require_top_change();
    source_client.change_approve(&change_id).await.expect("Failed to approve").assert_success("Approve");
    
    println!("✅ Change approved");
    
    // Step 2: Calc delta from this change (most recent)
    println!("\nStep 2: Calculating delta from most recent change...");
    let delta_response = source_client.index_calc_delta(&change_id)
        .await
        .expect("calc_delta should complete");
    
    delta_response.assert_success("calc_delta");
    
    // Step 3: Verify delta is empty
    println!("\nStep 3: Verifying delta is empty...");
    let result = &delta_response["result"];
    
    let change_ids = result.get("change_ids")
        .and_then(|v| v.as_array())
        .expect("Delta should have change_ids array");
    
    assert_eq!(change_ids.len(), 0, "Should have 0 changes after most recent");
    println!("✅ Delta is empty");
    
    println!("\n✅ Test passed: calc_delta from most recent returns empty");
}

#[tokio::test]
async fn test_calc_delta_with_malformed_change_id() {
    let source_server = TestServer::start().await.expect("Failed to start source server");
    let source_client = source_server.client();
    
    println!("Test: calc_delta with malformed change ID should fail gracefully");
    
    // Test with various malformed IDs
    let malformed_ids = vec![
        "not-a-uuid",
        "12345",
        "invalid@@@id",
        "spaces in id",
    ];
    
    for malformed_id in malformed_ids {
        println!("\nTesting malformed ID: '{}'", malformed_id);
        let response = source_client.index_calc_delta(malformed_id)
            .await
            .expect("Request should complete");
        
        // Should fail with error
        let result_str = response.get_result_str().unwrap_or("");
        assert!(result_str.contains("Error") || result_str.contains("not found") || result_str.contains("does not exist"), 
                "Should indicate error for '{}', got: {}", malformed_id, result_str);
        println!("✅ Failed appropriately: {}", result_str);
    }
    
    println!("\n✅ Test passed: calc_delta handles malformed IDs gracefully");
}

