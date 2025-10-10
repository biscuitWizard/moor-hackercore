//! Tests for filtering ignored properties and verbs during object update operations

use crate::common::*;

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

