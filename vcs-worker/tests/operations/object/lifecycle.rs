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
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Duplicate content detection");
    
    // Step 1: Create an object
    let object_name = "test_duplicate";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    let content_str = object_content.join("\n");
    let sha256_hash = TestServer::calculate_sha256(&content_str);
    
    println!("\nStep 1: Creating object with content A...");
    
    client.object_update(object_name, object_content.clone())
        .await
        .expect("Failed to update object")
        .assert_success("Object update");
    
    println!("✅ Object created with SHA256: {}", sha256_hash);
    
    db.assert_sha256_exists(&sha256_hash);
    let ref1 = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    assert_eq!(ref1, sha256_hash);
    
    let initial_count = server.database().objects().count();
    println!("Objects in DB: {}", initial_count);
    
    // Step 2: Submit the EXACT same content again (duplicate)
    println!("\nStep 2: Submitting duplicate content...");
    
    let response2 = client.object_update(object_name, object_content)
        .await
        .expect("Failed to update object");
    
    // Should detect duplicate and skip
    response2.assert_success("Duplicate content update");
    assert!(response2["result"].as_str().unwrap_or("").contains("unchanged"), "Should indicate unchanged");
    
    // Verify objects count hasn't increased (no duplicate storage)
    let final_count = server.database().objects().count();
    assert_eq!(final_count, initial_count, "Should not store duplicate SHA256");
    println!("✅ Duplicate detected, no extra storage");
    
    // Verify ref still points to same SHA256
    let ref2 = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    assert_eq!(ref2, sha256_hash);
    
    println!("\n✅ Test passed: Duplicate content properly detected");
}

#[tokio::test]
async fn test_sha256_cleanup_on_content_change() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: SHA256 cleanup when content changes");
    
    let object_name = "test_cleanup";
    
    // Step 1: Create object with content A
    println!("\nStep 1: Creating object with content A...");
    let content_a_lines = moo_to_lines(&load_moo_file("test_object.moo"));
    let content_a_str = content_a_lines.join("\n");
    let sha256_a = TestServer::calculate_sha256(&content_a_str);
    
    client.object_update(object_name, content_a_lines)
        .await
        .expect("Failed to update with content A")
        .assert_success("Update with content A");
    
    println!("Created with SHA256_A: {}", sha256_a);
    
    db.assert_sha256_exists(&sha256_a);
    let ref_a = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    assert_eq!(ref_a, sha256_a);
    
    let count_after_a = server.database().objects().count();
    println!("Objects after A: {}", count_after_a);
    
    // Step 2: Update object with content B (different content)
    println!("\nStep 2: Updating object with content B...");
    let content_b_lines = moo_to_lines(&load_moo_file("detailed_test_object.moo"));
    let content_b_str = content_b_lines.join("\n");
    let sha256_b = TestServer::calculate_sha256(&content_b_str);
    
    assert_ne!(sha256_a, sha256_b, "Content A and B should have different hashes");
    
    client.object_update(object_name, content_b_lines)
        .await
        .expect("Failed to update with content B")
        .assert_success("Update with content B");
    
    println!("Updated with SHA256_B: {}", sha256_b);
    
    db.assert_sha256_exists(&sha256_b);
    let ref_b = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    assert_eq!(ref_b, sha256_b);
    
    let count_after_b = server.database().objects().count();
    println!("Objects after B: {}", count_after_b);
    
    // CRITICAL CHECK: SHA256_A should be deleted since it's no longer referenced
    db.assert_sha256_not_exists(&sha256_a);
    println!("✅ Old SHA256_A properly deleted");
    
    // Verify object count hasn't increased (old SHA was cleaned up)
    assert_eq!(count_after_b, count_after_a, "Object count should remain same");
    
    println!("\n✅ Test passed: SHA256 cleanup working correctly");
}

