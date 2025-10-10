//! Tests for filtering ignored properties and verbs during object get operations

use crate::common::*;

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

