//! Integration tests for workspace operations
//!
//! These tests verify:
//! 1. workspace/list - Listing workspace changes with optional status filtering
//! 2. workspace/submit - Submitting changes to the workspace for review

use crate::common::*;
use moor_vcs_worker::types::{Change, ChangeStatus, ObjectInfo};
use moor_vcs_worker::providers::workspace::WorkspaceProvider;

#[tokio::test]
async fn test_workspace_list_empty() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: List workspace changes when empty");
    
    // Step 1: List workspace changes (should be empty)
    println!("\nStep 1: Listing workspace changes...");
    let response = client.workspace_list(None)
        .await
        .expect("Failed to list workspace changes");
    
    response.assert_success("List workspace");
    let result = response.require_result_str("List result");
    assert!(result.contains("No workspace changes found"), "Should indicate no changes found");
    println!("✅ Empty workspace list: {}", result);
    
    // Verify via database as well
    let all_changes = server.database().workspace().list_all_workspace_changes()
        .expect("Failed to list changes directly");
    assert_eq!(all_changes.len(), 0, "Should have 0 workspace changes");
    
    println!("\n✅ Test passed: Workspace list is empty");
}

#[tokio::test]
async fn test_workspace_submit_and_list() {
    let server = TestServer::start().await.expect("Failed to start test server");
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
    let submit_response = client.workspace_submit(&change_json)
        .await
        .expect("Failed to submit change");
    
    submit_response.assert_success("Submit to workspace");
    let result = submit_response.require_result_str("Submit result");
    assert!(result.contains("successfully submitted"), "Should confirm submission");
    println!("✅ Submit response: {}", result);
    
    // Step 3: Verify change is stored in workspace
    println!("\nStep 3: Verifying change in workspace...");
    let stored_change = server.database().workspace().get_workspace_change(&change.id)
        .expect("Failed to get workspace change")
        .expect("Change should exist in workspace");
    
    assert_eq!(stored_change.id, change.id, "Change ID should match");
    assert_eq!(stored_change.name, change.name, "Change name should match");
    assert_eq!(stored_change.status, ChangeStatus::Review, "Status should be Review");
    println!("✅ Change stored in workspace");
    
    // Step 4: List workspace changes
    println!("\nStep 4: Listing workspace changes...");
    let list_response = client.workspace_list(None)
        .await
        .expect("Failed to list workspace changes");
    
    list_response.assert_success("List workspace");
    let list_result = list_response.require_result_str("List result");
    
    assert!(list_result.contains(&change.id), "Should contain change ID");
    assert!(list_result.contains(&change.name), "Should contain change name");
    assert!(list_result.contains("test_author"), "Should contain author");
    assert!(list_result.contains("Review"), "Should show Review status");
    println!("✅ List shows submitted change");
    
    println!("\n✅ Test passed: Submit and list workspace changes");
}

#[tokio::test]
async fn test_workspace_list_filter_by_status() {
    let server = TestServer::start().await.expect("Failed to start test server");
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
    client.workspace_submit(&serde_json::to_string(&review_change).unwrap())
        .await
        .expect("Failed to submit review change");
    
    client.workspace_submit(&serde_json::to_string(&idle_change).unwrap())
        .await
        .expect("Failed to submit idle change");
    
    println!("✅ Submitted 2 changes: 1 Review, 1 Idle");
    
    // Step 2: List all changes (no filter)
    println!("\nStep 2: Listing all workspace changes...");
    let all_response = client.workspace_list(None)
        .await
        .expect("Failed to list all changes");
    
    let all_result = all_response.require_result_str("List all");
    assert!(all_result.contains(&review_change.name), "Should contain review change");
    assert!(all_result.contains(&idle_change.name), "Should contain idle change");
    assert!(all_result.contains("Total: 2 changes"), "Should show 2 total changes");
    println!("✅ All changes listed (2 total)");
    
    // Step 3: Filter by Review status
    println!("\nStep 3: Filtering by Review status...");
    let review_response = client.workspace_list(Some("review"))
        .await
        .expect("Failed to list review changes");
    
    let review_result = review_response.require_result_str("List review");
    assert!(review_result.contains(&review_change.name), "Should contain review change");
    assert!(!review_result.contains(&idle_change.name), "Should NOT contain idle change");
    assert!(review_result.contains("status: Review") || review_result.contains("Review"), "Should indicate Review filter");
    println!("✅ Review filter works correctly");
    
    // Step 4: Filter by Idle status
    println!("\nStep 4: Filtering by Idle status...");
    let idle_response = client.workspace_list(Some("idle"))
        .await
        .expect("Failed to list idle changes");
    
    let idle_result = idle_response.require_result_str("List idle");
    assert!(!idle_result.contains(&review_change.name), "Should NOT contain review change");
    assert!(idle_result.contains(&idle_change.name), "Should contain idle change");
    assert!(idle_result.contains("status: Idle"), "Should indicate Idle filter");
    println!("✅ Idle filter works correctly");
    
    println!("\n✅ Test passed: Status filtering works correctly");
}

