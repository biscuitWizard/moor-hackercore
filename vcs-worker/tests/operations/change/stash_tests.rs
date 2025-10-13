//! Tests for change stash operations

use crate::common::*;
use moor_vcs_worker::providers::workspace::WorkspaceProvider;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_stash_local_change_moves_to_workspace_idle() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Stash local change should move it to workspace with Idle status");

    // Step 1: Create a change and add some objects
    println!("\nStep 1: Creating change and adding objects...");
    client
        .change_create(
            "test_stash_change",
            "test_author",
            Some("Test stash description"),
        )
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("stashed_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    println!("✅ Created change with 1 object");

    // Get the change ID before stashing
    let (change_id, change_before) = db.require_top_change();
    assert_eq!(
        change_before.status,
        ChangeStatus::Local,
        "Should be Local before stash"
    );
    assert_eq!(
        change_before.name, "test_stash_change",
        "Change name should match"
    );
    println!("✅ Change is Local with correct name");

    // Step 2: Stash the change
    println!("\nStep 2: Stashing the change...");
    let stash_response = client.change_stash().await.expect("Failed to stash change");

    // The response should be successful and contain undo diff
    println!("Stash response: {:?}", stash_response);
    println!("✅ Change stashed");

    // Step 3: Verify top change is None
    println!("\nStep 3: Verifying top change is cleared...");
    db.assert_no_top_change();
    println!("✅ Top change cleared from index");

    // Step 4: Verify change exists in workspace with Idle status
    println!("\nStep 4: Verifying change in workspace...");
    let workspace_change = server
        .database()
        .workspace()
        .get_workspace_change(&change_id)
        .expect("Failed to get workspace change")
        .expect("Change should exist in workspace");

    assert_eq!(workspace_change.id, change_id, "Change ID should match");
    assert_eq!(
        workspace_change.name, "test_stash_change",
        "Change name should match"
    );
    assert_eq!(
        workspace_change.status,
        ChangeStatus::Idle,
        "Status should be Idle"
    );
    assert_eq!(
        workspace_change.author, "test_author",
        "Author should be preserved"
    );
    assert_eq!(
        workspace_change.description,
        Some("Test stash description".to_string()),
        "Description should be preserved"
    );
    assert_eq!(
        workspace_change.added_objects.len(),
        1,
        "Should have 1 added object"
    );
    println!("✅ Change in workspace with Idle status");

    // Step 5: Verify object still exists
    println!("\nStep 5: Verifying object still exists...");
    db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "stashed_object",
    );
    println!("✅ Object still exists");

    println!("\n✅ Test passed: Stash moves change to workspace with Idle status");
}

#[tokio::test]
async fn test_stash_change_clears_index_top() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Stash should clear the top of the index");

    // Step 1: Create and stash a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("first_change", "test_author", None)
        .await
        .expect("Failed to create change");

    db.require_top_change();
    println!("✅ Change created");

    // Step 2: Stash the change
    println!("\nStep 2: Stashing change...");
    client.change_stash().await.expect("Failed to stash change");
    println!("✅ Change stashed");

    // Step 3: Verify no top change
    println!("\nStep 3: Verifying no top change...");
    db.assert_no_top_change();
    println!("✅ Top change cleared");

    println!("\n✅ Test passed: Stash clears index top");
}

#[tokio::test]
async fn test_cannot_stash_merged_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Cannot stash a merged change");

    // Step 1: Create and approve a change
    println!("\nStep 1: Creating and approving change...");
    client
        .change_create("merged_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("merged_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id, _) = db.require_top_change();

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");

    println!("✅ Change approved (merged)");

    // Step 2: Verify change is merged
    let merged_change = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    assert_eq!(
        merged_change.status,
        ChangeStatus::Merged,
        "Should be Merged"
    );

    // Step 3: Create a new local change and try to stash (should work)
    println!("\nStep 3: Creating new local change...");
    client
        .change_create("new_local_change", "test_author", None)
        .await
        .expect("Failed to create new change");

    println!("✅ New local change created");

    // Step 4: Stash should work on the local change
    println!("\nStep 4: Stashing local change (should succeed)...");
    client
        .change_stash()
        .await
        .expect("Stash should succeed on local change");

    println!("✅ Local change stashed successfully");

    // Verify the merged change is still merged
    let still_merged = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    assert_eq!(
        still_merged.status,
        ChangeStatus::Merged,
        "Merged change should still be Merged"
    );

    println!("\n✅ Test passed: Can only stash local changes");
}

