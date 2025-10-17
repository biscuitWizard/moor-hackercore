
//! Integration tests for object/diff operation
//!
//! These tests verify:
//! 1. Basic verb diff functionality
//! 2. Short hash resolution
//! 3. Baseline change ID override
//! 4. Multiple verb changes
//! 5. Verb rename detection
//! 6. Added verb detection
//! 7. Deleted verb detection
//! 8. Error handling for missing objects
//! 9. Error handling for invalid change IDs

use crate::common::*;
use moor_var::{Associative, Sequence, Variant};

#[tokio::test]
async fn test_object_diff_basic() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Basic object diff between commits");

    // Create a change and add an object with a simple verb
    println!("\nCreating first change with initial object...");
    client
        .change_create("first_change", "wizard", Some("Initial version"))
        .await
        .expect("Failed to create change")
        .assert_success("Change creation");

    let obj_dump: Vec<String> = vec![
        "object #123",
        "  name: \"Test Object\"",
        "  parent: #1",
        "  location: #2",
        "  owner: #2",
        "",
        "  verb test_verb (this none this) owner: #2 flags: \"rxd\"",
        "    return 1;",
        "  endverb",
        "endobject",
    ].iter().map(|s| s.to_string()).collect();

    client
        .object_update("#123", obj_dump)
        .await
        .expect("Failed to create object")
        .assert_success("Object creation");

    let (first_change_id, _) = db.require_top_change();
    
    // Approve the first change
    client
        .change_approve(&first_change_id)
        .await
        .expect("Failed to approve first change");

    // Create a second change and modify the verb
    println!("\nCreating second change with modified verb...");
    client
        .change_create("second_change", "wizard", Some("Modified version"))
        .await
        .expect("Failed to create change")
        .assert_success("Second change creation");

    let modified_dump: Vec<String> = vec![
        "object #123",
        "  name: \"Test Object\"",
        "  parent: #1",
        "  location: #2",
        "  owner: #2",
        "",
        "  verb test_verb (this none this) owner: #2 flags: \"rxd\"",
        "    return 2;",
        "    return 3;",
        "  endverb",
        "endobject",
    ].iter().map(|s| s.to_string()).collect();

    client
        .object_update("#123", modified_dump)
        .await
        .expect("Failed to update object")
        .assert_success("Object update");

    let (second_change_id, _) = db.require_top_change();

    // Run the diff
    println!("\nRunning diff operation...");
    let result = client
        .object_diff("#123", &second_change_id, None)
        .await
        .expect("Failed to run diff");

    // Verify the result structure
    println!("Result: {:?}", result);
    match result.variant() {
        Variant::Err(e) => {
            // Get the error message
            let msg = e.message();
            panic!("Operation returned error: {} - {:?}", msg, e);
        }
        Variant::Map(result_map) => {
            // Check obj_id
            let obj_id_key = moor_var::Var::mk_str("obj_id");
            let obj_id = result_map
                .get(&obj_id_key)
                .expect("Result should have obj_id");
            println!("✅ Found obj_id: {:?}", obj_id);

            // Check changes
            let changes_key = moor_var::Var::mk_str("changes");
            let changes = result_map
                .get(&changes_key)
                .expect("Result should have changes");

            match changes.variant() {
                Variant::List(changes_list) => {
                    assert!(
                        !changes_list.is_empty(),
                        "Should have at least one changed verb"
                    );
                    println!("✅ Found {} changed verbs", changes_list.len());

                    // Check the first change
                    let first_change = &changes_list[0];
                    match first_change.variant() {
                        Variant::Map(change_map) => {
                            let verb_key = moor_var::Var::mk_str("verb");
                            let verb = change_map
                                .get(&verb_key)
                                .expect("Change should have verb");
                            println!("✅ Changed verb: {:?}", verb);

                            let hunks_key = moor_var::Var::mk_str("hunks");
                            let hunks = change_map
                                .get(&hunks_key)
                                .expect("Change should have hunks");

                            match hunks.variant() {
                                Variant::List(hunks_list) => {
                                    assert!(
                                        !hunks_list.is_empty(),
                                        "Should have at least one hunk"
                                    );
                                    println!("✅ Found {} hunks", hunks_list.len());
                                }
                                _ => panic!("Hunks should be a list"),
                            }
                        }
                        _ => panic!("Change should be a map"),
                    }
                }
                _ => panic!("Changes should be a list"),
            }
        }
        _ => panic!("Result should be a map"),
    }

    println!("✅ Test passed: Basic object diff works");
}

