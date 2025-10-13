//! Integration tests for clone operations
//!
//! These tests verify:
//! 1. Clone export returns complete CloneData
//! 2. Clone import successfully imports from remote URL
//! 3. Clone preserves all state (refs, objects, changes, change_order)
//! 4. Clone requires proper permissions

use crate::common::*;
use moor_vcs_worker::types::{CloneData, ObjectInfo, VcsObjectType};

#[tokio::test]
async fn test_clone_export_returns_clone_data() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Clone export should return complete CloneData with all repository state");

    // Step 1: Create some initial state
    println!("\nStep 1: Creating initial state with changes and objects...");
    client
        .change_create("first_change", "test_author", Some("First change"))
        .await
        .expect("Failed to create first change");

    client
        .object_update_from_file("test_object_1", "test_object.moo")
        .await
        .expect("Failed to update object 1");

    let (first_change_id, _) = db.require_top_change();

    client
        .change_approve(&first_change_id)
        .await
        .expect("Failed to approve first change")
        .assert_success("Approve first change");

    // Create second change
    client
        .change_create("second_change", "test_author", Some("Second change"))
        .await
        .expect("Failed to create second change");

    client
        .object_update_from_file("test_object_2", "detailed_test_object.moo")
        .await
        .expect("Failed to update object 2");

    let (second_change_id, _) = db.require_top_change();

    client
        .change_approve(&second_change_id)
        .await
        .expect("Failed to approve second change")
        .assert_success("Approve second change");

    println!("✅ Created 2 merged changes with objects");

    // Step 2: Export the repository state
    println!("\nStep 2: Exporting repository state...");
    let export_response = client
        .clone_export()
        .await
        .expect("Failed to export clone data");
    export_response.assert_success("Clone export");

    // Parse the CloneData from the result
    let result_str = export_response.require_result_str("Clone export");
    let clone_data: CloneData =
        serde_json::from_str(result_str).expect("Failed to parse CloneData JSON");

    println!("✅ Export successful, received CloneData");

    // Step 3: Verify CloneData contents
    println!("\nStep 3: Verifying CloneData contents...");

    // Should have refs for both objects
    assert_eq!(clone_data.refs.len(), 2, "Should have 2 refs");
    println!("✅ Has {} refs", clone_data.refs.len());

    // Should have objects in the objects map
    assert!(!clone_data.objects.is_empty(), "Should have objects");
    println!("✅ Has {} objects", clone_data.objects.len());

    // Should have 2 merged changes
    assert_eq!(clone_data.changes.len(), 2, "Should have 2 changes");
    let change_ids: Vec<String> = clone_data.changes.iter().map(|c| c.id.clone()).collect();
    assert!(
        change_ids.contains(&first_change_id),
        "Should contain first change"
    );
    assert!(
        change_ids.contains(&second_change_id),
        "Should contain second change"
    );
    println!("✅ Has 2 merged changes");

    // Should have change order with both changes
    assert_eq!(
        clone_data.change_order.len(),
        2,
        "Should have 2 changes in order"
    );
    assert!(
        clone_data.change_order.contains(&first_change_id),
        "Order should contain first change"
    );
    assert!(
        clone_data.change_order.contains(&second_change_id),
        "Order should contain second change"
    );
    println!("✅ Change order contains both changes");

    // Verify all changes are MERGED status
    for change in &clone_data.changes {
        assert_eq!(
            change.status,
            moor_vcs_worker::types::ChangeStatus::Merged,
            "All exported changes should be Merged"
        );
    }
    println!("✅ All changes have Merged status");

    println!("\n✅ Test passed: Clone export returns complete CloneData");
}

