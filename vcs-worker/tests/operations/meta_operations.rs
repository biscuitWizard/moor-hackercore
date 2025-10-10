//! Integration tests for meta operations (add/remove ignored properties and verbs)

use crate::common::*;
use moor_vcs_worker::types::{VcsObjectType};

#[tokio::test]
async fn test_meta_add_ignored_property() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object first
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    let update_response = make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    assert!(
        update_response["success"].as_bool().unwrap_or(false),
        "Object update should succeed, got: {}",
        update_response
    );
    
    // Add an ignored property
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    let add_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(add_property_request),
    )
    .await
    .expect("Failed to add ignored property");
    
    assert!(
        add_response["success"].as_bool().unwrap_or(false),
        "Adding ignored property should succeed, got: {}",
        add_response
    );
    
    // Verify the meta was created and tracked in the change
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to get meta ref");
    
    assert!(
        meta_ref.is_some(),
        "Meta ref should exist for object '{}'",
        object_name
    );
    
    let meta_sha256 = meta_ref.unwrap();
    let meta_yaml = server.database().objects().get(&meta_sha256)
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");
    
    let meta = server.database().objects().parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");
    
    assert!(
        meta.ignored_properties.contains("test_property"),
        "Meta should contain ignored property 'test_property'"
    );
    
    // Verify the change tracks the meta object
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Top change should exist");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let meta_in_change = change.added_objects.iter()
        .any(|obj| obj.object_type == VcsObjectType::MooMetaObject && obj.name == object_name);
    
    assert!(
        meta_in_change,
        "Change should track the meta object as added"
    );
}

#[tokio::test]
async fn test_meta_add_ignored_verb() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object first
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add an ignored verb
    let add_verb_request = json!({
        "operation": "meta/add_ignored_verb",
        "args": [
            object_name,
            "test_verb"
        ]
    });
    
    let add_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(add_verb_request),
    )
    .await
    .expect("Failed to add ignored verb");
    
    assert!(
        add_response["success"].as_bool().unwrap_or(false),
        "Adding ignored verb should succeed, got: {}",
        add_response
    );
    
    // Verify the meta contains the ignored verb
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to get meta ref")
        .expect("Meta ref should exist");
    
    let meta_yaml = server.database().objects().get(&meta_ref)
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");
    
    let meta = server.database().objects().parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");
    
    assert!(
        meta.ignored_verbs.contains("test_verb"),
        "Meta should contain ignored verb 'test_verb'"
    );
}

#[tokio::test]
async fn test_meta_remove_ignored_property() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object first
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add an ignored property
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    // Remove the ignored property
    let remove_property_request = json!({
        "operation": "meta/remove_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    let remove_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(remove_property_request),
    )
    .await
    .expect("Failed to remove ignored property");
    
    assert!(
        remove_response["success"].as_bool().unwrap_or(false),
        "Removing ignored property should succeed, got: {}",
        remove_response
    );
    
    // Verify the property was removed
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to get meta ref")
        .expect("Meta ref should exist");
    
    let meta_yaml = server.database().objects().get(&meta_ref)
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");
    
    let meta = server.database().objects().parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");
    
    assert!(
        !meta.ignored_properties.contains("test_property"),
        "Meta should not contain ignored property 'test_property' after removal"
    );
}

#[tokio::test]
async fn test_meta_remove_ignored_verb() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object first
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add an ignored verb
    let add_verb_request = json!({
        "operation": "meta/add_ignored_verb",
        "args": [
            object_name,
            "test_verb"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_verb_request))
        .await
        .expect("Failed to add ignored verb");
    
    // Remove the ignored verb
    let remove_verb_request = json!({
        "operation": "meta/remove_ignored_verb",
        "args": [
            object_name,
            "test_verb"
        ]
    });
    
    let remove_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(remove_verb_request),
    )
    .await
    .expect("Failed to remove ignored verb");
    
    assert!(
        remove_response["success"].as_bool().unwrap_or(false),
        "Removing ignored verb should succeed, got: {}",
        remove_response
    );
    
    // Verify the verb was removed
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to get meta ref")
        .expect("Meta ref should exist");
    
    let meta_yaml = server.database().objects().get(&meta_ref)
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");
    
    let meta = server.database().objects().parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");
    
    assert!(
        !meta.ignored_verbs.contains("test_verb"),
        "Meta should not contain ignored verb 'test_verb' after removal"
    );
}

