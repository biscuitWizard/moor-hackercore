//! Integration tests for index operations (list, calc_delta, update)
//!
//! These tests verify:
//! 1. index/list returns merged changes in reverse chronological order (newest first)
//! 2. index/list only shows merged changes (filters out local, review, idle)
//! 3. Approved changes remain visible in index/list

use crate::common::*;

#[tokio::test]
async fn test_index_list_shows_approved_changes() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: index/list should show merged changes in reverse chronological order (newest first)");

    // Step 1: Verify empty change list initially
    println!("\nStep 1: Verifying empty change list initially...");
    let initial_list = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    println!("Initial list: {:?}", initial_list);
    // Should be empty list initially

    // Step 2: Create and approve first change
    println!("\nStep 2: Creating and approving first change...");
    client
        .change_create("first_change", "test_author", Some("First test change"))
        .await
        .expect("Failed to create first change");

    client
        .object_update_from_file("first_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (first_change_id, _) = db.require_top_change();

    client
        .change_approve(&first_change_id)
        .await
        .expect("Failed to approve first change")
        .assert_success("Approve first change");

    println!("✅ First change approved");

    // Step 3: Verify first change appears in index/list
    println!("\nStep 3: Verifying first change appears in index/list...");
    let list_after_first = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    println!("List after first approval: {:?}", list_after_first);

    // Parse the list and check for the first change
    let changes = parse_change_list(&list_after_first);
    assert_eq!(changes.len(), 1, "Should have 1 merged change in the list");
    assert_eq!(
        changes[0].change_id, first_change_id,
        "First change should be in the list"
    );
    assert_eq!(
        changes[0].status, "merged",
        "First change should have merged status"
    );
    assert_eq!(
        changes[0].name, "first_change",
        "First change should have correct name"
    );

    println!("✅ First change appears in index/list with merged status");

    // Step 4: Create and approve second change
    println!("\nStep 4: Creating and approving second change...");
    client
        .change_create("second_change", "test_author", Some("Second test change"))
        .await
        .expect("Failed to create second change");

    client
        .object_update_from_file("second_object", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");

    let (second_change_id, _) = db.require_top_change();

    client
        .change_approve(&second_change_id)
        .await
        .expect("Failed to approve second change")
        .assert_success("Approve second change");

    println!("✅ Second change approved");

    // Step 5: Verify both changes appear in index/list
    println!("\nStep 5: Verifying both changes appear in index/list...");
    let list_after_second = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    println!("List after second approval: {:?}", list_after_second);

    // Parse the list and check for both changes
    let changes = parse_change_list(&list_after_second);
    assert_eq!(changes.len(), 2, "Should have 2 merged changes in the list");

    // Changes should be in reverse chronological order (newest first)
    assert_eq!(
        changes[0].change_id, second_change_id,
        "Second change should be first in the list (newest first)"
    );
    assert_eq!(
        changes[0].status, "merged",
        "Second change should have merged status"
    );

    assert_eq!(
        changes[1].change_id, first_change_id,
        "First change should be second in the list (oldest last)"
    );
    assert_eq!(
        changes[1].status, "merged",
        "First change should have merged status"
    );

    println!("✅ Both changes appear in index/list in reverse chronological order");

    // Step 6: Create a third change but DON'T approve it
    println!("\nStep 6: Creating third change without approving...");
    client
        .change_create("third_change", "test_author", Some("Third test change"))
        .await
        .expect("Failed to create third change");

    client
        .object_update_from_file("third_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (_third_change_id, _) = db.require_top_change();

    println!("✅ Third change created (not approved)");

    // Step 7: Verify only merged changes appear in index/list (local changes are filtered out)
    println!("\nStep 7: Verifying only merged changes appear in index/list (local changes filtered out)...");
    let list_with_local = client
        .index_list(None, None)
        .await
        .expect("Failed to get index list");

    println!("List with local change: {:?}", list_with_local);

    // Parse the list and check for only merged changes
    let changes = parse_change_list(&list_with_local);
    assert_eq!(
        changes.len(),
        2,
        "Should have 2 merged changes in the list (local changes are filtered out)"
    );

    // Changes should be in reverse chronological order (newest first)
    assert_eq!(
        changes[0].change_id, second_change_id,
        "Second change should be first (newest first)"
    );
    assert_eq!(changes[0].status, "merged", "Second change should be merged");

    assert_eq!(
        changes[1].change_id, first_change_id,
        "First change should be second (oldest last)"
    );
    assert_eq!(
        changes[1].status, "merged",
        "First change should be merged"
    );

    println!("✅ Only merged changes appear in index/list (local changes filtered out)");

    println!(
        "\n✅ Test passed: index/list shows only merged changes in reverse chronological order (newest first)"
    );
}

// Helper struct to parse change information
#[derive(Debug)]
#[allow(dead_code)]
struct ChangeInfo {
    change_id: String,
    name: String,
    status: String,
    author: String,
    timestamp: i64,
}

// Helper function to parse the change list response
fn parse_change_list(response: &serde_json::Value) -> Vec<ChangeInfo> {
    let mut changes = Vec::new();

    // The response has a "result" field that contains the list
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
                        author: map
                            .get("author")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        timestamp: map.get("timestamp").and_then(|v| v.as_i64()).unwrap_or(0),
                    };
                    changes.push(change);
                }
            }
        }
    }

    changes
}