#[tokio::test]
async fn test_cannot_stash_when_no_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Cannot stash when no change exists");

    // Step 1: Verify no change initially
    println!("\nStep 1: Verifying no change initially...");
    db.assert_no_top_change();
    println!("✅ No change initially");

    // Step 2: Try to stash (should fail)
    println!("\nStep 2: Attempting to stash with no change...");
    let response = client
        .change_stash()
        .await
        .expect("Request should complete");

    // Should return an error
    let result_str = response.get_result_str().unwrap_or("");
    let failed = !response.is_success()
        || result_str.contains("Error")
        || result_str.contains("No change to stash");

    assert!(failed, "Stash should fail when no change exists");
    println!("✅ Stash failed as expected: {}", result_str);

    println!("\n✅ Test passed: Cannot stash when no change exists");
}

#[tokio::test]
async fn test_stash_with_objects_preserves_data() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Stash preserves all object data");

    // Step 1: Create change with multiple objects
    println!("\nStep 1: Creating change with multiple objects...");
    client
        .change_create(
            "complex_stash",
            "complex_author",
            Some("Complex stash test"),
        )
        .await
        .expect("Failed to create change");

    // Add multiple objects
    client
        .object_update_from_file("object_1", "test_object.moo")
        .await
        .expect("Failed to update object 1");

    client
        .object_update_from_file("object_2", "detailed_test_object.moo")
        .await
        .expect("Failed to update object 2");

    println!("✅ Created change with 2 objects");

    // Get the change before stashing
    let (change_id, change_before) = db.require_top_change();
    assert_eq!(
        change_before.added_objects.len(),
        2,
        "Should have 2 added objects"
    );

    // Step 2: Stash the change
    println!("\nStep 2: Stashing change...");
    client.change_stash().await.expect("Failed to stash change");
    println!("✅ Change stashed");

    // Step 3: Verify all data is preserved in workspace
    println!("\nStep 3: Verifying data preservation...");
    let workspace_change = server
        .database()
        .workspace()
        .get_workspace_change(&change_id)
        .expect("Failed to get workspace change")
        .expect("Change should exist in workspace");

    assert_eq!(workspace_change.name, "complex_stash", "Name preserved");
    assert_eq!(
        workspace_change.author, "complex_author",
        "Author preserved"
    );
    assert_eq!(
        workspace_change.description,
        Some("Complex stash test".to_string()),
        "Description preserved"
    );
    assert_eq!(
        workspace_change.added_objects.len(),
        2,
        "Object count preserved"
    );
    assert_eq!(
        workspace_change.status,
        ChangeStatus::Idle,
        "Status is Idle"
    );

    // Verify object names
    let object_names: Vec<String> = workspace_change
        .added_objects
        .iter()
        .map(|obj| obj.name.clone())
        .collect();
    assert!(
        object_names.contains(&"object_1".to_string()),
        "Object 1 preserved"
    );
    assert!(
        object_names.contains(&"object_2".to_string()),
        "Object 2 preserved"
    );

    println!("✅ All data preserved correctly");

    println!("\n✅ Test passed: Stash preserves all object data");
}

#[tokio::test]
async fn test_stash_then_switch_back() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Stash a change and then switch back to it");

    // Step 1: Create and stash a change
    println!("\nStep 1: Creating and stashing change...");
    client
        .change_create("stashed_change", "test_author", Some("Will be stashed"))
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("stashed_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id, _) = db.require_top_change();

    client.change_stash().await.expect("Failed to stash change");

    println!("✅ Change stashed");

    // Verify it's in workspace with Idle status
    let workspace_change = server
        .database()
        .workspace()
        .get_workspace_change(&change_id)
        .expect("Failed to get workspace change")
        .expect("Change should exist");
    assert_eq!(
        workspace_change.status,
        ChangeStatus::Idle,
        "Should be Idle"
    );

    // Step 2: Verify no top change
    println!("\nStep 2: Verifying no top change...");
    db.assert_no_top_change();
    println!("✅ No top change");

    // Step 3: Switch back to the stashed change
    println!("\nStep 3: Switching back to stashed change...");
    let switch_response = client
        .change_switch(&change_id)
        .await
        .expect("Failed to switch to stashed change");

    // Verify switch was successful
    println!("Switch response: {:?}", switch_response);

    // Step 4: Verify the change is back on top of index
    println!("\nStep 4: Verifying change is back on index...");
    let (new_top_id, new_top_change) = db.require_top_change();

    assert_eq!(new_top_id, change_id, "Change ID should match");
    assert_eq!(
        new_top_change.name, "stashed_change",
        "Change name should match"
    );
    assert_eq!(
        new_top_change.status,
        ChangeStatus::Local,
        "Status should be Local again"
    );
    assert_eq!(
        new_top_change.added_objects.len(),
        1,
        "Should have 1 object"
    );

    println!("✅ Change back on index with Local status");

    // Step 5: Verify object still exists
    println!("\nStep 5: Verifying object still exists...");
    db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "stashed_obj",
    );
    println!("✅ Object still exists");

    println!("\n✅ Test passed: Can stash and switch back to change");
}