#[tokio::test]
async fn test_meta_clear_ignored_properties() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object first
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add multiple ignored properties
    for prop in &["prop1", "prop2", "prop3"] {
        let add_property_request = json!({
            "operation": "meta/add_ignored_property",
            "args": [
                object_name,
                prop
            ]
        });
        
        make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
            .await
            .expect("Failed to add ignored property");
    }
    
    // Clear all ignored properties
    let clear_request = json!({
        "operation": "meta/clear_ignored_properties",
        "args": [
            object_name
        ]
    });
    
    let clear_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(clear_request),
    )
    .await
    .expect("Failed to clear ignored properties");
    
    assert!(
        clear_response["success"].as_bool().unwrap_or(false),
        "Clearing ignored properties should succeed, got: {}",
        clear_response
    );
    
    // Verify all properties were cleared
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to get meta ref")
        .expect("Meta ref should exist");
    
    let meta_yaml = server.database().objects().get(&meta_ref)
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");
    
    let meta = server.database().objects().parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");
    
    assert!(
        meta.ignored_properties.is_empty(),
        "Meta should have no ignored properties after clearing"
    );
}

#[tokio::test]
async fn test_meta_clear_ignored_verbs() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object first
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add multiple ignored verbs
    for verb in &["verb1", "verb2", "verb3"] {
        let add_verb_request = json!({
            "operation": "meta/add_ignored_verb",
            "args": [
                object_name,
                verb
            ]
        });
        
        make_request("POST", &format!("{}/rpc", base_url), Some(add_verb_request))
            .await
            .expect("Failed to add ignored verb");
    }
    
    // Clear all ignored verbs
    let clear_request = json!({
        "operation": "meta/clear_ignored_verbs",
        "args": [
            object_name
        ]
    });
    
    let clear_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(clear_request),
    )
    .await
    .expect("Failed to clear ignored verbs");
    
    assert!(
        clear_response["success"].as_bool().unwrap_or(false),
        "Clearing ignored verbs should succeed, got: {}",
        clear_response
    );
    
    // Verify all verbs were cleared
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to get meta ref")
        .expect("Meta ref should exist");
    
    let meta_yaml = server.database().objects().get(&meta_ref)
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");
    
    let meta = server.database().objects().parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");
    
    assert!(
        meta.ignored_verbs.is_empty(),
        "Meta should have no ignored verbs after clearing"
    );
}

#[tokio::test]
async fn test_meta_clear_ignored_properties_when_no_meta_exists() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object without any meta
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Try to clear ignored properties when no meta exists
    let clear_request = json!({
        "operation": "meta/clear_ignored_properties",
        "args": [
            object_name
        ]
    });
    
    let clear_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(clear_request),
    )
    .await
    .expect("Failed to clear ignored properties");
    
    assert!(
        clear_response["success"].as_bool().unwrap_or(false),
        "Clearing when no meta exists should succeed, got: {}",
        clear_response
    );
    
    // Verify the response indicates 0 properties were cleared
    let result_str = clear_response["result"].as_str().unwrap_or("");
    assert!(
        result_str.contains("0 ignored properties") || result_str.contains("No meta exists"),
        "Response should indicate 0 properties cleared or no meta exists, got: {}",
        result_str
    );
    
    // Verify no meta was created
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to get meta ref");
    
    assert!(
        meta_ref.is_none(),
        "No meta should have been created"
    );
}

#[tokio::test]
async fn test_meta_clear_ignored_verbs_when_no_meta_exists() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object without any meta
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Try to clear ignored verbs when no meta exists
    let clear_request = json!({
        "operation": "meta/clear_ignored_verbs",
        "args": [
            object_name
        ]
    });
    
    let clear_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(clear_request),
    )
    .await
    .expect("Failed to clear ignored verbs");
    
    assert!(
        clear_response["success"].as_bool().unwrap_or(false),
        "Clearing when no meta exists should succeed, got: {}",
        clear_response
    );
    
    // Verify the response indicates 0 verbs were cleared
    let result_str = clear_response["result"].as_str().unwrap_or("");
    assert!(
        result_str.contains("0 ignored verbs") || result_str.contains("No meta exists"),
        "Response should indicate 0 verbs cleared or no meta exists, got: {}",
        result_str
    );
    
    // Verify no meta was created
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to get meta ref");
    
    assert!(
        meta_ref.is_none(),
        "No meta should have been created"
    );
}

