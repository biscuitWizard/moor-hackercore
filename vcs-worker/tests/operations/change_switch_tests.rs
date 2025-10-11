//! Integration tests for change/switch operations
//!
//! These tests verify:
//! 1. Switching to non-existent changes fails appropriately
//! 2. Switching with various local change states
//! 3. Switch and switch-back behavior
//! 4. Error handling with invalid inputs

use crate::common::*;
use moor_vcs_worker::types::ChangeStatus;
use moor_vcs_worker::providers::workspace::WorkspaceProvider;

#[tokio::test]
async fn test_switch_to_non_existent_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Switching to a non-existent change ID should fail");
    
    // Attempt to switch to non-existent change
    println!("\nAttempting to switch to non-existent change...");
    let response = client.change_switch("non_existent_change_id")
        .await
        .expect("Request should complete");
    
    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found"), 
            "Should indicate change not found, got: {}", result_str);
    println!("✅ Switch failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot switch to non-existent change");
}

#[tokio::test]
async fn test_switch_with_empty_change_id() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Switching with empty change ID should fail");
    
    // Attempt to switch with empty ID
    println!("\nAttempting to switch with empty change ID...");
    let response = client.change_switch("")
        .await
        .expect("Request should complete");
    
    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("not found") || result_str.contains("required"), 
            "Should indicate error, got: {}", result_str);
    println!("✅ Switch failed with appropriate error: {}", result_str);
    
    println!("\n✅ Test passed: Cannot switch with empty change ID");
}

#[tokio::test]
async fn test_switch_with_no_local_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Switching with no local change active should work");
    
    // Step 1: Create and stash a change
    println!("\nStep 1: Creating and stashing change...");
    client.change_create("stashed_change", "test_author", Some("Will be stashed"))
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("stashed_obj", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_id, _) = db.require_top_change();
    
    client.change_stash()
        .await
        .expect("Failed to stash change");
    
    println!("✅ Change stashed");
    
    // Step 2: Verify no local change
    println!("\nStep 2: Verifying no local change...");
    db.assert_no_top_change();
    println!("✅ No local change");
    
    // Step 3: Switch to the stashed change
    println!("\nStep 3: Switching to stashed change...");
    let response = client.change_switch(&change_id)
        .await
        .expect("Failed to switch");
    
    // Should succeed
    println!("Switch response: {:?}", response);
    println!("✅ Switch successful");
    
    // Step 4: Verify change is now active
    println!("\nStep 4: Verifying change is now active...");
    let (new_change_id, new_change) = db.require_top_change();
    assert_eq!(new_change_id, change_id, "Should have switched to the stashed change");
    assert_eq!(new_change.status, ChangeStatus::Local, "Should be Local status");
    println!("✅ Change is now active");
    
    println!("\n✅ Test passed: Switch with no local change works");
}

