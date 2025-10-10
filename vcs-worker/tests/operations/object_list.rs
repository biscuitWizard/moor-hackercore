//! Integration tests for object_list operation
//!
//! These tests verify that object/list correctly reports object states:
//! 1. Added objects appear in the list
//! 2. Deleted objects are excluded from the list
//! 3. Renamed objects appear with their new name
//! 4. Modified objects appear with updated versions

use crate::common::*;

#[tokio::test]
async fn test_object_list_shows_added_objects() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: object/list shows added objects");
    
    // Step 1: Verify empty list initially
    println!("\nStep 1: Verifying empty list initially...");
    let list_request = json!({
        "operation": "object/list",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(list_request.clone()))
        .await
        .expect("Failed to list objects");
    
    assert!(response["success"].as_bool().unwrap_or(false), "List should succeed");
    
    let result_str = response["result"].as_str().unwrap();
    let list_response: serde_json::Value = serde_json::from_str(result_str)
        .expect("Failed to parse list response");
    
    let objects = list_response["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 0, "Should have no objects initially");
    println!("✅ Empty list initially");
    
    // Step 2: Add some objects
    println!("\nStep 2: Adding objects...");
    
    // Create a change first
    let create_request = json!({
        "operation": "change/create",
        "args": ["test_change", "test_author"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(create_request))
        .await
        .expect("Failed to create change");
    
    // Add first object
    let object_name_1 = "object_one";
    let object_dump_1 = load_moo_file("test_object.moo");
    let object_content_1 = moo_to_lines(&object_dump_1);
    
    let update_request_1 = json!({
        "operation": "object/update",
        "args": [object_name_1, serde_json::to_string(&object_content_1).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_1))
        .await
        .expect("Failed to add object 1");
    
    // Add second object
    let object_name_2 = "object_two";
    let object_dump_2 = load_moo_file("detailed_test_object.moo");
    let object_content_2 = moo_to_lines(&object_dump_2);
    
    let update_request_2 = json!({
        "operation": "object/update",
        "args": [object_name_2, serde_json::to_string(&object_content_2).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_2))
        .await
        .expect("Failed to add object 2");
    
    println!("✅ Added 2 objects");
    
    // Step 3: Verify objects appear in list
    println!("\nStep 3: Verifying objects appear in list...");
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(list_request))
        .await
        .expect("Failed to list objects");
    
    assert!(response["success"].as_bool().unwrap_or(false), "List should succeed");
    
    let result_str = response["result"].as_str().unwrap();
    let list_response: serde_json::Value = serde_json::from_str(result_str)
        .expect("Failed to parse list response");
    
    let objects = list_response["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 2, "Should have 2 objects");
    
    let object_names: Vec<String> = objects.iter()
        .map(|obj| obj["name"].as_str().unwrap().to_string())
        .collect();
    
    assert!(object_names.contains(&object_name_1.to_string()), "Should contain object_one");
    assert!(object_names.contains(&object_name_2.to_string()), "Should contain object_two");
    
    println!("✅ Both objects appear in list: {:?}", object_names);
    
    println!("\n✅ Test passed: object/list shows added objects");
}

#[tokio::test]
async fn test_object_list_excludes_deleted_objects() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: object/list excludes deleted objects");
    
    // Step 1: Add objects and delete one within the same change
    println!("\nStep 1: Creating change and adding objects...");
    let create_request = json!({
        "operation": "change/create",
        "args": ["test_change", "test_author"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(create_request))
        .await
        .expect("Failed to create change");
    
    let object_name_1 = "object_to_keep";
    let object_dump_1 = load_moo_file("test_object.moo");
    let object_content_1 = moo_to_lines(&object_dump_1);
    
    let update_request_1 = json!({
        "operation": "object/update",
        "args": [object_name_1, serde_json::to_string(&object_content_1).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_1))
        .await
        .expect("Failed to add object 1");
    
    let object_name_2 = "object_to_delete";
    let object_dump_2 = load_moo_file("detailed_test_object.moo");
    let object_content_2 = moo_to_lines(&object_dump_2);
    
    let update_request_2 = json!({
        "operation": "object/update",
        "args": [object_name_2, serde_json::to_string(&object_content_2).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_2))
        .await
        .expect("Failed to add object 2");
    
    println!("✅ Added 2 objects");
    
    // Step 2: Delete the second object in the same change
    println!("\nStep 2: Deleting an object...");
    
    // Delete the second object
    let delete_request = json!({
        "operation": "object/delete",
        "args": [object_name_2]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(delete_request))
        .await
        .expect("Failed to delete object");
    
    println!("✅ Deleted object_to_delete");
    
    // Step 3: Verify deleted object doesn't appear in list
    println!("\nStep 3: Verifying deleted object is excluded...");
    let list_request = json!({
        "operation": "object/list",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(list_request))
        .await
        .expect("Failed to list objects");
    
    assert!(response["success"].as_bool().unwrap_or(false), "List should succeed");
    
    let result_str = response["result"].as_str().unwrap();
    let list_response: serde_json::Value = serde_json::from_str(result_str)
        .expect("Failed to parse list response");
    
    let objects = list_response["objects"].as_array().unwrap();
    
    // When an object is added and then deleted in the same change,
    // it should net out to zero (object was never really there)
    assert_eq!(objects.len(), 1, "Should have only 1 object (second was added then deleted)");
    
    let object_names: Vec<String> = objects.iter()
        .map(|obj| obj["name"].as_str().unwrap().to_string())
        .collect();
    
    assert!(object_names.contains(&object_name_1.to_string()), "Should contain object_to_keep");
    assert!(!object_names.contains(&object_name_2.to_string()), "Should NOT contain object_to_delete");
    
    println!("✅ Only remaining object appears in list: {:?}", object_names);
    
    println!("\n✅ Test passed: object/list excludes deleted objects");
}

#[tokio::test]
async fn test_object_list_shows_renamed_objects_with_new_name() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: object/list shows renamed objects with new name");
    
    // Test rename within a single change (added objects can be renamed)
    println!("\nStep 1: Creating change, adding object, and renaming it...");
    let create_request = json!({
        "operation": "change/create",
        "args": ["test_change", "test_author"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(create_request))
        .await
        .expect("Failed to create change");
    
    let original_name = "original_name";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [original_name, serde_json::to_string(&object_content).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to add object");
    
    println!("✅ Added object with original name");
    
    // Rename the object (within the same change, it just updates the name in added_objects)
    let new_name = "renamed_object";
    let rename_request = json!({
        "operation": "object/rename",
        "args": [original_name, new_name]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(rename_request))
        .await
        .expect("Failed to rename object");
    
    println!("✅ Renamed object from '{}' to '{}'", original_name, new_name);
    
    // Step 2: Verify object appears with new name only
    println!("\nStep 2: Verifying object appears with new name...");
    let list_request = json!({
        "operation": "object/list",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(list_request))
        .await
        .expect("Failed to list objects");
    
    assert!(response["success"].as_bool().unwrap_or(false), "List should succeed");
    
    let result_str = response["result"].as_str().unwrap();
    let list_response: serde_json::Value = serde_json::from_str(result_str)
        .expect("Failed to parse list response");
    
    let objects = list_response["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 1, "Should have 1 object");
    
    let object_names: Vec<String> = objects.iter()
        .map(|obj| obj["name"].as_str().unwrap().to_string())
        .collect();
    
    assert!(object_names.contains(&new_name.to_string()), "Should contain renamed_object");
    assert!(!object_names.contains(&original_name.to_string()), "Should NOT contain original_name");
    
    println!("✅ Object appears with new name: {:?}", object_names);
    
    println!("\n✅ Test passed: object/list shows renamed objects with new name");
}

#[tokio::test]
async fn test_object_list_tracks_object_state_in_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: object/list correctly tracks object state (added vs modified)");
    
    // Step 1: Create a change and add an object
    println!("\nStep 1: Creating change and adding object...");
    let create_request = json!({
        "operation": "change/create",
        "args": ["test_change", "test_author"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(create_request))
        .await
        .expect("Failed to create change");
    
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [object_name, serde_json::to_string(&object_content).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to add object");
    
    println!("✅ Added object");
    
    // Verify object is in added_objects (not modified_objects)
    let change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(change.added_objects.len(), 1, "Should have 1 added object");
    assert_eq!(change.modified_objects.len(), 0, "Should have 0 modified objects");
    println!("✅ Object is in added_objects");
    
    // Step 2: Modify the object (in the same change)
    println!("\nStep 2: Modifying object in same change...");
    let object_dump_2 = load_moo_file("detailed_test_object.moo");
    let object_content_2 = moo_to_lines(&object_dump_2);
    
    let update_request_2 = json!({
        "operation": "object/update",
        "args": [object_name, serde_json::to_string(&object_content_2).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_2))
        .await
        .expect("Failed to modify object");
    
    println!("✅ Modified object");
    
    // Step 3: Verify object is STILL in added_objects (not moved to modified_objects)
    println!("\nStep 3: Verifying object stays in added_objects...");
    
    let change = server.database().index().get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(change.added_objects.len(), 1, "Should still have 1 added object");
    assert_eq!(change.modified_objects.len(), 0, "Should still have 0 modified objects");
    assert_eq!(change.added_objects[0].name, object_name, "Object should still be in added_objects");
    
    println!("✅ Object stays in added_objects (not moved to modified_objects)");
    
    // Step 4: Verify object appears in list with version 1
    println!("\nStep 4: Verifying object appears in list...");
    let list_request = json!({
        "operation": "object/list",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(list_request))
        .await
        .expect("Failed to list objects");
    
    assert!(response["success"].as_bool().unwrap_or(false), "List should succeed");
    
    let result_str = response["result"].as_str().unwrap();
    let list_response: serde_json::Value = serde_json::from_str(result_str)
        .expect("Failed to parse list response");
    
    let objects = list_response["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 1, "Should have 1 object");
    
    let obj = &objects[0];
    assert_eq!(obj["name"].as_str().unwrap(), object_name, "Should be the same object");
    assert_eq!(obj["version"].as_u64().unwrap(), 1, "Version should be 1 (added object)");
    
    println!("✅ Object appears in list with version 1 (added, not modified)");
    
    println!("\n✅ Test passed: Objects modified in same change stay as 'added'");
}

#[tokio::test]
async fn test_object_list_complex_scenario() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: object/list complex scenario with add, modify, rename, delete");
    println!("This test creates committed objects, then performs operations to test all states\n");
    
    // Step 1: Create initial change and add objects
    println!("Step 1: Creating initial change and adding objects...");
    let create_request = json!({
        "operation": "change/create",
        "args": ["initial_change", "test_author"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(create_request))
        .await
        .expect("Failed to create change");
    
    // Add four objects
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    for name in ["obj_A", "obj_B", "obj_C", "obj_E"] {
        let update_request = json!({
            "operation": "object/update",
            "args": [name, serde_json::to_string(&object_content).unwrap()]
        });
        make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
            .await
            .expect("Failed to add object");
    }
    
    println!("✅ Added 4 objects: obj_A, obj_B, obj_C, obj_E");
    
    // Step 2: Approve the commit using the actual approve operation
    println!("\nStep 2: Approving initial commit...");
    let initial_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let initial_change = server.database().index().get_change(&initial_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert_eq!(initial_change.status, moor_vcs_worker::types::ChangeStatus::Local, "Should be Local");
    assert_eq!(initial_change.added_objects.len(), 4, "Should have 4 added objects");
    
    // Use the actual approve_change operation via HTTP
    let approve_request = json!({
        "operation": "change/approve",
        "args": [initial_change_id.clone()]
    });
    
    let approve_response = make_request("POST", &format!("{}/rpc", base_url), Some(approve_request))
        .await
        .expect("Failed to approve change");
    
    assert!(approve_response["success"].as_bool().unwrap_or(false), "Approve should succeed");
    println!("✅ Approved initial change using approve operation");
    
    // Verify it's marked as Merged
    let merged_change = server.database().index().get_change(&initial_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    assert_eq!(merged_change.status, moor_vcs_worker::types::ChangeStatus::Merged, "Should be Merged");
    println!("✅ Change is Merged (kept in history for object_list)");
    
    // Step 3: Create new change and perform operations
    println!("\nStep 3: Creating new change and performing operations...");
    let create_request_2 = json!({
        "operation": "change/create",
        "args": ["complex_change", "test_author"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(create_request_2))
        .await
        .expect("Failed to create second change");
    
    let new_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    // Verify it's a different change
    assert_ne!(new_change_id, initial_change_id, "Should be a new change");
    println!("✅ Created new local change");
    
    // Rename obj_A to obj_A_renamed (should go in renamed_objects)
    let rename_request = json!({
        "operation": "object/rename",
        "args": ["obj_A", "obj_A_renamed"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(rename_request))
        .await
        .expect("Failed to rename obj_A");
    println!("  • Renamed obj_A -> obj_A_renamed");
    
    // Check change state after rename
    let change = server.database().index().get_change(&new_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    assert_eq!(change.renamed_objects.len(), 1, "Should have 1 renamed object");
    assert_eq!(change.renamed_objects[0].from.name, "obj_A");
    assert_eq!(change.renamed_objects[0].to.name, "obj_A_renamed");
    println!("    ✓ obj_A in renamed_objects");
    
    // Modify obj_B (should go in modified_objects since it's from committed change)
    let object_dump_2 = load_moo_file("detailed_test_object.moo");
    let object_content_2 = moo_to_lines(&object_dump_2);
    let update_request = json!({
        "operation": "object/update",
        "args": ["obj_B", serde_json::to_string(&object_content_2).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to modify obj_B");
    println!("  • Modified obj_B");
    
    // Check change state after modify
    let change = server.database().index().get_change(&new_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    assert_eq!(change.modified_objects.len(), 1, "Should have 1 modified object");
    assert_eq!(change.modified_objects[0].name, "obj_B");
    println!("    ✓ obj_B in modified_objects");
    
    // Delete obj_C (should go in deleted_objects)
    let delete_request = json!({
        "operation": "object/delete",
        "args": ["obj_C"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(delete_request))
        .await
        .expect("Failed to delete obj_C");
    println!("  • Deleted obj_C");
    
    // Check change state after delete
    let change = server.database().index().get_change(&new_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    assert_eq!(change.deleted_objects.len(), 1, "Should have 1 deleted object");
    assert_eq!(change.deleted_objects[0].name, "obj_C");
    println!("    ✓ obj_C in deleted_objects");
    
    // Add new obj_D (should go in added_objects)
    let update_request_d = json!({
        "operation": "object/update",
        "args": ["obj_D", serde_json::to_string(&object_content).unwrap()]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_d))
        .await
        .expect("Failed to add obj_D");
    println!("  • Added obj_D");
    
    // Check change state after add
    let change = server.database().index().get_change(&new_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    assert_eq!(change.added_objects.len(), 1, "Should have 1 added object");
    assert_eq!(change.added_objects[0].name, "obj_D");
    println!("    ✓ obj_D in added_objects");
    
    println!("\n✅ Verified all objects are in correct state fields:");
    println!("   • renamed_objects: [obj_A -> obj_A_renamed]");
    println!("   • modified_objects: [obj_B]");
    println!("   • deleted_objects: [obj_C]");
    println!("   • added_objects: [obj_D]");
    
    // Step 4: Verify final object list
    println!("\nStep 4: Verifying final object list...");
    let list_request = json!({
        "operation": "object/list",
        "args": []
    });
    
    let response = make_request("POST", &format!("{}/rpc", base_url), Some(list_request))
        .await
        .expect("Failed to list objects");
    
    assert!(response["success"].as_bool().unwrap_or(false), "List should succeed");
    
    let result_str = response["result"].as_str().unwrap();
    let list_response: serde_json::Value = serde_json::from_str(result_str)
        .expect("Failed to parse list response");
    
    let objects = list_response["objects"].as_array().unwrap();
    
    // Should have 4 objects (A renamed to A_renamed, B modified, C deleted, D added, E unchanged)
    assert_eq!(objects.len(), 4, "Should have 4 objects (A renamed, B modified, D added, E unchanged, C deleted)");
    
    let object_names: Vec<String> = objects.iter()
        .map(|obj| obj["name"].as_str().unwrap().to_string())
        .collect();
    
    assert!(object_names.contains(&"obj_A_renamed".to_string()), "Should contain obj_A_renamed");
    assert!(!object_names.contains(&"obj_A".to_string()), "Should NOT contain obj_A (renamed)");
    assert!(object_names.contains(&"obj_B".to_string()), "Should contain obj_B (modified)");
    assert!(!object_names.contains(&"obj_C".to_string()), "Should NOT contain obj_C (deleted)");
    assert!(object_names.contains(&"obj_D".to_string()), "Should contain obj_D (added)");
    assert!(object_names.contains(&"obj_E".to_string()), "Should contain obj_E (unchanged)");
    
    println!("✅ Final objects: {:?}", object_names);
    println!("   Expected: [obj_A_renamed, obj_B, obj_D, obj_E]");
    
    println!("\n✅ Test passed: object/list handles complex scenarios with proper state tracking");
}