#[tokio::test]
async fn test_meta_rename_with_object() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add some metadata
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    // Rename the object
    let new_name = "renamed_object";
    let rename_request = json!({
        "operation": "object/rename",
        "args": [
            object_name,
            new_name
        ]
    });
    
    let rename_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(rename_request),
    )
    .await
    .expect("Failed to rename object");
    
    assert!(
        rename_response["success"].as_bool().unwrap_or(false),
        "Renaming object should succeed, got: {}",
        rename_response
    );
    
    // Verify the meta was also renamed
    let meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, new_name, None)
        .expect("Failed to get meta ref");
    
    assert!(
        meta_ref.is_some(),
        "Meta ref should exist for renamed object '{}'",
        new_name
    );
    
    // Verify old meta ref no longer exists
    let old_meta_ref = server.database().refs().get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .expect("Failed to check old meta ref");
    
    assert!(
        old_meta_ref.is_none(),
        "Meta ref should not exist for old object name '{}'",
        object_name
    );
    
    // Verify the meta content is preserved
    let meta_yaml = server.database().objects().get(&meta_ref.unwrap())
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");
    
    let meta = server.database().objects().parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");
    
    assert!(
        meta.ignored_properties.contains("test_property"),
        "Meta should still contain the ignored property after rename"
    );
}

#[tokio::test]
async fn test_meta_delete_with_object() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add some metadata
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    // Delete the object
    let delete_request = json!({
        "operation": "object/delete",
        "args": [
            object_name
        ]
    });
    
    let delete_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(delete_request),
    )
    .await
    .expect("Failed to delete object");
    
    assert!(
        delete_response["success"].as_bool().unwrap_or(false),
        "Deleting object should succeed, got: {}",
        delete_response
    );
    
    // Verify the meta was also marked for deletion in the change
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Top change should exist");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let meta_in_deleted = change.deleted_objects.iter()
        .any(|obj| obj.object_type == VcsObjectType::MooMetaObject && obj.name == object_name);
    
    assert!(
        meta_in_deleted,
        "Change should track the meta object as deleted"
    );
}

#[tokio::test]
async fn test_object_get_filters_ignored_properties() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object with properties
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Get the object before adding meta (should return full object)
    let get_request_before = json!({
        "operation": "object/get",
        "args": [object_name]
    });
    
    let get_response_before = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(get_request_before),
    )
    .await
    .expect("Failed to get object");
    
    assert!(
        get_response_before["success"].as_bool().unwrap_or(false),
        "Getting object should succeed"
    );
    
    let content_before = get_response_before["result"].as_str().unwrap();
    
    // The object should contain 'test_property' in property definitions
    assert!(
        content_before.contains("test_property"),
        "Object should contain test_property before filtering"
    );
    
    // Add test_property to ignored properties
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    // Get the object after adding meta (should filter out test_property)
    let get_request_after = json!({
        "operation": "object/get",
        "args": [object_name]
    });
    
    let get_response_after = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(get_request_after),
    )
    .await
    .expect("Failed to get object");
    
    assert!(
        get_response_after["success"].as_bool().unwrap_or(false),
        "Getting object should succeed after adding meta"
    );
    
    let content_after = get_response_after["result"].as_str().unwrap();
    
    // The filtered object should NOT contain 'test_property'
    assert!(
        !content_after.contains("test_property"),
        "Object should not contain test_property after filtering"
    );
}

#[tokio::test]
async fn test_object_get_filters_ignored_verbs() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object with verbs
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Get the object before adding meta
    let get_request_before = json!({
        "operation": "object/get",
        "args": [object_name]
    });
    
    let get_response_before = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(get_request_before),
    )
    .await
    .expect("Failed to get object");
    
    let content_before = get_response_before["result"].as_str().unwrap();
    
    // The object should contain 'test_verb' in verb definitions
    assert!(
        content_before.contains("test_verb"),
        "Object should contain test_verb before filtering"
    );
    
    // Add test_verb to ignored verbs
    let add_verb_request = json!({
        "operation": "meta/add_ignored_verb",
        "args": [
            object_name,
            "test_verb"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_verb_request))
        .await
        .expect("Failed to add ignored verb");
    
    // Get the object after adding meta
    let get_request_after = json!({
        "operation": "object/get",
        "args": [object_name]
    });
    
    let get_response_after = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(get_request_after),
    )
    .await
    .expect("Failed to get object");
    
    let content_after = get_response_after["result"].as_str().unwrap();
    
    // The filtered object should NOT contain 'test_verb'
    assert!(
        !content_after.contains("test_verb"),
        "Object should not contain test_verb after filtering"
    );
}

