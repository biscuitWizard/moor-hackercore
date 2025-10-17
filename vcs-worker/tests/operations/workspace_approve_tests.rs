//! Integration tests for approving changes from workspace back into index
//!
//! These tests verify:
//! 1. Changes submitted to remote (moved to workspace with Review status) can be approved
//! 2. Approved workspace changes are added back to the index with Merged status
//! 3. Approved workspace changes appear in index/list

use crate::common::*;
use moor_vcs_worker::providers::index::IndexProvider;
use moor_vcs_worker::providers::workspace::WorkspaceProvider;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_approve_change_from_workspace_adds_to_index() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!(
        "Test: Approving a change from workspace should add it back to the index with Merged status"
    );

    // Step 1: Set a source URL to enable remote submission workflow
    println!("\nStep 1: Setting source URL to enable remote workflow...");
    let source_url = "http://example.com/vcs";
    server
        .database()
        .index()
        .set_source(source_url)
        .expect("Failed to set source URL");
    println!("✅ Source URL configured: {}", source_url);

    // Step 2: Create a change with objects
    println!("\nStep 2: Creating change with objects...");
    client
        .change_create(
            "test_workspace_approve",
            "test_author",
            Some("Test workspace approval"),
        )
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("workspace_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id, _) = db.require_top_change();
    println!("✅ Change created: {}", change_id);

    // Step 3: Verify change is initially NOT in index/list (local changes are filtered out)
    println!("\nStep 3: Verifying local change does NOT appear in index/list...");
    let initial_list = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    let initial_changes = parse_change_list(&initial_list);
    assert_eq!(
        initial_changes.len(),
        0,
        "Should have no changes in index (local changes are filtered out)"
    );
    println!("✅ Local change correctly filtered out from index/list");

    // Step 4: Submit the change (moves to workspace with Review status, removes from index)
    println!("\nStep 4: Submitting change (moves to workspace)...");
    client
        .change_submit()
        .await
        .expect("Failed to submit change");

    println!("✅ Change submitted to workspace");

    // Step 5: Verify change is now in workspace with Review status
    println!("\nStep 5: Verifying change is in workspace...");
    let workspace_change = server
        .database()
        .workspace()
        .get_workspace_change(&change_id)
        .expect("Failed to check workspace")
        .expect("Change should be in workspace");

    assert_eq!(
        workspace_change.status,
        ChangeStatus::Review,
        "Should have Review status in workspace"
    );
    println!("✅ Change in workspace with Review status");

    // Step 6: Verify change is still NOT in index/list (local changes filtered out)
    println!("\nStep 6: Verifying change is still not in index/list (local changes filtered out)...");
    let list_after_submit = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    let changes_after_submit = parse_change_list(&list_after_submit);
    assert_eq!(
        changes_after_submit.len(),
        0,
        "Change should still not be in index (local changes filtered out)"
    );
    println!("✅ Change still not in index/list (local changes filtered out)");

    // Step 7: Approve the change from workspace
    println!("\nStep 7: Approving change from workspace...");
    let approve_response = client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve change");

    println!("Approve response: {:?}", approve_response);
    approve_response.assert_success("Approve workspace change");

    println!("✅ Change approved from workspace");

    // Step 8: Verify change is now back in index with Merged status
    println!("\nStep 8: Verifying change is back in index with Merged status...");
    let list_after_approve = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    let changes_after_approve = parse_change_list(&list_after_approve);
    assert_eq!(
        changes_after_approve.len(),
        1,
        "Should have 1 merged change in index"
    );
    assert_eq!(
        changes_after_approve[0].change_id, change_id,
        "Approved change should be in index"
    );
    assert_eq!(
        changes_after_approve[0].status, "merged",
        "Approved change should have merged status"
    );
    println!("✅ Change back in index with Merged status");

    // Step 9: Verify change is removed from workspace
    println!("\nStep 9: Verifying change is removed from workspace...");
    let workspace_after_approve = server
        .database()
        .workspace()
        .get_workspace_change(&change_id)
        .expect("Failed to check workspace");

    assert!(
        workspace_after_approve.is_none(),
        "Change should be removed from workspace after approval"
    );
    println!("✅ Change removed from workspace");

    // Step 10: Verify change appears in index/list
    println!("\nStep 10: Verifying change appears in index/list...");

    // Give a moment for any async operations to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // First check the raw change_order in the database
    let change_order = server
        .database()
        .index()
        .get_change_order()
        .expect("Failed to get change_order");
    println!("Change order in database: {:?}", change_order);

    let list_after_approve = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    println!("Index list response: {:?}", list_after_approve);

    let changes_after_approve = parse_change_list(&list_after_approve);
    println!(
        "Parsed {} changes from index/list",
        changes_after_approve.len()
    );

    assert_eq!(
        changes_after_approve.len(),
        1,
        "Should have 1 merged change in index"
    );
    assert_eq!(
        changes_after_approve[0].change_id, change_id,
        "Should be the approved change"
    );
    assert_eq!(
        changes_after_approve[0].status, "merged",
        "Should have merged status"
    );

    println!("✅ Approved change appears in index/list with merged status");

    println!("\n✅ Test passed: Workspace change approval adds change back to index");
}

