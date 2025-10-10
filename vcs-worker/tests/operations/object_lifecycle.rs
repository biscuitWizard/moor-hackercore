//! Integration tests for object lifecycle and SHA256 management
//!
//! These tests verify proper handling of:
//! - Duplicate content detection
//! - SHA256 cleanup when content changes
//! - Version management
//! - Ref updates

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_duplicate_content_detection() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: Duplicate content detection");
    
    // Step 1: Create an object
    let object_name = "test_duplicate";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    let content_str = object_content.join("\n");
    let sha256_hash = TestServer::calculate_sha256(&content_str);
    
    println!("\nStep 1: Creating object with content A...");
    let update_request = json!({
        "operation": "object/update",
        "args": [object_name, serde_json::to_string(&object_content).unwrap()]
    });
    
    let response1 = make_request("POST", &format!("{}/rpc", base_url), Some(update_request.clone()))
        .await
        .expect("Failed to update object");
    
    assert!(response1["success"].as_bool().unwrap_or(false));
    println!("✅ Object created with SHA256: {}", sha256_hash);
    
    // Verify SHA256 is stored
    let stored1 = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object");
    assert!(stored1.is_some(), "SHA256 should be stored");
    
    // Verify ref points to this SHA256
    let ref1 = server.database().refs().get_ref(VcsObjectType::MooObject, object_name, None)
        .expect("Failed to get ref");
    assert_eq!(ref1, Some(sha256_hash.clone()));
    
    // Get initial objects count
    let initial_count = server.database().objects().count();
    println!("Objects in DB: {}", initial_count);
    
    // Step 2: Submit the EXACT same content again (duplicate)
    println!("\nStep 2: Submitting duplicate content...");
    let response2 = make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    println!("Response: {}", serde_json::to_string_pretty(&response2).unwrap());
    
    // Should detect duplicate and skip
    assert!(response2["success"].as_bool().unwrap_or(false));
    assert!(response2["result"].as_str().unwrap_or("").contains("unchanged"));
    
    // Verify objects count hasn't increased (no duplicate storage)
    let final_count = server.database().objects().count();
    assert_eq!(final_count, initial_count, "Should not store duplicate SHA256");
    
    println!("✅ Duplicate detected, no extra storage");
    
    // Verify ref still points to same SHA256
    let ref2 = server.database().refs().get_ref(VcsObjectType::MooObject, object_name, None)
        .expect("Failed to get ref");
    assert_eq!(ref2, Some(sha256_hash));
    
    println!("\n✅ Test passed: Duplicate content properly detected");
}

#[tokio::test]
async fn test_sha256_cleanup_on_content_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: SHA256 cleanup when content changes");
    
    let object_name = "test_cleanup";
    
    // Step 1: Create object with content A
    println!("\nStep 1: Creating object with content A...");
    let content_a = load_moo_file("test_object.moo");
    let content_a_lines = moo_to_lines(&content_a);
    let content_a_str = content_a_lines.join("\n");
    let sha256_a = TestServer::calculate_sha256(&content_a_str);
    
    let update_a = json!({
        "operation": "object/update",
        "args": [object_name, serde_json::to_string(&content_a_lines).unwrap()]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_a))
        .await
        .expect("Failed to update with content A");
    
    println!("Created with SHA256_A: {}", sha256_a);
    
    // Verify SHA256_A is stored
    assert!(server.database().objects().get(&sha256_a).expect("Failed to get object").is_some());
    
    // Verify ref points to SHA256_A
    let ref_a = server.database().refs().get_ref(VcsObjectType::MooObject, object_name, None).expect("Failed to get ref");
    assert_eq!(ref_a, Some(sha256_a.clone()));
    
    let count_after_a = server.database().objects().count();
    println!("Objects after A: {}", count_after_a);
    
    // Step 2: Update object with content B (different content)
    println!("\nStep 2: Updating object with content B...");
    let content_b = load_moo_file("detailed_test_object.moo");
    let content_b_lines = moo_to_lines(&content_b);
    let content_b_str = content_b_lines.join("\n");
    let sha256_b = TestServer::calculate_sha256(&content_b_str);
    
    assert_ne!(sha256_a, sha256_b, "Content A and B should have different hashes");
    
    let update_b = json!({
        "operation": "object/update",
        "args": [object_name, serde_json::to_string(&content_b_lines).unwrap()]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_b))
        .await
        .expect("Failed to update with content B");
    
    println!("Updated with SHA256_B: {}", sha256_b);
    
    // Verify SHA256_B is stored
    assert!(server.database().objects().get(&sha256_b).expect("Failed to get object").is_some());
    
    // Verify ref now points to SHA256_B
    let ref_b = server.database().refs().get_ref(VcsObjectType::MooObject, object_name, None).expect("Failed to get ref");
    assert_eq!(ref_b, Some(sha256_b.clone()));
    
    let count_after_b = server.database().objects().count();
    println!("Objects after B: {}", count_after_b);
    
    // CRITICAL CHECK: SHA256_A should be deleted since it's no longer referenced
    // and not in any committed history
    let sha256_a_exists = server.database().objects().get(&sha256_a).expect("Failed to check SHA256_A");
    
    if sha256_a_exists.is_some() {
        eprintln!("⚠️  ISSUE DETECTED: Old SHA256_A still exists after being replaced!");
        eprintln!("   SHA256_A: {}", sha256_a);
        eprintln!("   SHA256_B: {}", sha256_b);
        eprintln!("   This is an orphaned SHA256 that should have been cleaned up.");
        eprintln!("   Objects count: {} (should be {} if cleanup worked)", 
                  count_after_b, count_after_a);
        
        // This test will fail, exposing the issue
        panic!("SHA256 cleanup is not working! Old hash still exists: {}", sha256_a);
    } else {
        println!("✅ Old SHA256_A properly deleted");
    }
    
    // Verify object count hasn't increased (old SHA was cleaned up)
    assert_eq!(count_after_b, count_after_a, 
               "Object count should remain same (old SHA deleted, new SHA added)");
    
    println!("\n✅ Test passed: SHA256 cleanup working correctly");
}

