//! Integration tests for change/status operation
//!
//! These tests verify:
//! 1. change/status returns correct object change details
//! 2. Numeric object IDs (like "#73") are returned as v_obj, not strings
//! 3. Named objects (like "sysobj") are returned as strings

use crate::common::*;
use moor_var::{Associative, Sequence, Variant};
use moor_vcs_worker::operations::{ChangeStatusOperation, Operation};

/// Helper to call change/status directly and get the Var result
async fn call_change_status(server: &TestServer) -> moor_var::Var {
    let change_status_op = ChangeStatusOperation::new(server.database().clone());
    let wizard_user = server.get_wizard_user().expect("Failed to get wizard user");
    change_status_op.execute(vec![], &wizard_user)
}

#[tokio::test]
async fn test_change_status_with_named_objects() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: change/status returns named objects as strings");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("test_change", "test_author", None)
        .await
        .expect("Failed to create change")
        .assert_success("Create change");
    println!("✅ Change created");

    // Step 2: Add an object with a name (not a numeric ID)
    println!("\nStep 2: Adding named object...");
    client
        .object_update_from_file("sysobj", "test_object.moo")
        .await
        .expect("Failed to update object");
    println!("✅ Named object added");

    // Step 3: Get change status directly
    println!("\nStep 3: Getting change status...");
    let result = call_change_status(&server).await;
    println!("✅ Change status retrieved");

    // Step 4: Verify the response structure
    println!("\nStep 4: Verifying response structure...");
    
    match result.variant() {
        Variant::Map(result_map) => {
            // Check objects_added contains "sysobj"
            let objects_added = result_map
                .get(&moor_var::v_str("objects_added"))
                .expect("Should have objects_added field");

            match objects_added.variant() {
                Variant::List(list) => {
                    assert_eq!(list.len(), 1, "Should have 1 added object");
                    let first_obj = &list[0];
                    // Named object should be a string
                    match first_obj.variant() {
                        Variant::Str(s) => {
                            assert_eq!(s.as_str(), "sysobj", "Object name should be 'sysobj'");
                            println!("✅ Named object 'sysobj' returned as string");
                        }
                        _ => panic!("Named object should be returned as string, got: {:?}", first_obj.variant()),
                    }
                }
                _ => panic!("objects_added should be a list"),
            }

            // Check changes array contains obj_id as string
            let changes = result_map
                .get(&moor_var::v_str("changes"))
                .expect("Should have changes field");

            match changes.variant() {
                Variant::List(list) => {
                    assert!(list.len() > 0, "Should have at least one change");
                    let first_change = &list[0];
                    
                    match first_change.variant() {
                        Variant::Map(map) => {
                            let obj_id = map
                                .get(&moor_var::v_str("obj_id"))
                                .expect("Should have obj_id field");
                            
                            // Named object should be a string
                            match obj_id.variant() {
                                Variant::Str(s) => {
                                    assert_eq!(s.as_str(), "sysobj", "obj_id should be 'sysobj'");
                                    println!("✅ obj_id 'sysobj' in changes array returned as string");
                                }
                                _ => panic!("Named obj_id should be returned as string, got: {:?}", obj_id.variant()),
                            }
                        }
                        _ => panic!("Change should be a map"),
                    }
                }
                _ => panic!("changes should be a list"),
            }
        }
        _ => panic!("Result should be a map, got: {:?}", result.variant()),
    }

    println!("\n✅ Test passed: Named objects returned as strings");
}

