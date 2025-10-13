//! Tests for managing user permissions

use crate::common::*;
use moor_vcs_worker::types::Permission;

#[tokio::test]
async fn test_add_permission() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Add permission to user");

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

    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("eve".to_string()),
                serde_json::Value::String("eve@example.com".to_string()),
                serde_json::Value::String("104".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");

    // Step 1: Add Clone permission to eve
    println!("\nStep 1: Adding Clone permission to eve...");
    let response = client
        .rpc_call(
            "user/add_permission",
            vec![
                serde_json::Value::String("eve".to_string()),
                serde_json::Value::String("Clone".to_string()),
            ],
        )
        .await
        .expect("Failed to add permission");

    response.assert_success("Add permission");
    println!("✅ Permission added");

    // Step 2: Verify permission was added
    println!("\nStep 2: Verifying permission...");
    let eve = server
        .database()
        .users()
        .get_user("eve")
        .expect("Failed to get user")
        .expect("User not found");

    assert!(
        eve.has_permission(&Permission::Clone),
        "User should have Clone permission"
    );
    println!("✅ User has Clone permission");

    println!("\n✅ Test passed: Add permission success");
}

#[tokio::test]
async fn test_remove_permission() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Remove permission from user");

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

    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("frank".to_string()),
                serde_json::Value::String("frank@example.com".to_string()),
                serde_json::Value::String("105".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");

    server
        .database()
        .users()
        .add_permission("frank", Permission::Clone)
        .expect("Failed to add permission");

    // Step 1: Remove Clone permission from frank
    println!("\nStep 1: Removing Clone permission from frank...");
    let response = client
        .rpc_call(
            "user/remove_permission",
            vec![
                serde_json::Value::String("frank".to_string()),
                serde_json::Value::String("Clone".to_string()),
            ],
        )
        .await
        .expect("Failed to remove permission");

    response.assert_success("Remove permission");
    println!("✅ Permission removed");

    // Step 2: Verify permission was removed
    println!("\nStep 2: Verifying permission removal...");
    let frank = server
        .database()
        .users()
        .get_user("frank")
        .expect("Failed to get user")
        .expect("User not found");

    assert!(
        !frank.has_permission(&Permission::Clone),
        "User should not have Clone permission"
    );
    println!("✅ User does not have Clone permission");

    println!("\n✅ Test passed: Remove permission success");
}

