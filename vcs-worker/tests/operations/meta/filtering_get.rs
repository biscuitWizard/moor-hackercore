//! Tests for filtering ignored properties and verbs during object get operations

use crate::common::*;

#[tokio::test]
async fn test_object_get_filters_ignored_properties() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    // Create an object with properties
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Get the object before adding meta (should return full object)
    let get_response_before = client.object_get("test_object_with_meta")
        .await
        .expect("Failed to get object");
    
    get_response_before.assert_success("Get object");
    let content_before = get_response_before.require_result_str("Get object");
    
    // The object should contain 'test_property' in property definitions
    assert!(
        content_before.contains("test_property"),
        "Object should contain test_property before filtering"
    );
    
    // Add test_property to ignored properties
    client.meta_add_ignored_property("test_object_with_meta", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    // Get the object after adding meta (should filter out test_property)
    let get_response_after = client.object_get("test_object_with_meta")
        .await
        .expect("Failed to get object");
    
    get_response_after.assert_success("Get object after adding meta");
    let content_after = get_response_after.require_result_str("Get object");
    
    // The filtered object should NOT contain 'test_property'
    assert!(
        !content_after.contains("test_property"),
        "Object should not contain test_property after filtering"
    );
}

#[tokio::test]
async fn test_object_get_filters_ignored_verbs() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    // Create an object with verbs
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Get the object before adding meta
    let get_response_before = client.object_get("test_object_with_meta")
        .await
        .expect("Failed to get object");
    
    let content_before = get_response_before.require_result_str("Get object");
    
    // The object should contain 'test_verb' in verb definitions
    assert!(
        content_before.contains("test_verb"),
        "Object should contain test_verb before filtering"
    );
    
    // Add test_verb to ignored verbs
    client.meta_add_ignored_verb("test_object_with_meta", "test_verb")
        .await
        .expect("Failed to add ignored verb");
    
    // Get the object after adding meta
    let get_response_after = client.object_get("test_object_with_meta")
        .await
        .expect("Failed to get object");
    
    let content_after = get_response_after.require_result_str("Get object");
    
    // The filtered object should NOT contain 'test_verb'
    assert!(
        !content_after.contains("test_verb"),
        "Object should not contain test_verb after filtering"
    );
}

#[tokio::test]
async fn test_object_get_filters_multiple_properties_and_verbs() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    // Create an object with both properties and verbs
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Add multiple ignored properties and verbs
    client.meta_add_ignored_property("test_object_with_meta", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    client.meta_add_ignored_verb("test_object_with_meta", "test_verb")
        .await
        .expect("Failed to add ignored verb");
    
    // Get the filtered object
    let get_response = client.object_get("test_object_with_meta")
        .await
        .expect("Failed to get object");
    
    get_response.assert_success("Get object");
    let content = get_response.require_result_str("Get object");
    
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