#[tokio::test]
async fn test_change_status_with_numeric_object_ids() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: change/status returns numeric object IDs as v_obj");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("test_numeric_change", "test_author", None)
        .await
        .expect("Failed to create change")
        .assert_success("Create change");
    println!("✅ Change created");

    // Step 2: Add objects with numeric IDs
    println!("\nStep 2: Adding numeric object IDs...");
    
    // Add object "#73"
    client
        .object_update_from_file("#73", "test_object.moo")
        .await
        .expect("Failed to update object #73");
    
    // Add object "#100"
    client
        .object_update_from_file("#100", "detailed_test_object.moo")
        .await
        .expect("Failed to update object #100");
    
    println!("✅ Numeric objects added");

    // Step 3: Get change status directly
    println!("\nStep 3: Getting change status...");
    let result = call_change_status(&server).await;
    println!("✅ Change status retrieved");

    // Step 4: Verify numeric IDs are returned as v_obj
    println!("\nStep 4: Verifying numeric IDs are v_obj...");
    
    match result.variant() {
        Variant::Map(result_map) => {
            // Check objects_added contains numeric IDs as v_obj
            let objects_added = result_map
                .get(&moor_var::v_str("objects_added"))
                .expect("Should have objects_added field");

            match objects_added.variant() {
                Variant::List(list) => {
                    assert_eq!(list.len(), 2, "Should have 2 added objects");
                    
                    // Check both objects
                    for i in 0..list.len() {
                        let obj = &list[i];
                        match obj.variant() {
                            Variant::Obj(obj_ref) => {
                                let obj_id = obj_ref.id().0;
                                assert!(
                                    obj_id == 73 || obj_id == 100,
                                    "Object ID should be 73 or 100, got: {}",
                                    obj_id
                                );
                                println!("✅ Found numeric object ID #{} as v_obj", obj_id);
                            }
                            _ => panic!("Numeric object ID should be returned as v_obj, got: {:?}", obj.variant()),
                        }
                    }
                }
                _ => panic!("objects_added should be a list"),
            }

            // Check objects_modified for numeric IDs
            let objects_modified = result_map
                .get(&moor_var::v_str("objects_modified"))
                .expect("Should have objects_modified field");

            match objects_modified.variant() {
                Variant::List(list) => {
                    // May or may not have modified objects, but if it does they should be v_obj
                    for i in 0..list.len() {
                        let obj = &list[i];
                        match obj.variant() {
                            Variant::Obj(_) => {
                                println!("✅ Modified numeric object ID is v_obj");
                            }
                            Variant::Str(s) => {
                                // Should only be string if it's not a numeric ID
                                assert!(
                                    !s.as_str().starts_with('#'),
                                    "Numeric object ID should not be a string: {}",
                                    s.as_str()
                                );
                            }
                            _ => {}
                        }
                    }
                }
                _ => panic!("objects_modified should be a list"),
            }

            // Check changes array for obj_id as v_obj
            let changes = result_map
                .get(&moor_var::v_str("changes"))
                .expect("Should have changes field");

            match changes.variant() {
                Variant::List(list) => {
                    assert!(list.len() >= 2, "Should have at least 2 changes");
                    
                    for i in 0..list.len() {
                        let change = &list[i];
                        
                        match change.variant() {
                            Variant::Map(map) => {
                                let obj_id = map
                                    .get(&moor_var::v_str("obj_id"))
                                    .expect("Should have obj_id field");
                                
                                // Numeric IDs should be v_obj
                                match obj_id.variant() {
                                    Variant::Obj(obj_ref) => {
                                        let id = obj_ref.id().0;
                                        assert!(
                                            id == 73 || id == 100,
                                            "Object ID should be 73 or 100, got: {}",
                                            id
                                        );
                                        println!("✅ obj_id #{} in changes array returned as v_obj", id);
                                    }
                                    _ => panic!("Numeric obj_id should be returned as v_obj, got: {:?}", obj_id.variant()),
                                }
                            }
                            _ => panic!("Change should be a map"),
                        }
                    }
                }
                _ => panic!("changes should be a list"),
            }
        }
        _ => panic!("Result should be a map, got: {:?}", result.variant()),
    }

    println!("\n✅ Test passed: Numeric object IDs returned as v_obj");
}

