//! Advanced integration tests for rename workflows
//!
//! These tests cover complex rename scenarios that involve multiple operations:
//! - Rename → Modify workflows
//! - Rename → Approve workflows  
//! - Rename → Rename back for modified/added objects
//! - Change switching with renamed modified objects
//! - Meta object tracking through renames

use crate::common::*;

#[tokio::test]
async fn test_rename_modified_then_modify_again() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Rename modified object then modify it again");

    // Step 1: Create and approve object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("initial", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("original_obj", "test_object.moo")
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object created and approved");

    // Step 2: Modify the object
    println!("\nStep 2: Modifying object...");
    client
        .object_update_from_file("original_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to modify");

    let (_change_id, change_after_mod) = db.require_top_change();
    let version_after_first_mod = change_after_mod.modified_objects[0].version;
    println!("✅ Modified to version {}", version_after_first_mod);

    // Step 3: Rename the modified object
    println!("\nStep 3: Renaming modified object...");
    client
        .object_rename("original_obj", "renamed_obj")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    println!("✅ Renamed to renamed_obj");

    // Step 4: Modify again using the new name
    println!("\nStep 4: Modifying again with new name...");
    client
        .object_update_from_file("renamed_obj", "test_object.moo")
        .await
        .expect("Failed to modify after rename");

    let (_change_id, change_after_second_mod) = db.require_top_change();
    let version_after_second_mod = change_after_second_mod.modified_objects[0].version;
    
    // Version should NOT increment - it's the same change!
    assert_eq!(
        version_after_second_mod, version_after_first_mod,
        "Version should stay same - it's all one change"
    );
    assert_eq!(
        change_after_second_mod.modified_objects[0].name, "renamed_obj",
        "Object should still have new name"
    );
    
    println!(
        "✅ Modified again, version stays {} (collapsed in same change)",
        version_after_second_mod
    );

    // Step 5: Verify can get by new name
    println!("\nStep 5: Verifying object accessible by new name...");
    client
        .object_get("renamed_obj")
        .await
        .expect("Failed to get")
        .assert_success("Get by new name");

    // Step 6: Verify old name doesn't work
    println!("\nStep 6: Verifying old name doesn't work...");
    let get_old = client
        .object_get("original_obj")
        .await
        .expect("Request should complete");
    
    let result_str = get_old.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Old name should not be accessible"
    );

    println!("\n✅ Test passed: Rename modified then modify again works correctly");
}

#[tokio::test]
async fn test_rename_added_then_modify() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Rename added object then modify it");

    // Step 1: Create object (added_objects)
    println!("\nStep 1: Creating object...");
    client
        .object_update_from_file("new_obj", "test_object.moo")
        .await
        .expect("Failed to create");

    println!("✅ Object created");

    // Step 2: Rename the added object
    println!("\nStep 2: Renaming added object...");
    client
        .object_rename("new_obj", "renamed_new_obj")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    let (_change_id, change_after_rename) = db.require_top_change();
    assert_eq!(
        change_after_rename.added_objects[0].name, "renamed_new_obj",
        "Name should be updated in added_objects"
    );
    assert_eq!(
        change_after_rename.renamed_objects.len(),
        0,
        "Should not have renamed_objects entry for added object"
    );

    println!("✅ Renamed to renamed_new_obj");

    // Step 3: Modify using new name
    println!("\nStep 3: Modifying with new name...");
    client
        .object_update_from_file("renamed_new_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to modify after rename");

    let (_change_id, change_after_mod) = db.require_top_change();
    let version_after_mod = change_after_mod.added_objects[0].version;
    
    // Version stays the same - it's all one change
    assert_eq!(
        version_after_mod, 1,
        "Version should stay 1 - it's all one change"
    );
    assert_eq!(
        change_after_mod.added_objects[0].name, "renamed_new_obj",
        "Object should still have new name"
    );

    println!("✅ Modified, version stays {} (collapsed in same change)", version_after_mod);

    // Step 4: Verify accessible by new name
    println!("\nStep 4: Verifying accessible by new name...");
    client
        .object_get("renamed_new_obj")
        .await
        .expect("Failed to get")
        .assert_success("Get by new name");

    println!("\n✅ Test passed: Rename added then modify works correctly");
}

