//! Tests for clone authentication with external user API keys

use crate::common::*;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
async fn test_clone_import_with_external_user_api_key_valid() {
    println!("Test: Clone import with valid external user API key should validate and store credentials");

    // Step 1: Start mock server to act as remote VCS worker
    let mock_server = MockServer::start().await;
    
    // Step 2: Create source server with actual data to clone
    let source_server = TestServer::start()
        .await
        .expect("Failed to start source server");
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();

    // Create some state on source server
    println!("\nStep 1: Creating state on source server...");
    source_client
        .change_create("test_change", "test_author", Some("Test change"))
        .await
        .expect("Failed to create change");
    
    source_client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_id, _) = source_db.require_top_change();
    source_client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve change");
    
    println!("✅ Source server has state");

    // Step 3: Export clone data from source
    let export_response = source_client
        .clone_export()
        .await
        .expect("Failed to export");
    let clone_data_json = export_response.require_result_str("Export");

    // Step 4: Mock the /api/user/stat endpoint for API key validation
    println!("\nStep 2: Setting up mock /api/user/stat endpoint...");
    Mock::given(method("GET"))
        .and(path("/api/user/stat"))
        .and(header("X-API-Key", "test-api-key-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": ["external_user_id", "external@example.com", {"type": "obj", "id": 200}, ["Read", "Write"]]
        })))
        .mount(&mock_server)
        .await;
    
    println!("✅ Mock stat endpoint configured");

    // Step 5: Mock the /api/clone endpoint to return clone data
    println!("\nStep 3: Setting up mock /api/clone endpoint...");
    Mock::given(method("GET"))
        .and(path("/api/clone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": clone_data_json
        })))
        .mount(&mock_server)
        .await;
    
    println!("✅ Mock clone endpoint configured");

    // Step 6: Create target server and import with external user API key
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");
    
    println!("\nStep 4: Importing to target with external user API key...");
    let clone_url = format!("{}/api/clone", mock_server.uri());
    
    // Use the import_from_url_async method directly with API key
    let clone_op = moor_vcs_worker::operations::CloneOperation::new(target_server.database().clone());
    let result = clone_op
        .import_from_url_async(&clone_url, Some("test-api-key-123"))
        .await;
    
    assert!(result.is_ok(), "Clone import should succeed: {:?}", result);
    println!("✅ Clone import succeeded");

    // Step 7: Verify external user credentials are stored
    println!("\nStep 5: Verifying external user credentials are stored...");
    
    let stored_api_key = target_server
        .database()
        .index()
        .get_external_user_api_key()
        .expect("Failed to get API key")
        .expect("API key should be set");
    
    assert_eq!(stored_api_key, "test-api-key-123", "API key should be stored");
    println!("✅ External user API key stored: {}", stored_api_key);

    let stored_user_id = target_server
        .database()
        .index()
        .get_external_user_id()
        .expect("Failed to get user ID")
        .expect("User ID should be set");
    
    assert_eq!(stored_user_id, "external_user_id", "User ID should be stored");
    println!("✅ External user ID stored: {}", stored_user_id);

    // Verify base URL is stored
    let stored_source = target_server
        .database()
        .index()
        .get_source()
        .expect("Failed to get source")
        .expect("Source should be set");
    
    assert_eq!(stored_source, mock_server.uri(), "Source URL should be stored");
    println!("✅ Source URL stored: {}", stored_source);

    println!("\n✅ Test passed: Clone import with valid external user API key");
}

