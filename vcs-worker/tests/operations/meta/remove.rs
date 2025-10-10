//! Tests for removing ignored properties and verbs from meta objects

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

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