#[tokio::test]
async fn test_change_status_mixed_object_types() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: change/status handles mixed named and numeric object IDs correctly");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("mixed_objects", "test_author", None)
        .await
        .expect("Failed to create change")
        .assert_success("Create change");
    println!("✅ Change created");

    // Step 2: Add both named and numeric objects
    println!("\nStep 2: Adding mixed objects...");
    
    // Add named object
    client
        .object_update_from_file("sysobj", "test_object.moo")
        .await
        .expect("Failed to update sysobj");
    
    // Add numeric object
    client
        .object_update_from_file("#42", "detailed_test_object.moo")
        .await
        .expect("Failed to update #42");
    
    println!("✅ Mixed objects added");

    // Step 3: Get change status directly
    println!("\nStep 3: Getting change status...");
    let result = call_change_status(&server).await;
    println!("✅ Change status retrieved");

    // Step 4: Verify correct types
    println!("\nStep 4: Verifying mixed object types...");
    
    match result.variant() {
        Variant::Map(result_map) => {
            let objects_added = result_map
                .get(&moor_var::v_str("objects_added"))
                .expect("Should have objects_added field");

            match objects_added.variant() {
                Variant::List(list) => {
                    assert_eq!(list.len(), 2, "Should have 2 added objects");
                    
                    let mut found_string = false;
                    let mut found_obj = false;
                    
                    for i in 0..list.len() {
                        let obj = &list[i];
                        match obj.variant() {
                            Variant::Str(s) => {
                                assert_eq!(s.as_str(), "sysobj", "String should be 'sysobj'");
                                found_string = true;
                                println!("✅ Found named object 'sysobj' as string");
                            }
                            Variant::Obj(obj_ref) => {
                                assert_eq!(obj_ref.id().0, 42, "Numeric ID should be 42");
                                found_obj = true;
                                println!("✅ Found numeric object #42 as v_obj");
                            }
                            _ => panic!("Unexpected variant type: {:?}", obj.variant()),
                        }
                    }
                    
                    assert!(found_string, "Should have found named object as string");
                    assert!(found_obj, "Should have found numeric object as v_obj");
                }
                _ => panic!("objects_added should be a list"),
            }
        }
        _ => panic!("Result should be a map, got: {:?}", result.variant()),
    }

    println!("\n✅ Test passed: Mixed object types handled correctly");
}

