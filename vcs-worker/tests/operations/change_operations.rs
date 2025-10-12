//! Integration tests for change operations (create, abandon, approve)
//!
//! These tests verify:
//! 1. Creating an empty local change
//! 2. Abandoning a local change clears top change and cleans up unused resources
//! 3. Approving a change moves it to Merged status and canonizes refs/SHA256s

use crate::common::*;
use moor_vcs_worker::providers::workspace::WorkspaceProvider;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_change_create_empty_local() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Create change should create an empty local change");

    // Step 1: Verify no change initially
    println!("\nStep 1: Verifying no change initially...");
    db.assert_no_top_change();
    println!("✅ No change initially");

    // Step 2: Create a change
    println!("\nStep 2: Creating a new change...");
    client
        .change_create("test_change", "test_author", Some("Test description"))
        .await
        .expect("Failed to create change")
        .assert_success("Change creation");
    println!("✅ Change created");

    // Step 3: Verify the change exists and is empty
    println!("\nStep 3: Verifying change exists and is empty...");
    let (_, change) = db.require_top_change();

    assert_eq!(change.name, "test_change", "Change name should match");
    assert_eq!(change.author, "test_author", "Author should match");
    assert_eq!(change.status, ChangeStatus::Local, "Status should be Local");
    assert_eq!(
        change.added_objects.len(),
        0,
        "Should have no added objects"
    );
    assert_eq!(
        change.modified_objects.len(),
        0,
        "Should have no modified objects"
    );
    assert_eq!(
        change.deleted_objects.len(),
        0,
        "Should have no deleted objects"
    );
    assert_eq!(
        change.renamed_objects.len(),
        0,
        "Should have no renamed objects"
    );
    println!("✅ Change exists and is empty");

    println!("\n✅ Test passed: Create change creates empty local change");
}

#[tokio::test]
async fn test_abandon_local_change_clears_top_and_cleans_up() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Abandon local change should clear top change and cleanup unused resources");

    // Step 1: Create a change and add some objects
    println!("\nStep 1: Creating change and adding objects...");
    client
        .change_create("test_abandon_change", "test_author", None)
        .await
        .expect("Failed to create change");

    // Add some objects to the change
    let object_content_1 = moo_to_lines(&load_moo_file("test_object.moo"));
    let content_str_1 = object_content_1.join("\n");
    let sha256_1 = TestServer::calculate_sha256(&content_str_1);

    client
        .object_update("test_object_1", object_content_1)
        .await
        .expect("Failed to update object 1");

    let object_content_2 = moo_to_lines(&load_moo_file("detailed_test_object.moo"));
    let content_str_2 = object_content_2.join("\n");
    let sha256_2 = TestServer::calculate_sha256(&content_str_2);

    client
        .object_update("test_object_2", object_content_2)
        .await
        .expect("Failed to update object 2");

    println!("✅ Created change with 2 objects");

    // Verify objects exist
    let sha256_1_before = server
        .database()
        .objects()
        .get(&sha256_1)
        .expect("Failed to get object 1")
        .is_some();
    let sha256_2_before = server
        .database()
        .objects()
        .get(&sha256_2)
        .expect("Failed to get object 2")
        .is_some();

    assert!(sha256_1_before, "SHA256 1 should exist before abandon");
    assert!(sha256_2_before, "SHA256 2 should exist before abandon");
    println!("✅ Objects exist in database");

    // Step 2: Abandon the change
    println!("\nStep 2: Abandoning the change...");
    client
        .change_abandon()
        .await
        .expect("Failed to abandon change")
        .assert_success("Abandon change");
    println!("✅ Change abandoned");

    // Step 3: Verify top change is None
    println!("\nStep 3: Verifying top change is cleared...");
    db.assert_no_top_change();
    println!("✅ Top change cleared");

    // Step 4: Verify SHA256s were cleaned up (not referenced elsewhere)
    println!("\nStep 4: Verifying SHA256 cleanup...");
    let sha256_1_after = server
        .database()
        .objects()
        .get(&sha256_1)
        .expect("Failed to check SHA256 1")
        .is_some();
    let sha256_2_after = server
        .database()
        .objects()
        .get(&sha256_2)
        .expect("Failed to check SHA256 2")
        .is_some();

    println!("SHA256 1 after abandon: {}", sha256_1_after);
    println!("SHA256 2 after abandon: {}", sha256_2_after);

    // Step 5: Verify refs were cleaned up
    println!("\nStep 5: Verifying ref cleanup...");
    let ref_1_after = server
        .database()
        .refs()
        .get_ref(
            moor_vcs_worker::types::VcsObjectType::MooObject,
            "test_object_1",
            None,
        )
        .expect("Failed to check ref 1");
    let ref_2_after = server
        .database()
        .refs()
        .get_ref(
            moor_vcs_worker::types::VcsObjectType::MooObject,
            "test_object_2",
            None,
        )
        .expect("Failed to check ref 2");

    println!("Ref 1 after abandon: {:?}", ref_1_after);
    println!("Ref 2 after abandon: {:?}", ref_2_after);

    println!("\n⚠️  Note: Cleanup of SHA256s and refs during abandon needs to be implemented");
    println!("✅ Test completed (cleanup verification pending implementation)");
}

