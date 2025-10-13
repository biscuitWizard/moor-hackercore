//! Integration tests for object/switch operations
//!
//! These tests verify:
//! 1. Basic object switching between changes
//! 2. Force parameter behavior
//! 3. Error handling for various invalid scenarios
//! 4. Meta object handling
//! 5. Modified object switching

use crate::common::*;
use moor_vcs_worker::providers::index::IndexProvider;
use moor_vcs_worker::providers::workspace::WorkspaceProvider;
use moor_vcs_worker::types::VcsObjectType;

#[tokio::test]
async fn test_object_switch_basic() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Basic object switch from local change to target workspace change");

    // Step 1: Create local change with an object
    println!("\nStep 1: Creating local change with object...");
    client
        .object_update_from_file("test_obj", "test_object.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Object create");

    let (local_change_id, local_change) = db.require_top_change();
    println!("✅ Local change created: {}", local_change_id);

    // Verify object is in added_objects
    assert!(
        local_change
            .added_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == "test_obj"),
        "Object should be in added_objects"
    );

    // Step 2: Create target change in workspace (stash current, create new, stash that)
    println!("\nStep 2: Creating target change in workspace...");
    client
        .change_stash()
        .await
        .expect("Failed to stash local change");

    client
        .change_create("target_change", "test_author", Some("Target"))
        .await
        .expect("Failed to create target change");

    let (target_change_id, _) = db.require_top_change();

    client
        .change_stash()
        .await
        .expect("Failed to stash target change");

    // Switch back to local change
    client
        .change_switch(&local_change_id)
        .await
        .expect("Failed to switch back to local");

    println!("✅ Target change created in workspace: {}", target_change_id);

    // Step 3: Switch object from local to target
    println!("\nStep 3: Switching object to target change...");
    let response = client
        .object_switch("test_obj", &target_change_id, None)
        .await
        .expect("Failed to switch object");

    response.assert_success("Object switch");
    println!("✅ Object switched successfully");

    // Step 4: Verify object removed from local change
    println!("\nStep 4: Verifying object removed from local change...");
    let updated_local = server
        .database()
        .index()
        .get_change(&local_change_id)
        .expect("Failed to get local change")
        .expect("Local change should exist");

    assert!(
        !updated_local
            .added_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == "test_obj"),
        "Object should be removed from local change"
    );
    println!("✅ Object removed from local change");

    // Step 5: Verify object added to target change
    println!("\nStep 5: Verifying object added to target change...");
    let updated_target = server
        .database()
        .workspace()
        .get_workspace_change(&target_change_id)
        .expect("Failed to get target change")
        .expect("Target change should exist");

    assert!(
        updated_target
            .added_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == "test_obj"),
        "Object should be in target change"
    );
    println!("✅ Object added to target change");

    println!("\n✅ Test passed: Basic object switch works");
}

#[tokio::test]
async fn test_object_switch_force_overwrite() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Force parameter allows overwriting existing objects");

    // Step 1: Create local change with object A
    println!("\nStep 1: Creating local change with object...");
    client
        .object_update_from_file("test_obj", "test_object.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Object create");

    let (local_change_id, _) = db.require_top_change();

    // Step 2: Create target change with same object (different version)
    println!("\nStep 2: Creating target change with same object...");
    client
        .change_stash()
        .await
        .expect("Failed to stash local");

    client
        .change_create("target_change", "test_author", Some("Target"))
        .await
        .expect("Failed to create target");

    client
        .object_update_from_file("test_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to create object in target")
        .assert_success("Object create in target");

    let (target_change_id, _) = db.require_top_change();

    client
        .change_stash()
        .await
        .expect("Failed to stash target");

    client
        .change_switch(&local_change_id)
        .await
        .expect("Failed to switch back");

    println!("✅ Both changes have the same object");

    // Step 3: Try switch without force - should fail
    println!("\nStep 3: Trying switch without force (should fail)...");
    let response = client
        .object_switch("test_obj", &target_change_id, None)
        .await
        .expect("Request should complete");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("already exists"),
        "Should fail with error about existing object, got: {}",
        result_str
    );
    println!("✅ Switch without force failed as expected");

    // Step 4: Try switch with force - should succeed
    println!("\nStep 4: Trying switch with force=true (should succeed)...");
    let response = client
        .object_switch("test_obj", &target_change_id, Some(true))
        .await
        .expect("Failed to switch with force");

    response.assert_success("Object switch with force");
    println!("✅ Switch with force succeeded");

    // Verify object is in target
    let updated_target = server
        .database()
        .workspace()
        .get_workspace_change(&target_change_id)
        .expect("Failed to get target")
        .expect("Target should exist");

    let obj_count = updated_target
        .added_objects
        .iter()
        .filter(|o| o.object_type == VcsObjectType::MooObject && o.name == "test_obj")
        .count();

    assert_eq!(obj_count, 1, "Should have exactly one instance of the object");
    println!("✅ Object successfully overwrote existing one");

    println!("\n✅ Test passed: Force parameter works correctly");
}