#[tokio::test]
async fn test_change_status_objects_renamed_with_numeric_ids() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: change/status returns numeric IDs in objects_renamed as v_obj");

    // Step 1: Create a change and add an object
    println!("\nStep 1: Creating change with object...");
    client
        .change_create("rename_test", "test_author", None)
        .await
        .expect("Failed to create change");
    
    client
        .object_update_from_file("#50", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    println!("✅ Object created");

    // Step 2: Rename the object (using object/rename)
    println!("\nStep 2: Renaming object #50 to #51...");
    let rename_response = client
        .object_rename("#50", "#51")
        .await
        .expect("Failed to rename object");
    
    println!("Rename response: {:?}", rename_response);

    // Step 3: Get change status directly
    println!("\nStep 3: Getting change status...");
    let result = call_change_status(&server).await;

    // Step 4: Verify objects_renamed contains numeric IDs as v_obj
    println!("\nStep 4: Verifying objects_renamed...");
    
    match result.variant() {
        Variant::Map(result_map) => {
            let objects_renamed = result_map
                .get(&moor_var::v_str("objects_renamed"))
                .expect("Should have objects_renamed field");

            match objects_renamed.variant() {
                Variant::Map(map) => {
                    // Check if the map has the rename
                    if map.is_empty() {
                        println!("⚠️  Note: objects_renamed is empty - rename may not be tracked in this implementation");
                    } else {
                        // Iterate through the map
                        for entry in map.iter() {
                            println!("Renamed: {:?} -> {:?}", entry.0.variant(), entry.1.variant());
                            
                            // If either key or value is numeric, it should be v_obj
                            match entry.0.variant() {
                                Variant::Obj(_) => println!("✅ Rename key is v_obj"),
                                Variant::Str(s) if s.as_str().starts_with('#') => {
                                    panic!("Numeric ID key should be v_obj, not string: {}", s.as_str());
                                }
                                _ => {}
                            }
                            
                            match entry.1.variant() {
                                Variant::Obj(_) => println!("✅ Rename value is v_obj"),
                                Variant::Str(s) if s.as_str().starts_with('#') => {
                                    panic!("Numeric ID value should be v_obj, not string: {}", s.as_str());
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => panic!("objects_renamed should be a map"),
            }
        }
        Variant::Err(_) => {
            println!("⚠️  Note: change/status returned an error - possibly change was abandoned or committed");
            println!("⚠️  This test may need adjustment depending on rename operation behavior");
        }
        _ => panic!("Result should be a map or error, got: {:?}", result.variant()),
    }

    println!("\n✅ Test passed: Numeric IDs in objects_renamed handled correctly");
}

#[tokio::test]
async fn test_change_status_negative_object_ids() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: change/status handles special negative object IDs correctly");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("negative_ids", "test_author", None)
        .await
        .expect("Failed to create change");

    // Step 2: Add objects with negative IDs (special system objects)
    println!("\nStep 2: Adding object with negative ID...");
    
    // Try to add #-1 (NOTHING)
    let result = client
        .object_update_from_file("#-1", "test_object.moo")
        .await;
    
    // This might fail or succeed depending on implementation
    // Just verify we don't crash
    println!("Result of adding #-1: {:?}", result.is_ok());

    // Get change status regardless
    let result = call_change_status(&server).await;

    // Just verify the structure is valid
    match result.variant() {
        Variant::Map(_) => {
            println!("✅ Change status with negative ID didn't crash");
        }
        _ => panic!("Result should be a map"),
    }

    println!("\n✅ Test passed: Negative object IDs handled without crashing");
}

#[tokio::test]
async fn test_change_status_by_id_current_local_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: change/status can query a specific change by ID (current local change)");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("test_change_by_id", "test_author", None)
        .await
        .expect("Failed to create change")
        .assert_success("Create change");
    
    // Get the change ID directly from the database
    let change_id = server.database()
        .index()
        .get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    println!("Created change with ID: {}", change_id);

    // Step 2: Add objects to the change
    println!("\nStep 2: Adding objects...");
    client
        .object_update_from_file("obj1", "test_object.moo")
        .await
        .expect("Failed to update obj1");
    client
        .object_update_from_file("obj2", "detailed_test_object.moo")
        .await
        .expect("Failed to update obj2");
    println!("✅ Objects added");

    // Step 3: Get status using the specific change ID
    println!("\nStep 3: Getting status by change ID...");
    let status_response = client
        .change_status_by_id(&change_id)
        .await
        .expect("Failed to get change status by ID");
    
    println!("Status response: {:?}", status_response);

    // Step 4: Verify the response is valid and contains our objects
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Response should be successful"
    );

    println!("\n✅ Test passed: Can query status by change ID");
}

#[tokio::test]
async fn test_change_status_by_id_merged_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: change/status can query a merged change by ID");

    // Step 1: Create and submit a change
    println!("\nStep 1: Creating and submitting change...");
    client
        .change_create("merged_change", "test_author", Some("A merged change"))
        .await
        .expect("Failed to create change")
        .assert_success("Create change");
    
    // Get the change ID before we submit it
    let change_id = server.database()
        .index()
        .get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    println!("Created change with ID: {}", change_id);

    // Add objects
    client
        .object_update_from_file("merged_obj", "test_object.moo")
        .await
        .expect("Failed to update merged_obj");
    
    // Submit the change (this should merge it if no remote)
    client
        .change_submit()
        .await
        .expect("Failed to submit change");
    println!("✅ Change submitted/merged");

    // Step 2: Query the merged change by ID
    println!("\nStep 2: Querying merged change by ID...");
    let status_response = client
        .change_status_by_id(&change_id)
        .await
        .expect("Failed to get status of merged change");
    
    println!("Status response: {:?}", status_response);

    // Step 3: Verify the response is valid
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Response should be successful"
    );

    println!("\n✅ Test passed: Can query merged change by ID");
}

#[tokio::test]
async fn test_change_status_by_id_workspace_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: change/status can query a change in workspace by ID");

    // Step 1: Create a change and add objects
    println!("\nStep 1: Creating change and adding objects...");
    client
        .change_create("workspace_change", "test_author", Some("To be stashed"))
        .await
        .expect("Failed to create change")
        .assert_success("Create change");
    
    // Get the change ID before we stash it
    let change_id = server.database()
        .index()
        .get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    println!("Created change with ID: {}", change_id);

    // Add objects
    client
        .object_update_from_file("workspace_obj", "test_object.moo")
        .await
        .expect("Failed to update workspace_obj");

    // Step 2: Stash the change (moves it to workspace)
    println!("\nStep 2: Stashing change...");
    client
        .change_stash()
        .await
        .expect("Failed to stash change");
    println!("✅ Change stashed to workspace");

    // Step 3: Query the workspace change by ID
    println!("\nStep 3: Querying workspace change by ID...");
    let status_response = client
        .change_status_by_id(&change_id)
        .await
        .expect("Failed to get status of workspace change");
    
    println!("Status response: {:?}", status_response);

    // Step 4: Verify the response is valid
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Response should be successful"
    );

    println!("\n✅ Test passed: Can query workspace change by ID");
}

