//! Integration tests for verb and property rename detection
//!
//! This module tests the rename detection logic when:
//! 1. A verb is renamed (same code, different name) - should show as verbs_renamed
//! 2. A property is renamed (same value, different name) - should show as props_renamed
//! 3. Edge cases with special characters in verb/property names

use crate::common::*;

#[tokio::test]
async fn test_verb_rename_simple() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Renaming a verb from 'look' to 'examine' should show as verbs_renamed");

    // Step 1: Create an object with a verb named "look"
    println!("\nStep 1: Creating object with verb 'look'...");
    client
        .object_update_from_file("#3", "object_with_look_verb.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Create object with verb 'look'");

    // Step 2: Submit the change
    println!("\nStep 2: Submitting the change...");
    client
        .change_submit()
        .await
        .expect("Failed to submit change")
        .assert_success("Submit change");

    println!("✅ Object with verb 'look' submitted");

    // Step 3: Update the object with the verb renamed to 'examine'
    println!("\nStep 3: Updating object with verb renamed to 'examine'...");
    client
        .object_update_from_file("#3", "object_with_examine_verb.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update object with renamed verb");

    println!("✅ Object updated with renamed verb");

    // Step 4: Check change status to see the diff
    println!("\nStep 4: Checking change status...");
    let status = client
        .change_status()
        .await
        .expect("Failed to get change status");

    println!("Status response: {:?}", status);

    // Parse the response - it's a nested JSON structure
    let status_map = status.as_object().expect("Status should be an object");
    let result = &status_map["result"];
    let result_obj = result.as_object().expect("Result should be an object");
    
    println!("Result object: {:?}", result_obj);

    // Get the changes array
    let changes = result_obj["changes"].as_array().expect("changes should be an array");
    assert!(!changes.is_empty(), "Should have at least one change");
    
    let change = &changes[0];
    let change_obj = change.as_object().expect("change should be an object");
    
    println!("Change details: {:?}", change_obj);
    
    // Check verbs_added, verbs_deleted, verbs_renamed
    let verbs_added = change_obj["verbs_added"].as_array().expect("verbs_added should be an array");
    let verbs_deleted = change_obj["verbs_deleted"].as_array().expect("verbs_deleted should be an array");
    let verbs_renamed = change_obj["verbs_renamed"].as_object().expect("verbs_renamed should be an object");
    
    println!("verbs_added: {:?}", verbs_added);
    println!("verbs_deleted: {:?}", verbs_deleted);
    println!("verbs_renamed: {:?}", verbs_renamed);
    
    // Validate format: verbs_renamed should be a map with old_name -> new_name
    assert!(
        verbs_renamed.len() > 0,
        "verbs_renamed should contain the renamed verb, but got: {:?}. verbs_added: {:?}, verbs_deleted: {:?}",
        verbs_renamed, verbs_added, verbs_deleted
    );
    
    // Verify it's a map (object in JSON) with the correct old -> new mapping
    let look_rename = verbs_renamed.get("look");
    assert!(
        look_rename.is_some(),
        "verbs_renamed should have 'look' as key (old name)"
    );
    assert_eq!(
        look_rename.unwrap().as_str().unwrap(),
        "examine",
        "verbs_renamed['look'] should be 'examine' (new name)"
    );
    
    assert!(
        verbs_added.is_empty(),
        "verbs_added should be empty (rename, not add), but got: {:?}",
        verbs_added
    );
    
    assert!(
        verbs_deleted.is_empty(),
        "verbs_deleted should be empty (rename, not delete), but got: {:?}",
        verbs_deleted
    );

    println!("\n✅ Test passed - verb rename detected correctly with old_name -> new_name format");
}