#[tokio::test]
async fn test_object_switch_no_local_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Cannot switch object when no local change exists");

    // Verify no local change
    db.assert_no_top_change();

    // Create a dummy workspace change to switch to
    client
        .change_create("target", "author", None)
        .await
        .expect("Failed to create change");

    let (target_id, _) = db.require_top_change();

    client
        .change_stash()
        .await
        .expect("Failed to stash");

    // Try to switch object without local change
    println!("\nAttempting to switch object without local change...");
    let response = client
        .object_switch("any_obj", &target_id, None)
        .await
        .expect("Request should complete");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("No local change"),
        "Should fail with no local change error, got: {}",
        result_str
    );
    println!("✅ Failed as expected: {}", result_str);

    println!("\n✅ Test passed: Cannot switch without local change");
}

#[tokio::test]
async fn test_object_switch_object_not_in_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Cannot switch object not in current change");

    // Create local change with object A
    client
        .object_update_from_file("obj_a", "test_object.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Create obj_a");

    let (local_id, _) = db.require_top_change();

    // Create target change
    client
        .change_stash()
        .await
        .expect("Failed to stash");

    client
        .change_create("target", "author", None)
        .await
        .expect("Failed to create target");

    let (target_id, _) = db.require_top_change();

    client
        .change_stash()
        .await
        .expect("Failed to stash target");

    client
        .change_switch(&local_id)
        .await
        .expect("Failed to switch back");

    // Try to switch obj_b which doesn't exist
    println!("\nAttempting to switch non-existent object...");
    let response = client
        .object_switch("obj_b", &target_id, None)
        .await
        .expect("Request should complete");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Should fail with object not found error, got: {}",
        result_str
    );
    println!("✅ Failed as expected: {}", result_str);

    println!("\n✅ Test passed: Cannot switch object not in change");
}

#[tokio::test]
async fn test_object_switch_target_merged() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Cannot switch object to a Merged change");

    // Create and approve a change (becomes Merged)
    println!("\nStep 1: Creating and approving a change...");
    client
        .object_update_from_file("obj1", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (merged_change_id, _) = db.require_top_change();

    client
        .change_approve(&merged_change_id)
        .await
        .expect("Failed to approve");

    println!("✅ Change approved (now Merged)");

    // Create new local change with different object
    println!("\nStep 2: Creating new local change...");
    client
        .object_update_from_file("obj2", "test_object.moo")
        .await
        .expect("Failed to create object");

    // Try to switch to merged change
    println!("\nStep 3: Attempting to switch to merged change...");
    let response = client
        .object_switch("obj2", &merged_change_id, None)
        .await
        .expect("Request should complete");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("Merged"),
        "Should fail with merged change error, got: {}",
        result_str
    );
    println!("✅ Failed as expected: {}", result_str);

    println!("\n✅ Test passed: Cannot switch to merged change");
}

#[tokio::test]
async fn test_object_switch_deleted_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Cannot switch deleted objects");

    // Create and commit an object
    println!("\nStep 1: Creating and committing object...");
    client
        .object_update_from_file("test_obj", "test_object.moo")
        .await
        .expect("Failed to create");

    let (change_id, _) = db.require_top_change();

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve");

    // Delete the object
    println!("\nStep 2: Deleting object...");
    client
        .object_delete("test_obj")
        .await
        .expect("Failed to delete");

    let (local_id, local_change) = db.require_top_change();

    // Verify it's in deleted_objects
    assert!(
        local_change
            .deleted_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == "test_obj"),
        "Object should be in deleted_objects"
    );

    // Create target change
    println!("\nStep 3: Creating target change...");
    client
        .change_stash()
        .await
        .expect("Failed to stash");

    client
        .change_create("target", "author", None)
        .await
        .expect("Failed to create target");

    let (target_id, _) = db.require_top_change();

    client
        .change_stash()
        .await
        .expect("Failed to stash target");

    client
        .change_switch(&local_id)
        .await
        .expect("Failed to switch back");

    // Try to switch deleted object
    println!("\nStep 4: Attempting to switch deleted object...");
    let response = client
        .object_switch("test_obj", &target_id, None)
        .await
        .expect("Request should complete");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Should fail because deleted objects can't be switched, got: {}",
        result_str
    );
    println!("✅ Failed as expected: {}", result_str);

    println!("\n✅ Test passed: Cannot switch deleted objects");
}

