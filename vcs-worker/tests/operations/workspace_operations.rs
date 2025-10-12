//! Integration tests for workspace operations
//!
//! These tests verify:
//! 1. workspace/list - Listing workspace changes with optional status filtering
//! 2. workspace/submit - Submitting changes to the workspace for review

use crate::common::*;
use moor_vcs_worker::providers::workspace::WorkspaceProvider;
use moor_vcs_worker::types::{Change, ChangeStatus, ObjectInfo};

#[tokio::test]
async fn test_workspace_list_empty() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: List workspace changes when empty");

    // Step 1: List workspace changes (should be empty)
    println!("\nStep 1: Listing workspace changes...");
    let response = client
        .workspace_list(None)
        .await
        .expect("Failed to list workspace changes");

    response.assert_success("List workspace");
    let result = response.require_result_list("List result");
    assert_eq!(result.len(), 0, "Should return empty list");
    println!("✅ Empty workspace list returned");

    // Verify via database as well
    let all_changes = server
        .database()
        .workspace()
        .list_all_workspace_changes()
        .expect("Failed to list changes directly");
    assert_eq!(all_changes.len(), 0, "Should have 0 workspace changes");

    println!("\n✅ Test passed: Workspace list is empty");
}

#[tokio::test]
async fn test_workspace_submit_and_list() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Submit a change to workspace and list it");

    // Step 1: Create a change object to submit
    println!("\nStep 1: Creating a change to submit...");
    let change = Change {
        id: uuid::Uuid::new_v4().to_string(),
        name: "test_workspace_change".to_string(),
        description: Some("A test change for workspace".to_string()),
        author: "test_author".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: ChangeStatus::Review,
        added_objects: vec![ObjectInfo {
            object_type: moor_vcs_worker::types::VcsObjectType::MooObject,
            name: "test_object".to_string(),
            version: 1,
        }],
        modified_objects: vec![],
        deleted_objects: vec![],
        renamed_objects: vec![],
        index_change_id: None,
    };

    let change_json = serde_json::to_string(&change).expect("Failed to serialize change");
    println!("✅ Created change: {} ({})", change.name, change.id);

    // Step 2: Submit the change to workspace
    println!("\nStep 2: Submitting change to workspace...");
    let submit_response = client
        .workspace_submit(&change_json)
        .await
        .expect("Failed to submit change");

    submit_response.assert_success("Submit to workspace");
    let result = submit_response.require_result_str("Submit result");
    assert!(
        result.contains("successfully submitted"),
        "Should confirm submission"
    );
    println!("✅ Submit response: {}", result);

    // Step 3: Verify change is stored in workspace
    println!("\nStep 3: Verifying change in workspace...");
    let stored_change = server
        .database()
        .workspace()
        .get_workspace_change(&change.id)
        .expect("Failed to get workspace change")
        .expect("Change should exist in workspace");

    assert_eq!(stored_change.id, change.id, "Change ID should match");
    assert_eq!(stored_change.name, change.name, "Change name should match");
    assert_eq!(
        stored_change.status,
        ChangeStatus::Review,
        "Status should be Review"
    );
    println!("✅ Change stored in workspace");

    // Step 4: List workspace changes
    println!("\nStep 4: Listing workspace changes...");
    let list_response = client
        .workspace_list(None)
        .await
        .expect("Failed to list workspace changes");

    list_response.assert_success("List workspace");
    let list_result = list_response.require_result_list("List result");

    assert_eq!(list_result.len(), 1, "Should return 1 change");
    let change_map = list_result[0].as_object().expect("Change should be a map");

    assert_eq!(
        change_map.get("id").and_then(|v| v.as_str()),
        Some(change.id.as_str()),
        "Should contain change ID"
    );
    assert_eq!(
        change_map.get("name").and_then(|v| v.as_str()),
        Some(change.name.as_str()),
        "Should contain change name"
    );
    assert_eq!(
        change_map.get("author").and_then(|v| v.as_str()),
        Some("test_author"),
        "Should contain author"
    );
    assert_eq!(
        change_map.get("status").and_then(|v| v.as_str()),
        Some("Review"),
        "Should show Review status"
    );
    println!("✅ List shows submitted change");

    println!("\n✅ Test passed: Submit and list workspace changes");
}

