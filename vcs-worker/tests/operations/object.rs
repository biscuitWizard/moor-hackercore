//! Integration tests for object operations (create, update, get, etc.)

use crate::common::*;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_object_create_and_verify() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test server started at: {}", base_url);
    println!("Database path: {:?}", server.db_path());
    
    // Step 1: Verify no active change initially
    println!("\nStep 1: Verifying no active change in database initially...");
    
    let top_change = server.database().index().get_top_change()
        .expect("Failed to get top change");
    
    assert!(
        top_change.is_none(),
        "Expected no top change initially, but found: {:?}",
        top_change
    );
    
    println!("✅ Confirmed: No active change in database initially");
    
    // Step 2: Create an object update
    println!("\nStep 2: Creating object update for test object...");
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    let update_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(update_request),
    )
    .await
    .expect("Failed to update object");
    
    println!("Update response: {}", serde_json::to_string_pretty(&update_response).unwrap());
    
    // Verify the update was successful via API
    assert!(
        update_response["success"].as_bool().unwrap_or(false),
        "Object update should succeed, got: {}",
        update_response
    );
    
    // Now verify the internal state directly
    // The API joins the lines with "\n", so we need to match that for hash calculation
    let object_dump = object_content.join("\n");
    let sha256_hash = TestServer::calculate_sha256(&object_dump);
    
    println!("Calculated SHA256: {}", sha256_hash);
    
    // Check that the object exists in the objects provider with this hash
    let stored_object = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object from provider");
    
    assert!(
        stored_object.is_some(),
        "Object with hash {} should exist in objects provider",
        sha256_hash
    );
    
    println!("✅ Confirmed: Object exists in objects provider with correct hash");
    
    // Check that the ref was created
    let object_ref = server.database().refs().get_ref(object_name, None)
        .expect("Failed to get object ref");
    
    assert!(
        object_ref.is_some(),
        "Object ref for '{}' should exist",
        object_name
    );
    
    assert_eq!(
        object_ref.unwrap(),
        sha256_hash,
        "Object ref should point to the correct SHA256 hash"
    );
    
    println!("✅ Confirmed: Object ref points to correct hash");
    
    // Step 3: Verify change tracks the object
    println!("\nStep 3: Verifying change tracks the object...");
    
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change after creating object");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Verify the change has our object
    assert_eq!(
        change.status,
        ChangeStatus::Local,
        "Change should be in Local status"
    );
    
    let object_in_change = change.added_objects.iter()
        .any(|obj| obj.name == object_name);
    
    assert!(
        object_in_change,
        "Object '{}' should be in added_objects list, found: {:?}",
        object_name,
        change.added_objects
    );
    
    println!("✅ Confirmed: Change exists with our object in added_objects list");
    
    // Step 4: Verify stored content matches submission exactly
    println!("\nStep 4: Verifying stored content matches submission...");
    
    let stored_content = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object")
        .expect("Object should exist");
    
    assert_eq!(
        stored_content,
        object_dump,
        "Stored content should match exactly what was submitted"
    );
    
    println!("✅ Confirmed: Stored content matches submission exactly");
    
    println!("\n✅ Test completed successfully!");
}

#[tokio::test]
async fn test_object_content_integrity() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test server started at: {}", base_url);
    
    // Create an object with specific content
    let object_name = "detailed_test_object";
    let object_dump = load_moo_file("detailed_test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    let update_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(update_request),
    )
    .await
    .expect("Failed to update object");
    
    assert!(
        update_response["success"].as_bool().unwrap_or(false),
        "Object update should succeed"
    );
    
    // Verify directly from database
    // The API joins the lines with "\n", so calculate hash on that
    let object_dump = object_content.join("\n");
    let sha256_hash = TestServer::calculate_sha256(&object_dump);
    
    println!("Calculated SHA256: {}", sha256_hash);
    
    // Check that the object exists in the objects provider
    let stored_content = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object from provider")
        .expect("Object should exist in objects provider");
    
    // Verify exact content match
    assert_eq!(
        stored_content,
        object_dump,
        "Stored content should match exactly"
    );
    
    println!("✅ Stored content matches exactly");
    
    // Verify the ref points to the correct hash
    let ref_hash = server.database().refs().get_ref(object_name, None)
        .expect("Failed to get ref")
        .expect("Ref should exist");
    
    assert_eq!(
        ref_hash,
        sha256_hash,
        "Ref should point to the correct SHA256 hash"
    );
    
    println!("✅ Ref points to correct hash");
    
    // Verify specific fields in the content
    assert!(
        stored_content.contains("Detailed Test Object"),
        "Content should contain object name"
    );
    assert!(
        stored_content.contains("#12345"),
        "Content should contain object ID"
    );
    assert!(
        stored_content.contains("readable: true"),
        "Content should contain readable flag"
    );
    
    println!("✅ All content fields verified");
    
    // Verify the object is in a change
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let object_in_change = change.added_objects.iter()
        .any(|obj| obj.name == object_name);
    
    assert!(
        object_in_change,
        "Object should be tracked in the change"
    );
    
    println!("✅ Object is tracked in change");
    
    println!("\n✅ Content verification test passed!");
}