#[tokio::test]
async fn test_content_flip_flop_version_handling() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: Content flip-flop and version handling");
    
    let object_name = "test_flipflop";
    
    // Content A
    let content_a = load_moo_file("test_object.moo");
    let content_a_lines = moo_to_lines(&content_a);
    let content_a_str = content_a_lines.join("\n");
    let sha256_a = TestServer::calculate_sha256(&content_a_str);
    
    // Content B
    let content_b = load_moo_file("detailed_test_object.moo");
    let content_b_lines = moo_to_lines(&content_b);
    let content_b_str = content_b_lines.join("\n");
    let sha256_b = TestServer::calculate_sha256(&content_b_str);
    
    println!("\nSHA256_A: {}", sha256_a);
    println!("SHA256_B: {}", sha256_b);
    
    // Step 1: Create with content A
    println!("\nStep 1: Create with content A...");
    make_request("POST", &format!("{}/rpc", base_url), Some(json!({
        "operation": "object/update",
        "args": [object_name, serde_json::to_string(&content_a_lines).unwrap()]
    }))).await.expect("Failed to create object");
    
    let version1 = server.database().refs().get_ref(VcsObjectType::MooObject, object_name, None)
        .expect("Failed to get ref")
        .expect("Ref should exist");
    println!("Version after A: ref points to {}", version1);
    assert_eq!(version1, sha256_a);
    
    // Step 2: Update to content B
    println!("\nStep 2: Update to content B...");
    make_request("POST", &format!("{}/rpc", base_url), Some(json!({
        "operation": "object/update",
        "args": [object_name, serde_json::to_string(&content_b_lines).unwrap()]
    }))).await.expect("Failed to update with content B");
    
    let version2 = server.database().refs().get_ref(VcsObjectType::MooObject, object_name, None)
        .expect("Failed to get ref")
        .expect("Ref should exist");
    println!("Version after B: ref points to {}", version2);
    assert_eq!(version2, sha256_b);
    
    // Step 3: Switch back to content A
    println!("\nStep 3: Switch back to content A...");
    make_request("POST", &format!("{}/rpc", base_url), Some(json!({
        "operation": "object/update",
        "args": [object_name, serde_json::to_string(&content_a_lines).unwrap()]
    }))).await.expect("Failed to switch back to content A");
    
    let version3 = server.database().refs().get_ref(VcsObjectType::MooObject, object_name, None)
        .expect("Failed to get ref")
        .expect("Ref should exist");
    println!("Version after A again: ref points to {}", version3);
    
    // CRITICAL: Should point to SHA256_A
    // The version number behavior depends on whether we're in the same change
    // Since we're in a Local change, switching back shouldn't increment version
    assert_eq!(version3, sha256_a, "Ref should point back to SHA256_A");
    
    // Check which SHA256s exist
    let a_exists = server.database().objects().get(&sha256_a).expect("Failed to check A").is_some();
    let b_exists = server.database().objects().get(&sha256_b).expect("Failed to check B").is_some();
    
    println!("\nSHA256 existence:");
    println!("  SHA256_A exists: {}", a_exists);
    println!("  SHA256_B exists: {}", b_exists);
    
    // Since we switched back to A, B should be deleted (not in history)
    assert!(a_exists, "SHA256_A should exist (current content)");
    
    if b_exists {
        eprintln!("⚠️  ISSUE: SHA256_B still exists after switching back to A");
        eprintln!("   This orphaned SHA256 should have been cleaned up");
        panic!("SHA256_B should be deleted when switching back to A");
    } else {
        println!("✅ SHA256_B properly deleted when switching back");
    }
    
    // Verify the change tracks this correctly
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have change");
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    println!("\nChange status: {:?}", change.status);
    assert_eq!(change.status, ChangeStatus::Local);
    
    println!("\n✅ Test passed: Content flip-flop handled correctly");
}

