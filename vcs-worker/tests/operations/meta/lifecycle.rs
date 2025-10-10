//! Tests for meta behavior during object lifecycle operations (rename, delete)

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

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

