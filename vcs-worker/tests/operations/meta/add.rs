//! Tests for adding ignored properties and verbs to meta objects

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

#[tokio::test]
async fn test_meta_add_ignored_property() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    // Create an object first
    client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object")
        .assert_success("Object update");

    // Add an ignored property
    client
        .meta_add_ignored_property("test_object", "test_property")
        .await
        .expect("Failed to add ignored property")
        .assert_success("Add ignored property");

    // Verify the meta was created
    let meta_sha256 = db.assert_ref_exists(VcsObjectType::MooMetaObject, "test_object");

    let meta_yaml = server
        .database()
        .objects()
        .get(&meta_sha256)
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");

    let meta = server
        .database()
        .objects()
        .parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");

    assert!(
        meta.ignored_properties.contains("test_property"),
        "Meta should contain ignored property 'test_property'"
    );

    // Verify the change tracks the meta object
    let (_, change) = db.require_top_change();
    let meta_in_change = change
        .added_objects
        .iter()
        .any(|obj| obj.object_type == VcsObjectType::MooMetaObject && obj.name == "test_object");

    assert!(
        meta_in_change,
        "Change should track the meta object as added"
    );
}

#[tokio::test]
async fn test_meta_add_ignored_verb() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    // Create an object first
    client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    // Add an ignored verb
    client
        .meta_add_ignored_verb("test_object", "test_verb")
        .await
        .expect("Failed to add ignored verb")
        .assert_success("Add ignored verb");

    // Verify the meta contains the ignored verb
    let meta_ref = db.assert_ref_exists(VcsObjectType::MooMetaObject, "test_object");

    let meta_yaml = server
        .database()
        .objects()
        .get(&meta_ref)
        .expect("Failed to get meta YAML")
        .expect("Meta YAML should exist");

    let meta = server
        .database()
        .objects()
        .parse_meta_dump(&meta_yaml)
        .expect("Failed to parse meta");

    assert!(
        meta.ignored_verbs.contains("test_verb"),
        "Meta should contain ignored verb 'test_verb'"
    );
}
