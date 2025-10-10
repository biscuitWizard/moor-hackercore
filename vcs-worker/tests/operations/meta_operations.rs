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

