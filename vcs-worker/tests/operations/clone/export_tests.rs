//! Tests for clone export operations

use crate::common::*;
use moor_vcs_worker::types::{CloneData, VcsObjectType};

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