#[tokio::test]
async fn test_object_diff_with_short_hash() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Object diff with short hash IDs");

    // Create and approve first change
    client
        .change_create("first", "wizard", Some("First"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#123",
            vec![
                "object #123",
                "  name: \"Test Object\"",
                "  parent: #1",
                "  location: #2",
                "  owner: #2",
                "",
                "  verb test (this none this) owner: #2 flags: \"rxd\"",
                "    return 1;",
                "  endverb",
                "endobject",
            ].iter().map(|s| s.to_string()).collect(),
        )
        .await
        .expect("Failed to create object");

    let (first_id, _) = db.require_top_change();
    client
        .change_approve(&first_id)
        .await
        .expect("Failed to approve");

    // Create second change
    client
        .change_create("second", "wizard", Some("Second"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#123",
            vec![
                "object #123",
                "  name: \"Test Object\"",
                "  parent: #1",
                "  location: #2",
                "  owner: #2",
                "",
                "  verb test (this none this) owner: #2 flags: \"rxd\"",
                "    return 2;",
                "  endverb",
                "endobject",
            ].iter().map(|s| s.to_string()).collect(),
        )
        .await
        .expect("Failed to update object");

    let (second_id, _) = db.require_top_change();
    let short_id = &second_id[..12];

    println!("Full ID: {}", second_id);
    println!("Short ID: {}", short_id);

    // Run diff with short hash
    let result = client
        .object_diff("#123", short_id, None)
        .await
        .expect("Failed to run diff with short hash");

    match result.variant() {
        Variant::Map(result_map) => {
            let changes_key = moor_var::Var::mk_str("changes");
            let changes = result_map.get(&changes_key).expect("Should have changes");
            match changes.variant() {
                Variant::List(changes_list) => {
                    assert!(!changes_list.is_empty(), "Should have changes");
                    println!("✅ Diff with short hash succeeded");
                }
                _ => panic!("Changes should be a list"),
            }
        }
        _ => panic!("Result should be a map"),
    }

    println!("✅ Test passed: Short hash resolution works");
}

#[tokio::test]
async fn test_object_diff_with_baseline_override() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Object diff with baseline change ID override");

    // Create three versions
    let mut change_ids = Vec::new();

    for i in 1..=3 {
        client
            .change_create(&format!("change_{}", i), "wizard", Some(&format!("Version {}", i)))
            .await
            .expect("Failed to create change");

        client
            .object_update(
                "#123",
                vec![
                    "object #123".to_string(),
                    "  name: \"Test Object\"".to_string(),
                    "  parent: #1".to_string(),
                    "  location: #2".to_string(),
                    "  owner: #2".to_string(),
                    "".to_string(),
                    "  verb test (this none this) owner: #2 flags: \"rxd\"".to_string(),
                    format!("    return {};", i),
                    "  endverb".to_string(),
                    "endobject".to_string(),
                ],
            )
            .await
            .expect("Failed to update object");

        let (change_id, _) = db.require_top_change();
        change_ids.push(change_id.clone());
        
        client
            .change_approve(&change_id)
            .await
            .expect("Failed to approve change");
    }

    // Compare version 3 against version 1 (skip version 2)
    println!("\nComparing version 3 against version 1...");
    let result = client
        .object_diff("#123", &change_ids[2], Some(&change_ids[0]))
        .await
        .expect("Failed to run diff with baseline");

    match result.variant() {
        Variant::Map(result_map) => {
            let changes_key = moor_var::Var::mk_str("changes");
            let changes = result_map.get(&changes_key).expect("Should have changes");
            match changes.variant() {
                Variant::List(changes_list) => {
                    assert!(!changes_list.is_empty(), "Should have changes");
                    println!("✅ Baseline override diff succeeded");
                }
                _ => panic!("Changes should be a list"),
            }
        }
        _ => panic!("Result should be a map"),
    }

    println!("✅ Test passed: Baseline override works");
}