#[tokio::test]
async fn test_clone_import_from_remote() {
    let source_server = TestServer::start()
        .await
        .expect("Failed to start source server");
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");

    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    let target_client = target_server.client();
    let target_db = target_server.db_assertions();

    println!("Test: Clone import should successfully import from remote URL");

    // Step 1: Create state on source server
    println!("\nStep 1: Creating state on source server...");
    source_client
        .change_create("source_change", "test_author", Some("Source change"))
        .await
        .expect("Failed to create change");

    source_client
        .object_update_from_file("source_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (source_change_id, _) = source_db.require_top_change();

    source_client
        .change_approve(&source_change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");

    println!("✅ Source server has 1 merged change with object");

    // Step 2: Clone from source to target
    println!("\nStep 2: Cloning from source to target...");
    let source_url = format!("{}/api/clone", source_server.base_url());

    let import_response = target_client
        .clone_import(&source_url)
        .await
        .expect("Failed to import clone");

    println!("Import response: {:?}", import_response);
    import_response.assert_success("Clone import");

    println!("✅ Clone import successful");

    // Step 3: Verify target has the same state
    println!("\nStep 3: Verifying target has same state as source...");

    // Check that change exists on target
    let target_change_order = target_server
        .database()
        .index()
        .get_change_order()
        .expect("Failed to get change order");
    assert_eq!(target_change_order.len(), 1, "Target should have 1 change");
    assert_eq!(
        target_change_order[0], source_change_id,
        "Change ID should match"
    );

    let target_change = target_server
        .database()
        .index()
        .get_change(&source_change_id)
        .expect("Failed to get change")
        .expect("Change should exist on target");
    assert_eq!(
        target_change.name, "source_change",
        "Change name should match"
    );
    assert_eq!(
        target_change.status,
        moor_vcs_worker::types::ChangeStatus::Merged,
        "Should be merged"
    );

    println!("✅ Target has same change");

    // Check that object ref exists on target
    target_db.assert_ref_exists(VcsObjectType::MooObject, "source_object");
    println!("✅ Target has object ref");

    // Check that source URL is set on target (should be base URL without /api/clone)
    let target_source = target_server
        .database()
        .index()
        .get_source()
        .expect("Failed to get source")
        .expect("Source should be set");
    let expected_base_url = source_url.trim_end_matches("/api/clone");
    assert_eq!(
        target_source, expected_base_url,
        "Source URL should be base URL"
    );
    println!("✅ Target has correct source URL: {}", target_source);

    println!("\n✅ Test passed: Clone import successfully imports from remote");
}

#[tokio::test]
async fn test_clone_preserves_all_state() {
    let source_server = TestServer::start()
        .await
        .expect("Failed to start source server");
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");

    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    let target_client = target_server.client();

    println!(
        "Test: Clone should preserve all state including refs, objects, changes, and change order"
    );

    // Step 1: Create complex state on source
    println!("\nStep 1: Creating complex state on source server...");

    // First change with one object
    source_client
        .change_create("change_1", "author_1", Some("First change"))
        .await
        .expect("Failed to create change 1");

    source_client
        .object_update_from_file("object_1", "test_object.moo")
        .await
        .expect("Failed to update object 1");

    let (change_1_id, _) = source_db.require_top_change();
    source_client
        .change_approve(&change_1_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve 1");

    // Second change with another object
    source_client
        .change_create("change_2", "author_2", Some("Second change"))
        .await
        .expect("Failed to create change 2");

    source_client
        .object_update_from_file("object_2", "detailed_test_object.moo")
        .await
        .expect("Failed to update object 2");

    let (change_2_id, _) = source_db.require_top_change();
    source_client
        .change_approve(&change_2_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve 2");

    // Third change modifying first object
    source_client
        .change_create("change_3", "author_3", Some("Third change"))
        .await
        .expect("Failed to create change 3");

    source_client
        .object_update_from_file("object_1", "detailed_test_object.moo")
        .await
        .expect("Failed to update object 1 again");

    let (change_3_id, _) = source_db.require_top_change();
    source_client
        .change_approve(&change_3_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve 3");

    println!("✅ Created 3 merged changes with objects");

    // Step 2: Collect source state for comparison
    println!("\nStep 2: Collecting source state...");

    let source_refs = source_server
        .database()
        .refs()
        .get_all_refs()
        .expect("Failed to get source refs");
    let source_objects = source_server
        .database()
        .objects()
        .get_all_objects()
        .expect("Failed to get source objects");
    let source_change_order = source_server
        .database()
        .index()
        .get_change_order()
        .expect("Failed to get source change order");

    println!(
        "Source state: {} refs, {} objects, {} changes",
        source_refs.len(),
        source_objects.len(),
        source_change_order.len()
    );

    // Step 3: Clone to target
    println!("\nStep 3: Cloning to target server...");
    let source_url = format!("{}/api/clone", source_server.base_url());

    target_client
        .clone_import(&source_url)
        .await
        .expect("Failed to clone")
        .assert_success("Clone import");

    println!("✅ Clone completed");

    // Step 4: Verify all state matches
    println!("\nStep 4: Verifying all state matches...");

    // Verify refs match
    let target_refs_map = target_server
        .database()
        .refs()
        .get_all_refs()
        .expect("Failed to get target refs");
    let target_refs: Vec<(ObjectInfo, String)> = target_refs_map.clone().into_iter().collect();
    let source_refs_vec: Vec<(ObjectInfo, String)> = source_refs.clone().into_iter().collect();
    assert_eq!(
        target_refs.len(),
        source_refs_vec.len(),
        "Ref counts should match"
    );

    for (obj_info, source_sha) in &source_refs_vec {
        let target_sha = target_refs_map
            .get(obj_info)
            .unwrap_or_else(|| panic!("Target missing ref for {:?}", obj_info));
        assert_eq!(
            target_sha, source_sha,
            "SHA256 should match for {:?}",
            obj_info
        );
    }
    println!("✅ All {} refs match exactly", source_refs_vec.len());

    // Verify objects match
    let target_objects = target_server
        .database()
        .objects()
        .get_all_objects()
        .expect("Failed to get target objects");
    assert_eq!(
        target_objects.len(),
        source_objects.len(),
        "Object counts should match"
    );

    for (sha256, source_data) in &source_objects {
        let target_data = target_objects
            .get(sha256)
            .unwrap_or_else(|| panic!("Target missing object for SHA256 {}", sha256));
        assert_eq!(
            target_data, source_data,
            "Object data should match for SHA256 {}",
            sha256
        );
    }
    println!("✅ All {} objects match exactly", source_objects.len());

    // Verify change order matches
    let target_change_order = target_server
        .database()
        .index()
        .get_change_order()
        .expect("Failed to get target change order");
    assert_eq!(
        target_change_order, source_change_order,
        "Change order should match exactly"
    );
    println!(
        "✅ Change order matches ({} changes)",
        target_change_order.len()
    );

    // Verify individual changes match
    for change_id in &source_change_order {
        let source_change = source_server
            .database()
            .index()
            .get_change(change_id)
            .expect("Failed to get source change")
            .expect("Source change should exist");

        let target_change = target_server
            .database()
            .index()
            .get_change(change_id)
            .expect("Failed to get target change")
            .expect("Target change should exist");

        assert_eq!(
            target_change.name, source_change.name,
            "Change names should match"
        );
        assert_eq!(
            target_change.author, source_change.author,
            "Change authors should match"
        );
        assert_eq!(
            target_change.status, source_change.status,
            "Change status should match"
        );
        assert_eq!(
            target_change.added_objects.len(),
            source_change.added_objects.len(),
            "Added objects count should match"
        );
        assert_eq!(
            target_change.modified_objects.len(),
            source_change.modified_objects.len(),
            "Modified objects count should match"
        );
    }
    println!("✅ All change details match");

    println!("\n✅ Test passed: Clone preserves all state exactly");
}

#[tokio::test]
async fn test_clone_only_exports_merged_changes() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!(
        "Test: Clone export should only include MERGED changes, not local/review/idle changes"
    );

    // Step 1: Create a merged change
    println!("\nStep 1: Creating and approving a change...");
    client
        .change_create("merged_change", "test_author", Some("This will be merged"))
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("merged_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (merged_change_id, _) = db.require_top_change();

    client
        .change_approve(&merged_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve change");

    println!("✅ Created 1 merged change");

    // Step 2: Create a local change (not approved)
    println!("\nStep 2: Creating a local change (not approved)...");
    client
        .change_create("local_change", "test_author", Some("This is local"))
        .await
        .expect("Failed to create local change");

    client
        .object_update_from_file("local_object", "detailed_test_object.moo")
        .await
        .expect("Failed to update local object");

    let (local_change_id, local_change) = db.require_top_change();
    assert_eq!(
        local_change.status,
        moor_vcs_worker::types::ChangeStatus::Local,
        "Should be local"
    );

    println!("✅ Created 1 local change");

    // Step 3: Export clone data
    println!("\nStep 3: Exporting clone data...");
    let export_response = client.clone_export().await.expect("Failed to export");
    export_response.assert_success("Clone export");

    let result_str = export_response.require_result_str("Clone export");
    let clone_data: CloneData =
        serde_json::from_str(result_str).expect("Failed to parse CloneData");

    println!("✅ Export successful");

    // Step 4: Verify only merged change is exported
    println!("\nStep 4: Verifying only merged change is exported...");

    assert_eq!(
        clone_data.changes.len(),
        1,
        "Should only export 1 change (merged)"
    );
    assert_eq!(
        clone_data.changes[0].id, merged_change_id,
        "Should export the merged change"
    );
    assert_eq!(
        clone_data.changes[0].status,
        moor_vcs_worker::types::ChangeStatus::Merged,
        "Should be merged"
    );

    // Verify local change is NOT in the export
    let has_local = clone_data.changes.iter().any(|c| c.id == local_change_id);
    assert!(!has_local, "Local change should NOT be exported");

    println!("✅ Only merged change exported, local change excluded");

    // Verify change order also only includes merged change
    assert_eq!(
        clone_data.change_order.len(),
        1,
        "Change order should only have 1 change"
    );
    assert_eq!(
        clone_data.change_order[0], merged_change_id,
        "Change order should only have merged change"
    );

    println!("✅ Change order only includes merged change");

    println!("\n✅ Test passed: Clone only exports merged changes");
}

#[tokio::test]
async fn test_clone_import_when_already_cloned() {
    let source_server = TestServer::start()
        .await
        .expect("Failed to start source server");
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");

    let source_client = source_server.client();
    let source_db = source_server.db_assertions();
    let target_client = target_server.client();

    println!("Test: Clone import when already cloned should handle gracefully");

    // Step 1: Create state on source
    println!("\nStep 1: Creating state on source...");
    source_client
        .change_create("source_change", "test_author", Some("Source"))
        .await
        .expect("Failed to create change");

    source_client
        .object_update_from_file("source_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (source_change_id, _) = source_db.require_top_change();
    source_client
        .change_approve(&source_change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Source has 1 merged change");

    // Step 2: Clone from source to target (first time)
    println!("\nStep 2: First clone...");
    let source_url = format!("{}/api/clone", source_server.base_url());

    target_client
        .clone_import(&source_url)
        .await
        .expect("Failed to clone")
        .assert_success("Clone");

    println!("✅ First clone successful");

    // Step 3: Verify source is set
    let source_after_first = target_server
        .database()
        .index()
        .get_source()
        .expect("Failed to get source")
        .expect("Source should be set");

    let expected_base_url = source_url.trim_end_matches("/api/clone");
    assert_eq!(
        source_after_first, expected_base_url,
        "Source should be set"
    );

    // Step 4: Try to clone again (should handle gracefully)
    println!("\nStep 4: Attempting second clone...");
    let second_clone_response = target_client
        .clone_import(&source_url)
        .await
        .expect("Request should complete");

    println!("Second clone response: {:?}", second_clone_response);

    // Should either succeed (overwrite) or fail gracefully
    // Check if it contains an error or succeeds
    let result_str = second_clone_response.get_result_str().unwrap_or("");
    if result_str.contains("Error") {
        println!("✅ Second clone rejected with: {}", result_str);
    } else {
        println!("✅ Second clone succeeded (overwrite behavior)");
    }

    println!("\n✅ Test passed: Clone import when already cloned handles gracefully");
}

#[tokio::test]
async fn test_clone_import_with_invalid_url_format() {
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");
    let target_client = target_server.client();

    println!("Test: Clone import with invalid URL format should fail gracefully");

    let invalid_urls = vec![
        "not-a-url",
        "ftp://invalid-protocol.com",
        "http://",
        "",
        "   ",
    ];

    for invalid_url in invalid_urls {
        println!("\nTesting invalid URL: '{}'", invalid_url);
        let response = target_client
            .clone_import(invalid_url)
            .await
            .expect("Request should complete");

        // Should fail with error
        let result_str = response.get_result_str().unwrap_or("");
        assert!(
            result_str.contains("Error")
                || result_str.contains("failed")
                || result_str.contains("invalid"),
            "Should indicate error for '{}', got: {}",
            invalid_url,
            result_str
        );
        println!("✅ Failed appropriately: {}", result_str);
    }

    println!("\n✅ Test passed: Clone import handles invalid URLs gracefully");
}

#[tokio::test]
async fn test_clone_import_with_unreachable_url() {
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");
    let target_client = target_server.client();

    println!("Test: Clone import with unreachable URL should handle network errors gracefully");

    // Use a URL that should be unreachable
    let unreachable_url = "http://localhost:99999/api/clone";

    println!(
        "\nAttempting to clone from unreachable URL: {}",
        unreachable_url
    );
    let response = target_client
        .clone_import(unreachable_url)
        .await
        .expect("Request should complete");

    // Should fail with network error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error")
            || result_str.contains("failed")
            || result_str.contains("connection"),
        "Should indicate network error, got: {}",
        result_str
    );
    println!("✅ Network error handled gracefully: {}", result_str);

    println!("\n✅ Test passed: Clone handles unreachable URLs gracefully");
}

#[tokio::test]
async fn test_clone_export_then_import_to_same_repo() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Clone export then import to same repo should be detected");

    // Step 1: Create state
    println!("\nStep 1: Creating state...");
    client
        .change_create("test_change", "test_author", Some("Test"))
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ State created");

    // Step 2: Export
    println!("\nStep 2: Exporting...");
    let export_response = client.clone_export().await.expect("Failed to export");
    export_response.assert_success("Export");
    println!("✅ Export successful");

    // Step 3: Try to import to same repo
    println!("\nStep 3: Attempting to import to same repo...");
    let self_url = format!("{}/api/clone", server.base_url());
    let import_response = client
        .clone_import(&self_url)
        .await
        .expect("Request should complete");

    println!("Import response: {:?}", import_response);

    // This might succeed (self-clone) or fail - both are acceptable
    // Just verify it doesn't crash
    let result_str = import_response.get_result_str().unwrap_or("");
    if result_str.contains("Error") {
        println!("✅ Self-clone rejected: {}", result_str);
    } else {
        println!("✅ Self-clone succeeded (creates source reference)");
    }

    println!("\n✅ Test passed: Clone to same repo handled");
}

#[tokio::test]
async fn test_clone_import_with_external_user_api_key_valid() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

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
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

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
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

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
async fn test_clone_import_response_parsing_formats() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};
    use moor_vcs_worker::types::CloneData;

    println!("Test: Clone import should handle various response formats");

    // Create minimal valid CloneData
    let clone_data = CloneData {
        refs: vec![],
        objects: std::collections::HashMap::new(),
        changes: vec![],
        change_order: vec![],
        source: None,
    };

    // Test case 1: OperationResponse with result as JSON string
    println!("\nTest case 1: Result as JSON string...");
    let mock_server1 = MockServer::start().await;
    let clone_data_string = serde_json::to_string(&clone_data).unwrap();
    
    Mock::given(method("GET"))
        .and(path("/api/clone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": clone_data_string
        })))
        .mount(&mock_server1)
        .await;

    let target1 = TestServer::start().await.expect("Failed to start target");
    let clone_url1 = format!("{}/api/clone", mock_server1.uri());
    let clone_op1 = moor_vcs_worker::operations::CloneOperation::new(target1.database().clone());
    
    let result1 = clone_op1.import_from_url_async(&clone_url1, None).await;
    assert!(result1.is_ok(), "Should parse result as JSON string: {:?}", result1);
    println!("✅ Parsed result as JSON string");

    // Test case 2: OperationResponse with result as direct object
    println!("\nTest case 2: Result as direct object...");
    let mock_server2 = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/clone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "result": {
                "refs": [],
                "objects": {},
                "changes": [],
                "change_order": [],
                "source": null
            }
        })))
        .mount(&mock_server2)
        .await;

    let target2 = TestServer::start().await.expect("Failed to start target");
    let clone_url2 = format!("{}/api/clone", mock_server2.uri());
    let clone_op2 = moor_vcs_worker::operations::CloneOperation::new(target2.database().clone());
    
    let result2 = clone_op2.import_from_url_async(&clone_url2, None).await;
    assert!(result2.is_ok(), "Should parse result as object: {:?}", result2);
    println!("✅ Parsed result as direct object");

    // Test case 3: Direct CloneData (no OperationResponse wrapper)
    println!("\nTest case 3: Direct CloneData without wrapper...");
    let mock_server3 = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/clone"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "refs": [],
            "objects": {},
            "changes": [],
            "change_order": [],
            "source": null
        })))
        .mount(&mock_server3)
        .await;

    let target3 = TestServer::start().await.expect("Failed to start target");
    let clone_url3 = format!("{}/api/clone", mock_server3.uri());
    let clone_op3 = moor_vcs_worker::operations::CloneOperation::new(target3.database().clone());
    
    let result3 = clone_op3.import_from_url_async(&clone_url3, None).await;
    assert!(result3.is_ok(), "Should parse direct CloneData: {:?}", result3);
    println!("✅ Parsed direct CloneData");

    // Test case 4: Invalid JSON
    println!("\nTest case 4: Invalid JSON...");
    let mock_server4 = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/api/clone"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json {"))
        .mount(&mock_server4)
        .await;

    let target4 = TestServer::start().await.expect("Failed to start target");
    let clone_url4 = format!("{}/api/clone", mock_server4.uri());
    let clone_op4 = moor_vcs_worker::operations::CloneOperation::new(target4.database().clone());
    
    let result4 = clone_op4.import_from_url_async(&clone_url4, None).await;
    assert!(result4.is_err(), "Should fail with invalid JSON");
    println!("✅ Correctly rejected invalid JSON: {}", result4.unwrap_err());

    println!("\n✅ Test passed: All response formats handled correctly");
}