#[tokio::test]
async fn test_multiple_objects_persistence() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    let db_path = server.db_path();
    
    println!("Test server started at: {}", base_url);
    println!("Database path: {:?}", db_path);
    
    // Step 1: Create multiple object updates
    println!("\nStep 1: Creating multiple object updates...");
    
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        let object_dump = load_moo_file(&format!("test_object_{}.moo", i));
        let object_content = moo_to_lines(&object_dump);
        
        let update_request = json!({
            "operation": "object/update",
            "args": [
                object_name,
                serde_json::to_string(&object_content).unwrap()
            ]
        });
        
        let update_response = make_request(
            "POST",
            &format!("{}/rpc", base_url),
            Some(update_request),
        )
        .await
        .expect("Failed to update object");
        
        assert!(
            update_response["success"].as_bool().unwrap_or(false),
            "Object update {} should succeed",
            i
        );
        
        // The API joins the lines with "\n", so calculate hash on that
        let object_dump = object_content.join("\n");
        let sha256_hash = TestServer::calculate_sha256(&object_dump);
        
        // Verify the object exists in the database
        let stored = server.database().objects().get(&sha256_hash)
            .expect("Failed to query objects provider");
        assert!(stored.is_some(), "Object {} should be stored", i);
        
        // Verify the ref exists
        let ref_hash = server.database().refs().get_ref(&object_name, None)
            .expect("Failed to query refs provider");
        assert_eq!(ref_hash, Some(sha256_hash.clone()), "Ref should point to correct hash");
        
        println!("Created test_object_{} with hash {}", i, sha256_hash);
    }
    
    // Step 2: Verify the database was built and contains the changes
    println!("\nStep 2: Verifying database persistence...");
    
    // Check that the database directory was created
    assert!(db_path.exists(), "Database directory should exist");
    
    // The fjall database should have created subdirectories
    let db_contents: Vec<_> = std::fs::read_dir(&db_path)
        .expect("Failed to read database directory")
        .filter_map(|entry| entry.ok())
        .collect();
    
    assert!(
        !db_contents.is_empty(),
        "Database directory should contain files"
    );
    
    println!("Database contains {} entries", db_contents.len());
    
    // Verify all 3 objects are in the objects provider
    let objects_count = server.database().objects().count();
    assert!(
        objects_count >= 3,
        "Objects provider should contain at least 3 objects, found {}",
        objects_count
    );
    
    println!("✅ Confirmed: {} objects stored in objects provider", objects_count);
    
    // Step 3: Verify the change is at the top of the index
    println!("\nStep 3: Verifying change is tracked in index...");
    
    // Check database state directly
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    println!("Top change: {} (status: {:?})", top_change_id, change.status);
    
    // Verify all 3 objects are in the change
    assert_eq!(
        change.added_objects.len(),
        3,
        "Change should have 3 added objects, found: {:?}",
        change.added_objects
    );
    
    // Verify each object is in the change
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        let found = change.added_objects.iter()
            .any(|obj| obj.name == object_name);
        assert!(found, "Object {} should be in change", object_name);
    }
    
    println!("✅ Confirmed: All 3 objects are in the top change");
    
    // Step 4: Verify the DB contains the object updates in top of changes
    println!("\nStep 4: Verifying DB contains object updates at top of changes...");
    
    // Check that it's a Local change (top of changes)
    assert_eq!(
        change.status,
        ChangeStatus::Local,
        "Top change should be Local status"
    );
    
    // Verify the change order - our change should be at the top
    let all_changes = server.database().index().get_change_order()
        .expect("Failed to get change order");
    
    if !all_changes.is_empty() {
        assert_eq!(
            all_changes[all_changes.len() - 1],
            top_change_id,
            "Our change should be at the top (end) of the change order"
        );
        println!("✅ Confirmed: Change is at the top of the change order");
    }
    
    // Verify we can retrieve each object by its ref
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        let ref_hash = server.database().refs().get_ref(&object_name, None)
            .expect("Failed to get ref");
        assert!(ref_hash.is_some(), "Ref for {} should exist", object_name);
        
        let content = server.database().objects().get(&ref_hash.unwrap())
            .expect("Failed to get object");
        assert!(content.is_some(), "Object content for {} should exist", object_name);
        assert!(
            content.unwrap().contains(&format!("Test Object {}", i)),
            "Object {} should contain correct name",
            i
        );
    }
    
    println!("✅ Confirmed: All objects retrievable and contain correct content");
    
    println!("\n✅ Test completed successfully!");
}