#[tokio::test]
async fn test_object_get_filters_multiple_properties_and_verbs() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object with both properties and verbs
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add multiple ignored properties and verbs
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [object_name, "test_property"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    let add_verb_request = json!({
        "operation": "meta/add_ignored_verb",
        "args": [object_name, "test_verb"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(add_verb_request))
        .await
        .expect("Failed to add ignored verb");
    
    // Get the filtered object
    let get_request = json!({
        "operation": "object/get",
        "args": [object_name]
    });
    
    let get_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(get_request),
    )
    .await
    .expect("Failed to get object");
    
    assert!(
        get_response["success"].as_bool().unwrap_or(false),
        "Getting object should succeed"
    );
    
    let content = get_response["result"].as_str().unwrap();
    
    // Verify both property and verb are filtered out
    assert!(
        !content.contains("test_property"),
        "Object should not contain test_property"
    );
    assert!(
        !content.contains("test_verb"),
        "Object should not contain test_verb"
    );
    
    // Verify the object structure is still valid
    assert!(
        content.contains("object #"),
        "Object should still be a valid objdef"
    );
    assert!(
        content.contains("endobject"),
        "Object should have endobject marker"
    );
}

#[tokio::test]
async fn test_object_update_filters_ignored_properties() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add test_property to ignored properties
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    // Try to update the object again with test_property included
    let update_request_2 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    let update_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(update_request_2),
    )
    .await
    .expect("Failed to update object");
    
    assert!(
        update_response["success"].as_bool().unwrap_or(false),
        "Update should succeed, got: {}",
        update_response
    );
    
    // Get the object and verify test_property is not in it
    let get_request = json!({
        "operation": "object/get",
        "args": [object_name]
    });
    
    let get_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(get_request),
    )
    .await
    .expect("Failed to get object");
    
    let content = get_response["result"].as_str().unwrap();
    
    // Verify test_property was filtered out during update
    assert!(
        !content.contains("test_property"),
        "Object should not contain test_property after filtered update"
    );
    
    // Verify another_property is still present
    assert!(
        content.contains("another_property"),
        "Object should still contain another_property"
    );
}

#[tokio::test]
async fn test_object_update_filters_ignored_verbs() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add test_verb to ignored verbs
    let add_verb_request = json!({
        "operation": "meta/add_ignored_verb",
        "args": [
            object_name,
            "test_verb"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_verb_request))
        .await
        .expect("Failed to add ignored verb");
    
    // Try to update the object again with test_verb included
    let update_request_2 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    let update_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(update_request_2),
    )
    .await
    .expect("Failed to update object");
    
    assert!(
        update_response["success"].as_bool().unwrap_or(false),
        "Update should succeed, got: {}",
        update_response
    );
    
    // Get the object and verify test_verb is not in it
    let get_request = json!({
        "operation": "object/get",
        "args": [object_name]
    });
    
    let get_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(get_request),
    )
    .await
    .expect("Failed to get object");
    
    let content = get_response["result"].as_str().unwrap();
    
    // Verify test_verb was filtered out during update
    assert!(
        !content.contains("test_verb"),
        "Object should not contain test_verb after filtered update"
    );
    
    // Verify another_verb is still present
    assert!(
        content.contains("another_verb"),
        "Object should still contain another_verb"
    );
}

#[tokio::test]
async fn test_object_update_preserves_non_ignored_items() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Add ignored property and verb
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [object_name, "test_property"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    let add_verb_request = json!({
        "operation": "meta/add_ignored_verb",
        "args": [object_name, "test_verb"]
    });
    make_request("POST", &format!("{}/rpc", base_url), Some(add_verb_request))
        .await
        .expect("Failed to add ignored verb");
    
    // Update with the same content (should filter during update)
    let update_request_2 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_2))
        .await
        .expect("Failed to update object");
    
    // Get the object
    let get_request = json!({
        "operation": "object/get",
        "args": [object_name]
    });
    
    let get_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(get_request),
    )
    .await
    .expect("Failed to get object");
    
    let content = get_response["result"].as_str().unwrap();
    
    // Verify ignored items are not present
    assert!(
        !content.contains("test_property"),
        "Object should not contain test_property"
    );
    assert!(
        !content.contains("test_verb"),
        "Object should not contain test_verb"
    );
    
    // Verify non-ignored items are still present
    assert!(
        content.contains("another_property"),
        "Object should still contain another_property"
    );
    assert!(
        content.contains("another_verb"),
        "Object should still contain another_verb"
    );
    
    // Verify the object is still valid
    assert!(
        content.contains("object #"),
        "Object should still be valid"
    );
    assert!(
        content.contains("endobject"),
        "Object should have endobject marker"
    );
}