#[tokio::test]
async fn test_property_rename() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Renaming a property from 'description' to 'long_description' should show as props_renamed");

    // Step 1: Create an object with a property named "description"
    println!("\nStep 1: Creating object with property 'description'...");
    client
        .object_update_from_file("#4", "object_with_property.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Create object with property 'description'");

    // Step 2: Submit the change
    println!("\nStep 2: Submitting the change...");
    client
        .change_submit()
        .await
        .expect("Failed to submit change")
        .assert_success("Submit change");

    println!("✅ Object with property 'description' submitted");

    // Step 3: Update the object with the property renamed to 'long_description'
    println!("\nStep 3: Updating object with property renamed to 'long_description'...");
    client
        .object_update_from_file("#4", "object_with_property_renamed.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update object with renamed property");

    println!("✅ Object updated with renamed property");

    // Step 4: Check change status to see the diff
    println!("\nStep 4: Checking change status...");
    let status = client
        .change_status()
        .await
        .expect("Failed to get change status");

    println!("Status response: {:?}", status);

    let status_map = status.as_object().expect("Status should be an object");
    let result = &status_map["result"];
    let result_obj = result.as_object().expect("Result should be an object");
    
    let changes = result_obj["changes"].as_array().expect("changes should be an array");
    let change_obj = changes[0].as_object().expect("change should be an object");
    
    let props_renamed = change_obj["props_renamed"].as_object().expect("props_renamed should be an object");
    let props_added = change_obj["props_added"].as_array().expect("props_added should be an array");
    let props_deleted = change_obj["props_deleted"].as_array().expect("props_deleted should be an array");
    
    println!("props_added: {:?}", props_added);
    println!("props_deleted: {:?}", props_deleted);
    println!("props_renamed: {:?}", props_renamed);

    // The property should show as renamed, not as added + deleted
    // Validate format: props_renamed should be a map with old_name -> new_name
    assert!(
        props_renamed.len() > 0,
        "props_renamed should contain the renamed property"
    );
    
    // Verify it's a map (object in JSON) with the correct old -> new mapping
    let desc_rename = props_renamed.get("description");
    assert!(
        desc_rename.is_some(),
        "props_renamed should have 'description' as key (old name)"
    );
    assert_eq!(
        desc_rename.unwrap().as_str().unwrap(),
        "long_description",
        "props_renamed['description'] should be 'long_description' (new name)"
    );
    
    assert!(props_added.is_empty(), "props_added should be empty");
    assert!(props_deleted.is_empty(), "props_deleted should be empty");

    println!("\n✅ Test passed - property rename detected with old_name -> new_name format");
}

#[tokio::test]
async fn test_verb_modification_not_rename() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Modifying a verb's code should show as verbs_modified, not verbs_renamed");

    // Step 1: Create an object with a verb
    println!("\nStep 1: Creating object with verb...");
    client
        .object_update_from_file("#3", "object_with_look_verb.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Create object");

    // Step 2: Submit the change
    println!("\nStep 2: Submitting the change...");
    client
        .change_submit()
        .await
        .expect("Failed to submit change")
        .assert_success("Submit change");

    // Step 3: Update the same verb with different code but same name
    println!("\nStep 3: Updating verb with modified code...");
    let modified_object = r#"object #3
  name: "Test Object"
  parent: #1
  location: #2
  owner: #2

  verb look (this none this) owner: #2 flags: "rxd"
    player:tell("You look around more carefully.");
    return 2;
  endverb
endobject
"#;
    client
        .object_update("#3", moo_to_lines(modified_object))
        .await
        .expect("Failed to update object")
        .assert_success("Update object with modified verb");

    // Step 4: Check change status
    println!("\nStep 4: Checking change status...");
    let status = client
        .change_status()
        .await
        .expect("Failed to get change status");

    let status_map = status.as_object().expect("Status should be an object");
    let result = &status_map["result"];
    let result_obj = result.as_object().expect("Result should be an object");
    
    let changes = result_obj["changes"].as_array().expect("changes should be an array");
    let change_obj = changes[0].as_object().expect("change should be an object");
    
    let verbs_modified = change_obj["verbs_modified"].as_array().expect("verbs_modified should be an array");
    let verbs_renamed = change_obj["verbs_renamed"].as_object().expect("verbs_renamed should be an object");
    
    println!("verbs_modified: {:?}", verbs_modified);
    println!("verbs_renamed: {:?}", verbs_renamed);

    // Should show as modified, not renamed
    assert!(
        verbs_modified.len() > 0,
        "verbs_modified should contain the modified verb"
    );
    assert!(
        verbs_modified.contains(&serde_json::Value::String("look".to_string())),
        "verbs_modified should contain 'look'"
    );
    assert!(
        verbs_renamed.is_empty(),
        "verbs_renamed should be empty (modified, not renamed)"
    );

    println!("\n✅ Test passed - verb modification correctly detected as modified, not renamed");
}

