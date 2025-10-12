//! Integration test for renaming modified objects - refs handling
//!
//! This test verifies that when renaming an object that's in modified_objects,
//! the refs are properly updated to point from the new name to the SHA256.
//!
//! Bug: When renaming a modified object, the code updates the name in modified_objects
//! but doesn't update the refs, causing "version N not found" errors.

use crate::common::*;

#[tokio::test]
async fn test_rename_modified_object_refs_handling() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Renaming a modified object should update refs properly");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object #73...");
    client
        .change_create("initial", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("#73", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object #73 created and approved");

    // Step 2: Modify the object (this will create version 2)
    println!("\nStep 2: Modifying object #73...");
    client
        .object_update_from_file("#73", "detailed_test_object.moo")
        .await
        .expect("Failed to modify object");

    let (_change_id, change_after_modify) = db.require_top_change();
    assert_eq!(
        change_after_modify.modified_objects.len(),
        1,
        "Should have 1 modified object"
    );
    assert_eq!(
        change_after_modify.modified_objects[0].name, "#73",
        "Modified object should be #73"
    );
    let version_after_modify = change_after_modify.modified_objects[0].version;
    println!("✅ Object #73 modified (version {})", version_after_modify);

    // Step 3: Rename the modified object
    println!("\nStep 3: Renaming #73 to prog_feature...");
    let rename_result = client
        .object_rename("#73", "prog_feature")
        .await
        .expect("Failed to rename");
    
    rename_result.assert_success("Rename");
    println!("✅ Renamed #73 to prog_feature");

    // Step 4: Verify the modified_objects list was updated
    println!("\nStep 4: Verifying modified_objects was updated...");
    let (_change_id, change_after_rename) = db.require_top_change();
    assert_eq!(
        change_after_rename.modified_objects.len(),
        1,
        "Should still have 1 modified object"
    );
    assert_eq!(
        change_after_rename.modified_objects[0].name, "prog_feature",
        "Modified object should now be prog_feature"
    );
    assert_eq!(
        change_after_rename.modified_objects[0].version, version_after_modify,
        "Version should remain the same"
    );
    println!("✅ modified_objects updated correctly");

    // Step 5: Try to get the object by new name (this is where the bug occurs)
    println!("\nStep 5: Getting object by new name prog_feature...");
    let get_result = client
        .object_get("prog_feature")
        .await
        .expect("Request should complete");

    // This should succeed but currently fails with "version N not found"
    get_result.assert_success("Get renamed modified object");
    println!("✅ Object retrieved successfully by new name");

    // Step 6: Verify old name no longer works
    println!("\nStep 6: Verifying old name #73 no longer works...");
    let get_old_result = client
        .object_get("#73")
        .await
        .expect("Request should complete");
    
    let result_str = get_old_result.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Old name should not be accessible, got: {}",
        result_str
    );
    println!("✅ Old name no longer accessible");

    println!("\n✅ Test passed: Modified object rename with proper ref handling");
}

#[tokio::test]
async fn test_rename_added_object_refs_handling() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Renaming an added object should update refs properly");

    // Step 1: Create an object (not approved, so it's in added_objects)
    println!("\nStep 1: Creating object #99...");
    client
        .object_update_from_file("#99", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (_change_id, change_after_add) = db.require_top_change();
    assert_eq!(
        change_after_add.added_objects.len(),
        1,
        "Should have 1 added object"
    );
    assert_eq!(
        change_after_add.added_objects[0].name, "#99",
        "Added object should be #99"
    );
    println!("✅ Object #99 added");

    // Step 2: Rename the added object
    println!("\nStep 2: Renaming #99 to new_feature...");
    let rename_result = client
        .object_rename("#99", "new_feature")
        .await
        .expect("Failed to rename");
    
    rename_result.assert_success("Rename");
    println!("✅ Renamed #99 to new_feature");

    // Step 3: Try to get the object by new name
    println!("\nStep 3: Getting object by new name new_feature...");
    let get_result = client
        .object_get("new_feature")
        .await
        .expect("Request should complete");

    get_result.assert_success("Get renamed added object");
    println!("✅ Object retrieved successfully by new name");

    println!("\n✅ Test passed: Added object rename with proper ref handling");
}

#[tokio::test]
async fn test_rename_modified_object_multiple_versions() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Renaming modified object with multiple version updates");

    // Step 1: Create and approve object
    println!("\nStep 1: Creating initial object...");
    client
        .change_create("initial", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_obj", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Initial object created");

    // Step 2: Modify multiple times
    println!("\nStep 2: Modifying object multiple times...");
    for i in 1..=3 {
        client
            .object_update_from_file("test_obj", "detailed_test_object.moo")
            .await
            .expect("Failed to modify");
        println!("  Modification {}", i);
    }

    let (_change_id, change_after_mods) = db.require_top_change();
    let final_version = change_after_mods.modified_objects[0].version;
    println!("✅ Object modified to version {}", final_version);

    // Step 3: Rename
    println!("\nStep 3: Renaming test_obj to renamed_obj...");
    client
        .object_rename("test_obj", "renamed_obj")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    // Step 4: Get by new name
    println!("\nStep 4: Getting by new name...");
    client
        .object_get("renamed_obj")
        .await
        .expect("Failed to get")
        .assert_success("Get renamed object");

    println!("\n✅ Test passed: Multiple modifications then rename works correctly");
}

