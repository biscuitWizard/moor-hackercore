//! Tests for managing user API keys

use crate::common::*;
use moor_vcs_worker::types::Permission;

#[tokio::test]
async fn test_generate_api_key_self() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: User generates their own API key (self-service)");

    // Generate API key for Wizard (self)
    println!("\nStep 1: Generating API key...");
    let response = client
        .rpc_call("user/generate_api_key", vec![])
        .await
        .expect("Failed to generate API key");

    let api_key = response.require_result_str("Generate API key");
    assert!(
        !api_key.contains("Error"),
        "Should not get error: {}",
        api_key
    );
    assert!(api_key.len() > 20, "API key should be a reasonable length");
    println!(
        "✅ API key generated: {} (length: {})",
        api_key,
        api_key.len()
    );

    // Verify the key was added to the user
    println!("\nStep 2: Verifying API key was added...");
    let wizard = server.get_wizard_user().expect("Failed to get Wizard user");
    assert!(
        wizard.authorized_keys.contains(&api_key.to_string()),
        "User should have the new API key"
    );
    println!("✅ API key verified in user's authorized keys");

    println!("\n✅ Test passed: Generate API key (self-service) success");
}

#[tokio::test]
async fn test_generate_api_key_other_user() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Generate API key for another user (requires ManageApiKeys)");

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
        .add_permission(&wizard_user.id, Permission::ManageApiKeys)
        .expect("Failed to add ManageApiKeys permission");

    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("grace".to_string()),
                serde_json::Value::String("grace@example.com".to_string()),
                serde_json::Value::String("106".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");

    // Generate API key for grace
    println!("\nStep 1: Generating API key for grace...");
    let response = client
        .rpc_call(
            "user/generate_api_key",
            vec![serde_json::Value::String("grace".to_string())],
        )
        .await
        .expect("Failed to generate API key");

    let api_key = response.require_result_str("Generate API key");
    assert!(
        !api_key.contains("Error"),
        "Should not get error: {}",
        api_key
    );
    println!("✅ API key generated for grace");

    // Verify the key was added
    let grace = server
        .database()
        .users()
        .get_user("grace")
        .expect("Failed to get user")
        .expect("User not found");
    assert!(
        grace.authorized_keys.contains(&api_key.to_string()),
        "User should have the new API key"
    );
    println!("✅ API key verified");

    println!("\n✅ Test passed: Generate API key for other user success");
}

#[tokio::test]
async fn test_delete_api_key_self() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: User deletes their own API key");

    // Generate an API key first
    let response = client
        .rpc_call("user/generate_api_key", vec![])
        .await
        .expect("Failed to generate API key");
    let api_key = response.require_result_str("Generate API key").to_string();

    // Delete the API key
    println!("\nStep 1: Deleting API key...");
    let response = client
        .rpc_call(
            "user/delete_api_key",
            vec![serde_json::Value::String(api_key.clone())],
        )
        .await
        .expect("Failed to delete API key");

    response.assert_success("Delete API key");
    println!("✅ API key deleted");

    // Verify the key was removed
    println!("\nStep 2: Verifying API key removal...");
    let wizard = server.get_wizard_user().expect("Failed to get Wizard user");
    assert!(
        !wizard.authorized_keys.contains(&api_key),
        "User should not have the API key"
    );
    println!("✅ API key removed from user");

    println!("\n✅ Test passed: Delete API key (self) success");
}

#[tokio::test]
async fn test_delete_api_key_other_user() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Delete API key from another user (requires ManageApiKeys)");

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
        .add_permission(&wizard_user.id, Permission::ManageApiKeys)
        .expect("Failed to add ManageApiKeys permission");

    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("henry".to_string()),
                serde_json::Value::String("henry@example.com".to_string()),
                serde_json::Value::String("107".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");

    // Generate API key for henry
    let api_key = server
        .database()
        .users()
        .generate_api_key("henry")
        .expect("Failed to generate API key");

    // Delete the API key
    println!("\nStep 1: Deleting API key from henry...");
    let response = client
        .rpc_call(
            "user/delete_api_key",
            vec![
                serde_json::Value::String(api_key.clone()),
                serde_json::Value::String("henry".to_string()),
            ],
        )
        .await
        .expect("Failed to delete API key");

    response.assert_success("Delete API key");
    println!("✅ API key deleted from henry");

    // Verify the key was removed
    let henry = server
        .database()
        .users()
        .get_user("henry")
        .expect("Failed to get user")
        .expect("User not found");
    assert!(
        !henry.authorized_keys.contains(&api_key),
        "User should not have the API key"
    );
    println!("✅ API key removed");

    println!("\n✅ Test passed: Delete API key from other user success");
}