#[tokio::test]
async fn test_workspace_list_filter_by_status() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Filter workspace changes by status");

    // Step 1: Submit multiple changes with different statuses
    println!("\nStep 1: Submitting changes with different statuses...");

    // Create Review status change
    let review_change = Change {
        id: uuid::Uuid::new_v4().to_string(),
        name: "review_change".to_string(),
        description: Some("Change awaiting review".to_string()),
        author: "reviewer".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: ChangeStatus::Review,
        added_objects: vec![],
        modified_objects: vec![],
        deleted_objects: vec![],
        renamed_objects: vec![],
        index_change_id: None,
    };

    // Create Idle status change
    let idle_change = Change {
        id: uuid::Uuid::new_v4().to_string(),
        name: "idle_change".to_string(),
        description: Some("Idle change".to_string()),
        author: "idler".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: ChangeStatus::Idle,
        added_objects: vec![],
        modified_objects: vec![],
        deleted_objects: vec![],
        renamed_objects: vec![],
        index_change_id: None,
    };

    // Submit both changes
    client
        .workspace_submit(&serde_json::to_string(&review_change).unwrap())
        .await
        .expect("Failed to submit review change");

    client
        .workspace_submit(&serde_json::to_string(&idle_change).unwrap())
        .await
        .expect("Failed to submit idle change");

    println!("✅ Submitted 2 changes: 1 Review, 1 Idle");

    // Step 2: List all changes (no filter)
    println!("\nStep 2: Listing all workspace changes...");
    let all_response = client
        .workspace_list(None)
        .await
        .expect("Failed to list all changes");

    let all_result = all_response.require_result_list("List all");
    assert_eq!(all_result.len(), 2, "Should return 2 changes");

    let names: Vec<&str> = all_result
        .iter()
        .filter_map(|v| {
            v.as_object()
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
        })
        .collect();
    assert!(
        names.contains(&review_change.name.as_str()),
        "Should contain review change"
    );
    assert!(
        names.contains(&idle_change.name.as_str()),
        "Should contain idle change"
    );
    println!("✅ All changes listed (2 total)");

    // Step 3: Filter by Review status
    println!("\nStep 3: Filtering by Review status...");
    let review_response = client
        .workspace_list(Some("review"))
        .await
        .expect("Failed to list review changes");

    let review_result = review_response.require_result_list("List review");
    assert_eq!(review_result.len(), 1, "Should return 1 review change");

    let review_map = review_result[0]
        .as_object()
        .expect("Change should be a map");
    assert_eq!(
        review_map.get("name").and_then(|v| v.as_str()),
        Some(review_change.name.as_str()),
        "Should contain review change"
    );
    assert_eq!(
        review_map.get("status").and_then(|v| v.as_str()),
        Some("Review"),
        "Should have Review status"
    );
    println!("✅ Review filter works correctly");

    // Step 4: Filter by Idle status
    println!("\nStep 4: Filtering by Idle status...");
    let idle_response = client
        .workspace_list(Some("idle"))
        .await
        .expect("Failed to list idle changes");

    let idle_result = idle_response.require_result_list("List idle");
    assert_eq!(idle_result.len(), 1, "Should return 1 idle change");

    let idle_map = idle_result[0].as_object().expect("Change should be a map");
    assert_eq!(
        idle_map.get("name").and_then(|v| v.as_str()),
        Some(idle_change.name.as_str()),
        "Should contain idle change"
    );
    assert_eq!(
        idle_map.get("status").and_then(|v| v.as_str()),
        Some("Idle"),
        "Should have Idle status"
    );
    println!("✅ Idle filter works correctly");

    println!("\n✅ Test passed: Status filtering works correctly");
}