#[tokio::test]
async fn test_clone_import_with_external_user_api_key_invalid() {
    println!("Test: Clone import with invalid external user API key should fail");

    // Step 1: Start mock server
    let mock_server = MockServer::start().await;
    
    // Step 2: Create source server with data
    let source_server = TestServer::start()
        .await
        .expect("Failed to start source server");
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();

    source_client
        .change_create("test_change", "test_author", Some("Test"))
        .await
        .expect("Failed to create change");
    
    source_client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_id, _) = source_db.require_top_change();
    source_client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    let export_response = source_client.clone_export().await.expect("Failed to export");
    let clone_data_json = export_response.require_result_str("Export");

    // Step 3: Mock /api/user/stat to return 403 (invalid API key)
    println!("\nSetting up mock to return 403 for invalid API key...");
    Mock::given(method("GET"))
        .and(path("/api/user/stat"))
        .and(header("X-API-Key", "invalid-api-key"))
        .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "success": false,
            "error": "Invalid API key"
        })))
        .mount(&mock_server)
        .await;

    // Mock clone endpoint (won't be reached due to validation failure)
    Mock::given(method("GET"))
        .and(path("/api/clone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": clone_data_json
        })))
        .mount(&mock_server)
        .await;

    // Step 4: Try to import with invalid API key
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");
    
    println!("\nAttempting to import with invalid API key...");
    let clone_url = format!("{}/api/clone", mock_server.uri());
    
    let clone_op = moor_vcs_worker::operations::CloneOperation::new(target_server.database().clone());
    let result = clone_op
        .import_from_url_async(&clone_url, Some("invalid-api-key"))
        .await;
    
    assert!(result.is_err(), "Clone should fail with invalid API key");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("403") || error_msg.contains("invalid API key") || error_msg.contains("failed"),
        "Error should indicate API key validation failure: {}",
        error_msg
    );
    println!("✅ Clone correctly failed with: {}", error_msg);

    // Verify no credentials were stored
    let stored_api_key = target_server
        .database()
        .index()
        .get_external_user_api_key()
        .expect("Failed to get API key");
    
    assert!(stored_api_key.is_none(), "API key should NOT be stored on failure");
    println!("✅ No credentials stored on failure");

    println!("\n✅ Test passed: Clone import with invalid API key correctly fails");
}

#[tokio::test]
async fn test_clone_import_with_malformed_stat_response() {
    println!("Test: Clone import with malformed stat response should fail gracefully");

    let mock_server = MockServer::start().await;
    
    // Create source with data
    let source_server = TestServer::start().await.expect("Failed to start source");
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();

    source_client
        .change_create("test_change", "test_author", Some("Test"))
        .await
        .expect("Failed to create change");
    source_client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    let (change_id, _) = source_db.require_top_change();
    source_client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    let export_response = source_client.clone_export().await.expect("Failed to export");
    let _clone_data_json = export_response.require_result_str("Export");

    // Test case 1: Missing 'result' field
    println!("\nTest case 1: Missing 'result' field...");
    Mock::given(method("GET"))
        .and(path("/api/user/stat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .mount(&mock_server)
        .await;

    let target_server = TestServer::start().await.expect("Failed to start target");
    let clone_url = format!("{}/api/clone", mock_server.uri());
    let clone_op = moor_vcs_worker::operations::CloneOperation::new(target_server.database().clone());
    
    let result = clone_op.import_from_url_async(&clone_url, Some("test-key")).await;
    assert!(result.is_err(), "Should fail with missing result field");
    println!("✅ Correctly failed: {}", result.unwrap_err());

    // Test case 2: Result array with insufficient elements
    println!("\nTest case 2: Result array too short...");
    let mock_server2 = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/user/stat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": ["user1", "email"]  // Missing elements
        })))
        .mount(&mock_server2)
        .await;

    let target_server2 = TestServer::start().await.expect("Failed to start target");
    let clone_url2 = format!("{}/api/clone", mock_server2.uri());
    let clone_op2 = moor_vcs_worker::operations::CloneOperation::new(target_server2.database().clone());
    
    let result2 = clone_op2.import_from_url_async(&clone_url2, Some("test-key")).await;
    assert!(result2.is_err(), "Should fail with incomplete result array");
    println!("✅ Correctly failed: {}", result2.unwrap_err());

    // Test case 3: Result with non-string user_id
    println!("\nTest case 3: Non-string user_id...");
    let mock_server3 = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/user/stat"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": [123, "email@test.com", {"type": "obj", "id": 100}, []]
        })))
        .mount(&mock_server3)
        .await;

    let target_server3 = TestServer::start().await.expect("Failed to start target");
    let clone_url3 = format!("{}/api/clone", mock_server3.uri());
    let clone_op3 = moor_vcs_worker::operations::CloneOperation::new(target_server3.database().clone());
    
    let result3 = clone_op3.import_from_url_async(&clone_url3, Some("test-key")).await;
    assert!(result3.is_err(), "Should fail with non-string user_id");
    println!("✅ Correctly failed: {}", result3.unwrap_err());

    println!("\n✅ Test passed: Malformed stat responses handled correctly");
}