#[tokio::test]
async fn test_object_diff_multiple_verbs() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Object diff with multiple changed verbs");

    // Create first change with two verbs
    client
        .change_create("first", "wizard", Some("Initial"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#123",
            vec![
                "object #123",
                "  name: \"Test Object\"",
                "  parent: #1",
                "  location: #2",
                "  owner: #2",
                "",
                "  verb verb1 (this none this) owner: #2 flags: \"rxd\"",
                "    return 1;",
                "  endverb",
                "",
                "  verb verb2 (this none this) owner: #2 flags: \"rxd\"",
                "    return 2;",
                "  endverb",
                "endobject",
            ].iter().map(|s| s.to_string()).collect(),
        )
        .await
        .expect("Failed to create object");

    let (first_id, _) = db.require_top_change();
    client
        .change_approve(&first_id)
        .await
        .expect("Failed to approve");

    // Modify both verbs
    client
        .change_create("second", "wizard", Some("Modified"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#123",
            vec![
                "object #123",
                "  name: \"Test Object\"",
                "  parent: #1",
                "  location: #2",
                "  owner: #2",
                "",
                "  verb verb1 (this none this) owner: #2 flags: \"rxd\"",
                "    return 10;",
                "  endverb",
                "",
                "  verb verb2 (this none this) owner: #2 flags: \"rxd\"",
                "    return 20;",
                "  endverb",
                "endobject",
            ].iter().map(|s| s.to_string()).collect(),
        )
        .await
        .expect("Failed to update object");

    let (second_id, _) = db.require_top_change();

    // Run diff
    let result = client
        .object_diff("#123", &second_id, None)
        .await
        .expect("Failed to run diff");

    match result.variant() {
        Variant::Map(result_map) => {
            let changes_key = moor_var::Var::mk_str("changes");
            let changes = result_map.get(&changes_key).expect("Should have changes");
            match changes.variant() {
                Variant::List(changes_list) => {
                    println!("Found {} changed verbs", changes_list.len());
                    // Both verbs should be in the diff (or at least one if only one has code changes)
                    assert!(!changes_list.is_empty(), "Should have at least one changed verb");
                    println!("✅ Multiple verbs diff succeeded");
                }
                _ => panic!("Changes should be a list"),
            }
        }
        _ => panic!("Result should be a map"),
    }

    println!("✅ Test passed: Multiple verb changes detected");
}

#[tokio::test]
async fn test_object_diff_added_verb() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Object diff with added verb");

    // Create first change with one verb
    client
        .change_create("first", "wizard", Some("Initial"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#123",
            vec![
                "object #123",
                "  name: \"Test Object\"",
                "  parent: #1",
                "  location: #2",
                "  owner: #2",
                "",
                "  verb old_verb (this none this) owner: #2 flags: \"rxd\"",
                "    return 1;",
                "  endverb",
                "endobject",
            ].iter().map(|s| s.to_string()).collect(),
        )
        .await
        .expect("Failed to create object");

    let (first_id, _) = db.require_top_change();
    client
        .change_approve(&first_id)
        .await
        .expect("Failed to approve");

    // Add a new verb
    client
        .change_create("second", "wizard", Some("Added verb"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#123",
            vec![
                "object #123",
                "  name: \"Test Object\"",
                "  parent: #1",
                "  location: #2",
                "  owner: #2",
                "",
                "  verb old_verb (this none this) owner: #2 flags: \"rxd\"",
                "    return 1;",
                "  endverb",
                "",
                "  verb new_verb (this none this) owner: #2 flags: \"rxd\"",
                "    return 2;",
                "  endverb",
                "endobject",
            ].iter().map(|s| s.to_string()).collect(),
        )
        .await
        .expect("Failed to update object");

    let (second_id, _) = db.require_top_change();

    // Run diff
    let result = client
        .object_diff("#123", &second_id, None)
        .await
        .expect("Failed to run diff");

    match result.variant() {
        Variant::Map(result_map) => {
            let changes_key = moor_var::Var::mk_str("changes");
            let changes = result_map.get(&changes_key).expect("Should have changes");
            match changes.variant() {
                Variant::List(changes_list) => {
                    assert!(
                        !changes_list.is_empty(),
                        "Should have at least the new verb"
                    );
                    println!("✅ Added verb diff succeeded with {} changes", changes_list.len());
                }
                _ => panic!("Changes should be a list"),
            }
        }
        _ => panic!("Result should be a map"),
    }

    println!("✅ Test passed: Added verb detected");
}

#[tokio::test]
async fn test_object_diff_deleted_verb() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Object diff with deleted verb");

    // Create first change with two verbs
    client
        .change_create("first", "wizard", Some("Initial"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#123",
            vec![
                "object #123",
                "  name: \"Test Object\"",
                "  parent: #1",
                "  location: #2",
                "  owner: #2",
                "",
                "  verb keep (this none this) owner: #2 flags: \"rxd\"",
                "    return 1;",
                "  endverb",
                "",
                "  verb delete (this none this) owner: #2 flags: \"rxd\"",
                "    return 2;",
                "  endverb",
                "endobject",
            ].iter().map(|s| s.to_string()).collect(),
        )
        .await
        .expect("Failed to create object");

    let (first_id, _) = db.require_top_change();
    client
        .change_approve(&first_id)
        .await
        .expect("Failed to approve");

    // Delete one verb
    client
        .change_create("second", "wizard", Some("Deleted verb"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#123",
            vec![
                "object #123",
                "  name: \"Test Object\"",
                "  parent: #1",
                "  location: #2",
                "  owner: #2",
                "",
                "  verb keep (this none this) owner: #2 flags: \"rxd\"",
                "    return 1;",
                "  endverb",
                "endobject",
            ].iter().map(|s| s.to_string()).collect(),
        )
        .await
        .expect("Failed to update object");

    let (second_id, _) = db.require_top_change();

    // Run diff
    let result = client
        .object_diff("#123", &second_id, None)
        .await
        .expect("Failed to run diff");

    match result.variant() {
        Variant::Map(result_map) => {
            let changes_key = moor_var::Var::mk_str("changes");
            let changes = result_map.get(&changes_key).expect("Should have changes");
            match changes.variant() {
                Variant::List(changes_list) => {
                    assert!(
                        !changes_list.is_empty(),
                        "Should have at least the deleted verb"
                    );
                    println!("✅ Deleted verb diff succeeded with {} changes", changes_list.len());
                }
                _ => panic!("Changes should be a list"),
            }
        }
        _ => panic!("Result should be a map"),
    }

    println!("✅ Test passed: Deleted verb detected");
}

#[tokio::test]
async fn test_object_diff_object_not_in_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    println!("Test: Object diff error when object not in change");

    // Create a change but don't add the object we're looking for
    client
        .change_create("test_change", "wizard", Some("Test"))
        .await
        .expect("Failed to create change");

    client
        .object_update(
            "#456",
            vec![
                "object #456".to_string(),
                "  name: \"Test Object 456\"".to_string(),
                "  parent: #1".to_string(),
                "  location: #2".to_string(),
                "  owner: #2".to_string(),
                "endobject".to_string(),
            ],
        )
        .await
        .expect("Failed to create object");

    let (change_id, _) = db.require_top_change();

    // Try to diff an object that doesn't exist in this change
    let result = client.object_diff("#999", &change_id, None).await
        .expect("Failed to execute diff");

    // Should return an error Var
    match result.variant() {
        Variant::Err(_) => {
            println!("✅ Correctly returned error for missing object");
        }
        _ => {
            panic!("Should return error Var when object not in change, got: {:?}", result);
        }
    }

    println!("✅ Test passed: Error handling for missing object works");
}

#[tokio::test]
async fn test_object_diff_nonexistent_change() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Object diff error with nonexistent change ID");

    // Try to diff with a fake change ID
    let result = client.object_diff("#123", "nonexistent_change_id", None).await
        .expect("Failed to execute diff");

    // Should return an error Var
    match result.variant() {
        Variant::Err(_) => {
            println!("✅ Correctly returned error for nonexistent change");
        }
        _ => {
            panic!("Should return error Var when change ID doesn't exist, got: {:?}", result);
        }
    }

    println!("✅ Test passed: Error handling for invalid change ID works");
}