#[tokio::test]
async fn test_workspace_list_invalid_status_filter() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Invalid status filter should return error");
    
    println!("\nTesting invalid status filter...");
    let response = client.workspace_list(Some("invalid_status"))
        .await
        .expect("Request should complete");
    
    // Check if it's an error response
    let success = response.is_success();
    let result = response.get_result_str().unwrap_or("");
    
    assert!(!success || result.contains("Invalid status filter") || result.contains("Error"), 
            "Should indicate invalid status, got: {}", result);
    println!("✅ Invalid status filter rejected: {}", result);
    
    println!("\n✅ Test passed: Invalid status filter returns error");
}

#[tokio::test]
async fn test_workspace_submit_invalid_json() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Submit with invalid JSON should fail");
    
    println!("\nSubmitting invalid JSON...");
    let response = client.workspace_submit("this is not valid json")
        .await
        .expect("Request should complete");
    
    let success = response.is_success();
    let result = response.get_result_str().unwrap_or("");
    
    assert!(!success || result.contains("Error") || result.contains("Failed to deserialize"), 
            "Should indicate deserialization error");
    println!("✅ Invalid JSON rejected: {}", result);
    
    println!("\n✅ Test passed: Invalid JSON is rejected");
}

#[tokio::test]
async fn test_workspace_submit_missing_args() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Submit without arguments should fail");
    
    println!("\nSubmitting without arguments...");
    let response = client.rpc_call("workspace/submit", vec![])
        .await
        .expect("Request should complete");
    
    let success = response.is_success();
    let result = response.get_result_str().unwrap_or("");
    
    assert!(!success || result.contains("Error") || result.contains("requires"), 
            "Should indicate missing argument");
    println!("✅ Missing args rejected: {}", result);
    
    println!("\n✅ Test passed: Missing arguments are rejected");
}

#[tokio::test]
async fn test_workspace_submit_multiple_changes() {
    let server = TestServer::start().await.expect("Failed to start test server");
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
        client.workspace_submit(&change_json)
            .await
            .expect(&format!("Failed to submit change {}", i));
    }
    
    println!("✅ Submitted 3 changes");
    
    // List all changes
    println!("\nListing all workspace changes...");
    let response = client.workspace_list(None)
        .await
        .expect("Failed to list changes");
    
    let result = response.require_result_str("List result");
    
    // Verify all 3 changes are present
    for (i, change_id) in change_ids.iter().enumerate() {
        assert!(result.contains(change_id), "Should contain change {} ID", i + 1);
        assert!(result.contains(&format!("multi_change_{}", i + 1)), 
                "Should contain change {} name", i + 1);
    }
    
    assert!(result.contains("Total: 3 changes"), "Should show 3 total changes");
    println!("✅ All 3 changes listed correctly");
    
    // Verify via database
    let all_changes = server.database().workspace().list_all_workspace_changes()
        .expect("Failed to list changes directly");
    assert_eq!(all_changes.len(), 3, "Database should have 3 changes");
    
    println!("\n✅ Test passed: Multiple changes submitted and listed");
}