#[tokio::test]
async fn test_diff_excludes_ignored_properties_from_deleted() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object with properties
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Submit the change
    let submit_request = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request))
        .await
        .expect("Failed to submit change");
    
    // Add test_property to ignored properties
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    // Submit the meta change
    let submit_request_2 = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request_2))
        .await
        .expect("Failed to submit meta change");
    
    // Update the object again (test_property will be filtered out due to meta)
    let update_request_3 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_3))
        .await
        .expect("Failed to update object");
    
    // Get the change status to see the diff
    let status_request = json!({
        "operation": "change/status",
        "args": []
    });
    
    let status_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(status_request),
    )
    .await
    .expect("Failed to get change status");
    
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Getting change status should succeed"
    );
    
    // Verify using the direct database access to check object changes
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Build the diff using our diff function
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    // Find the object change for our test object
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == object_name);
    
    if let Some(obj_change) = obj_change {
        // test_property should NOT be in props_deleted
        assert!(
            !obj_change.props_deleted.contains("test_property"),
            "test_property should not be in props_deleted since it's ignored"
        );
    }
}

#[tokio::test]
async fn test_diff_excludes_ignored_verbs_from_deleted() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object with verbs
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Submit the change
    let submit_request = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request))
        .await
        .expect("Failed to submit change");
    
    // Add test_verb to ignored verbs
    let add_verb_request = json!({
        "operation": "meta/add_ignored_verb",
        "args": [
            object_name,
            "test_verb"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_verb_request))
        .await
        .expect("Failed to add ignored verb");
    
    // Submit the meta change
    let submit_request_2 = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request_2))
        .await
        .expect("Failed to submit meta change");
    
    // Update the object again (test_verb will be filtered out due to meta)
    let update_request_3 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_3))
        .await
        .expect("Failed to update object");
    
    // Get the change status to see the diff
    let status_request = json!({
        "operation": "change/status",
        "args": []
    });
    
    let status_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(status_request),
    )
    .await
    .expect("Failed to get change status");
    
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Getting change status should succeed"
    );
    
    // Verify using the direct database access to check object changes
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Build the diff using our diff function
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    // Find the object change for our test object
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == object_name);
    
    if let Some(obj_change) = obj_change {
        // test_verb should NOT be in verbs_deleted
        assert!(
            !obj_change.verbs_deleted.contains("test_verb"),
            "test_verb should not be in verbs_deleted since it's ignored"
        );
    }
}

#[tokio::test]
async fn test_diff_shows_actual_deletions_with_ignored_present() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    // Create an object with multiple properties
    let object_name = "test_object_with_meta";
    let object_dump = load_moo_file("test_object_with_meta.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request))
        .await
        .expect("Failed to update object");
    
    // Submit the initial change
    let submit_request = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request))
        .await
        .expect("Failed to submit change");
    
    // Add test_property to ignored (but not another_property)
    let add_property_request = json!({
        "operation": "meta/add_ignored_property",
        "args": [
            object_name,
            "test_property"
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(add_property_request))
        .await
        .expect("Failed to add ignored property");
    
    // Submit the meta change
    let submit_request_2 = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request_2))
        .await
        .expect("Failed to submit meta change");
    
    // Create an object without another_property (actually deleting it)
    let simple_object = vec![
        "object #9999",
        "  name: \"Test Object With Properties and Verbs\"",
        "  parent: #1",
        "  location: #2",
        "  owner: #2",
        "",
        "  verb test_verb (this none this) owner: #2 flags: \"rxd\"",
        "    x = 42;",
        "    return x;",
        "  endverb",
        "",
        "  verb another_verb (this none this) owner: #2 flags: \"r\"",
        "    return \"hello\";",
        "  endverb",
        "endobject",
    ];
    
    let update_request_3 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&simple_object).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_3))
        .await
        .expect("Failed to update object");
    
    // Get the diff
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == object_name)
        .expect("Should have object change");
    
    // another_property should be in props_deleted (it was actually removed)
    assert!(
        obj_change.props_deleted.contains("another_property"),
        "another_property should be in props_deleted (actually deleted)"
    );
    
    // test_property should NOT be in props_deleted (it's ignored)
    assert!(
        !obj_change.props_deleted.contains("test_property"),
        "test_property should not be in props_deleted (ignored, not deleted)"
    );
}

