//! Tests for filtering ignored properties and verbs during object update operations

use crate::common::*;

#[tokio::test]
async fn test_object_update_filters_ignored_properties() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    // Create an object
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Add test_property to ignored properties
    client.meta_add_ignored_property("test_object_with_meta", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    // Try to update the object again with test_property included
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update object");
    
    // Get the object and verify test_property is not in it
    let get_response = client.object_get("test_object_with_meta")
        .await
        .expect("Failed to get object");
    
    let content = get_response.require_result_str("Get object");
    
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
    let client = server.client();
    
    // Create an object
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Add test_verb to ignored verbs
    client.meta_add_ignored_verb("test_object_with_meta", "test_verb")
        .await
        .expect("Failed to add ignored verb");
    
    // Try to update the object again with test_verb included
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Update object");
    
    // Get the object and verify test_verb is not in it
    let get_response = client.object_get("test_object_with_meta")
        .await
        .expect("Failed to get object");
    
    let content = get_response.require_result_str("Get object");
    
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
    let client = server.client();
    
    // Create an object
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Add ignored property and verb
    client.meta_add_ignored_property("test_object_with_meta", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    client.meta_add_ignored_verb("test_object_with_meta", "test_verb")
        .await
        .expect("Failed to add ignored verb");
    
    // Update with the same content (should filter during update)
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Get the object
    let get_response = client.object_get("test_object_with_meta")
        .await
        .expect("Failed to get object");
    
    let content = get_response.require_result_str("Get object");
    
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