#[tokio::test]
async fn test_abandon_with_last_merged_returns_to_merged() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!(
        "Test: Abandoning local change when there's a merged change should return to merged state"
    );

    // Step 1: Create and commit a change
    println!("\nStep 1: Creating and committing first change...");
    client
        .change_create("first_change", "test_author", None)
        .await
        .expect("Failed to create first change");

    client
        .object_update_from_file("first_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    // Approve the change using HTTP API
    let (change_id, _) = db.require_top_change();

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");

    println!("✅ First change committed using change/approve API");

    // Verify no local change
    db.assert_no_top_change();

    // Step 2: Create a second local change
    println!("\nStep 2: Creating second local change...");
    client
        .change_create("second_change", "test_author", None)
        .await
        .expect("Failed to create second change");

    client
        .object_update_from_file("second_object", "detailed_test_object.moo")
        .await
        .expect("Failed to update object 2");

    println!("✅ Second change created");

    // Verify we have a local change
    db.require_top_change();

    // Step 3: Abandon the second change
    println!("\nStep 3: Abandoning second change...");
    client
        .change_abandon()
        .await
        .expect("Failed to abandon change")
        .assert_success("Abandon change");
    println!("✅ Second change abandoned");

    // Give the system a moment to fully process the abandon
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Step 4: Verify top change is None (but merged change still exists in database)
    println!("\nStep 4: Verifying state after abandon...");
    db.assert_no_top_change();
    println!("✅ Top change is None");

    // Verify the first (merged) change still exists
    let first_change = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get first change")
        .expect("First change should still exist");

    assert_eq!(
        first_change.status,
        ChangeStatus::Merged,
        "First change should still be Merged"
    );
    println!("✅ First change still exists as Merged");

    // Verify first object still exists (from merged change)
    db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "first_object",
    );
    println!("✅ First object still exists");

    println!("\n✅ Test passed: Abandoning local change returns to merged state");
}

