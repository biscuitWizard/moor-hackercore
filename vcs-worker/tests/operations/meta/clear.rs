//! Tests for clearing all ignored properties or verbs from meta objects

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

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