#[tokio::test]
async fn test_workspace_list_invalid_status_filter() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Invalid status filter should return error");

    println!("\nTesting invalid status filter...");
    let response = client
        .workspace_list(Some("invalid_status"))
        .await
        .expect("Request should complete");

    // Check if it's an error response
    let success = response.is_success();
    let result = response.get_result_str().unwrap_or("");

    assert!(
        !success || result.contains("Invalid status filter") || result.contains("Error"),
        "Should indicate invalid status, got: {}",
        result
    );
    println!("✅ Invalid status filter rejected: {}", result);

    println!("\n✅ Test passed: Invalid status filter returns error");
}

#[tokio::test]
async fn test_workspace_submit_invalid_json() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Submit with invalid JSON should fail");

    println!("\nSubmitting invalid JSON...");
    let response = client
        .workspace_submit("this is not valid json")
        .await
        .expect("Request should complete");

    let success = response.is_success();
    let result = response.get_result_str().unwrap_or("");

    assert!(
        !success || result.contains("Error") || result.contains("Failed to deserialize"),
        "Should indicate deserialization error"
    );
    println!("✅ Invalid JSON rejected: {}", result);

    println!("\n✅ Test passed: Invalid JSON is rejected");
}

#[tokio::test]
async fn test_workspace_submit_missing_args() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Submit without arguments should fail");

    println!("\nSubmitting without arguments...");
    let response = client
        .rpc_call("workspace/submit", vec![])
        .await
        .expect("Request should complete");

    let success = response.is_success();
    let result = response.get_result_str().unwrap_or("");

    assert!(
        !success || result.contains("Error") || result.contains("requires"),
        "Should indicate missing argument"
    );
    println!("✅ Missing args rejected: {}", result);

    println!("\n✅ Test passed: Missing arguments are rejected");
}

#[tokio::test]
async fn test_workspace_submit_multiple_changes() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Submit multiple changes and verify all are listed");

    // Submit 3 changes
    println!("\nSubmitting 3 changes...");
    let mut change_ids = Vec::new();

    for i in 1..=3 {
        let change = Change {
            id: uuid::Uuid::new_v4().to_string(),
            name: format!("multi_change_{}", i),
            description: Some(format!("Change number {}", i)),
            author: "batch_author".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: ChangeStatus::Review,
            added_objects: vec![],
            modified_objects: vec![],
            deleted_objects: vec![],
            renamed_objects: vec![],
            index_change_id: None,
        };

        change_ids.push(change.id.clone());

        let change_json = serde_json::to_string(&change).unwrap();
        client
            .workspace_submit(&change_json)
            .await
            .expect(&format!("Failed to submit change {}", i));
    }

    println!("✅ Submitted 3 changes");

    // List all changes
    println!("\nListing all workspace changes...");
    let response = client
        .workspace_list(None)
        .await
        .expect("Failed to list changes");

    let result = response.require_result_list("List result");
    assert_eq!(result.len(), 3, "Should return 3 changes");

    // Verify all 3 changes are present
    let change_maps: Vec<_> = result.iter().filter_map(|v| v.as_object()).collect();

    for (i, change_id) in change_ids.iter().enumerate() {
        let found = change_maps
            .iter()
            .any(|m| m.get("id").and_then(|v| v.as_str()) == Some(change_id.as_str()));
        assert!(found, "Should contain change {} ID", i + 1);

        let name = format!("multi_change_{}", i + 1);
        let found_name = change_maps
            .iter()
            .any(|m| m.get("name").and_then(|v| v.as_str()) == Some(name.as_str()));
        assert!(found_name, "Should contain change {} name", i + 1);
    }

    println!("✅ All 3 changes listed correctly");

    // Verify via database
    let all_changes = server
        .database()
        .workspace()
        .list_all_workspace_changes()
        .expect("Failed to list changes directly");
    assert_eq!(all_changes.len(), 3, "Database should have 3 changes");

    println!("\n✅ Test passed: Multiple changes submitted and listed");
}