#[tokio::test]
async fn test_object_update_twice_in_same_change_trims_unused_sha256() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test server started at: {}", base_url);
    
    // Step 1: Create an object update
    println!("\nStep 1: Creating initial object update...");
    let object_name = "test_object_trimming";
    let object_dump_v1 = load_moo_file("test_object.moo");
    let object_content_v1 = moo_to_lines(&object_dump_v1);
    
    let update_request_v1 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content_v1).unwrap()
        ]
    });
    
    let update_response_v1 = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(update_request_v1),
    )
    .await
    .expect("Failed to update object v1");
    
    assert!(
        update_response_v1["success"].as_bool().unwrap_or(false),
        "Object update v1 should succeed"
    );
    
    // Calculate SHA256 for the first version
    let object_dump_v1 = object_content_v1.join("\n");
    let sha256_v1 = TestServer::calculate_sha256(&object_dump_v1);
    
    println!("First update SHA256: {}", sha256_v1);
    
    // Verify the first version exists in objects provider
    let stored_v1 = server.database().objects().get(&sha256_v1)
        .expect("Failed to get object v1");
    assert!(
        stored_v1.is_some(),
        "First version should exist in objects provider"
    );
    
    // Verify ref points to first version
    let ref_sha_v1 = server.database().refs().get_ref(object_name, None)
        .expect("Failed to get ref v1")
        .expect("Ref should exist");
    assert_eq!(ref_sha_v1, sha256_v1, "Ref should point to first version");
    
    // Get the version number
    let version_v1 = server.database().refs().get_current_version(object_name)
        .expect("Failed to get current version")
        .expect("Version should exist");
    
    println!("Initial version: {}", version_v1);
    
    // Verify object is in the change
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change_v1 = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    assert!(
        change_v1.added_objects.iter().any(|obj| obj.name == object_name),
        "Object should be in added_objects"
    );
    
    println!("✅ First update completed successfully");
    
    // Step 2: Update the same object again in the same change with different content
    println!("\nStep 2: Updating the same object again in the same change...");
    let object_dump_v2 = load_moo_file("detailed_test_object.moo");
    let object_content_v2 = moo_to_lines(&object_dump_v2);
    
    let update_request_v2 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content_v2).unwrap()
        ]
    });
    
    let update_response_v2 = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(update_request_v2),
    )
    .await
    .expect("Failed to update object v2");
    
    assert!(
        update_response_v2["success"].as_bool().unwrap_or(false),
        "Object update v2 should succeed"
    );
    
    // Calculate SHA256 for the second version
    let object_dump_v2 = object_content_v2.join("\n");
    let sha256_v2 = TestServer::calculate_sha256(&object_dump_v2);
    
    println!("Second update SHA256: {}", sha256_v2);
    
    // Verify the SHA256s are different
    assert_ne!(
        sha256_v1,
        sha256_v2,
        "The two versions should have different SHA256s"
    );
    
    // Step 3: Verify the old SHA256 was deleted from objects provider
    println!("\nStep 3: Verifying old SHA256 was cleaned up...");
    
    let stored_v1_after = server.database().objects().get(&sha256_v1)
        .expect("Failed to check for old SHA256");
    
    assert!(
        stored_v1_after.is_none(),
        "Old SHA256 '{}' should have been deleted from objects provider",
        sha256_v1
    );
    
    println!("✅ Old SHA256 was successfully deleted");
    
    // Step 4: Verify the new SHA256 exists in objects provider
    println!("\nStep 4: Verifying new SHA256 exists...");
    
    let stored_v2 = server.database().objects().get(&sha256_v2)
        .expect("Failed to get object v2");
    
    assert!(
        stored_v2.is_some(),
        "New SHA256 '{}' should exist in objects provider",
        sha256_v2
    );
    
    println!("✅ New SHA256 exists in objects provider");
    
    // Step 5: Verify ref points to new SHA256
    println!("\nStep 5: Verifying ref points to new SHA256...");
    
    let ref_sha_v2 = server.database().refs().get_ref(object_name, None)
        .expect("Failed to get ref v2")
        .expect("Ref should exist");
    
    assert_eq!(
        ref_sha_v2,
        sha256_v2,
        "Ref should point to new SHA256"
    );
    
    println!("✅ Ref points to new SHA256");
    
    // Step 6: Verify version number did NOT increment
    println!("\nStep 6: Verifying version number did not increment...");
    
    let version_v2 = server.database().refs().get_current_version(object_name)
        .expect("Failed to get current version")
        .expect("Version should exist");
    
    assert_eq!(
        version_v1,
        version_v2,
        "Version should not have incremented (stayed at {})",
        version_v1
    );
    
    println!("✅ Version number remained the same: {}", version_v2);
    
    // Step 7: Verify the change still only tracks the object once (not duplicated)
    println!("\nStep 7: Verifying change tracking...");
    
    let change_v2 = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let object_count = change_v2.added_objects.iter()
        .filter(|obj| obj.name == object_name)
        .count();
    
    assert_eq!(
        object_count,
        1,
        "Object should only appear once in added_objects"
    );
    
    println!("✅ Object appears exactly once in change");
    
    println!("\n✅ Test completed successfully! Old SHA256 was trimmed, version stayed the same, and ref points to new content.");
}

