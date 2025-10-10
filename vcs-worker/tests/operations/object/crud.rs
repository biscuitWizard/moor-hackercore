//! Integration tests for object operations (create, update, get, etc.)

use crate::common::*;
use moor_vcs_worker::types::{ChangeStatus, VcsObjectType};

#[tokio::test]
async fn test_object_create_and_verify() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Create and verify object");
    
    // Step 1: Verify no active change initially
    println!("\nStep 1: Verifying no active change initially...");
    db.assert_no_top_change();
    println!("✅ No active change initially");
    
    // Step 2: Create an object update
    println!("\nStep 2: Creating object...");
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    client.object_update(object_name, object_content.clone())
        .await
        .expect("Failed to update object")
        .assert_success("Object update");
    
    println!("✅ Object created successfully");
    
    // Verify the object exists with correct hash
    let object_dump_str = object_content.join("\n");
    let sha256_hash = TestServer::calculate_sha256(&object_dump_str);
    println!("Calculated SHA256: {}", sha256_hash);
    
    db.assert_sha256_exists(&sha256_hash);
    println!("✅ Object exists in objects provider with correct hash");
    
    // Verify ref points to correct hash
    let ref_hash = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    assert_eq!(ref_hash, sha256_hash, "Ref should point to correct SHA256");
    println!("✅ Object ref points to correct hash");
    
    // Step 3: Verify change tracks the object
    println!("\nStep 3: Verifying change tracks the object...");
    
    let (_, change) = db.require_top_change();
    assert_eq!(change.status, ChangeStatus::Local, "Change should be Local");
    
    db.assert_object_in_top_change(object_name);
    println!("✅ Change exists with object in added_objects list");
    
    // Step 4: Verify stored content matches submission exactly
    println!("\nStep 4: Verifying stored content matches submission...");
    
    let stored_content = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object")
        .expect("Object should exist");
    
    assert_eq!(stored_content, object_dump_str, "Stored content should match submission");
    println!("✅ Stored content matches submission exactly");
    
    println!("\n✅ Test completed successfully!");
}

#[tokio::test]
async fn test_object_content_integrity() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Object content integrity");
    
    // Create an object with specific content
    let object_name = "detailed_test_object";
    
    client.object_update_from_file(object_name, "detailed_test_object.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Object update");
    
    // Verify directly from database
    let object_dump = load_moo_file("detailed_test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    let object_dump_str = object_content.join("\n");
    let sha256_hash = TestServer::calculate_sha256(&object_dump_str);
    
    println!("Calculated SHA256: {}", sha256_hash);
    
    // Check that the object exists in the objects provider
    let stored_content = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object from provider")
        .expect("Object should exist in objects provider");
    
    // Verify exact content match
    assert_eq!(stored_content, object_dump_str, "Stored content should match exactly");
    println!("✅ Stored content matches exactly");
    
    // Verify the ref points to the correct hash
    let ref_hash = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    assert_eq!(ref_hash, sha256_hash, "Ref should point to correct SHA256");
    println!("✅ Ref points to correct hash");
    
    // Verify specific fields in the content
    assert!(stored_content.contains("Detailed Test Object"), "Content should contain object name");
    assert!(stored_content.contains("#12345"), "Content should contain object ID");
    assert!(stored_content.contains("readable: true"), "Content should contain readable flag");
    println!("✅ All content fields verified");
    
    // Verify the object is in a change
    db.assert_object_in_top_change(object_name);
    println!("✅ Object is tracked in change");
    
    println!("\n✅ Content verification test passed!");
}

#[tokio::test]
async fn test_multiple_objects_persistence() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    let db_path = server.db_path();
    
    println!("Test: Multiple objects persistence");
    
    // Step 1: Create multiple object updates
    println!("\nStep 1: Creating multiple objects...");
    
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        
        client.object_update_from_file(&object_name, &format!("test_object_{}.moo", i))
            .await
            .expect("Failed to update object")
            .assert_success("Object update");
        
        // Verify the object and ref exist
        let ref_hash = db.assert_ref_exists(VcsObjectType::MooObject, &object_name);
        db.assert_sha256_exists(&ref_hash);
        
        println!("Created {} with hash {}", object_name, ref_hash);
    }
    
    // Step 2: Verify the database was built and contains the changes
    println!("\nStep 2: Verifying database persistence...");
    
    assert!(db_path.exists(), "Database directory should exist");
    
    let db_contents: Vec<_> = std::fs::read_dir(&db_path)
        .expect("Failed to read database directory")
        .filter_map(|entry| entry.ok())
        .collect();
    
    assert!(!db_contents.is_empty(), "Database directory should contain files");
    println!("Database contains {} entries", db_contents.len());
    
    let objects_count = server.database().objects().count();
    assert!(objects_count >= 3, "Should have at least 3 objects, found {}", objects_count);
    println!("✅ {} objects stored in objects provider", objects_count);
    
    // Step 3: Verify the change is at the top of the index
    println!("\nStep 3: Verifying change is tracked in index...");
    
    let (top_change_id, change) = db.require_top_change();
    println!("Top change: {} (status: {:?})", top_change_id, change.status);
    
    assert_eq!(change.added_objects.len(), 3, "Should have 3 added objects");
    
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        let found = change.added_objects.iter().any(|obj| obj.name == object_name);
        assert!(found, "Object {} should be in change", object_name);
    }
    
    println!("✅ All 3 objects are in the top change");
    
    // Step 4: Verify the DB contains the object updates at top of changes
    println!("\nStep 4: Verifying DB contains object updates...");
    
    assert_eq!(change.status, ChangeStatus::Local, "Change should be Local");
    
    let all_changes = server.database().index().get_change_order()
        .expect("Failed to get change order");
    
    if !all_changes.is_empty() {
        assert_eq!(
            all_changes[all_changes.len() - 1],
            top_change_id,
            "Our change should be at the top"
        );
        println!("✅ Change is at the top of the change order");
    }
    
    // Verify we can retrieve each object by its ref
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        let ref_hash = db.assert_ref_exists(VcsObjectType::MooObject, &object_name);
        
        let content = server.database().objects().get(&ref_hash)
            .expect("Failed to get object")
            .expect("Object content should exist");
        
        assert!(content.contains(&format!("Test Object {}", i)), "Object {} should contain correct name", i);
    }
    
    println!("✅ All objects retrievable and contain correct content");
    println!("\n✅ Test completed successfully!");
}