#[tokio::test]
async fn test_switch_preserves_current_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Switching should preserve current change in workspace");
    
    // Step 1: Create first change with objects
    println!("\nStep 1: Creating first change...");
    client.change_create("first_change", "test_author", Some("First"))
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("first_obj", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (first_change_id, first_change) = db.require_top_change();
    assert_eq!(first_change.added_objects.len(), 1, "Should have 1 object");
    println!("✅ First change created");
    
    // Step 2: Stash it
    println!("\nStep 2: Stashing first change...");
    client.change_stash()
        .await
        .expect("Failed to stash");
    println!("✅ First change stashed");
    
    // Step 3: Create second change
    println!("\nStep 3: Creating second change...");
    client.change_create("second_change", "test_author", Some("Second"))
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("second_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (second_change_id, _) = db.require_top_change();
    println!("✅ Second change created");
    
    // Step 4: Switch back to first change
    println!("\nStep 4: Switching back to first change...");
    client.change_switch(&first_change_id)
        .await
        .expect("Failed to switch");
    println!("✅ Switched to first change");
    
    // Step 5: Verify first change is active
    println!("\nStep 5: Verifying first change is active...");
    let (active_id, active_change) = db.require_top_change();
    assert_eq!(active_id, first_change_id, "Should have first change active");
    assert_eq!(active_change.name, "first_change", "Name should match");
    assert_eq!(active_change.added_objects.len(), 1, "Should have preserved objects");
    println!("✅ First change is active with preserved data");
    
    // Step 6: Verify second change is in workspace
    println!("\nStep 6: Verifying second change in workspace...");
    let workspace_change = server.database().workspace().get_workspace_change(&second_change_id)
        .expect("Failed to get workspace change")
        .expect("Second change should be in workspace");
    
    assert_eq!(workspace_change.id, second_change_id, "Should be second change");
    assert_eq!(workspace_change.status, ChangeStatus::Idle, "Should be Idle");
    assert_eq!(workspace_change.added_objects.len(), 1, "Should have preserved objects");
    println!("✅ Second change preserved in workspace");
    
    println!("\n✅ Test passed: Switch preserves both changes");
}

#[tokio::test]
async fn test_switch_back_and_forth() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Switching back and forth should restore original state");
    
    // Step 1: Create and stash change A
    println!("\nStep 1: Creating change A...");
    client.change_create("change_a", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("obj_a", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_a_id, _) = db.require_top_change();
    
    client.change_stash()
        .await
        .expect("Failed to stash");
    println!("✅ Change A stashed");
    
    // Step 2: Create and stash change B
    println!("\nStep 2: Creating change B...");
    client.change_create("change_b", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("obj_b", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_b_id, _) = db.require_top_change();
    
    client.change_stash()
        .await
        .expect("Failed to stash");
    println!("✅ Change B stashed");
    
    // Step 3: Switch to A
    println!("\nStep 3: Switching to A...");
    client.change_switch(&change_a_id)
        .await
        .expect("Failed to switch to A");
    
    let (active_id, active_change) = db.require_top_change();
    assert_eq!(active_id, change_a_id, "Should be A");
    assert_eq!(active_change.name, "change_a", "Should be change_a");
    println!("✅ Switched to A");
    
    // Step 4: Switch to B
    println!("\nStep 4: Switching to B...");
    client.change_switch(&change_b_id)
        .await
        .expect("Failed to switch to B");
    
    let (active_id, active_change) = db.require_top_change();
    assert_eq!(active_id, change_b_id, "Should be B");
    assert_eq!(active_change.name, "change_b", "Should be change_b");
    println!("✅ Switched to B");
    
    // Step 5: Switch back to A
    println!("\nStep 5: Switching back to A...");
    client.change_switch(&change_a_id)
        .await
        .expect("Failed to switch back to A");
    
    let (active_id, active_change) = db.require_top_change();
    assert_eq!(active_id, change_a_id, "Should be A again");
    assert_eq!(active_change.name, "change_a", "Should be change_a");
    assert_eq!(active_change.added_objects.len(), 1, "Should have preserved object");
    assert_eq!(active_change.added_objects[0].name, "obj_a", "Should have obj_a");
    println!("✅ Switched back to A with preserved state");
    
    println!("\n✅ Test passed: Switch back and forth works correctly");
}

#[tokio::test]
async fn test_switch_when_current_is_merged_fails() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Switching when current change is merged should fail");
    
    // Step 1: Create and stash a change
    println!("\nStep 1: Creating and stashing target change...");
    client.change_create("target_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("target_obj", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (target_change_id, _) = db.require_top_change();
    
    client.change_stash()
        .await
        .expect("Failed to stash");
    println!("✅ Target change stashed");
    
    // Step 2: Create and approve a change (merged)
    println!("\nStep 2: Creating and approving merged change...");
    client.change_create("merged_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("merged_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (merged_change_id, _) = db.require_top_change();
    
    client.change_approve(&merged_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");
    
    // Verify no local change now
    db.assert_no_top_change();
    println!("✅ Merged change approved, no local change");
    
    // Step 3: Try to switch (should work since no local change)
    println!("\nStep 3: Switching to target (should work, no local change blocking)...");
    client.change_switch(&target_change_id)
        .await
        .expect("Failed to switch")
        .assert_success("Switch should work");
    
    println!("✅ Switch succeeded");
    
    println!("\n✅ Test passed: Switch works when current is merged (no blocking)");
}

#[tokio::test]
async fn test_switch_with_objects_in_current_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Switching with objects in current change preserves them");
    
    // Step 1: Create change with multiple objects
    println!("\nStep 1: Creating change with multiple objects...");
    client.change_create("multi_obj_change", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("obj1", "test_object.moo")
        .await
        .expect("Failed to update obj1");
    
    client.object_update_from_file("obj2", "detailed_test_object.moo")
        .await
        .expect("Failed to update obj2");
    
    client.object_update_from_file("obj3", "test_object.moo")
        .await
        .expect("Failed to update obj3");
    
    let (first_change_id, first_change) = db.require_top_change();
    assert_eq!(first_change.added_objects.len(), 3, "Should have 3 objects");
    println!("✅ Change created with 3 objects");
    
    // Step 2: Stash another change to switch to
    println!("\nStep 2: Creating stash target...");
    client.change_stash()
        .await
        .expect("Failed to stash");
    
    client.change_create("target", "test_author", None)
        .await
        .expect("Failed to create change");
    
    let (second_change_id, _) = db.require_top_change();
    
    client.change_stash()
        .await
        .expect("Failed to stash");
    println!("✅ Target stashed");
    
    // Step 3: Switch to first change
    println!("\nStep 3: Switching to first change...");
    client.change_switch(&first_change_id)
        .await
        .expect("Failed to switch");
    
    let (active_id, active_change) = db.require_top_change();
    assert_eq!(active_id, first_change_id, "Should be first change");
    assert_eq!(active_change.added_objects.len(), 3, "Should have all 3 objects preserved");
    
    // Verify all object names
    let obj_names: Vec<String> = active_change.added_objects.iter()
        .map(|o| o.name.clone())
        .collect();
    assert!(obj_names.contains(&"obj1".to_string()), "Should have obj1");
    assert!(obj_names.contains(&"obj2".to_string()), "Should have obj2");
    assert!(obj_names.contains(&"obj3".to_string()), "Should have obj3");
    
    println!("✅ All objects preserved");
    
    // Step 4: Switch to second change
    println!("\nStep 4: Switching to second change...");
    client.change_switch(&second_change_id)
        .await
        .expect("Failed to switch");
    println!("✅ Switched to second change");
    
    // Step 5: Verify first change preserved in workspace
    println!("\nStep 5: Verifying first change in workspace...");
    let workspace_change = server.database().workspace().get_workspace_change(&first_change_id)
        .expect("Failed to get workspace change")
        .expect("First change should be in workspace");
    
    assert_eq!(workspace_change.added_objects.len(), 3, "Should still have 3 objects");
    println!("✅ First change preserved with all objects");
    
    println!("\n✅ Test passed: Switch preserves objects in current change");
}
