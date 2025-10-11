//! Tests for meta behavior during object lifecycle operations (rename, delete)

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

#[tokio::test]
async fn test_meta_rename_with_object() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    // Create an object
    client.object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    // Add some metadata
    client.meta_add_ignored_property("test_object", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    // Rename the object
    client.object_rename("test_object", "renamed_object")
        .await
        .expect("Failed to rename object")
        .assert_success("Rename object");
    
    // Verify the meta was also renamed
    db.assert_ref_exists(VcsObjectType::MooMetaObject, "renamed_object");
    
    // Verify old meta ref no longer exists
    db.assert_ref_not_exists(VcsObjectType::MooMetaObject, "test_object");
    
    // Verify the meta content is preserved
    let meta_ref = db.assert_ref_exists(VcsObjectType::MooMetaObject, "renamed_object");
    let meta_yaml = server.database().objects().get(&meta_ref)
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
    let client = server.client();
    let db = server.db_assertions();
    
    // Create an object
    client.object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    // Add some metadata
    client.meta_add_ignored_property("test_object", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    // Delete the object
    client.object_delete("test_object")
        .await
        .expect("Failed to delete object")
        .assert_success("Delete object");
    
    // Verify the meta was also marked for deletion in the change
    let (_, change) = db.require_top_change();
    
    let meta_in_deleted = change.deleted_objects.iter()
        .any(|obj| obj.object_type == VcsObjectType::MooMetaObject && obj.name == "test_object");
    
    assert!(
        meta_in_deleted,
        "Change should track the meta object as deleted"
    );
}
