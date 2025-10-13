//! Integration tests for change/abandon diff inversion
//!
//! These tests verify that the abandon operation returns properly inverted diffs
//! that show the exact operations needed to undo a change.

use crate::common::*;
use serde_json::Value;

/// Helper to extract a list field from a MOO-style map
fn get_list_field(map: &Value, field: &str) -> Vec<String> {
    map.get(field)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// Helper to extract a map field from a MOO-style map
#[allow(dead_code)]
fn get_map_field(map: &Value, field: &str) -> Vec<(String, String)> {
    map.get(field)
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| {
                    v.as_str().map(|s| (k.to_string(), s.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn test_abandon_verb_added_becomes_verb_deleted() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: When a change adds a verb, abandon should show that verb as deleted");

    // Step 1: Create a change and add an object with a verb
    println!("\nStep 1: Creating change with object containing 'test' verb...");
    client
        .change_create("test_abandon_verb", "test_author", None)
        .await
        .expect("Failed to create change");

    // Create an object with a 'test' verb
    let object_content = vec![
        "object #9".to_string(),
        "  name: \"vcs\"".to_string(),
        "  parent: #1".to_string(),
        "  location: #2".to_string(),
        "  owner: #2".to_string(),
        "".to_string(),
        "  verb test (this none this) owner: #2 flags: \"rxd\"".to_string(),
        "    return 1;".to_string(),
        "  endverb".to_string(),
        "endobject".to_string(),
        "".to_string(),
    ];

    client
        .object_update("vcs", object_content)
        .await
        .expect("Failed to update object");

    println!("✅ Created change with object 'vcs' having verb 'test'");

    // Step 2: Abandon the change and inspect the returned diff
    println!("\nStep 2: Abandoning change and inspecting diff...");
    let response = client
        .change_abandon()
        .await
        .expect("Failed to abandon change");

    // The response should be a success with a diff model
    let result = response.get("result").expect("No result field");
    
    println!("Abandon result: {}", serde_json::to_string_pretty(result).unwrap());

    // Extract the changes array
    let changes = result
        .get("changes")
        .and_then(|v| v.as_array())
        .expect("No changes array in result");

    // Find the change for object 'vcs'
    let vcs_change = changes
        .iter()
        .find(|c| {
            c.get("obj_id")
                .and_then(|v| v.as_str())
                .map(|s| s == "vcs")
                .unwrap_or(false)
        })
        .expect("No change found for object 'vcs'");

    println!("\nVCS change: {}", serde_json::to_string_pretty(vcs_change).unwrap());

    // Verify that verbs_deleted contains "test" (because we're undoing an addition)
    let verbs_deleted = get_list_field(vcs_change, "verbs_deleted");
    assert!(
        verbs_deleted.contains(&"test".to_string()),
        "Expected 'test' in verbs_deleted, got: {:?}",
        verbs_deleted
    );

    // Verify that verbs_added is empty (we're not adding anything back)
    let verbs_added = get_list_field(vcs_change, "verbs_added");
    assert!(
        verbs_added.is_empty(),
        "Expected empty verbs_added, got: {:?}",
        verbs_added
    );

    println!("✅ Abandon correctly shows verb 'test' as deleted (inverse of adding)");
}

#[tokio::test]
async fn test_abandon_modified_verb_shows_modification() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: When a change modifies a verb, abandon should show that verb as modified");

    // Step 1: Create initial change with a verb
    println!("\nStep 1: Creating initial change with 'examine' verb...");
    client
        .change_create("initial_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("test_obj", "object_with_examine_verb.moo")
        .await
        .expect("Failed to create object with examine verb");

    // Approve the change to make it the baseline
    let (change_id, _) = server.db_assertions().require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve change")
        .assert_success("Approve change");

    println!("✅ Initial change approved with 'examine' verb");

    // Step 2: Create a new change that modifies the verb
    println!("\nStep 2: Creating change that modifies the verb...");
    client
        .change_create("modify_change", "test_author", None)
        .await
        .expect("Failed to create change");

    // Modify the object by changing the verb content
    let modified_content = vec![
        "object #3".to_string(),
        "  name: \"Test Object\"".to_string(),
        "  parent: #1".to_string(),
        "  location: #2".to_string(),
        "  owner: #2".to_string(),
        "".to_string(),
        "  verb examine (this none this) owner: #2 flags: \"rxd\"".to_string(),
        "    player:tell(\"Modified examine!\");".to_string(), // Changed content
        "    return 1;".to_string(),
        "  endverb".to_string(),
        "endobject".to_string(),
        "".to_string(),
    ];

    client
        .object_update("test_obj", modified_content)
        .await
        .expect("Failed to modify object");

    println!("✅ Created change with modified 'examine' verb");

    // Step 3: Abandon and check the diff
    println!("\nStep 3: Abandoning change and checking diff...");
    let response = client
        .change_abandon()
        .await
        .expect("Failed to abandon change");

    let result = response.get("result").expect("No result field");
    let changes = result
        .get("changes")
        .and_then(|v| v.as_array())
        .expect("No changes array");

    let obj_change = changes
        .iter()
        .find(|c| {
            c.get("obj_id")
                .and_then(|v| v.as_str())
                .map(|s| s == "test_obj")
                .unwrap_or(false)
        })
        .expect("No change found for test_obj");

    // Verify that verbs_modified contains "examine"
    let verbs_modified = get_list_field(obj_change, "verbs_modified");
    assert!(
        verbs_modified.contains(&"examine".to_string()),
        "Expected 'examine' in verbs_modified, got: {:?}",
        verbs_modified
    );

    println!("✅ Abandon correctly shows verb 'examine' as modified");
}

#[tokio::test]
async fn test_abandon_property_added_becomes_property_deleted() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: When a change adds a property, abandon should show that property as deleted");

    // Step 1: Create a change with an object having a property
    println!("\nStep 1: Creating change with object containing property...");
    client
        .change_create("test_prop_change", "test_author", None)
        .await
        .expect("Failed to create change");

    // Use the object_with_property.moo test file
    client
        .object_update_from_file("prop_obj", "object_with_property.moo")
        .await
        .expect("Failed to update object");

    println!("✅ Created change with object having property");

    // Step 2: Abandon and check
    println!("\nStep 2: Abandoning change and checking diff...");
    let response = client
        .change_abandon()
        .await
        .expect("Failed to abandon change");

    let result = response.get("result").expect("No result field");
    let changes = result
        .get("changes")
        .and_then(|v| v.as_array())
        .expect("No changes array");

    let obj_change = changes
        .iter()
        .find(|c| {
            c.get("obj_id")
                .and_then(|v| v.as_str())
                .map(|s| s == "prop_obj")
                .unwrap_or(false)
        })
        .expect("No change found for prop_obj");

    // Verify that props_deleted contains the property from the file
    let props_deleted = get_list_field(obj_change, "props_deleted");
    assert!(
        !props_deleted.is_empty(),
        "Expected at least one property in props_deleted (inverse of adding), got: {:?}",
        props_deleted
    );

    println!("✅ Abandon correctly shows properties as deleted (inverse of adding)");
}

#[tokio::test]
async fn test_abandon_object_added_becomes_object_deleted() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: When a change adds an object, abandon should show that object as deleted");

    // Step 1: Create a change with a new object
    println!("\nStep 1: Creating change with new object...");
    client
        .change_create("add_obj_change", "test_author", None)
        .await
        .expect("Failed to create change");

    client
        .object_update_from_file("new_object", "test_object.moo")
        .await
        .expect("Failed to add object");

    println!("✅ Created change with new object");

    // Step 2: Abandon and check
    println!("\nStep 2: Abandoning change and checking diff...");
    let response = client
        .change_abandon()
        .await
        .expect("Failed to abandon change");

    let result = response.get("result").expect("No result field");
    
    // Verify that objects_deleted contains the new object
    let objects_deleted = result
        .get("objects_deleted")
        .and_then(|v| v.as_array())
        .expect("No objects_deleted array");

    let has_new_object = objects_deleted
        .iter()
        .any(|v| v.as_str().map(|s| s == "new_object").unwrap_or(false));

    assert!(
        has_new_object,
        "Expected 'new_object' in objects_deleted, got: {:?}",
        objects_deleted
    );

    println!("✅ Abandon correctly shows object 'new_object' as deleted");
}

#[tokio::test]
async fn test_abandon_verb_rename_shows_reverse_rename() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: When a change renames a verb, abandon should show the reverse rename");

    // Step 1: Create initial object with a verb
    println!("\nStep 1: Creating initial object with 'old_verb'...");
    client
        .change_create("initial", "test_author", None)
        .await
        .expect("Failed to create change");

    let initial_content = vec![
        "object #7".to_string(),
        "  name: \"Rename Test\"".to_string(),
        "  parent: #1".to_string(),
        "  location: #2".to_string(),
        "  owner: #2".to_string(),
        "".to_string(),
        "  verb old_verb (this none this) owner: #2 flags: \"rxd\"".to_string(),
        "    return 1;".to_string(),
        "  endverb".to_string(),
        "endobject".to_string(),
        "".to_string(),
    ];

    client
        .object_update("rename_test", initial_content)
        .await
        .expect("Failed to create object");

    let (change_id, _) = server.db_assertions().require_top_change();
    client
        .change_approve(&change_id)
        .await
        .expect("Failed to approve")
        .assert_success("Approve");

    println!("✅ Initial object created with 'old_verb'");

    // Step 2: Create change that renames the verb
    println!("\nStep 2: Creating change that renames verb to 'new_verb'...");
    client
        .change_create("rename_change", "test_author", None)
        .await
        .expect("Failed to create change");

    let renamed_content = vec![
        "object #7".to_string(),
        "  name: \"Rename Test\"".to_string(),
        "  parent: #1".to_string(),
        "  location: #2".to_string(),
        "  owner: #2".to_string(),
        "".to_string(),
        "  verb new_verb (this none this) owner: #2 flags: \"rxd\"".to_string(),
        "    return 1;".to_string(),
        "  endverb".to_string(),
        "endobject".to_string(),
        "".to_string(),
    ];

    client
        .object_update("rename_test", renamed_content)
        .await
        .expect("Failed to rename verb");

    println!("✅ Created change with renamed verb");

    // Step 3: Abandon and check for inversion
    // Without a hint, the system sees this as deleted old_verb + added new_verb
    // So the inversion should show: deleted new_verb + added old_verb
    println!("\nStep 3: Abandoning and checking for inversion...");
    let response = client
        .change_abandon()
        .await
        .expect("Failed to abandon");

    let result = response.get("result").expect("No result field");
    let changes = result
        .get("changes")
        .and_then(|v| v.as_array())
        .expect("No changes array");

    let obj_change = changes
        .iter()
        .find(|c| {
            c.get("obj_id")
                .and_then(|v| v.as_str())
                .map(|s| s == "rename_test")
                .unwrap_or(false)
        })
        .expect("No change found for rename_test");

    // Without a hint, the change sees: old_verb deleted, new_verb added
    // So the inversion should show: new_verb deleted (undo the addition), old_verb added (undo the deletion)
    let verbs_deleted = get_list_field(obj_change, "verbs_deleted");
    let verbs_added = get_list_field(obj_change, "verbs_added");

    assert!(
        verbs_deleted.contains(&"new_verb".to_string()),
        "Expected 'new_verb' in verbs_deleted (undo the addition), got: {:?}",
        verbs_deleted
    );

    assert!(
        verbs_added.contains(&"old_verb".to_string()),
        "Expected 'old_verb' in verbs_added (undo the deletion), got: {:?}",
        verbs_added
    );

    println!("✅ Abandon correctly inverts the verb change (delete new, add old)");
}