#[tokio::test]
async fn test_multiple_workspace_approvals_maintain_order() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!(
        "Test: Multiple workspace approvals should maintain chronological order in index/list"
    );

    // Step 1: Set source URL
    println!("\nStep 1: Setting source URL...");
    server
        .database()
        .index()
        .set_source("http://example.com/vcs")
        .expect("Failed to set source URL");

    // Step 2: Create and submit first change
    println!("\nStep 2: Creating and submitting first change...");
    client
        .change_create("first_workspace_change", "test_author", None)
        .await
        .expect("Failed to create first change");

    client
        .object_update_from_file("first_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (first_change_id, _) = db.require_top_change();

    client
        .change_submit()
        .await
        .expect("Failed to submit first change");

    println!("✅ First change submitted to workspace");

    // Step 3: Create and submit second change
    println!("\nStep 3: Creating and submitting second change...");
    client
        .change_create("second_workspace_change", "test_author", None)
        .await
        .expect("Failed to create second change");

    client
        .object_update_from_file("second_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");

    let (second_change_id, _) = db.require_top_change();

    client
        .change_submit()
        .await
        .expect("Failed to submit second change");

    println!("✅ Second change submitted to workspace");

    // Step 4: Verify both are in workspace, none in index
    println!("\nStep 4: Verifying both in workspace, none in index...");
    let list_before_approval = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    let changes_before = parse_change_list(&list_before_approval);
    assert_eq!(changes_before.len(), 0, "Should have no changes in index");
    println!("✅ No changes in index");

    // Step 5: Approve first change
    println!("\nStep 5: Approving first change...");
    client
        .change_approve(&first_change_id)
        .await
        .expect("Failed to approve first change")
        .assert_success("Approve first change");

    println!("✅ First change approved");

    // Step 6: Verify first change in index
    println!("\nStep 6: Verifying first change in index...");
    let list_after_first = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    let changes_after_first = parse_change_list(&list_after_first);
    assert_eq!(
        changes_after_first.len(),
        1,
        "Should have 1 change in index"
    );
    assert_eq!(
        changes_after_first[0].change_id, first_change_id,
        "Should be first change"
    );
    println!("✅ First change appears in index");

    // Step 7: Approve second change
    println!("\nStep 7: Approving second change...");
    client
        .change_approve(&second_change_id)
        .await
        .expect("Failed to approve second change")
        .assert_success("Approve second change");

    println!("✅ Second change approved");

    // Step 8: Verify both changes in index in reverse chronological order (newest first)
    println!("\nStep 8: Verifying both changes in reverse chronological order (newest first)...");
    let list_after_second = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    let changes_after_second = parse_change_list(&list_after_second);
    assert_eq!(
        changes_after_second.len(),
        2,
        "Should have 2 merged changes in index"
    );
    // Changes should be in reverse chronological order (newest first)
    assert_eq!(
        changes_after_second[0].change_id, second_change_id,
        "Second approved should be first (newest first)"
    );
    assert_eq!(
        changes_after_second[1].change_id, first_change_id,
        "First approved should be second (oldest last)"
    );
    assert_eq!(
        changes_after_second[0].status, "merged",
        "Second should be merged"
    );
    assert_eq!(
        changes_after_second[1].status, "merged",
        "First should be merged"
    );

    println!("✅ Both changes appear in reverse chronological order (newest first)");

    println!("\n✅ Test passed: Multiple workspace approvals maintain order");
}

// Helper function to parse the change list response
fn parse_change_list(response: &serde_json::Value) -> Vec<ChangeInfo> {
    let mut changes = Vec::new();

    if let Some(result) = response.get("result") {
        if let Some(list) = result.as_array() {
            for item in list {
                if let Some(map) = item.as_object() {
                    let change = ChangeInfo {
                        change_id: map
                            .get("change_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        name: map
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        status: map
                            .get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    };
                    changes.push(change);
                }
            }
        }
    }

    changes
}

#[derive(Debug)]
#[allow(dead_code)]
struct ChangeInfo {
    change_id: String,
    name: String,
    status: String,
}