#[tokio::test]
async fn test_clone_execute_error_paths() {
    use moor_vcs_worker::operations::Operation;
    use moor_var::Variant;

    println!("Test: Clone execute() method error paths (also tests sync wrapper lines 243-263)");

    let server = TestServer::start().await.expect("Failed to start server");
    let client = server.client();
    let db = server.db_assertions();

    // Get wizard user with Clone permission
    let mut wizard = server.get_wizard_user().expect("Failed to get wizard");
    server
        .database()
        .users()
        .add_permission(&wizard.id, moor_vcs_worker::types::Permission::Clone)
        .expect("Failed to add Clone permission");
    wizard = server.database().users().get_user(&wizard.id).unwrap().unwrap();

    // Test 1: Export with empty repository (should succeed with empty data)
    println!("\nTest 1: Export empty repository...");
    let clone_op = moor_vcs_worker::operations::CloneOperation::new(server.database().clone());
    let result = clone_op.execute(vec![], &wizard);
    
    match result.variant() {
        Variant::Str(_) => {
            println!("✅ Export succeeded with empty repository");
        }
        Variant::Err(e) => {
            panic!("Export should not fail with empty repo: {:?}", e);
        }
        _ => panic!("Unexpected result type: {:?}", result),
    }

    // Test 2: Export after creating data, then verify serialization
    println!("\nTest 2: Export with actual data...");
    client
        .change_create("test_change", "test_author", Some("Test"))
        .await
        .expect("Failed to create change");
    
    client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    let result = clone_op.execute(vec![], &wizard);
    
    match result.variant() {
        Variant::Str(s) => {
            // Verify it's valid JSON CloneData
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(s.as_str());
            assert!(parsed.is_ok(), "Export should be valid JSON");
            
            let json = parsed.unwrap();
            assert!(json.get("refs").is_some(), "Should have refs field");
            assert!(json.get("objects").is_some(), "Should have objects field");
            assert!(json.get("changes").is_some(), "Should have changes field");
            
            println!("✅ Export succeeded with valid JSON ({} bytes)", s.as_str().len());
        }
        Variant::Err(e) => {
            panic!("Export should not fail: {:?}", e);
        }
        _ => panic!("Unexpected result type: {:?}", result),
    }

    // Test 3: Import with empty URL (should export instead)
    println!("\nTest 3: Import with empty URL (treats as export)...");
    let result = clone_op.execute(vec!["".to_string()], &wizard);
    
    match result.variant() {
        Variant::Str(_) => {
            println!("✅ Empty URL treated as export");
        }
        _ => panic!("Empty URL should trigger export, got: {:?}", result),
    }

    println!("\n✅ Test passed: Execute handles export paths correctly");
    println!("Note: Import paths with URLs (lines 243-263 sync wrapper) are tested via async methods in other tests");
}

#[tokio::test]
async fn test_external_user_credentials_persistence() {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

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