#[tokio::test]
async fn test_rename_modified_back_to_original() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Rename modified object back to original name");

    // Step 1: Create and approve object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("initial", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_obj", "test_object.moo")
        .await
        .expect("Failed to create");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object created and approved");

    // Step 2: Modify the object
    println!("\nStep 2: Modifying object...");
    client
        .object_update_from_file("test_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to modify");

    println!("✅ Object modified");

    // Step 3: Rename it
    println!("\nStep 3: Renaming to temp_name...");
    client
        .object_rename("test_obj", "temp_name")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    let (_change_id, change_after_first_rename) = db.require_top_change();
    assert_eq!(
        change_after_first_rename.modified_objects[0].name, "temp_name",
        "Name should be temp_name"
    );

    println!("✅ Renamed to temp_name");

    // Step 4: Rename back to original
    println!("\nStep 4: Renaming back to test_obj...");
    client
        .object_rename("temp_name", "test_obj")
        .await
        .expect("Failed to rename back")
        .assert_success("Rename back");

    let (_change_id, change_after_rename_back) = db.require_top_change();
    assert_eq!(
        change_after_rename_back.modified_objects[0].name, "test_obj",
        "Name should be back to test_obj"
    );
    
    // The renamed_objects entry should be gone (cancelled out)
    assert_eq!(
        change_after_rename_back.renamed_objects.len(),
        0,
        "Rename should be cancelled when renamed back to original"
    );

    println!("✅ Renamed back, rename entry cancelled");

    // Step 5: Verify accessible by original name
    println!("\nStep 5: Verifying accessible by original name...");
    client
        .object_get("test_obj")
        .await
        .expect("Failed to get")
        .assert_success("Get by original name");

    // Step 6: Verify temp name doesn't work
    println!("\nStep 6: Verifying temp name doesn't work...");
    let get_temp = client
        .object_get("temp_name")
        .await
        .expect("Request should complete");
    
    let result_str = get_temp.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Temp name should not be accessible"
    );

    println!("\n✅ Test passed: Rename modified back to original cancels rename");
}

#[tokio::test]
async fn test_rename_modified_then_approve() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Rename modified object then approve the change");

    // Step 1: Create and approve object
    println!("\nStep 1: Creating and approving initial object...");
    client
        .change_create("initial", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("obj_v1", "test_object.moo")
        .await
        .expect("Failed to create");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Initial object approved");

    // Step 2: Modify the object
    println!("\nStep 2: Modifying object...");
    client
        .object_update_from_file("obj_v1", "detailed_test_object.moo")
        .await
        .expect("Failed to modify");

    println!("✅ Object modified");

    // Step 3: Rename the modified object
    println!("\nStep 3: Renaming to obj_v2...");
    client
        .object_rename("obj_v1", "obj_v2")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    println!("✅ Renamed to obj_v2");

    // Step 4: Approve the change
    println!("\nStep 4: Approving change with renamed modified object...");
    let (change_id, change_before_approve) = db.require_top_change();
    
    assert_eq!(
        change_before_approve.modified_objects[0].name, "obj_v2",
        "Modified object should have new name"
    );
    assert_eq!(
        change_before_approve.renamed_objects.len(),
        1,
        "Should have rename entry"
    );
    assert_eq!(
        change_before_approve.renamed_objects[0].from.name, "obj_v1",
        "Rename should be from obj_v1"
    );
    assert_eq!(
        change_before_approve.renamed_objects[0].to.name, "obj_v2",
        "Rename should be to obj_v2"
    );

    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Change approved");

    // Step 5: Verify new name works
    println!("\nStep 5: Verifying new name works after approval...");
    client
        .object_get("obj_v2")
        .await
        .expect("Failed to get")
        .assert_success("Get by new name");

    // Note: After approval, obj_v1 may still resolve to historical version 1
    // This is expected behavior - old refs remain for historical access
    
    println!("\n✅ Test passed: Rename modified then approve works correctly");
}