#[tokio::test]
async fn test_external_user_credentials_persistence() {
    println!("Test: External user credentials should persist and be retrievable");

    // Setup mock server
    let mock_server = MockServer::start().await;
    
    // Create source server
    let source_server = TestServer::start().await.expect("Failed to start source");
    let source_client = source_server.client();
    let source_db = source_server.db_assertions();

    source_client
        .change_create("test_change", "test_author", Some("Test"))
        .await
        .expect("Failed to create change");
    source_client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    let (change_id, _) = source_db.require_top_change();
    source_client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    let export_response = source_client.clone_export().await.expect("Failed to export");
    let clone_data_json = export_response.require_result_str("Export");

    // Mock stat endpoint
    Mock::given(method("GET"))
        .and(path("/api/user/stat"))
        .and(header("X-API-Key", "persistent-key-456"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": ["persistent_user", "persistent@example.com", {"type": "obj", "id": 300}, ["Admin"]]
        })))
        .mount(&mock_server)
        .await;

    // Mock clone endpoint
    Mock::given(method("GET"))
        .and(path("/api/clone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": clone_data_json
        })))
        .mount(&mock_server)
        .await;

    // Import with API key
    let target_server = TestServer::start().await.expect("Failed to start target");
    let clone_url = format!("{}/api/clone", mock_server.uri());
    
    println!("\nStep 1: Importing with external user API key...");
    let clone_op = moor_vcs_worker::operations::CloneOperation::new(target_server.database().clone());
    let result = clone_op
        .import_from_url_async(&clone_url, Some("persistent-key-456"))
        .await;
    
    assert!(result.is_ok(), "Clone should succeed");
    println!("✅ Clone completed");

    // Verify credentials persist - check multiple times
    println!("\nStep 2: Verifying credentials persist...");
    
    for i in 1..=3 {
        println!("\nCheck {}: Reading credentials...", i);
        
        let api_key = target_server
            .database()
            .index()
            .get_external_user_api_key()
            .expect("Failed to get API key")
            .expect("API key should exist");
        
        assert_eq!(api_key, "persistent-key-456", "API key should persist");
        
        let user_id = target_server
            .database()
            .index()
            .get_external_user_id()
            .expect("Failed to get user ID")
            .expect("User ID should exist");
        
        assert_eq!(user_id, "persistent_user", "User ID should persist");
        
        let source_url = target_server
            .database()
            .index()
            .get_source()
            .expect("Failed to get source")
            .expect("Source should exist");
        
        assert_eq!(source_url, mock_server.uri(), "Source URL should persist");
        
        println!("✅ Check {}: All credentials present", i);
    }

    // Verify they can be used for future operations (conceptually)
    println!("\nStep 3: Verifying credentials are ready for future use...");
    let final_api_key = target_server
        .database()
        .index()
        .get_external_user_api_key()
        .expect("Failed to get API key")
        .expect("API key should exist");
    
    assert!(!final_api_key.is_empty(), "API key should not be empty");
    println!("✅ Credentials ready for future update operations");

    println!("\n✅ Test passed: External user credentials persist correctly");
}