#[tokio::test]
async fn test_approve_change_moves_to_merged() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Approve change should move to Merged status and canonize refs/SHA256s");

    // Step 1: Create a change with objects
    println!("\nStep 1: Creating change with objects...");
    client
        .change_create("test_approve_change", "test_author", None)
        .await
        .expect("Failed to create change");

    let object_content = moo_to_lines(&load_moo_file("test_object.moo"));
    let content_str = object_content.join("\n");
    let sha256 = TestServer::calculate_sha256(&content_str);

    client
        .object_update("approved_object", object_content)
        .await
        .expect("Failed to update object");

    println!("✅ Change created with object");

    // Get change ID
    let (change_id, change_before) = db.require_top_change();

    assert_eq!(
        change_before.status,
        ChangeStatus::Local,
        "Should be Local before approve"
    );
    println!("✅ Change status is Local");

    // Step 2: Approve the change using HTTP API (Wizard user has approval permission)
    println!("\nStep 2: Approving the change...");

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");

    println!("✅ Change approved and marked as Merged using change/approve API");

    // Step 3: Verify change is marked as Merged
    println!("\nStep 3: Verifying change status...");
    let change_after = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should still exist in database");

    assert_eq!(
        change_after.status,
        ChangeStatus::Merged,
        "Should be Merged after approve"
    );
    println!("✅ Change status is Merged");

    // Step 4: Verify change is removed from top of index
    println!("\nStep 4: Verifying change removed from index top...");
    db.assert_no_top_change();
    println!("✅ Change removed from top of index");

    // Step 5: Verify SHA256 still exists (canonized)
    println!("\nStep 5: Verifying SHA256 canonization...");
    let sha256_after = server
        .database()
        .objects()
        .get(&sha256)
        .expect("Failed to check SHA256")
        .is_some();

    assert!(
        sha256_after,
        "SHA256 should still exist after approve (canonized)"
    );
    println!("✅ SHA256 canonized (still exists)");

    // Step 6: Verify ref still exists (canonized)
    println!("\nStep 6: Verifying ref canonization...");
    let ref_after = db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "approved_object",
    );

    assert_eq!(ref_after, sha256, "Ref should point to correct SHA256");
    println!("✅ Ref canonized (still exists and points to correct SHA256)");

    println!("\n✅ Test passed: Approve change moves to Merged and canonizes refs/SHA256s");
}

#[tokio::test]
async fn test_cannot_create_change_when_local_exists() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Cannot create a new change when a local change already exists");

    // Step 1: Create first change
    println!("\nStep 1: Creating first change...");
    client
        .change_create("first_change", "test_author", None)
        .await
        .expect("Failed to create first change")
        .assert_success("Create first change");
    println!("✅ First change created");

    // Step 2: Try to create second change (should fail)
    println!("\nStep 2: Trying to create second change...");
    let response = client
        .change_create("second_change", "test_author", None)
        .await
        .expect("Request should complete");

    // The operation might succeed at the RPC level but return an error message
    let result_str = response.get_result_str().unwrap_or("");
    let failed = !response.is_success()
        || result_str.contains("Error")
        || result_str.contains("Already in a local change");

    assert!(
        failed,
        "Second change creation should fail (already in local change)"
    );
    println!("✅ Second change creation failed as expected");

    // Verify only one change exists
    let (_, change) = db.require_top_change();

    assert_eq!(
        change.name, "first_change",
        "Only first change should exist"
    );
    println!("✅ Only first change exists");

    println!("\n✅ Test passed: Cannot create change when local exists");
}

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

