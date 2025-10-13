//! Tests for change creation operations

use crate::common::*;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_change_create_empty_local() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Create change should create an empty local change");

    // Step 1: Verify no change initially
    println!("\nStep 1: Verifying no change initially...");
    db.assert_no_top_change();
    println!("✅ No change initially");

    // Step 2: Create a change
    println!("\nStep 2: Creating a new change...");
    client
        .change_create("test_change", "test_author", Some("Test description"))
        .await
        .expect("Failed to create change")
        .assert_success("Change creation");
    println!("✅ Change created");

    // Step 3: Verify the change exists and is empty
    println!("\nStep 3: Verifying change exists and is empty...");
    let (_, change) = db.require_top_change();

    assert_eq!(change.name, "test_change", "Change name should match");
    assert_eq!(change.author, "test_author", "Author should match");
    assert_eq!(change.status, ChangeStatus::Local, "Status should be Local");
    assert_eq!(
        change.added_objects.len(),
        0,
        "Should have no added objects"
    );
    assert_eq!(
        change.modified_objects.len(),
        0,
        "Should have no modified objects"
    );
    assert_eq!(
        change.deleted_objects.len(),
        0,
        "Should have no deleted objects"
    );
    assert_eq!(
        change.renamed_objects.len(),
        0,
        "Should have no renamed objects"
    );
    println!("✅ Change exists and is empty");

    println!("\n✅ Test passed: Create change creates empty local change");
}

#[tokio::test]
async fn test_cannot_create_change_when_local_exists() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Cannot create a new change when a local change already exists");

    // Step 1: Create first change
    println!("\nStep 1: Creating first change...");
    client
        .change_create("first_change", "test_author", None)
        .await
        .expect("Failed to create first change")
        .assert_success("Create first change");
    println!("✅ First change created");

    // Step 2: Try to create second change (should fail)
    println!("\nStep 2: Trying to create second change...");
    let response = client
        .change_create("second_change", "test_author", None)
        .await
        .expect("Request should complete");

    // The operation might succeed at the RPC level but return an error message
    let result_str = response.get_result_str().unwrap_or("");
    let failed = !response.is_success()
        || result_str.contains("Error")
        || result_str.contains("Already in a local change");

    assert!(
        failed,
        "Second change creation should fail (already in local change)"
    );
    println!("✅ Second change creation failed as expected");

    // Verify only one change exists
    let (_, change) = db.require_top_change();

    assert_eq!(
        change.name, "first_change",
        "Only first change should exist"
    );
    println!("✅ Only first change exists");

    println!("\n✅ Test passed: Cannot create change when local exists");
}

