//! Tests for diff behavior with ignored properties and verbs

use crate::common::*;

#[tokio::test]
async fn test_diff_excludes_ignored_properties_from_deleted() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    // Create an object with properties
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Submit the change
    client.change_submit()
        .await
        .expect("Failed to submit change");
    
    // Add test_property to ignored properties
    client.meta_add_ignored_property("test_object_with_meta", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    // Submit the meta change
    client.change_submit()
        .await
        .expect("Failed to submit meta change");
    
    // Update the object again (test_property will be filtered out due to meta)
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Get the change status to see the diff
    client.change_status()
        .await
        .expect("Failed to get change status")
        .assert_success("Get change status");
    
    // Verify using the direct database access to check object changes
    let (top_change_id, _) = server.db_assertions().require_top_change();
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Build the diff using our diff function
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    // Find the object change for our test object
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == "test_object_with_meta");
    
    if let Some(obj_change) = obj_change {
        // test_property should NOT be in props_deleted
        assert!(
            !obj_change.props_deleted.contains("test_property"),
            "test_property should not be in props_deleted since it's ignored"
        );
    }
}

#[tokio::test]
async fn test_diff_excludes_ignored_verbs_from_deleted() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    // Create an object with verbs
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Submit the change
    client.change_submit()
        .await
        .expect("Failed to submit change");
    
    // Add test_verb to ignored verbs
    client.meta_add_ignored_verb("test_object_with_meta", "test_verb")
        .await
        .expect("Failed to add ignored verb");
    
    // Submit the meta change
    client.change_submit()
        .await
        .expect("Failed to submit meta change");
    
    // Update the object again (test_verb will be filtered out due to meta)
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Get the change status to see the diff
    client.change_status()
        .await
        .expect("Failed to get change status")
        .assert_success("Get change status");
    
    // Verify using the direct database access to check object changes
    let (top_change_id, _) = server.db_assertions().require_top_change();
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Build the diff using our diff function
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    // Find the object change for our test object
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == "test_object_with_meta");
    
    if let Some(obj_change) = obj_change {
        // test_verb should NOT be in verbs_deleted
        assert!(
            !obj_change.verbs_deleted.contains("test_verb"),
            "test_verb should not be in verbs_deleted since it's ignored"
        );
    }
}

#[tokio::test]
async fn test_diff_shows_actual_deletions_with_ignored_present() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    // Create an object with multiple properties
    client.object_update_from_file("test_object_with_meta", "test_object_with_meta.moo")
        .await
        .expect("Failed to update object");
    
    // Submit the initial change
    client.change_submit()
        .await
        .expect("Failed to submit change");
    
    // Add test_property to ignored (but not another_property)
    client.meta_add_ignored_property("test_object_with_meta", "test_property")
        .await
        .expect("Failed to add ignored property");
    
    // Submit the meta change
    client.change_submit()
        .await
        .expect("Failed to submit meta change");
    
    // Create an object without another_property (actually deleting it)
    let simple_object = vec![
        "object #9999",
        "  name: \"Test Object With Properties and Verbs\"",
        "  parent: #1",
        "  location: #2",
        "  owner: #2",
        "",
        "  verb test_verb (this none this) owner: #2 flags: \"rxd\"",
        "    x = 42;",
        "    return x;",
        "  endverb",
        "",
        "  verb another_verb (this none this) owner: #2 flags: \"r\"",
        "    return \"hello\";",
        "  endverb",
        "endobject",
    ];
    
    client.object_update("test_object_with_meta", simple_object.iter().map(|s| s.to_string()).collect())
        .await
        .expect("Failed to update object");
    
    // Get the diff
    let (top_change_id, _) = server.db_assertions().require_top_change();
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == "test_object_with_meta")
        .expect("Should have object change");
    
    // another_property should be in props_deleted (it was actually removed)
    assert!(
        obj_change.props_deleted.contains("another_property"),
        "another_property should be in props_deleted (actually deleted)"
    );
    
    // test_property should NOT be in props_deleted (it's ignored)
    assert!(
        !obj_change.props_deleted.contains("test_property"),
        "test_property should not be in props_deleted (ignored, not deleted)"
    );
}
