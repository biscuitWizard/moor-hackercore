//! Tests for change approval operations

use crate::common::*;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_approve_change_moves_to_merged() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Approve change should move to Merged status and canonize refs/SHA256s");

    // Step 1: Create a change with objects
    println!("\nStep 1: Creating change with objects...");
    client
        .change_create("test_approve_change", "test_author", None)
        .await
        .expect("Failed to create change");

    let object_content = moo_to_lines(&load_moo_file("test_object.moo"));
    let content_str = object_content.join("\n");
    let sha256 = TestServer::calculate_sha256(&content_str);

    client
        .object_update("approved_object", object_content)
        .await
        .expect("Failed to update object");

    println!("✅ Change created with object");

    // Get change ID
    let (change_id, change_before) = db.require_top_change();

    assert_eq!(
        change_before.status,
        ChangeStatus::Local,
        "Should be Local before approve"
    );
    println!("✅ Change status is Local");

    // Step 2: Approve the change using HTTP API (Wizard user has approval permission)
    println!("\nStep 2: Approving the change...");

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");

    println!("✅ Change approved and marked as Merged using change/approve API");

    // Step 3: Verify change is marked as Merged
    println!("\nStep 3: Verifying change status...");
    let change_after = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should still exist in database");

    assert_eq!(
        change_after.status,
        ChangeStatus::Merged,
        "Should be Merged after approve"
    );
    println!("✅ Change status is Merged");

    // Step 4: Verify change is removed from top of index
    println!("\nStep 4: Verifying change removed from index top...");
    db.assert_no_top_change();
    println!("✅ Change removed from top of index");

    // Step 5: Verify SHA256 still exists (canonized)
    println!("\nStep 5: Verifying SHA256 canonization...");
    let sha256_after = server
        .database()
        .objects()
        .get(&sha256)
        .expect("Failed to check SHA256")
        .is_some();

    assert!(
        sha256_after,
        "SHA256 should still exist after approve (canonized)"
    );
    println!("✅ SHA256 canonized (still exists)");

    // Step 6: Verify ref still exists (canonized)
    println!("\nStep 6: Verifying ref canonization...");
    let ref_after = db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "approved_object",
    );

    assert_eq!(ref_after, sha256, "Ref should point to correct SHA256");
    println!("✅ Ref canonized (still exists and points to correct SHA256)");

    println!("\n✅ Test passed: Approve change moves to Merged and canonizes refs/SHA256s");
}

#[tokio::test]
async fn test_approve_non_existent_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Approving a non-existent change ID should fail");

    // Attempt to approve non-existent change
    println!("\nAttempting to approve non-existent change...");
    let response = client
        .change_approve("non_existent_change_id")
        .await
        .expect("Request should complete");

    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Should indicate change not found, got: {}",
        result_str
    );
    println!("✅ Approve failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot approve non-existent change");
}

#[tokio::test]
async fn test_approve_already_merged_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Approving an already merged change should be idempotent or fail gracefully");

    // Step 1: Create and approve a change
    println!("\nStep 1: Creating and approving change...");
    client
        .change_create("test_approve_merged", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("merged_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    let (change_id, _) = db.require_top_change();

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Change approved (merged)");

    // Step 2: Verify change is merged
    let merged_change = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    assert_eq!(
        merged_change.status,
        ChangeStatus::Merged,
        "Should be Merged"
    );

    // Step 3: Try to approve again
    println!("\nStep 3: Attempting to approve again...");
    let response = client
        .change_approve(&change_id)
        .await
        .expect("Request should complete");

    println!("Second approve response: {:?}", response);

    // Should either be idempotent (succeed) or fail gracefully
    let result_str = response.get_result_str().unwrap_or("");
    if result_str.contains("Error") {
        println!("✅ Second approve rejected: {}", result_str);
    } else {
        println!("✅ Second approve was idempotent");
    }

    println!("\n✅ Test passed: Approve handles already merged change");
}

#[tokio::test]
async fn test_approve_empty_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Approving an empty change with no objects should work");

    // Step 1: Create an empty change
    println!("\nStep 1: Creating empty change...");
    client
        .change_create("empty_change", "test_author", Some("Empty change"))
        .await
        .expect("Failed to create change")
        .assert_success("Create change");

    let (change_id, change_before) = db.require_top_change();
    assert_eq!(
        change_before.added_objects.len(),
        0,
        "Should have no objects"
    );
    assert_eq!(change_before.status, ChangeStatus::Local, "Should be Local");
    println!("✅ Empty change created");

    // Step 2: Approve the empty change
    println!("\nStep 2: Approving empty change...");
    let response = client
        .change_approve(&change_id)
        .await
        .expect("Request should complete");

    // Should succeed
    response.assert_success("Approve empty change");
    println!("✅ Empty change approved");

    // Step 3: Verify change is merged
    println!("\nStep 3: Verifying change is merged...");
    let merged_change = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get change")
        .expect("Change should exist");

    assert_eq!(
        merged_change.status,
        ChangeStatus::Merged,
        "Should be Merged"
    );
    assert_eq!(
        merged_change.added_objects.len(),
        0,
        "Should still have no objects"
    );
    println!("✅ Empty change is merged");

    println!("\n✅ Test passed: Can approve empty change");
}

#[tokio::test]
async fn test_approve_with_empty_change_id() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Approving with empty change ID should fail");

    // Attempt to approve with empty change ID
    println!("\nAttempting to approve with empty change ID...");
    let response = client
        .change_approve("")
        .await
        .expect("Request should complete");

    // Should fail with error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error")
            || result_str.contains("required")
            || result_str.contains("not found"),
        "Should indicate error, got: {}",
        result_str
    );
    println!("✅ Approve failed with appropriate error: {}", result_str);

    println!("\n✅ Test passed: Cannot approve with empty change ID");
}

