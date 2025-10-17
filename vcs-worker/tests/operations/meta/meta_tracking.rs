//! Integration tests for meta change tracking in ObjectChange
//!
//! These tests verify that meta changes (ignored/unignored properties and verbs)
//! are correctly tracked in change/status and object/history operations.

use crate::common::*;
use moor_var::{Associative, Sequence, Variant};
use moor_vcs_worker::operations::Operation;

#[tokio::test]
async fn test_meta_tracking_add_ignored_property() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Meta tracking when adding an ignored property");

    // Step 1: Create an object with properties
    println!("\nStep 1: Creating object...");
    client
        .object_update_from_file("test_meta_track", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");

    client
        .change_submit()
        .await
        .expect("Failed to submit initial change");
    println!("✅ Initial object created and submitted");

    // Step 2: Add a property to ignored list
    println!("\nStep 2: Adding property to ignored list...");
    client
        .meta_add_ignored_property("test_meta_track", "test_property")
        .await
        .expect("Failed to add ignored property");
    println!("✅ Property added to ignored list");

    // Step 3: Get change status and verify meta tracking
    println!("\nStep 3: Checking change status...");
    let (top_change_id, _) = server.db_assertions().require_top_change();
    let change = server
        .database()
        .index()
        .get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");

    let diff =
        moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
            .expect("Failed to build diff");

    // Find the object change
    let obj_change = diff
        .changes
        .iter()
        .find(|c| c.obj_id == "test_meta_track")
        .expect("Should have object change for test_meta_track");

    // Verify meta_ignored_properties contains our property
    assert!(
        obj_change.meta_ignored_properties.contains("test_property"),
        "meta_ignored_properties should contain 'test_property'"
    );
    assert!(
        obj_change.meta_unignored_properties.is_empty(),
        "meta_unignored_properties should be empty"
    );
    println!("✅ Meta tracking shows property was ignored");
}

#[tokio::test]
async fn test_meta_tracking_remove_ignored_property() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Meta tracking when removing an ignored property");

    // Step 1: Create object with ignored property
    println!("\nStep 1: Creating object and ignoring property...");
    client
        .object_update_from_file("test_meta_track", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");

    client
        .change_submit()
        .await
        .expect("Failed to submit initial change");

    client
        .meta_add_ignored_property("test_meta_track", "test_property")
        .await
        .expect("Failed to add ignored property");

    client
        .change_submit()
        .await
        .expect("Failed to submit meta change");
    println!("✅ Object created with ignored property");

    // Step 2: Remove from ignored list
    println!("\nStep 2: Removing property from ignored list...");
    client
        .meta_remove_ignored_property("test_meta_track", "test_property")
        .await
        .expect("Failed to remove ignored property");
    println!("✅ Property removed from ignored list");

    // Step 3: Verify meta tracking
    println!("\nStep 3: Checking change status...");
    let (top_change_id, _) = server.db_assertions().require_top_change();
    let change = server
        .database()
        .index()
        .get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");

    let diff =
        moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
            .expect("Failed to build diff");

    let obj_change = diff
        .changes
        .iter()
        .find(|c| c.obj_id == "test_meta_track")
        .expect("Should have object change for test_meta_track");

    // Verify meta_unignored_properties contains our property
    assert!(
        obj_change.meta_unignored_properties.contains("test_property"),
        "meta_unignored_properties should contain 'test_property'"
    );
    assert!(
        obj_change.meta_ignored_properties.is_empty(),
        "meta_ignored_properties should be empty"
    );
    println!("✅ Meta tracking shows property was unignored");
}

#[tokio::test]
async fn test_meta_tracking_add_ignored_verb() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Meta tracking when adding an ignored verb");

    // Step 1: Create an object with verbs
    println!("\nStep 1: Creating object...");
    client
        .object_update_from_file("test_meta_track", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");

    client
        .change_submit()
        .await
        .expect("Failed to submit initial change");
    println!("✅ Initial object created and submitted");

    // Step 2: Add a verb to ignored list
    println!("\nStep 2: Adding verb to ignored list...");
    client
        .meta_add_ignored_verb("test_meta_track", "test_verb")
        .await
        .expect("Failed to add ignored verb");
    println!("✅ Verb added to ignored list");

    // Step 3: Verify meta tracking
    println!("\nStep 3: Checking change status...");
    let (top_change_id, _) = server.db_assertions().require_top_change();
    let change = server
        .database()
        .index()
        .get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");

    let diff =
        moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
            .expect("Failed to build diff");

    let obj_change = diff
        .changes
        .iter()
        .find(|c| c.obj_id == "test_meta_track")
        .expect("Should have object change for test_meta_track");

    // Verify meta_ignored_verbs contains our verb
    assert!(
        obj_change.meta_ignored_verbs.contains("test_verb"),
        "meta_ignored_verbs should contain 'test_verb'"
    );
    assert!(
        obj_change.meta_unignored_verbs.is_empty(),
        "meta_unignored_verbs should be empty"
    );
    println!("✅ Meta tracking shows verb was ignored");
}