#[tokio::test]
async fn test_rename_chain_on_modified_object() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Chain multiple renames on a modified object");

    // Step 1: Create and approve object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("initial", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("name_a", "test_object.moo")
        .await
        .expect("Failed to create");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object approved with name_a");

    // Step 2: Modify the object
    println!("\nStep 2: Modifying object...");
    client
        .object_update_from_file("name_a", "detailed_test_object.moo")
        .await
        .expect("Failed to modify");

    println!("✅ Object modified");

    // Step 3: Rename to name_b
    println!("\nStep 3: Renaming to name_b...");
    client
        .object_rename("name_a", "name_b")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    // Step 4: Rename to name_c
    println!("\nStep 4: Renaming to name_c...");
    client
        .object_rename("name_b", "name_c")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    // Step 5: Rename to name_d
    println!("\nStep 5: Renaming to name_d...");
    client
        .object_rename("name_c", "name_d")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    let (_change_id, change_after_renames) = db.require_top_change();

    // Should have single modified entry with final name
    assert_eq!(
        change_after_renames.modified_objects.len(),
        1,
        "Should have 1 modified object"
    );
    assert_eq!(
        change_after_renames.modified_objects[0].name, "name_d",
        "Should have final name"
    );

    // Should have single rename entry from original to final
    assert_eq!(
        change_after_renames.renamed_objects.len(),
        1,
        "Should have 1 rename entry (chained)"
    );
    assert_eq!(
        change_after_renames.renamed_objects[0].from.name, "name_a",
        "Should be from original name"
    );
    assert_eq!(
        change_after_renames.renamed_objects[0].to.name, "name_d",
        "Should be to final name"
    );

    println!("✅ Rename chain tracked correctly");

    // Step 6: Verify accessible by final name only
    println!("\nStep 6: Verifying only final name works...");
    client
        .object_get("name_d")
        .await
        .expect("Failed to get")
        .assert_success("Get by final name");

    for old_name in &["name_a", "name_b", "name_c"] {
        let get_old = client
            .object_get(old_name)
            .await
            .expect("Request should complete");
        
        let result_str = get_old.get_result_str().unwrap_or("");
        assert!(
            result_str.contains("Error") || result_str.contains("not found"),
            "Old name {} should not be accessible",
            old_name
        );
    }

    println!("\n✅ Test passed: Rename chain on modified object works correctly");
}

#[tokio::test]
async fn test_rename_modified_with_no_previous_versions() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Rename modified object that was just added (no previous versions)");

    // Step 1: Create and approve an object
    println!("\nStep 1: Creating and approving object...");
    client
        .change_create("initial", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("fresh_obj", "test_object.moo")
        .await
        .expect("Failed to create");

    let (change_id, _) = db.require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Object created and approved (version 1)");

    // Step 2: Modify immediately (this creates version 2, but version 1 doesn't exist in local change)
    println!("\nStep 2: Modifying to version 2...");
    client
        .object_update_from_file("fresh_obj", "detailed_test_object.moo")
        .await
        .expect("Failed to modify");

    let (_change_id, change_after_mod) = db.require_top_change();
    assert_eq!(
        change_after_mod.modified_objects[0].version, 2,
        "Should be version 2"
    );

    println!("✅ Modified to version 2");

    // Step 3: Rename the modified object
    println!("\nStep 3: Renaming fresh_obj to renamed_fresh...");
    client
        .object_rename("fresh_obj", "renamed_fresh")
        .await
        .expect("Failed to rename")
        .assert_success("Rename");

    let (_change_id, change_after_rename) = db.require_top_change();
    
    assert_eq!(
        change_after_rename.modified_objects[0].name, "renamed_fresh",
        "Modified object should have new name"
    );
    
    // Should have rename entry because there's a previous version (v1) in committed history
    assert_eq!(
        change_after_rename.renamed_objects.len(),
        1,
        "Should have rename entry (v1 exists in history)"
    );

    println!("✅ Renamed with proper rename entry");

    // Step 4: Verify new name works, old doesn't
    println!("\nStep 4: Verifying name resolution...");
    client
        .object_get("renamed_fresh")
        .await
        .expect("Failed to get")
        .assert_success("Get by new name");

    let get_old = client
        .object_get("fresh_obj")
        .await
        .expect("Request should complete");
    
    let result_str = get_old.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Old name should not work"
    );

    println!("\n✅ Test passed: Rename modified with previous version works correctly");
}


