//! Tests for diff behavior with ignored properties and verbs

use crate::common::*;

#[tokio::test]
async fn test_diff_excludes_ignored_properties_from_deleted() {
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
    
    // Submit the change
    let submit_request = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request))
        .await
        .expect("Failed to submit change");
    
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
    
    // Submit the meta change
    let submit_request_2 = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request_2))
        .await
        .expect("Failed to submit meta change");
    
    // Update the object again (test_property will be filtered out due to meta)
    let update_request_3 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_3))
        .await
        .expect("Failed to update object");
    
    // Get the change status to see the diff
    let status_request = json!({
        "operation": "change/status",
        "args": []
    });
    
    let status_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(status_request),
    )
    .await
    .expect("Failed to get change status");
    
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Getting change status should succeed"
    );
    
    // Verify using the direct database access to check object changes
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Build the diff using our diff function
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    // Find the object change for our test object
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == object_name);
    
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
    
    // Submit the change
    let submit_request = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request))
        .await
        .expect("Failed to submit change");
    
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
    
    // Submit the meta change
    let submit_request_2 = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request_2))
        .await
        .expect("Failed to submit meta change");
    
    // Update the object again (test_verb will be filtered out due to meta)
    let update_request_3 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_3))
        .await
        .expect("Failed to update object");
    
    // Get the change status to see the diff
    let status_request = json!({
        "operation": "change/status",
        "args": []
    });
    
    let status_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(status_request),
    )
    .await
    .expect("Failed to get change status");
    
    assert!(
        status_response["success"].as_bool().unwrap_or(false),
        "Getting change status should succeed"
    );
    
    // Verify using the direct database access to check object changes
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Build the diff using our diff function
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    // Find the object change for our test object
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == object_name);
    
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
    let base_url = server.base_url();
    
    // Create an object with multiple properties
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
    
    // Submit the initial change
    let submit_request = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request))
        .await
        .expect("Failed to submit change");
    
    // Add test_property to ignored (but not another_property)
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
    
    // Submit the meta change
    let submit_request_2 = json!({
        "operation": "change/submit",
        "args": []
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(submit_request_2))
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
    
    let update_request_3 = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&simple_object).unwrap()
        ]
    });
    
    make_request("POST", &format!("{}/rpc", base_url), Some(update_request_3))
        .await
        .expect("Failed to update object");
    
    // Get the diff
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let diff = moor_vcs_worker::object_diff::build_object_diff_from_change(server.database(), &change)
        .expect("Failed to build diff");
    
    let obj_change = diff.changes.iter()
        .find(|c| c.obj_id == object_name)
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