#[tokio::test]
async fn test_meta_tracking_ignore_then_unignore_same_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Meta tracking when ignoring then unignoring in same change");

    // Step 1: Create object
    println!("\nStep 1: Creating object...");
    client
        .object_update_from_file("test_meta_track", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");

    client
        .change_submit()
        .await
        .expect("Failed to submit initial change");
    println!("✅ Initial object created");

    // Step 2: Add property to ignored list
    println!("\nStep 2: Adding property to ignored list...");
    client
        .meta_add_ignored_property("test_meta_track", "test_property")
        .await
        .expect("Failed to add ignored property");
    println!("✅ Property added to ignored list");

    // Step 3: Remove from ignored list (same change)
    println!("\nStep 3: Removing property from ignored list in same change...");
    client
        .meta_remove_ignored_property("test_meta_track", "test_property")
        .await
        .expect("Failed to remove ignored property");
    println!("✅ Property removed from ignored list");

    // Step 4: Verify meta tracking shows nothing (should cancel out)
    println!("\nStep 4: Checking change status...");
    let (top_change_id, _) = server.db_assertions().require_top_change();
    let change = server
        .database()
        .index()
        .get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");

    let diff =
        moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
            .expect("Failed to build diff");

    // When a property is ignored then unignored in the same change, it cancels out completely.
    // In this case, no ObjectChange is created at all because there are no net changes.
    let obj_change = diff
        .changes
        .iter()
        .find(|c| c.obj_id == "test_meta_track");

    if let Some(obj_change) = obj_change {
        // If an ObjectChange exists, both should be empty (cancelled out)
        assert!(
            obj_change.meta_ignored_properties.is_empty(),
            "meta_ignored_properties should be empty (cancelled out)"
        );
        assert!(
            obj_change.meta_unignored_properties.is_empty(),
            "meta_unignored_properties should be empty (cancelled out)"
        );
        println!("✅ Meta tracking correctly cancelled out ignore/unignore (empty ObjectChange)");
    } else {
        // No ObjectChange at all is also correct - complete cancellation
        println!("✅ Meta tracking correctly cancelled out ignore/unignore (no ObjectChange)");
    }
}

#[tokio::test]
async fn test_meta_tracking_moo_var_output() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Meta tracking in MOO var output");

    // Step 1: Create object and add meta changes
    println!("\nStep 1: Creating object and meta changes...");
    client
        .object_update_from_file("test_meta_track", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");

    client
        .change_submit()
        .await
        .expect("Failed to submit initial change");

    client
        .meta_add_ignored_property("test_meta_track", "test_property")
        .await
        .expect("Failed to add ignored property");

    client
        .meta_add_ignored_verb("test_meta_track", "test_verb")
        .await
        .expect("Failed to add ignored verb");
    println!("✅ Meta changes added");

    // Step 2: Get the change status as MOO var
    println!("\nStep 2: Getting change status...");
    let change_status_op =
        moor_vcs_worker::operations::ChangeStatusOperation::new(server.database().clone());
    let wizard_user = server.get_wizard_user().expect("Failed to get wizard user");
    let result = change_status_op.execute(vec![], &wizard_user);

    // Step 3: Verify meta is present in the MOO var
    println!("\nStep 3: Verifying meta in MOO var output...");
    match result.variant() {
        Variant::Map(result_map) => {
            let changes = result_map
                .get(&moor_var::v_str("changes"))
                .expect("Should have changes field");

            match changes.variant() {
                Variant::List(list) => {
                    assert!(list.len() > 0, "Should have at least one change");
                    let first_change = &list[0];

                    match first_change.variant() {
                        Variant::Map(change_map) => {
                            // Check for meta field
                            let meta = change_map
                                .get(&moor_var::v_str("meta"))
                                .expect("Should have meta field");

                            match meta.variant() {
                                Variant::Map(meta_map) => {
                                    // Check ignored_properties
                                    let ignored_props = meta_map
                                        .get(&moor_var::v_str("ignored_properties"))
                                        .expect("Should have ignored_properties field");

                                    match ignored_props.variant() {
                                        Variant::List(props_list) => {
                                            assert_eq!(
                                                props_list.len(),
                                                1,
                                                "Should have 1 ignored property"
                                            );
                                            println!("✅ ignored_properties present with correct count");
                                        }
                                        _ => panic!("ignored_properties should be a list"),
                                    }

                                    // Check ignored_verbs
                                    let ignored_verbs = meta_map
                                        .get(&moor_var::v_str("ignored_verbs"))
                                        .expect("Should have ignored_verbs field");

                                    match ignored_verbs.variant() {
                                        Variant::List(verbs_list) => {
                                            assert_eq!(
                                                verbs_list.len(),
                                                1,
                                                "Should have 1 ignored verb"
                                            );
                                            println!("✅ ignored_verbs present with correct count");
                                        }
                                        _ => panic!("ignored_verbs should be a list"),
                                    }
                                }
                                _ => panic!("meta should be a map"),
                            }
                        }
                        _ => panic!("change should be a map"),
                    }
                }
                _ => panic!("changes should be a list"),
            }
        }
        Variant::Err(e) => panic!("Operation failed with error: {:?}", e),
        _ => panic!("Expected map result, got: {:?}", result.variant()),
    }

    println!("✅ Meta tracking correctly appears in MOO var output");
}