#[tokio::test]
async fn test_verb_rename_with_overlapping_aliases() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Renaming a verb with overlapping aliases should be detected as rename");
    println!("Example: 'look examine inspect' -> 'look observe watch' (shared: 'look')");

    // Step 1: Create an object with a verb with multiple aliases
    println!("\nStep 1: Creating object with verb 'look examine inspect'...");
    client
        .object_update_from_file("#3", "object_with_multiname_verb.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Create object with multi-alias verb");

    // Step 2: Submit the change
    println!("\nStep 2: Submitting the change...");
    client
        .change_submit()
        .await
        .expect("Failed to submit change")
        .assert_success("Submit change");

    println!("✅ Object with verb 'look examine inspect' submitted");

    // Step 3: Update the object with the verb renamed to 'look observe watch'
    // Note: "look" is shared between both, so it should be detected as a rename
    println!("\nStep 3: Updating object with verb renamed to 'look observe watch'...");
    client
        .object_update_from_file("#3", "object_with_multiname_verb_renamed.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update object with renamed verb");

    println!("✅ Object updated with renamed verb");

    // Step 4: Check change status to see the diff
    println!("\nStep 4: Checking change status...");
    let status = client
        .change_status()
        .await
        .expect("Failed to get change status");

    println!("Status response: {:?}", status);

    let status_map = status.as_object().expect("Status should be an object");
    let result = &status_map["result"];
    let result_obj = result.as_object().expect("Result should be an object");
    
    let changes = result_obj["changes"].as_array().expect("changes should be an array");
    let change_obj = changes[0].as_object().expect("change should be an object");
    
    let verbs_added = change_obj["verbs_added"].as_array().expect("verbs_added should be an array");
    let verbs_deleted = change_obj["verbs_deleted"].as_array().expect("verbs_deleted should be an array");
    let verbs_renamed = change_obj["verbs_renamed"].as_object().expect("verbs_renamed should be an object");
    
    println!("verbs_added: {:?}", verbs_added);
    println!("verbs_deleted: {:?}", verbs_deleted);
    println!("verbs_renamed: {:?}", verbs_renamed);
    
    // Validate: should detect renames for matching aliases
    // In MOO, "look examine inspect" creates 3 callable verb names
    // When changed to "look observe watch", we get:
    // - "look" -> "look" (no change, but could be detected as rename)
    // - "examine" -> "observe" (rename, same code)
    // - "inspect" -> "observe" (rename, same code)  
    // - "watch" is new (added)
    
    println!("Note: Verb names with overlapping aliases create multiple renames:");
    println!("  - Some aliases may map to the same new name");
    println!("  - New unique aliases appear in verbs_added");
    
    assert!(
        verbs_renamed.len() > 0,
        "verbs_renamed should contain renamed verb aliases (overlapping detected)"
    );
    
    // It's ok to have some verbs_added if they're truly new aliases with no matching old one
    println!("verbs_added (new unique aliases): {:?}", verbs_added);
    println!("verbs_deleted (removed aliases with no match): {:?}", verbs_deleted);
    println!("verbs_renamed (matched aliases): {:?}", verbs_renamed);

    println!("\n✅ Test passed - verb rename with overlapping aliases detected correctly");
}

#[tokio::test]
async fn test_property_rename_skips_empty_values() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Properties with empty values should NOT be detected as renames (false positive avoidance)");

    // Step 1: Create an object with two empty-valued properties
    println!("\nStep 1: Creating object with empty properties...");
    client
        .object_update_from_file("#5", "object_with_empty_props.moo")
        .await
        .expect("Failed to create object")
        .assert_success("Create object with empty properties");

    // Step 2: Submit the change
    println!("\nStep 2: Submitting the change...");
    client
        .change_submit()
        .await
        .expect("Failed to submit change")
        .assert_success("Submit change");

    println!("✅ Object with empty properties submitted");

    // Step 3: "Rename" the properties (actually delete + add with empty values)
    println!("\nStep 3: Updating object with 'renamed' empty properties...");
    client
        .object_update_from_file("#5", "object_with_empty_props_renamed.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update object");

    println!("✅ Object updated");

    // Step 4: Check change status - should show as added/deleted, NOT renamed
    println!("\nStep 4: Checking change status...");
    let status = client
        .change_status()
        .await
        .expect("Failed to get change status");

    let status_map = status.as_object().expect("Status should be an object");
    let result = &status_map["result"];
    let result_obj = result.as_object().expect("Result should be an object");
    
    let changes = result_obj["changes"].as_array().expect("changes should be an array");
    let change_obj = changes[0].as_object().expect("change should be an object");
    
    let props_renamed = change_obj["props_renamed"].as_object().expect("props_renamed should be an object");
    let props_added = change_obj["props_added"].as_array().expect("props_added should be an array");
    let props_deleted = change_obj["props_deleted"].as_array().expect("props_deleted should be an array");
    
    println!("props_added: {:?}", props_added);
    println!("props_deleted: {:?}", props_deleted);
    println!("props_renamed: {:?}", props_renamed);

    // Empty properties should NOT be detected as renames (false positive avoidance)
    assert!(
        props_renamed.is_empty(),
        "props_renamed should be empty for empty-valued properties (avoiding false positives)"
    );
    
    assert!(
        !props_added.is_empty(),
        "props_added should contain the new properties (not detected as renames)"
    );
    
    assert!(
        !props_deleted.is_empty(),
        "props_deleted should contain the old properties (not detected as renames)"
    );

    println!("\n✅ Test passed - empty property values correctly skip rename detection");
}

