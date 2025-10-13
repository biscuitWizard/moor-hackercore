//! Tests for change submit operations

use crate::common::*;
use moor_vcs_worker::providers::workspace::WorkspaceProvider;
use moor_vcs_worker::types::ChangeStatus;

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