#[tokio::test]
async fn test_meta_tracking_multiple_properties() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Meta tracking with multiple properties and verbs");

    // Step 1: Create object
    println!("\nStep 1: Creating object...");
    client
        .object_update_from_file("test_meta_track", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");

    client
        .change_submit()
        .await
        .expect("Failed to submit initial change");

    // Step 2: Add multiple properties and verbs to ignored
    println!("\nStep 2: Adding multiple items to ignored lists...");
    client
        .meta_add_ignored_property("test_meta_track", "test_property")
        .await
        .expect("Failed to add ignored property");

    client
        .meta_add_ignored_property("test_meta_track", "another_property")
        .await
        .expect("Failed to add another ignored property");

    client
        .meta_add_ignored_verb("test_meta_track", "test_verb")
        .await
        .expect("Failed to add ignored verb");

    // Step 3: Verify all are tracked
    println!("\nStep 3: Verifying all items are tracked...");
    let (top_change_id, _) = server.db_assertions().require_top_change();
    let change = server
        .database()
        .index()
        .get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");

    let diff =
        moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
            .expect("Failed to build diff");

    let obj_change = diff
        .changes
        .iter()
        .find(|c| c.obj_id == "test_meta_track")
        .expect("Should have object change for test_meta_track");

    assert_eq!(
        obj_change.meta_ignored_properties.len(),
        2,
        "Should have 2 ignored properties"
    );
    assert!(obj_change.meta_ignored_properties.contains("test_property"));
    assert!(obj_change.meta_ignored_properties.contains("another_property"));

    assert_eq!(
        obj_change.meta_ignored_verbs.len(),
        1,
        "Should have 1 ignored verb"
    );
    assert!(obj_change.meta_ignored_verbs.contains("test_verb"));

    println!("✅ All meta changes tracked correctly");
}

#[tokio::test]
async fn test_meta_tracking_in_object_history() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Meta tracking appears in object history");

    // Step 1: Create object
    println!("\nStep 1: Creating object...");
    client
        .object_update_from_file("test_meta_track", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");

    client
        .change_submit()
        .await
        .expect("Failed to submit initial change");

    // Step 2: Add meta changes and submit
    println!("\nStep 2: Adding meta changes...");
    client
        .meta_add_ignored_property("test_meta_track", "test_property")
        .await
        .expect("Failed to add ignored property");

    client
        .change_submit()
        .await
        .expect("Failed to submit meta change");

    // Step 3: Get object history
    println!("\nStep 3: Getting object history...");
    let history_op =
        moor_vcs_worker::operations::ObjectHistoryOperation::new(server.database().clone());
    let wizard_user = server.get_wizard_user().expect("Failed to get wizard user");
    let result = history_op.execute(vec!["test_meta_track".to_string()], &wizard_user);

    // Step 4: Verify meta appears in history
    println!("\nStep 4: Verifying meta in history...");
    match result.variant() {
        Variant::List(history_list) => {
            // Find the history entry with the meta change
            let mut found_meta = false;
            for entry in history_list.iter() {
                if let Variant::Map(entry_map) = entry.variant() {
                    if let Ok(details) = entry_map.get(&moor_var::v_str("details")) {
                        if let Variant::Map(details_map) = details.variant() {
                            if let Ok(meta) = details_map.get(&moor_var::v_str("meta")) {
                                found_meta = true;
                                println!("✅ Found meta in history entry");
                                
                                // Verify structure
                                if let Variant::Map(meta_map) = meta.variant() {
                                    let has_key = meta_map
                                        .contains_key(&moor_var::v_str("ignored_properties"), true)
                                        .unwrap_or(false);
                                    assert!(
                                        has_key,
                                        "Meta should have ignored_properties field"
                                    );
                                    println!("✅ Meta structure is correct");
                                }
                            }
                        }
                    }
                }
            }
            assert!(found_meta, "Should find meta in at least one history entry");
        }
        Variant::Err(e) => panic!("Operation failed with error: {:?}", e),
        _ => panic!("Expected list result, got: {:?}", result.variant()),
    }

    println!("✅ Meta tracking correctly appears in object history");
}

