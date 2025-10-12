//! Tests for clearing all ignored properties or verbs from meta objects

use crate::common::*;
use moor_vcs_worker::types::VcsObjectType;

#[tokio::test]
async fn test_meta_clear_ignored_properties() {
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

    // Add multiple ignored properties
    for prop in &["prop1", "prop2", "prop3"] {
        client
            .meta_add_ignored_property("test_object", prop)
            .await
            .expect("Failed to add ignored property");
    }

    // Clear all ignored properties
    client
        .meta_clear_ignored_properties("test_object")
        .await
        .expect("Failed to clear ignored properties")
        .assert_success("Clear ignored properties");

    // Verify all properties were cleared
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
        meta.ignored_properties.is_empty(),
        "Meta should have no ignored properties after clearing"
    );
}

#[tokio::test]
async fn test_meta_clear_ignored_verbs() {
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

    // Add multiple ignored verbs
    for verb in &["verb1", "verb2", "verb3"] {
        client
            .meta_add_ignored_verb("test_object", verb)
            .await
            .expect("Failed to add ignored verb");
    }

    // Clear all ignored verbs
    client
        .meta_clear_ignored_verbs("test_object")
        .await
        .expect("Failed to clear ignored verbs")
        .assert_success("Clear ignored verbs");

    // Verify all verbs were cleared
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
        meta.ignored_verbs.is_empty(),
        "Meta should have no ignored verbs after clearing"
    );
}

#[tokio::test]
async fn test_meta_clear_ignored_properties_when_no_meta_exists() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    // Create an object without any meta
    client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    // Try to clear ignored properties when no meta exists
    let clear_response = client
        .meta_clear_ignored_properties("test_object")
        .await
        .expect("Failed to clear ignored properties");

    assert!(
        clear_response.is_success(),
        "Clearing when no meta exists should succeed"
    );

    // Verify the response indicates 0 properties were cleared
    let result_str = clear_response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("0 ignored properties") || result_str.contains("No meta exists"),
        "Response should indicate 0 properties cleared or no meta exists, got: {}",
        result_str
    );

    // Verify no meta was created
    db.assert_ref_not_exists(VcsObjectType::MooMetaObject, "test_object");
}

#[tokio::test]
async fn test_meta_clear_ignored_verbs_when_no_meta_exists() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();
    let db = server.db_assertions();

    // Create an object without any meta
    client
        .object_update_from_file("test_object", "test_object.moo")
        .await
        .expect("Failed to update object");

    // Try to clear ignored verbs when no meta exists
    let clear_response = client
        .meta_clear_ignored_verbs("test_object")
        .await
        .expect("Failed to clear ignored verbs");

    assert!(
        clear_response.is_success(),
        "Clearing when no meta exists should succeed"
    );

    // Verify the response indicates 0 verbs were cleared
    let result_str = clear_response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("0 ignored verbs") || result_str.contains("No meta exists"),
        "Response should indicate 0 verbs cleared or no meta exists, got: {}",
        result_str
    );

    // Verify no meta was created
    db.assert_ref_not_exists(VcsObjectType::MooMetaObject, "test_object");
}