#[tokio::test]
async fn test_object_switch_with_meta() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Meta objects are moved along with main objects");

    // Create object with meta
    println!("\nStep 1: Creating object with meta...");
    client
        .object_update_from_file("test_obj", "test_object.moo")
        .await
        .expect("Failed to create object");

    client
        .meta_add_ignored_verb("test_obj", "test_verb")
        .await
        .expect("Failed to add ignored verb");

    let (local_id, local_change) = db.require_top_change();

    // Verify meta exists in local change
    let has_meta = local_change
        .added_objects
        .iter()
        .any(|o| o.object_type == VcsObjectType::MooMetaObject && o.name == "test_obj");

    assert!(has_meta, "Meta should exist in local change");
    println!("✅ Object and meta created");

    // Create target change
    println!("\nStep 2: Creating target change...");
    client
        .change_stash()
        .await
        .expect("Failed to stash");

    client
        .change_create("target", "author", None)
        .await
        .expect("Failed to create target");

    let (target_id, _) = db.require_top_change();

    client
        .change_stash()
        .await
        .expect("Failed to stash target");

    client
        .change_switch(&local_id)
        .await
        .expect("Failed to switch back");

    // Switch object
    println!("\nStep 3: Switching object (should move meta too)...");
    client
        .object_switch("test_obj", &target_id, None)
        .await
        .expect("Failed to switch")
        .assert_success("Object switch");

    // Verify meta removed from local
    println!("\nStep 4: Verifying meta removed from local...");
    let updated_local = server
        .database()
        .index()
        .get_change(&local_id)
        .expect("Failed to get local")
        .expect("Local should exist");

    let has_meta_in_local = updated_local.added_objects.iter().any(|o| {
        o.object_type == VcsObjectType::MooMetaObject && o.name == "test_obj"
    });

    assert!(!has_meta_in_local, "Meta should be removed from local");
    println!("✅ Meta removed from local");

    // Verify meta added to target
    println!("\nStep 5: Verifying meta added to target...");
    let updated_target = server
        .database()
        .workspace()
        .get_workspace_change(&target_id)
        .expect("Failed to get target")
        .expect("Target should exist");

    let has_meta_in_target = updated_target.added_objects.iter().any(|o| {
        o.object_type == VcsObjectType::MooMetaObject && o.name == "test_obj"
    });

    assert!(has_meta_in_target, "Meta should be in target");
    println!("✅ Meta added to target");

    println!("\n✅ Test passed: Meta objects move with main objects");
}

#[tokio::test]
async fn test_object_switch_modified_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Switching modified objects");

    // Create and commit object
    println!("\nStep 1: Creating and committing object...");
    client
        .object_update_from_file("test_obj", "test_object.moo")
        .await
        .expect("Failed to create");

    let (change_id, _) = db.require_top_change();

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve");

    // Modify object
    println!("\nStep 2: Modifying object...");
    client
        .object_update_from_file("test_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to modify");

    let (local_id, local_change) = db.require_top_change();

    // Verify it's in modified_objects
    assert!(
        local_change
            .modified_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == "test_obj"),
        "Object should be in modified_objects"
    );
    println!("✅ Object modified");

    // Create target change
    println!("\nStep 3: Creating target change...");
    client
        .change_stash()
        .await
        .expect("Failed to stash");

    client
        .change_create("target", "author", None)
        .await
        .expect("Failed to create target");

    let (target_id, _) = db.require_top_change();

    client
        .change_stash()
        .await
        .expect("Failed to stash target");

    client
        .change_switch(&local_id)
        .await
        .expect("Failed to switch back");

    // Switch modified object
    println!("\nStep 4: Switching modified object...");
    client
        .object_switch("test_obj", &target_id, None)
        .await
        .expect("Failed to switch")
        .assert_success("Object switch");

    // Verify object in target's modified_objects
    println!("\nStep 5: Verifying object in target's modified_objects...");
    let updated_target = server
        .database()
        .workspace()
        .get_workspace_change(&target_id)
        .expect("Failed to get target")
        .expect("Target should exist");

    assert!(
        updated_target
            .modified_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == "test_obj"),
        "Object should be in target's modified_objects"
    );
    println!("✅ Object in target's modified_objects");

    // Verify object removed from local
    let updated_local = server
        .database()
        .index()
        .get_change(&local_id)
        .expect("Failed to get local")
        .expect("Local should exist");

    assert!(
        !updated_local
            .modified_objects
            .iter()
            .filter(|o| o.object_type == VcsObjectType::MooObject)
            .any(|o| o.name == "test_obj"),
        "Object should be removed from local"
    );
    println!("✅ Object removed from local");

    println!("\n✅ Test passed: Modified objects can be switched");
}