#[tokio::test]
async fn test_workspace_change_with_objects() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Submit change with objects and verify details in list");

    println!("\nCreating change with multiple object operations...");
    let change = Change {
        id: uuid::Uuid::new_v4().to_string(),
        name: "complex_change".to_string(),
        description: Some("A change with multiple object operations".to_string()),
        author: "complex_author".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: ChangeStatus::Review,
        added_objects: vec![
            ObjectInfo {
                object_type: moor_vcs_worker::types::VcsObjectType::MooObject,
                name: "new_obj_1".to_string(),
                version: 1,
            },
            ObjectInfo {
                object_type: moor_vcs_worker::types::VcsObjectType::MooObject,
                name: "new_obj_2".to_string(),
                version: 1,
            },
        ],
        modified_objects: vec![ObjectInfo {
            object_type: moor_vcs_worker::types::VcsObjectType::MooObject,
            name: "mod_obj_1".to_string(),
            version: 2,
        }],
        deleted_objects: vec![ObjectInfo {
            object_type: moor_vcs_worker::types::VcsObjectType::MooObject,
            name: "del_obj_1".to_string(),
            version: 1,
        }],
        renamed_objects: vec![],
        index_change_id: Some("base_change_123".to_string()),
    };

    // Submit the change
    let change_json = serde_json::to_string(&change).unwrap();
    client
        .workspace_submit(&change_json)
        .await
        .expect("Failed to submit change");

    println!("✅ Submitted complex change");

    // List and verify details
    println!("\nListing workspace changes...");
    let response = client
        .workspace_list(None)
        .await
        .expect("Failed to list changes");

    let result = response.require_result_list("List result");
    assert_eq!(result.len(), 1, "Should return 1 change");

    let change_map = result[0].as_object().expect("Change should be a map");

    // Verify change details
    assert_eq!(
        change_map.get("name").and_then(|v| v.as_str()),
        Some("complex_change"),
        "Should contain change name"
    );
    assert_eq!(
        change_map.get("author").and_then(|v| v.as_str()),
        Some("complex_author"),
        "Should contain author"
    );
    assert_eq!(
        change_map.get("description").and_then(|v| v.as_str()),
        Some("A change with multiple object operations"),
        "Should contain description"
    );
    assert_eq!(
        change_map.get("based_on").and_then(|v| v.as_str()),
        Some("base_change_123"),
        "Should show base change ID"
    );

    // Verify changes map
    let changes_map = change_map
        .get("changes")
        .and_then(|v| v.as_object())
        .expect("Should have changes map");

    let added = changes_map
        .get("objects_added")
        .and_then(|v| v.as_array())
        .expect("Should have objects_added list");
    assert_eq!(added.len(), 2, "Should show 2 added objects");

    let modified = changes_map
        .get("objects_modified")
        .and_then(|v| v.as_array())
        .expect("Should have objects_modified list");
    assert_eq!(modified.len(), 1, "Should show 1 modified object");

    let deleted = changes_map
        .get("objects_deleted")
        .and_then(|v| v.as_array())
        .expect("Should have objects_deleted list");
    assert_eq!(deleted.len(), 1, "Should show 1 deleted object");

    println!("✅ All object details displayed correctly");

    println!("\n✅ Test passed: Change with objects listed correctly");
}