#[tokio::test]
async fn test_sha256_not_deleted_if_in_history() {
    let server = TestServer::start().await.expect("Failed to start test server");
    
    println!("Test: SHA256 not deleted if referenced in history");
    println!("⚠️  This test is a placeholder for future implementation");
    println!("   When changes are committed to history, the SHA256s should be preserved");
    println!("   even if the object is later updated with new content.");
    
    // TODO: This test requires:
    // 1. Create object with content A (SHA256_A)
    // 2. Commit the change to history
    // 3. Update object with content B (SHA256_B)
    // 4. Verify SHA256_A still exists (it's in history)
    // 5. Verify SHA256_B exists (current content)
    
    // For now, just verify the concept
    let _db = server.database();
    
    println!("\n⚠️  Test skipped: Requires change commit functionality");
    println!("   This will be critical once we have change submission/merge");
}

#[tokio::test]
async fn test_multiple_objects_same_content() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test: Multiple objects with same content share SHA256");
    
    // Use the same content for two different objects
    let content = load_moo_file("test_object.moo");
    let content_lines = moo_to_lines(&content);
    let content_str = content_lines.join("\n");
    let sha256 = TestServer::calculate_sha256(&content_str);
    
    println!("\nShared SHA256: {}", sha256);
    
    // Create object 1
    println!("\nCreating object_1 with shared content...");
    make_request("POST", &format!("{}/rpc", base_url), Some(json!({
        "operation": "object/update",
        "args": ["object_1", serde_json::to_string(&content_lines).unwrap()]
    }))).await.expect("Failed to create object_1");
    
    let ref1 = server.database().refs().get_ref(VcsObjectType::MooObject, "object_1", None).expect("Failed to get ref1");
    assert_eq!(ref1, Some(sha256.clone()));
    
    let count_after_1 = server.database().objects().count();
    
    // Create object 2 with SAME content
    println!("Creating object_2 with same content...");
    make_request("POST", &format!("{}/rpc", base_url), Some(json!({
        "operation": "object/update",
        "args": ["object_2", serde_json::to_string(&content_lines).unwrap()]
    }))).await.expect("Failed to create object_2");
    
    let ref2 = server.database().refs().get_ref(VcsObjectType::MooObject, "object_2", None).expect("Failed to get ref2");
    assert_eq!(ref2, Some(sha256.clone()));
    
    let count_after_2 = server.database().objects().count();
    
    // Should NOT have duplicated the SHA256
    assert_eq!(count_after_2, count_after_1, 
               "Should reuse SHA256, not duplicate");
    println!("✅ SHA256 properly reused for both objects");
    
    // Now update object_1 with different content
    let new_content = load_moo_file("detailed_test_object.moo");
    let new_content_lines = moo_to_lines(&new_content);
    let new_content_str = new_content_lines.join("\n");
    let new_sha256 = TestServer::calculate_sha256(&new_content_str);
    
    println!("\nUpdating object_1 with different content...");
    println!("New SHA256: {}", new_sha256);
    
    make_request("POST", &format!("{}/rpc", base_url), Some(json!({
        "operation": "object/update",
        "args": ["object_1", serde_json::to_string(&new_content_lines).unwrap()]
    }))).await.expect("Failed to update object_1");
    
    // Verify object_1 now points to new SHA256
    let ref1_new = server.database().refs().get_ref(VcsObjectType::MooObject, "object_1", None).expect("Failed to get ref1_new");
    assert_eq!(ref1_new, Some(new_sha256.clone()));
    
    // CRITICAL: Original SHA256 should NOT be deleted because object_2 still uses it
    let original_exists = server.database().objects().get(&sha256).expect("Failed to check original").is_some();
    assert!(original_exists, 
            "Original SHA256 should NOT be deleted - object_2 still references it!");
    
    println!("✅ Original SHA256 preserved (object_2 still uses it)");
    
    // Verify object_2 still points to original SHA256
    let ref2_check = server.database().refs().get_ref(VcsObjectType::MooObject, "object_2", None).expect("Failed to get ref2_check");
    assert_eq!(ref2_check, Some(sha256));
    
    println!("\n✅ Test passed: SHA256 reference counting works correctly");
}