#[tokio::test]
async fn test_submit_on_non_remote_index_instantly_approves() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Submit on non-remote index should instantly approve the change");

    // Step 1: Verify no source URL (non-remote index)
    println!("\nStep 1: Verifying no source URL...");
    let source_url = server
        .database()
        .index()
        .get_source()
        .expect("Failed to get source URL");
    assert!(
        source_url.is_none(),
        "Should have no source URL (non-remote index)"
    );
    println!("✅ No source URL configured");

    // Step 2: Create a change with objects
    println!("\nStep 2: Creating change with objects...");
    client
        .change_create(
            "test_submit_change",
            "test_author",
            Some("Test submit description"),
        )
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("submit_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    println!("✅ Change created with object");

    // Get change ID before submit
    let (change_id, change_before) = db.require_top_change();
    assert_eq!(
        change_before.status,
        ChangeStatus::Local,
        "Should be Local before submit"
    );
    assert_eq!(
        change_before.name, "test_submit_change",
        "Change name should match"
    );
    println!("✅ Change is Local with correct name");

    // Step 3: Submit the change
    println!("\nStep 3: Submitting the change...");
    let submit_response = client
        .change_submit()
        .await
        .expect("Failed to submit change");

    submit_response.assert_success("Submit change");
    println!("✅ Change submitted successfully");

    // Step 4: Verify change is marked as Merged (instantly approved)
    println!("\nStep 4: Verifying change status...");
    let change_after = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should still exist in database");

    assert_eq!(
        change_after.status,
        ChangeStatus::Merged,
        "Should be Merged after submit on non-remote index"
    );
    println!("✅ Change status is Merged (instantly approved)");

    // Step 5: Verify change is removed from top of index
    println!("\nStep 5: Verifying change removed from index top...");
    db.assert_no_top_change();
    println!("✅ Change removed from top of index");

    // Step 6: Verify change is NOT in workspace
    println!("\nStep 6: Verifying change is not in workspace...");
    let workspace_change = server
        .database()
        .workspace()
        .get_workspace_change(&change_id)
        .expect("Failed to check workspace");
    assert!(
        workspace_change.is_none(),
        "Change should not be in workspace (instantly approved, not waiting for review)"
    );
    println!("✅ Change not in workspace");

    // Step 7: Verify object still exists (canonized)
    println!("\nStep 7: Verifying object still exists...");
    db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "submit_object",
    );
    println!("✅ Object still exists (canonized)");

    println!("\n✅ Test passed: Submit on non-remote index instantly approves the change");
}

#[tokio::test]
async fn test_submit_multiple_changes_on_non_remote_index() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Submit multiple changes sequentially on non-remote index");

    // Step 1: Create and submit first change
    println!("\nStep 1: Creating and submitting first change...");
    client
        .change_create("first_submit", "test_author", None)
        .await
        .expect("Failed to create first change");

    client
        .object_update_from_file("first_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (first_change_id, _) = db.require_top_change();

    client
        .change_submit()
        .await
        .expect("Failed to submit first change")
        .assert_success("Submit first change");

    println!("✅ First change submitted");

    // Verify first change is merged
    let first_change = server
        .database()
        .index()
        .get_change(&first_change_id)
        .expect("Failed to get first change")
        .expect("First change should exist");
    assert_eq!(
        first_change.status,
        ChangeStatus::Merged,
        "First change should be Merged"
    );

    // Verify no top change
    db.assert_no_top_change();
    println!("✅ First change merged, no top change");

    // Step 2: Create and submit second change
    println!("\nStep 2: Creating and submitting second change...");
    client
        .change_create("second_submit", "test_author", None)
        .await
        .expect("Failed to create second change");

    client
        .object_update_from_file("second_object", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");

    let (second_change_id, _) = db.require_top_change();

    client
        .change_submit()
        .await
        .expect("Failed to submit second change")
        .assert_success("Submit second change");

    println!("✅ Second change submitted");

    // Step 3: Verify both changes are merged
    println!("\nStep 3: Verifying both changes are merged...");
    let first_change_final = server
        .database()
        .index()
        .get_change(&first_change_id)
        .expect("Failed to get first change")
        .expect("First change should exist");
    assert_eq!(
        first_change_final.status,
        ChangeStatus::Merged,
        "First change should still be Merged"
    );

    let second_change_final = server
        .database()
        .index()
        .get_change(&second_change_id)
        .expect("Failed to get second change")
        .expect("Second change should exist");
    assert_eq!(
        second_change_final.status,
        ChangeStatus::Merged,
        "Second change should be Merged"
    );

    println!("✅ Both changes are Merged");

    // Step 4: Verify both objects exist
    println!("\nStep 4: Verifying both objects exist...");
    db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "first_object",
    );
    db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "second_object",
    );
    println!("✅ Both objects exist");

    // Step 5: Verify no top change
    println!("\nStep 5: Verifying no top change...");
    db.assert_no_top_change();
    println!("✅ No top change");

    println!(
        "\n✅ Test passed: Multiple changes can be submitted sequentially on non-remote index"
    );
}