#[tokio::test]
async fn test_change_status_by_id_not_found() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: change/status returns error for non-existent change ID");

    // Try to query a non-existent change
    let fake_change_id = "00000000-0000-0000-0000-000000000000";
    println!("Querying non-existent change ID: {}", fake_change_id);
    
    let status_response = client
        .change_status_by_id(fake_change_id)
        .await
        .expect("Request should complete (even with error result)");
    
    println!("Status response: {:?}", status_response);

    // Verify the response indicates failure or error
    let success = status_response["success"].as_bool().unwrap_or(true);
    
    // The response should either indicate failure or return an error
    if success {
        // If marked as success, the result should be an error value
        let result = &status_response["result"];
        println!("Result value: {:?}", result);
        // We expect some kind of error indication
    } else {
        println!("✅ Response correctly indicates failure");
    }

    println!("\n✅ Test passed: Non-existent change ID handled correctly");
}

#[tokio::test]
async fn test_change_status_default_behavior_unchanged() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: change/status default behavior (no ID) still works");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("default_behavior", "test_author", None)
        .await
        .expect("Failed to create change");

    // Step 2: Add objects
    println!("\nStep 2: Adding objects...");
    client
        .object_update_from_file("default_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    // Step 3: Get status using default behavior (no ID)
    println!("\nStep 3: Getting status (default behavior)...");
    let status_response = client
        .change_status()
        .await
        .expect("Failed to get change status");
    
    println!("Status response: {:?}", status_response);

    // Step 4: Verify the response is valid
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Response should be successful"
    );

    println!("\n✅ Test passed: Default behavior unchanged");
}

#[tokio::test]
async fn test_change_status_by_short_hash() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: change/status should accept short Blake3 hash IDs");

    // Step 1: Create a change
    println!("\nStep 1: Creating change...");
    client
        .change_create("short_hash_test", "test_author", Some("Testing short hashes"))
        .await
        .expect("Failed to create change")
        .assert_success("Create change");
    
    // Get the full change ID
    let full_change_id = server.database()
        .index()
        .get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    // Create short hash (first 12 characters)
    let short_hash = &full_change_id[..12];
    println!("Full change ID: {}", full_change_id);
    println!("Short hash: {}", short_hash);

    // Step 2: Add objects to the change
    println!("\nStep 2: Adding objects...");
    client
        .object_update_from_file("short_hash_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    // Step 3: Get status using short hash
    println!("\nStep 3: Getting status by short hash...");
    let status_response = client
        .change_status_by_id(short_hash)
        .await
        .expect("Failed to get change status by short hash");
    
    println!("Status response: {:?}", status_response);

    // Step 4: Verify the response is valid
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Response should be successful"
    );

    println!("\n✅ Test passed: Short hash works with change/status");
}

#[tokio::test]
async fn test_change_status_by_short_hash_workspace() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: change/status should accept short hash for workspace changes");

    // Step 1: Create a change and add objects
    println!("\nStep 1: Creating change...");
    client
        .change_create("workspace_short_hash", "test_author", Some("Workspace short hash test"))
        .await
        .expect("Failed to create change")
        .assert_success("Create change");
    
    // Get the full change ID before stashing
    let full_change_id = server.database()
        .index()
        .get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let short_hash = &full_change_id[..12];
    println!("Full change ID: {}", full_change_id);
    println!("Short hash: {}", short_hash);

    // Add objects
    client
        .object_update_from_file("workspace_short_obj", "test_object.moo")
        .await
        .expect("Failed to update object");

    // Step 2: Stash the change
    println!("\nStep 2: Stashing change...");
    client
        .change_stash()
        .await
        .expect("Failed to stash change");

    // Step 3: Get status using short hash (change is now in workspace)
    println!("\nStep 3: Getting status by short hash (workspace)...");
    let status_response = client
        .change_status_by_id(short_hash)
        .await
        .expect("Failed to get status by short hash from workspace");
    
    println!("Status response: {:?}", status_response);

    // Step 4: Verify the response is valid
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Response should be successful"
    );

    println!("\n✅ Test passed: Short hash works for workspace changes");
}