#[tokio::test]
async fn test_object_update_twice_in_same_change_trims_unused_sha256() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Update twice in same change trims unused SHA256");
    
    // Step 1: Create an object update
    println!("\nStep 1: Creating initial object update...");
    let object_name = "test_object_trimming";
    let object_dump_v1 = load_moo_file("test_object.moo");
    let object_content_v1 = moo_to_lines(&object_dump_v1);
    
    client.object_update(object_name, object_content_v1.clone())
        .await
        .expect("Failed to update object v1")
        .assert_success("Object update v1");
    
    let object_dump_v1_str = object_content_v1.join("\n");
    let sha256_v1 = TestServer::calculate_sha256(&object_dump_v1_str);
    println!("First update SHA256: {}", sha256_v1);
    
    db.assert_sha256_exists(&sha256_v1);
    
    let ref_sha_v1 = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    assert_eq!(ref_sha_v1, sha256_v1, "Ref should point to first version");
    
    let version_v1 = server.database().refs().get_current_version(VcsObjectType::MooObject, object_name)
        .expect("Failed to get current version")
        .expect("Version should exist");
    println!("Initial version: {}", version_v1);
    
    let (top_change_id, change_v1) = db.require_top_change();
    assert!(change_v1.added_objects.iter().any(|obj| obj.name == object_name), "Object should be in added_objects");
    println!("✅ First update completed successfully");
    
    // Step 2: Update the same object again in the same change with different content
    println!("\nStep 2: Updating the same object again in same change...");
    
    client.object_update_from_file(object_name, "detailed_test_object.moo")
        .await
        .expect("Failed to update object v2")
        .assert_success("Object update v2");
    
    let object_dump_v2 = load_moo_file("detailed_test_object.moo");
    let object_content_v2 = moo_to_lines(&object_dump_v2);
    let object_dump_v2_str = object_content_v2.join("\n");
    let sha256_v2 = TestServer::calculate_sha256(&object_dump_v2_str);
    println!("Second update SHA256: {}", sha256_v2);
    
    assert_ne!(sha256_v1, sha256_v2, "The two versions should have different SHA256s");
    
    // Step 3: Verify the old SHA256 was deleted from objects provider
    println!("\nStep 3: Verifying old SHA256 was cleaned up...");
    db.assert_sha256_not_exists(&sha256_v1);
    println!("✅ Old SHA256 was successfully deleted");
    
    // Step 4: Verify the new SHA256 exists in objects provider
    println!("\nStep 4: Verifying new SHA256 exists...");
    db.assert_sha256_exists(&sha256_v2);
    println!("✅ New SHA256 exists in objects provider");
    
    // Step 5: Verify ref points to new SHA256
    println!("\nStep 5: Verifying ref points to new SHA256...");
    let ref_sha_v2 = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    assert_eq!(ref_sha_v2, sha256_v2, "Ref should point to new SHA256");
    println!("✅ Ref points to new SHA256");
    
    // Step 6: Verify version number did NOT increment
    println!("\nStep 6: Verifying version number did not increment...");
    let version_v2 = server.database().refs().get_current_version(VcsObjectType::MooObject, object_name)
        .expect("Failed to get current version")
        .expect("Version should exist");
    assert_eq!(version_v1, version_v2, "Version should not have incremented");
    println!("✅ Version number remained the same: {}", version_v2);
    
    // Step 7: Verify the change still only tracks the object once (not duplicated)
    println!("\nStep 7: Verifying change tracking...");
    let change_v2 = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let object_count = change_v2.added_objects.iter()
        .filter(|obj| obj.name == object_name)
        .count();
    assert_eq!(object_count, 1, "Object should only appear once in added_objects");
    println!("✅ Object appears exactly once in change");
    
    println!("\n✅ Test completed! Old SHA256 was trimmed, version stayed same, ref points to new content.");
}

