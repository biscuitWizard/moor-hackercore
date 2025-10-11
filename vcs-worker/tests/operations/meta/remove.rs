//! Tests for removing ignored properties and verbs from meta objects

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

#[tokio::test]
async fn test_meta_remove_ignored_property() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();
    
    // Create an object first
    client.object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    // Add an ignored property
    client.meta_add_ignored_property("test_object", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    // Remove the ignored property
    client.meta_remove_ignored_property("test_object", "test_property")
        .await
        .expect("Failed to remove ignored property")
        .assert_success("Remove ignored property");
    
    // Verify the property was removed
    let meta_ref = db.assert_ref_exists(VcsObjectType::MooMetaObject, "test_object");
    
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
    let client = server.client();
    let db = server.db_assertions();
    
    // Create an object first
    client.object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");
    
    // Add an ignored verb
    client.meta_add_ignored_verb("test_object", "test_verb")
        .await
        .expect("Failed to add ignored verb");
    
    // Remove the ignored verb
    client.meta_remove_ignored_verb("test_object", "test_verb")
        .await
        .expect("Failed to remove ignored verb")
        .assert_success("Remove ignored verb");
    
    // Verify the verb was removed
    let meta_ref = db.assert_ref_exists(VcsObjectType::MooMetaObject, "test_object");
    
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
