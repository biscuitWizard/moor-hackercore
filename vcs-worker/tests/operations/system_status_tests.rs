//! Integration tests for system/status operation
//!
//! These tests verify:
//! 1. system/status returns all required fields
//! 2. Counts are accurate for idle changes, pending review, and index changes
//! 3. Latest merged change information is correct
//! 4. Partition sizes are reported
//! 5. Remote URL is correctly reported

use crate::common::*;

#[tokio::test]
async fn test_system_status_basic() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: system/status returns basic status information");
    
    // Step 1: Get status on fresh repository
    println!("\nStep 1: Getting status on fresh repository...");
    let status_request = json!({
        "operation": "system/status",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(status_request))
        .await
        .expect("Failed to get status");
    
    println!("Status response: {}", serde_json::to_string_pretty(&response).unwrap());
    
    assert!(response["success"].as_bool().unwrap_or(false), "Status request should succeed");
    
    // Verify all required fields are present
    let result = &response["result"];
    assert!(result.is_object(), "Result should be a map/object");
    
    let result_obj = result.as_object().unwrap();
    assert!(result_obj.contains_key("top_change_id"), "Should have top_change_id");
    assert!(result_obj.contains_key("idle_changes"), "Should have idle_changes");
    assert!(result_obj.contains_key("pending_review"), "Should have pending_review");
    assert!(result_obj.contains_key("current_username"), "Should have current_username");
    assert!(result_obj.contains_key("changes_in_index"), "Should have changes_in_index");
    assert!(result_obj.contains_key("latest_merged_change"), "Should have latest_merged_change");
    assert!(result_obj.contains_key("index_partition_size"), "Should have index_partition_size");
    assert!(result_obj.contains_key("refs_partition_size"), "Should have refs_partition_size");
    assert!(result_obj.contains_key("objects_partition_size"), "Should have objects_partition_size");
    assert!(result_obj.contains_key("remote_url"), "Should have remote_url");
    assert!(result_obj.contains_key("pending_updates"), "Should have pending_updates");
    
    println!("✅ All required fields are present");
    
    // Verify fresh repository values
    assert_eq!(result_obj["top_change_id"].as_str().unwrap_or("x"), "", "Fresh repo should have empty top change ID");
    assert_eq!(result_obj["idle_changes"].as_i64().unwrap_or(-1), 0, "Fresh repo should have 0 idle changes");
    assert_eq!(result_obj["pending_review"].as_i64().unwrap_or(-1), 0, "Fresh repo should have 0 pending review");
    assert_eq!(result_obj["changes_in_index"].as_i64().unwrap_or(-1), 0, "Fresh repo should have 0 changes in index");
    assert_eq!(result_obj["remote_url"].as_str().unwrap_or("x"), "", "Fresh repo should have empty remote URL");
    assert_eq!(result_obj["pending_updates"].as_i64().unwrap_or(-1), 0, "Fresh repo should have 0 pending updates");
    
    // Check username
    assert_eq!(result_obj["current_username"].as_str().unwrap_or(""), "Wizard", "Should be Wizard user by default");
    
    // Verify latest_merged_change is empty string on fresh repo
    assert_eq!(result_obj["latest_merged_change"].as_str().unwrap_or("x"), "", "Fresh repo should have no merged changes");
    
    println!("✅ Fresh repository values are correct");
    
    println!("\n✅ Test passed: system/status returns correct basic information");
}

