//! Tests for adding ignored properties and verbs to meta objects

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

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