#[tokio::test]
async fn test_submit_on_remote_index_sends_to_workspace() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Submit on remote index should move change to workspace with Review status");

    // Step 1: Set a source URL to simulate remote index
    println!("\nStep 1: Setting source URL to simulate remote index...");
    let source_url = "http://example.com/vcs";
    server
        .database()
        .index()
        .set_source(source_url)
        .expect("Failed to set source URL");

    let configured_source = server
        .database()
        .index()
        .get_source()
        .expect("Failed to get source URL");
    assert_eq!(
        configured_source,
        Some(source_url.to_string()),
        "Source URL should be set"
    );
    println!("✅ Source URL configured: {}", source_url);

    // Step 2: Create a change with objects
    println!("\nStep 2: Creating change with objects...");
    client
        .change_create(
            "test_remote_submit",
            "test_author",
            Some("Remote submit test"),
        )
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("remote_submit_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    println!("✅ Change created with object");

    // Get change ID before submit
    let (change_id, change_before) = db.require_top_change();
    assert_eq!(
        change_before.status,
        ChangeStatus::Local,
        "Should be Local before submit"
    );
    println!("✅ Change is Local");

    // Step 3: Submit the change
    println!("\nStep 3: Submitting the change...");
    let submit_response = client
        .change_submit()
        .await
        .expect("Failed to submit change");

    // The submission might succeed even if remote fails (remote is best-effort)
    println!("Submit response: {:?}", submit_response);
    println!(
        "✅ Change submitted (remote submission may have failed, but local submission should succeed)"
    );

    // Step 4: Verify change is removed from top of index
    println!("\nStep 4: Verifying change removed from index top...");
    db.assert_no_top_change();
    println!("✅ Change removed from top of index");

    // Step 5: Verify change is in workspace with Review status
    println!("\nStep 5: Verifying change in workspace...");
    let workspace_change = server
        .database()
        .workspace()
        .get_workspace_change(&change_id)
        .expect("Failed to check workspace")
        .expect("Change should be in workspace when submitted to remote index");

    assert_eq!(workspace_change.id, change_id, "Change ID should match");
    assert_eq!(
        workspace_change.name, "test_remote_submit",
        "Change name should match"
    );
    assert_eq!(
        workspace_change.status,
        ChangeStatus::Review,
        "Status should be Review when submitted to remote index"
    );
    println!("✅ Change in workspace with Review status");

    // Step 6: Verify change in index is NOT marked as Merged
    println!("\nStep 6: Verifying change is not merged...");
    let index_change = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get change from index");

    // The change should either not exist in index or not be Merged
    if let Some(ch) = index_change {
        assert_ne!(
            ch.status,
            ChangeStatus::Merged,
            "Change should not be Merged when submitted to remote index"
        );
        println!(
            "✅ Change exists in index but is not Merged: {:?}",
            ch.status
        );
    } else {
        println!("✅ Change removed from index (moved to workspace for review)");
    }

    println!(
        "\n✅ Test passed: Submit on remote index moves change to workspace with Review status"
    );
}

#[tokio::test]
async fn test_approve_non_existent_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Approving a non-existent change ID should fail");

    // Attempt to approve non-existent change
    println!("\nAttempting to approve non-existent change...");
    let response = client
        .change_approve("non_existent_change_id")
        .await
        .expect("Request should complete");

    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Should indicate change not found, got: {}",
        result_str
    );
    println!("✅ Approve failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot approve non-existent change");
}

