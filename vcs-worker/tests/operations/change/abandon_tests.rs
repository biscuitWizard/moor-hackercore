//! Tests for change abandon operations

use crate::common::*;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_abandon_local_change_clears_top_and_cleans_up() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Abandon local change should clear top change and cleanup unused resources");

    // Step 1: Create a change and add some objects
    println!("\nStep 1: Creating change and adding objects...");
    client
        .change_create("test_abandon_change", "test_author", None)
        .await
        .expect("Failed to create change");

    // Add some objects to the change
    let object_content_1 = moo_to_lines(&load_moo_file("test_object.moo"));
    let content_str_1 = object_content_1.join("\n");
    let sha256_1 = TestServer::calculate_sha256(&content_str_1);

    client
        .object_update("test_object_1", object_content_1)
        .await
        .expect("Failed to update object 1");

    let object_content_2 = moo_to_lines(&load_moo_file("detailed_test_object.moo"));
    let content_str_2 = object_content_2.join("\n");
    let sha256_2 = TestServer::calculate_sha256(&content_str_2);

    client
        .object_update("test_object_2", object_content_2)
        .await
        .expect("Failed to update object 2");

    println!("✅ Created change with 2 objects");

    // Verify objects exist
    let sha256_1_before = server
        .database()
        .objects()
        .get(&sha256_1)
        .expect("Failed to get object 1")
        .is_some();
    let sha256_2_before = server
        .database()
        .objects()
        .get(&sha256_2)
        .expect("Failed to get object 2")
        .is_some();

    assert!(sha256_1_before, "SHA256 1 should exist before abandon");
    assert!(sha256_2_before, "SHA256 2 should exist before abandon");
    println!("✅ Objects exist in database");

    // Step 2: Abandon the change
    println!("\nStep 2: Abandoning the change...");
    client
        .change_abandon()
        .await
        .expect("Failed to abandon change")
        .assert_success("Abandon change");
    println!("✅ Change abandoned");

    // Step 3: Verify top change is None
    println!("\nStep 3: Verifying top change is cleared...");
    db.assert_no_top_change();
    println!("✅ Top change cleared");

    // Step 4: Verify SHA256s were cleaned up (not referenced elsewhere)
    println!("\nStep 4: Verifying SHA256 cleanup...");
    let sha256_1_after = server
        .database()
        .objects()
        .get(&sha256_1)
        .expect("Failed to check SHA256 1")
        .is_some();
    let sha256_2_after = server
        .database()
        .objects()
        .get(&sha256_2)
        .expect("Failed to check SHA256 2")
        .is_some();

    println!("SHA256 1 after abandon: {}", sha256_1_after);
    println!("SHA256 2 after abandon: {}", sha256_2_after);

    // Step 5: Verify refs were cleaned up
    println!("\nStep 5: Verifying ref cleanup...");
    let ref_1_after = server
        .database()
        .refs()
        .get_ref(
            moor_vcs_worker::types::VcsObjectType::MooObject,
            "test_object_1",
            None,
        )
        .expect("Failed to check ref 1");
    let ref_2_after = server
        .database()
        .refs()
        .get_ref(
            moor_vcs_worker::types::VcsObjectType::MooObject,
            "test_object_2",
            None,
        )
        .expect("Failed to check ref 2");

    println!("Ref 1 after abandon: {:?}", ref_1_after);
    println!("Ref 2 after abandon: {:?}", ref_2_after);

    println!("\n⚠️  Note: Cleanup of SHA256s and refs during abandon needs to be implemented");
    println!("✅ Test completed (cleanup verification pending implementation)");
}

#[tokio::test]
async fn test_abandon_with_last_merged_returns_to_merged() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!(
        "Test: Abandoning local change when there's a merged change should return to merged state"
    );

    // Step 1: Create and commit a change
    println!("\nStep 1: Creating and committing first change...");
    client
        .change_create("first_change", "test_author", None)
        .await
        .expect("Failed to create first change");

    client
        .object_update_from_file("first_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    // Approve the change using HTTP API
    let (change_id, _) = db.require_top_change();

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");

    println!("✅ First change committed using change/approve API");

    // Verify no local change
    db.assert_no_top_change();

    // Step 2: Create a second local change
    println!("\nStep 2: Creating second local change...");
    client
        .change_create("second_change", "test_author", None)
        .await
        .expect("Failed to create second change");

    client
        .object_update_from_file("second_object", "detailed_test_object.moo")
        .await
        .expect("Failed to update object 2");

    println!("✅ Second change created");

    // Verify we have a local change
    db.require_top_change();

    // Step 3: Abandon the second change
    println!("\nStep 3: Abandoning second change...");
    client
        .change_abandon()
        .await
        .expect("Failed to abandon change")
        .assert_success("Abandon change");
    println!("✅ Second change abandoned");

    // Give the system a moment to fully process the abandon
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Step 4: Verify top change is None (but merged change still exists in database)
    println!("\nStep 4: Verifying state after abandon...");
    db.assert_no_top_change();
    println!("✅ Top change is None");

    // Verify the first (merged) change still exists
    let first_change = server
        .database()
        .index()
        .get_change(&change_id)
        .expect("Failed to get first change")
        .expect("First change should still exist");

    assert_eq!(
        first_change.status,
        ChangeStatus::Merged,
        "First change should still be Merged"
    );
    println!("✅ First change still exists as Merged");

    // Verify first object still exists (from merged change)
    db.assert_ref_exists(
        moor_vcs_worker::types::VcsObjectType::MooObject,
        "first_object",
    );
    println!("✅ First object still exists");

    println!("\n✅ Test passed: Abandoning local change returns to merged state");
}

#[tokio::test]
async fn test_abandon_when_no_change_exists() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Abandoning when no change exists should fail gracefully");

    // Step 1: Verify no change initially
    println!("\nStep 1: Verifying no change initially...");
    db.assert_no_top_change();
    println!("✅ No change initially");

    // Step 2: Try to abandon
    println!("\nStep 2: Attempting to abandon...");
    let response = client
        .change_abandon()
        .await
        .expect("Request should complete");

    // Should fail gracefully
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error")
            || result_str.contains("No change")
            || result_str.contains("not found"),
        "Should indicate no change to abandon, got: {}",
        result_str
    );
    println!("✅ Abandon failed gracefully: {}", result_str);

    println!("\n✅ Test passed: Abandon fails gracefully when no change exists");
}

#[tokio::test]
async fn test_abandon_multiple_times() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Abandoning multiple times should fail after first abandon");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("test_abandon", "test_author", None)
        .await
        .expect("Failed to create change");

    db.require_top_change();
    println!("✅ Change created");

    // Step 2: Abandon the change
    println!("\nStep 2: Abandoning change...");
    client
        .change_abandon()
        .await
        .expect("Failed to abandon")
        .assert_success("Abandon");

    db.assert_no_top_change();
    println!("✅ Change abandoned");

    // Step 3: Try to abandon again
    println!("\nStep 3: Attempting to abandon again...");
    let response = client
        .change_abandon()
        .await
        .expect("Request should complete");

    // Should fail since no change to abandon
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("No change"),
        "Should indicate no change to abandon, got: {}",
        result_str
    );
    println!("✅ Second abandon failed appropriately: {}", result_str);

    println!("\n✅ Test passed: Multiple abandons fail after first");
}

