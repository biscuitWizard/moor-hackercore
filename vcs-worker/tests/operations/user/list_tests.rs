//! Tests for listing users

use crate::common::*;
use moor_vcs_worker::types::Permission;

#[tokio::test]
async fn test_list_users() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: List all users");

    // Setup
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::ManagePermissions)
        .expect("Failed to add ManagePermissions permission");

    // Create a few users
    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("iris".to_string()),
                serde_json::Value::String("iris@example.com".to_string()),
                serde_json::Value::String("108".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");
    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("jack".to_string()),
                serde_json::Value::String("jack@example.com".to_string()),
                serde_json::Value::String("109".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");

    // List users
    println!("\nStep 1: Listing users...");
    let response = client
        .rpc_call("user/list", vec![])
        .await
        .expect("Failed to list users");

    // Response should be a list
    let users_list = response["result"]
        .as_array()
        .expect("Result should be an array");

    // Should have at least 4 users (Everyone, Wizard, iris, jack)
    assert!(
        users_list.len() >= 4,
        "Should have at least 4 users, got {}",
        users_list.len()
    );
    println!("✅ Listed {} users", users_list.len());

    println!("\n✅ Test passed: List users success");
}