#[tokio::test]
async fn test_workspace_list_structure() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Verify workspace list data structure");

    // Submit changes with different statuses
    println!("\nSubmitting changes...");

    let review_change = Change {
        id: uuid::Uuid::new_v4().to_string(),
        name: "format_review".to_string(),
        description: None,
        author: "formatter".to_string(),
        timestamp: 1234567890,
        status: ChangeStatus::Review,
        added_objects: vec![],
        modified_objects: vec![],
        deleted_objects: vec![],
        renamed_objects: vec![],
        index_change_id: None,
    };

    let idle_change = Change {
        id: uuid::Uuid::new_v4().to_string(),
        name: "format_idle".to_string(),
        description: Some("Has description".to_string()),
        author: "formatter".to_string(),
        timestamp: 1234567890,
        status: ChangeStatus::Idle,
        added_objects: vec![],
        modified_objects: vec![],
        deleted_objects: vec![],
        renamed_objects: vec![],
        index_change_id: None,
    };

    for change in [review_change.clone(), idle_change.clone()] {
        let change_json = serde_json::to_string(&change).unwrap();
        client
            .workspace_submit(&change_json)
            .await
            .expect("Failed to submit change");
    }

    println!("✅ Submitted changes");

    // List and verify structure
    println!("\nListing and checking structure...");
    let response = client
        .workspace_list(None)
        .await
        .expect("Failed to list changes");

    let result = response.require_result_list("List result");
    assert_eq!(result.len(), 2, "Should return 2 changes");

    // Check structure of each change
    for (i, change_var) in result.iter().enumerate() {
        let change_map = change_var
            .as_object()
            .expect(&format!("Change {} should be a map", i));

        // Verify required fields exist
        assert!(change_map.contains_key("id"), "Change {} should have id", i);
        assert!(
            change_map.contains_key("short_id"),
            "Change {} should have short_id",
            i
        );
        assert!(
            change_map.contains_key("name"),
            "Change {} should have name",
            i
        );
        assert!(
            change_map.contains_key("author"),
            "Change {} should have author",
            i
        );
        assert!(
            change_map.contains_key("timestamp"),
            "Change {} should have timestamp",
            i
        );
        assert!(
            change_map.contains_key("status"),
            "Change {} should have status",
            i
        );
        assert!(
            change_map.contains_key("changes"),
            "Change {} should have changes map",
            i
        );

        // Verify changes map structure
        let changes_map = change_map
            .get("changes")
            .and_then(|v| v.as_object())
            .expect(&format!("Change {} should have changes map", i));

        assert!(
            changes_map.contains_key("objects_added"),
            "Changes should have objects_added"
        );
        assert!(
            changes_map.contains_key("objects_modified"),
            "Changes should have objects_modified"
        );
        assert!(
            changes_map.contains_key("objects_deleted"),
            "Changes should have objects_deleted"
        );
        assert!(
            changes_map.contains_key("objects_renamed"),
            "Changes should have objects_renamed"
        );

        println!("✅ Change {} has correct structure", i);
    }

    // Verify one has description and one doesn't
    let with_desc = result
        .iter()
        .filter_map(|v| v.as_object())
        .filter(|m| m.contains_key("description"))
        .count();
    assert_eq!(with_desc, 1, "Exactly one change should have description");

    println!("✅ Data structure verified");

    println!("\n✅ Test passed: List structure is correct");
}

#[tokio::test]
async fn test_workspace_operations_persistence() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Verify workspace changes persist across operations");

    // Submit a change
    println!("\nSubmitting change...");
    let change = Change {
        id: uuid::Uuid::new_v4().to_string(),
        name: "persist_test".to_string(),
        description: Some("Testing persistence".to_string()),
        author: "persister".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: ChangeStatus::Review,
        added_objects: vec![],
        modified_objects: vec![],
        deleted_objects: vec![],
        renamed_objects: vec![],
        index_change_id: None,
    };

    let change_id = change.id.clone();
    let change_json = serde_json::to_string(&change).unwrap();

    client
        .workspace_submit(&change_json)
        .await
        .expect("Failed to submit change");

    println!("✅ Change submitted");

    // Verify it's in the database
    println!("\nVerifying database persistence...");
    let stored = server
        .database()
        .workspace()
        .get_workspace_change(&change_id)
        .expect("Failed to query workspace")
        .expect("Change should exist");

    assert_eq!(stored.id, change_id, "ID should match");
    assert_eq!(stored.name, "persist_test", "Name should match");
    assert_eq!(stored.status, ChangeStatus::Review, "Status should match");
    println!("✅ Change persisted in database");

    // List via API multiple times
    println!("\nListing multiple times...");
    for i in 1..=3 {
        let response = client
            .workspace_list(None)
            .await
            .expect(&format!("Failed to list changes (iteration {})", i));

        let result = response.require_result_list("List result");
        assert_eq!(result.len(), 1, "Should return 1 change (iteration {})", i);

        let change_map = result[0].as_object().expect("Change should be a map");
        assert_eq!(
            change_map.get("id").and_then(|v| v.as_str()),
            Some(change_id.as_str()),
            "Should contain change ID (iteration {})",
            i
        );
        println!("✅ List iteration {} successful", i);
    }

    println!("\n✅ Test passed: Workspace changes persist correctly");
}