#[tokio::test]
async fn test_system_status_with_changes() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let base_url = server.base_url();
    
    println!("Test: system/status accurately counts changes");
    
    // Step 1: Create and approve first change
    println!("\nStep 1: Creating and approving first change...");
    client.change_create("first_change", "test_author", Some("First test change"))
        .await
        .expect("Failed to create first change");
    
    client.object_update_from_file("first_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let db = server.db_assertions();
    let (first_change_id, _) = db.require_top_change();
    
    client.change_approve(&first_change_id)
        .await
        .expect("Failed to approve first change")
        .assert_success("Approve first change");
    
    println!("✅ First change approved");
    
    // Step 2: Check status after first merged change
    println!("\nStep 2: Checking status after first merged change...");
    let status_request = json!({
        "operation": "system/status",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(status_request.clone()))
        .await
        .expect("Failed to get status");
    
    let result = &response["result"];
    let result_obj = result.as_object().unwrap();
    
    assert_eq!(result_obj["top_change_id"].as_str().unwrap_or("x"), "", "Should have no top change after approval");
    assert_eq!(result_obj["changes_in_index"].as_i64().unwrap_or(-1), 1, "Should have 1 change in index");
    
    // Verify latest_merged_change has correct structure
    let latest_merged = &result_obj["latest_merged_change"];
    assert!(latest_merged.is_object(), "latest_merged_change should be a map");
    let latest_merged_obj = latest_merged.as_object().unwrap();
    assert_eq!(latest_merged_obj["id"].as_str().unwrap_or(""), first_change_id, "Latest merged change ID should match");
    assert_eq!(latest_merged_obj["author"].as_str().unwrap_or(""), "test_author", "Author should match");
    assert!(latest_merged_obj.contains_key("timestamp"), "Should have timestamp");
    assert_eq!(latest_merged_obj["message"].as_str().unwrap_or(""), "First test change", "Message should match");
    
    println!("✅ Status correctly shows first merged change");
    
    // Step 3: Create a second local change (not approved)
    println!("\nStep 3: Creating second local change (not approved)...");
    client.change_create("second_change", "test_author", Some("Second test change"))
        .await
        .expect("Failed to create second change");
    
    client.object_update_from_file("second_object", "detailed_test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (second_change_id, _) = db.require_top_change();
    
    println!("✅ Second change created (not approved)");
    
    // Step 4: Check status with local change
    println!("\nStep 4: Checking status with local change...");
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(status_request.clone()))
        .await
        .expect("Failed to get status");
    
    let result = &response["result"];
    let result_obj = result.as_object().unwrap();
    
    assert_eq!(result_obj["top_change_id"].as_str().unwrap_or(""), second_change_id, "Should have second change as top");
    assert_eq!(result_obj["changes_in_index"].as_i64().unwrap_or(-1), 2, "Should have 2 changes in index (1 merged + 1 local)");
    
    // Latest merged should still be first change
    let latest_merged = &result_obj["latest_merged_change"];
    let latest_merged_obj = latest_merged.as_object().unwrap();
    assert_eq!(latest_merged_obj["id"].as_str().unwrap_or(""), first_change_id, "Latest merged change should still be first");
    
    println!("✅ Status correctly shows local change and maintains merged change info");
    
    // Step 5: Stash the second change (move to workspace as idle)
    println!("\nStep 5: Stashing second change...");
    let stash_request = json!({
        "operation": "change/stash",
        "args": []
    });
    
    let _stash_response = make_request("POST", &format!("{}/rpc", base_url), Some(stash_request))
        .await
        .expect("Failed to stash change");
    
    println!("✅ Second change stashed");
    
    // Step 6: Check status with idle change in workspace
    println!("\nStep 6: Checking status with idle change in workspace...");
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(status_request.clone()))
        .await
        .expect("Failed to get status");
    
    let result = &response["result"];
    let result_obj = result.as_object().unwrap();
    
    assert_eq!(result_obj["top_change_id"].as_str().unwrap_or("x"), "", "Should have no top change after stash");
    assert_eq!(result_obj["idle_changes"].as_i64().unwrap_or(-1), 1, "Should have 1 idle change");
    assert_eq!(result_obj["changes_in_index"].as_i64().unwrap_or(-1), 1, "Should have 1 change in index (only merged)");
    
    println!("✅ Status correctly shows idle change count");
    
    println!("\n✅ Test passed: system/status accurately counts all types of changes");
}

#[tokio::test]
async fn test_system_status_partition_sizes() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: system/status reports partition sizes");
    
    // Get status
    println!("\nStep 1: Getting partition sizes...");
    let status_request = json!({
        "operation": "system/status",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(status_request))
        .await
        .expect("Failed to get status");
    
    let result = &response["result"];
    let result_obj = result.as_object().unwrap();
    
    let index_size = result_obj["index_partition_size"].as_i64().unwrap_or(-1);
    let refs_size = result_obj["refs_partition_size"].as_i64().unwrap_or(-1);
    let objects_size = result_obj["objects_partition_size"].as_i64().unwrap_or(-1);
    
    println!("Partition sizes - index: {} bytes, refs: {} bytes, objects: {} bytes", 
             index_size, refs_size, objects_size);
    
    // All should be non-negative (0 is valid for empty/new partitions)
    assert!(index_size >= 0, "Index partition size should be non-negative");
    assert!(refs_size >= 0, "Refs partition size should be non-negative");
    assert!(objects_size >= 0, "Objects partition size should be non-negative");
    
    println!("✅ All partition sizes are non-negative");
    
    println!("\n✅ Test passed: system/status reports partition sizes correctly");
}

#[tokio::test]
async fn test_system_status_reports_correct_counts() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let base_url = server.base_url();
    
    println!("Test: system/status reports correct initial counts");
    
    // Get status on fresh repository
    println!("\nStep 1: Checking counts on fresh repository...");
    let status_request = json!({
        "operation": "system/status",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(status_request.clone()))
        .await
        .expect("Failed to get status");
    
    let result = &response["result"];
    let result_obj = result.as_object().unwrap();
    
    // Verify all counts are zero on fresh repo
    assert_eq!(result_obj["idle_changes"].as_i64().unwrap_or(-1), 0, "Should have 0 idle changes");
    assert_eq!(result_obj["pending_review"].as_i64().unwrap_or(-1), 0, "Should have 0 pending review");
    assert_eq!(result_obj["changes_in_index"].as_i64().unwrap_or(-1), 0, "Should have 0 changes in index");
    assert_eq!(result_obj["pending_updates"].as_i64().unwrap_or(-1), 0, "Should have 0 pending updates");
    
    println!("✅ All counts are zero on fresh repository");
    
    // Create a change
    println!("\nStep 2: Creating a change...");
    client.change_create("test_change", "test_author", Some("Test change"))
        .await
        .expect("Failed to create change");
    
    client.object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    println!("✅ Change created");
    
    // Check status with active change
    println!("\nStep 3: Checking counts with active change...");
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(status_request))
        .await
        .expect("Failed to get status");
    
    let result = &response["result"];
    let result_obj = result.as_object().unwrap();
    
    // Should have 1 change in index (the local one)
    assert_eq!(result_obj["changes_in_index"].as_i64().unwrap_or(-1), 1, "Should have 1 change in index");
    // Other counts should still be zero
    assert_eq!(result_obj["idle_changes"].as_i64().unwrap_or(-1), 0, "Should still have 0 idle changes");
    assert_eq!(result_obj["pending_review"].as_i64().unwrap_or(-1), 0, "Should still have 0 pending review");
    
    println!("✅ Counts correctly reflect active change");
    
    println!("\n✅ Test passed: system/status reports correct counts");
}

