//! Tests for enabling and disabling users

use crate::common::*;
use moor_vcs_worker::types::Permission;

#[tokio::test]
async fn test_disable_user() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Disable user and verify they cannot authenticate");

    // Step 1: Create and setup user
    println!("\nStep 1: Setting up user...");
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::DisableUser)
        .expect("Failed to add DisableUser permission");

    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("charlie".to_string()),
                serde_json::Value::String("charlie@example.com".to_string()),
                serde_json::Value::String("102".to_string()),
            ],
        )
        .await
        .expect("Failed to create user")
        .assert_success("Create user");
    println!("✅ User created");

    // Step 2: Disable the user
    println!("\nStep 2: Disabling user...");
    let response = client
        .rpc_call(
            "user/disable",
            vec![serde_json::Value::String("charlie".to_string())],
        )
        .await
        .expect("Failed to disable user");

    response.assert_success("Disable user");
    println!("✅ User disabled");

    // Step 3: Verify user is disabled
    println!("\nStep 3: Verifying user is disabled...");
    let is_disabled = server
        .database()
        .users()
        .is_disabled("charlie")
        .expect("Failed to check if user is disabled");

    assert!(is_disabled, "User should be disabled");
    println!("✅ User is disabled");

    println!("\n✅ Test passed: Disable user success");
}

#[tokio::test]
async fn test_enable_user() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Re-enable a disabled user");

    // Setup: Create and disable user
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::DisableUser)
        .expect("Failed to add DisableUser permission");

    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("dave".to_string()),
                serde_json::Value::String("dave@example.com".to_string()),
                serde_json::Value::String("103".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");

    server
        .database()
        .users()
        .disable_user("dave")
        .expect("Failed to disable user");

    // Step 1: Re-enable the user
    println!("\nStep 1: Re-enabling user...");
    let response = client
        .rpc_call(
            "user/enable",
            vec![serde_json::Value::String("dave".to_string())],
        )
        .await
        .expect("Failed to enable user");

    response.assert_success("Enable user");
    println!("✅ User enabled");

    // Step 2: Verify user is enabled
    println!("\nStep 2: Verifying user is enabled...");
    let is_disabled = server
        .database()
        .users()
        .is_disabled("dave")
        .expect("Failed to check if user is disabled");

    assert!(!is_disabled, "User should be enabled");
    println!("✅ User is enabled");

    println!("\n✅ Test passed: Enable user success");
}

#[tokio::test]
async fn test_cannot_disable_system_user() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Cannot disable system users");

    // Grant DisableUser permission
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::DisableUser)
        .expect("Failed to add DisableUser permission");

    // Try to disable the Wizard (system user)
    let response = client
        .rpc_call(
            "user/disable",
            vec![serde_json::Value::String("Wizard".to_string())],
        )
        .await
        .expect("Failed to call disable user");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("system user"),
        "Expected system user error, got: {}",
        result_str
    );

    println!("✅ Test passed: System users cannot be disabled");
}

