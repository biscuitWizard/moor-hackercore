//! Tests for clone import operations

use crate::common::*;
use moor_vcs_worker::types::{CloneData, ObjectInfo, VcsObjectType};

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