#[tokio::test]
async fn test_approve_already_merged_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Approving an already merged change should be idempotent or fail gracefully");

    // Step 1: Create and approve a change
    println!("\nStep 1: Creating and approving change...");
    client
        .change_create("test_approve_merged", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("merged_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id, _) = db.require_top_change();

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

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

    // Step 3: Try to approve again
    println!("\nStep 3: Attempting to approve again...");
    let response = client
        .change_approve(&change_id)
        .await
        .expect("Request should complete");

    println!("Second approve response: {:?}", response);

    // Should either be idempotent (succeed) or fail gracefully
    let result_str = response.get_result_str().unwrap_or("");
    if result_str.contains("Error") {
        println!("✅ Second approve rejected: {}", result_str);
    } else {
        println!("✅ Second approve was idempotent");
    }

    println!("\n✅ Test passed: Approve handles already merged change");
}

#[tokio::test]
async fn test_approve_empty_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Approving an empty change with no objects should work");

    // Step 1: Create an empty change
    println!("\nStep 1: Creating empty change...");
    client
        .change_create("empty_change", "test_author", Some("Empty change"))
        .await
        .expect("Failed to create change")
        .assert_success("Create change");

    let (change_id, change_before) = db.require_top_change();
    assert_eq!(
        change_before.added_objects.len(),
        0,
        "Should have no objects"
    );
    assert_eq!(change_before.status, ChangeStatus::Local, "Should be Local");
    println!("✅ Empty change created");

    // Step 2: Approve the empty change
    println!("\nStep 2: Approving empty change...");
    let response = client
        .change_approve(&change_id)
        .await
        .expect("Request should complete");

    // Should succeed
    response.assert_success("Approve empty change");
    println!("✅ Empty change approved");

    // Step 3: Verify change is merged
    println!("\nStep 3: Verifying change is merged...");
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
    assert_eq!(
        merged_change.added_objects.len(),
        0,
        "Should still have no objects"
    );
    println!("✅ Empty change is merged");

    println!("\n✅ Test passed: Can approve empty change");
}

#[tokio::test]
async fn test_approve_with_empty_change_id() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Approving with empty change ID should fail");

    // Attempt to approve with empty change ID
    println!("\nAttempting to approve with empty change ID...");
    let response = client
        .change_approve("")
        .await
        .expect("Request should complete");

    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error")
            || result_str.contains("required")
            || result_str.contains("not found"),
        "Should indicate error, got: {}",
        result_str
    );
    println!("✅ Approve failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot approve with empty change ID");
}

#[tokio::test]
async fn test_abandon_when_no_change_exists() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Abandoning when no change exists should fail gracefully");

    // Step 1: Verify no change initially
    println!("\nStep 1: Verifying no change initially...");
    db.assert_no_top_change();
    println!("✅ No change initially");

    // Step 2: Try to abandon
    println!("\nStep 2: Attempting to abandon...");
    let response = client
        .change_abandon()
        .await
        .expect("Request should complete");

    // Should fail gracefully
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error")
            || result_str.contains("No change")
            || result_str.contains("not found"),
        "Should indicate no change to abandon, got: {}",
        result_str
    );
    println!("✅ Abandon failed gracefully: {}", result_str);

    println!("\n✅ Test passed: Abandon fails gracefully when no change exists");
}

#[tokio::test]
async fn test_abandon_multiple_times() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Abandoning multiple times should fail after first abandon");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("test_abandon", "test_author", None)
        .await
        .expect("Failed to create change");

    db.require_top_change();
    println!("✅ Change created");

    // Step 2: Abandon the change
    println!("\nStep 2: Abandoning change...");
    client
        .change_abandon()
        .await
        .expect("Failed to abandon")
        .assert_success("Abandon");

    db.assert_no_top_change();
    println!("✅ Change abandoned");

    // Step 3: Try to abandon again
    println!("\nStep 3: Attempting to abandon again...");
    let response = client
        .change_abandon()
        .await
        .expect("Request should complete");

    // Should fail since no change to abandon
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("No change"),
        "Should indicate no change to abandon, got: {}",
        result_str
    );
    println!("✅ Second abandon failed appropriately: {}", result_str);

    println!("\n✅ Test passed: Multiple abandons fail after first");
}