#[tokio::test]
async fn test_workspace_change_with_objects() {
    let server = TestServer::start().await.expect("Failed to start test server");
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
            ObjectInfo { object_type: moor_vcs_worker::types::VcsObjectType::MooObject, name: "new_obj_1".to_string(), version: 1 },
            ObjectInfo { object_type: moor_vcs_worker::types::VcsObjectType::MooObject, name: "new_obj_2".to_string(), version: 1 },
        ],
        modified_objects: vec![
            ObjectInfo { object_type: moor_vcs_worker::types::VcsObjectType::MooObject, name: "mod_obj_1".to_string(), version: 2 },
        ],
        deleted_objects: vec![
            ObjectInfo { object_type: moor_vcs_worker::types::VcsObjectType::MooObject, name: "del_obj_1".to_string(), version: 1 },
        ],
        renamed_objects: vec![],
        index_change_id: Some("base_change_123".to_string()),
    };
    
    // Submit the change
    let change_json = serde_json::to_string(&change).unwrap();
    client.workspace_submit(&change_json)
        .await
        .expect("Failed to submit change");
    
    println!("✅ Submitted complex change");
    
    // List and verify details
    println!("\nListing workspace changes...");
    let response = client.workspace_list(None)
        .await
        .expect("Failed to list changes");
    
    let result = response.require_result_str("List result");
    
    // Verify change details
    assert!(result.contains("complex_change"), "Should contain change name");
    assert!(result.contains("complex_author"), "Should contain author");
    assert!(result.contains("A change with multiple object operations"), "Should contain description");
    assert!(result.contains("2 added"), "Should show 2 added objects");
    assert!(result.contains("1 modified"), "Should show 1 modified object");
    assert!(result.contains("1 deleted"), "Should show 1 deleted object");
    assert!(result.contains("4 total"), "Should show 4 total objects");
    assert!(result.contains("base_change_123"), "Should show base change ID");
    
    println!("✅ All object details displayed correctly");
    
    println!("\n✅ Test passed: Change with objects listed correctly");
}

#[tokio::test]
async fn test_workspace_list_formatting() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Verify workspace list output formatting");
    
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
    
    for change in [review_change, idle_change] {
        let change_json = serde_json::to_string(&change).unwrap();
        client.workspace_submit(&change_json)
            .await
            .expect("Failed to submit change");
    }
    
    println!("✅ Submitted changes");
    
    // List and verify formatting
    println!("\nListing and checking format...");
    let response = client.workspace_list(None)
        .await
        .expect("Failed to list changes");
    
    let result = response.require_result_str("List result");
    
    // Check for expected formatting elements
    assert!(result.contains("Workspace Changes"), "Should have header");
    assert!(result.contains("="), "Should have separator line");
    assert!(result.contains("Review Changes"), "Should have Review section");
    assert!(result.contains("Idle Changes"), "Should have Idle section");
    assert!(result.contains("Total: 2 changes"), "Should have total count");
    assert!(result.contains("ID:"), "Should have ID labels");
    assert!(result.contains("Name:"), "Should have Name labels");
    assert!(result.contains("Author:"), "Should have Author labels");
    assert!(result.contains("Created:"), "Should have timestamp");
    assert!(result.contains("Objects:"), "Should have object counts");
    
    println!("✅ Formatting elements present");
    println!("\nFormatted output:\n{}", result);
    
    println!("\n✅ Test passed: List formatting is correct");
}

#[tokio::test]
async fn test_workspace_operations_persistence() {
    let server = TestServer::start().await.expect("Failed to start test server");
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
    
    client.workspace_submit(&change_json)
        .await
        .expect("Failed to submit change");
    
    println!("✅ Change submitted");
    
    // Verify it's in the database
    println!("\nVerifying database persistence...");
    let stored = server.database().workspace().get_workspace_change(&change_id)
        .expect("Failed to query workspace")
        .expect("Change should exist");
    
    assert_eq!(stored.id, change_id, "ID should match");
    assert_eq!(stored.name, "persist_test", "Name should match");
    assert_eq!(stored.status, ChangeStatus::Review, "Status should match");
    println!("✅ Change persisted in database");
    
    // List via API multiple times
    println!("\nListing multiple times...");
    for i in 1..=3 {
        let response = client.workspace_list(None)
            .await
            .expect(&format!("Failed to list changes (iteration {})", i));
        
        let result = response.require_result_str("List result");
        assert!(result.contains(&change_id), "Should contain change ID (iteration {})", i);
        println!("✅ List iteration {} successful", i);
    }
    
    println!("\n✅ Test passed: Workspace changes persist correctly");
}