#[tokio::test]
async fn test_content_flip_flop_version_handling() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Content flip-flop and version handling");
    
    let object_name = "test_flipflop";
    
    // Prepare Content A and B
    let content_a_lines = moo_to_lines(&load_moo_file("test_object.moo"));
    let content_a_str = content_a_lines.join("\n");
    let sha256_a = TestServer::calculate_sha256(&content_a_str);
    
    let content_b_lines = moo_to_lines(&load_moo_file("detailed_test_object.moo"));
    let content_b_str = content_b_lines.join("\n");
    let sha256_b = TestServer::calculate_sha256(&content_b_str);
    
    println!("\nSHA256_A: {}", sha256_a);
    println!("SHA256_B: {}", sha256_b);
    
    // Step 1: Create with content A
    println!("\nStep 1: Create with content A...");
    client.object_update(object_name, content_a_lines.clone())
        .await
        .expect("Failed to create object")
        .assert_success("Create with content A");
    
    let version1 = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    println!("Version after A: ref points to {}", version1);
    assert_eq!(version1, sha256_a);
    
    // Step 2: Update to content B
    println!("\nStep 2: Update to content B...");
    client.object_update(object_name, content_b_lines)
        .await
        .expect("Failed to update with content B")
        .assert_success("Update with content B");
    
    let version2 = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    println!("Version after B: ref points to {}", version2);
    assert_eq!(version2, sha256_b);
    
    // Step 3: Switch back to content A
    println!("\nStep 3: Switch back to content A...");
    client.object_update(object_name, content_a_lines)
        .await
        .expect("Failed to switch back to content A")
        .assert_success("Switch back to content A");
    
    let version3 = db.assert_ref_exists(VcsObjectType::MooObject, object_name);
    println!("Version after A again: ref points to {}", version3);
    assert_eq!(version3, sha256_a, "Ref should point back to SHA256_A");
    
    // Check which SHA256s exist
    let a_exists = server.database().objects().get(&sha256_a).expect("Failed to check A").is_some();
    let b_exists = server.database().objects().get(&sha256_b).expect("Failed to check B").is_some();
    
    println!("\nSHA256 existence:");
    println!("  SHA256_A exists: {}", a_exists);
    println!("  SHA256_B exists: {}", b_exists);
    
    // Since we switched back to A, B should be deleted (not in history)
    assert!(a_exists, "SHA256_A should exist (current content)");
    db.assert_sha256_not_exists(&sha256_b);
    println!("✅ SHA256_B properly deleted when switching back");
    
    // Verify the change tracks this correctly
    let (_, change) = db.require_top_change();
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
    let client = server.client();
    let db = server.db_assertions();
    
    println!("Test: Multiple objects with same content share SHA256");
    
    // Use the same content for two different objects
    let content_lines = moo_to_lines(&load_moo_file("test_object.moo"));
    let content_str = content_lines.join("\n");
    let sha256 = TestServer::calculate_sha256(&content_str);
    
    println!("\nShared SHA256: {}", sha256);
    
    // Create object 1
    println!("\nCreating object_1 with shared content...");
    client.object_update("object_1", content_lines.clone())
        .await
        .expect("Failed to create object_1")
        .assert_success("Create object_1");
    
    let ref1 = db.assert_ref_exists(VcsObjectType::MooObject, "object_1");
    assert_eq!(ref1, sha256);
    
    let count_after_1 = server.database().objects().count();
    
    // Create object 2 with SAME content
    println!("Creating object_2 with same content...");
    client.object_update("object_2", content_lines)
        .await
        .expect("Failed to create object_2")
        .assert_success("Create object_2");
    
    let ref2 = db.assert_ref_exists(VcsObjectType::MooObject, "object_2");
    assert_eq!(ref2, sha256);
    
    let count_after_2 = server.database().objects().count();
    
    // Should NOT have duplicated the SHA256
    assert_eq!(count_after_2, count_after_1, "Should reuse SHA256, not duplicate");
    println!("✅ SHA256 properly reused for both objects");
    
    // Now update object_1 with different content
    println!("\nUpdating object_1 with different content...");
    let new_content_lines = moo_to_lines(&load_moo_file("detailed_test_object.moo"));
    let new_content_str = new_content_lines.join("\n");
    let new_sha256 = TestServer::calculate_sha256(&new_content_str);
    println!("New SHA256: {}", new_sha256);
    
    client.object_update("object_1", new_content_lines)
        .await
        .expect("Failed to update object_1")
        .assert_success("Update object_1");
    
    // Verify object_1 now points to new SHA256
    let ref1_new = db.assert_ref_exists(VcsObjectType::MooObject, "object_1");
    assert_eq!(ref1_new, new_sha256);
    
    // CRITICAL: Original SHA256 should NOT be deleted because object_2 still uses it
    db.assert_sha256_exists(&sha256);
    println!("✅ Original SHA256 preserved (object_2 still uses it)");
    
    // Verify object_2 still points to original SHA256
    let ref2_check = db.assert_ref_exists(VcsObjectType::MooObject, "object_2");
    assert_eq!(ref2_check, sha256);
    
    println!("\n✅ Test passed: SHA256 reference counting works correctly");
}

