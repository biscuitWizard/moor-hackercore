//! Integration tests for user management operations
//! 
//! Tests cover:
//! - Creating users
//! - Disabling/enabling users  
//! - Managing permissions
//! - Managing API keys
//! - Listing users
//! - External user configuration for clone operations

use crate::common::*;
use moor_vcs_worker::types::Permission;

#[tokio::test]
async fn test_create_user_success() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Create user with CreateUser permission");
    
    // Step 1: Grant CreateUser permission to Wizard
    println!("\nStep 1: Granting CreateUser permission to Wizard...");
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    println!("✅ Wizard has CreateUser permission");
    
    // Step 2: Create a new user
    println!("\nStep 2: Creating new user 'alice'...");
    let response = client.rpc_call("user/create", vec![
        serde_json::Value::String("alice".to_string()),
        serde_json::Value::String("alice@example.com".to_string()),
        serde_json::Value::String("100".to_string())
    ])
        .await
        .expect("Failed to create user");
    
    response.assert_success("Create user");
    println!("✅ User created successfully");
    
    // Step 3: Verify user exists
    println!("\nStep 3: Verifying user exists...");
    let alice = server.database().users().get_user("alice")
        .expect("Failed to get user")
        .expect("User not found");
    
    assert_eq!(alice.id, "alice");
    assert_eq!(alice.email, "alice@example.com");
    assert_eq!(alice.v_obj, moor_var::Obj::mk_id(100));
    assert!(!alice.is_disabled);
    assert!(!alice.is_system_user);
    println!("✅ User verified: {} ({})", alice.id, alice.email);
    
    println!("\n✅ Test passed: Create user success");
}

#[tokio::test]
async fn test_create_user_no_permission() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Create user without CreateUser permission should fail");
    
    // Try to create user without permission (Wizard doesn't have CreateUser by default)
    let response = client.rpc_call("user/create", vec![
        serde_json::Value::String("bob".to_string()),
        serde_json::Value::String("bob@example.com".to_string()),
        serde_json::Value::String("101".to_string())
    ])
        .await
        .expect("Failed to call create user");
    
    // Should get an error about permissions
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("permission"), 
            "Expected permission error, got: {}", result_str);
    
    println!("✅ Test passed: Create user correctly requires permission");
}

#[tokio::test]
async fn test_disable_user() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Disable user and verify they cannot authenticate");
    
    // Step 1: Create and setup user
    println!("\nStep 1: Setting up user...");
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server.database().users().add_permission(&wizard_user.id, Permission::DisableUser)
        .expect("Failed to add DisableUser permission");
    
    client.rpc_call("user/create", vec![
        serde_json::Value::String("charlie".to_string()),
        serde_json::Value::String("charlie@example.com".to_string()),
        serde_json::Value::String("102".to_string())
    ])
        .await
        .expect("Failed to create user")
        .assert_success("Create user");
    println!("✅ User created");
    
    // Step 2: Disable the user
    println!("\nStep 2: Disabling user...");
    let response = client.rpc_call("user/disable", vec![
        serde_json::Value::String("charlie".to_string())
    ])
        .await
        .expect("Failed to disable user");
    
    response.assert_success("Disable user");
    println!("✅ User disabled");
    
    // Step 3: Verify user is disabled
    println!("\nStep 3: Verifying user is disabled...");
    let is_disabled = server.database().users().is_disabled("charlie")
        .expect("Failed to check if user is disabled");
    
    assert!(is_disabled, "User should be disabled");
    println!("✅ User is disabled");
    
    println!("\n✅ Test passed: Disable user success");
}

#[tokio::test]
async fn test_enable_user() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Re-enable a disabled user");
    
    // Setup: Create and disable user
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server.database().users().add_permission(&wizard_user.id, Permission::DisableUser)
        .expect("Failed to add DisableUser permission");
    
    client.rpc_call("user/create", vec![
        serde_json::Value::String("dave".to_string()),
        serde_json::Value::String("dave@example.com".to_string()),
        serde_json::Value::String("103".to_string())
    ])
        .await.expect("Failed to create user");
    
    server.database().users().disable_user("dave")
        .expect("Failed to disable user");
    
    // Step 1: Re-enable the user
    println!("\nStep 1: Re-enabling user...");
    let response = client.rpc_call("user/enable", vec![
        serde_json::Value::String("dave".to_string())
    ])
        .await
        .expect("Failed to enable user");
    
    response.assert_success("Enable user");
    println!("✅ User enabled");
    
    // Step 2: Verify user is enabled
    println!("\nStep 2: Verifying user is enabled...");
    let is_disabled = server.database().users().is_disabled("dave")
        .expect("Failed to check if user is disabled");
    
    assert!(!is_disabled, "User should be enabled");
    println!("✅ User is enabled");
    
    println!("\n✅ Test passed: Enable user success");
}

#[tokio::test]
async fn test_cannot_disable_system_user() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Cannot disable system users");
    
    // Grant DisableUser permission
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::DisableUser)
        .expect("Failed to add DisableUser permission");
    
    // Try to disable the Wizard (system user)
    let response = client.rpc_call("user/disable", vec![
        serde_json::Value::String("Wizard".to_string())
    ])
        .await
        .expect("Failed to call disable user");
    
    let result_str = response.get_result_str().unwrap_or("");
    assert!(result_str.contains("Error") || result_str.contains("system user"), 
            "Expected system user error, got: {}", result_str);
    
    println!("✅ Test passed: System users cannot be disabled");
}

#[tokio::test]
async fn test_add_permission() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Add permission to user");
    
    // Setup
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server.database().users().add_permission(&wizard_user.id, Permission::ManagePermissions)
        .expect("Failed to add ManagePermissions permission");
    
    client.rpc_call("user/create", vec![
        serde_json::Value::String("eve".to_string()),
        serde_json::Value::String("eve@example.com".to_string()),
        serde_json::Value::String("104".to_string())
    ])
        .await.expect("Failed to create user");
    
    // Step 1: Add Clone permission to eve
    println!("\nStep 1: Adding Clone permission to eve...");
    let response = client.rpc_call("user/add_permission", vec![
        serde_json::Value::String("eve".to_string()),
        serde_json::Value::String("Clone".to_string())
    ])
        .await
        .expect("Failed to add permission");
    
    response.assert_success("Add permission");
    println!("✅ Permission added");
    
    // Step 2: Verify permission was added
    println!("\nStep 2: Verifying permission...");
    let eve = server.database().users().get_user("eve")
        .expect("Failed to get user")
        .expect("User not found");
    
    assert!(eve.has_permission(&Permission::Clone), "User should have Clone permission");
    println!("✅ User has Clone permission");
    
    println!("\n✅ Test passed: Add permission success");
}

#[tokio::test]
async fn test_remove_permission() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Remove permission from user");
    
    // Setup
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server.database().users().add_permission(&wizard_user.id, Permission::ManagePermissions)
        .expect("Failed to add ManagePermissions permission");
    
    client.rpc_call("user/create", vec![
        serde_json::Value::String("frank".to_string()),
        serde_json::Value::String("frank@example.com".to_string()),
        serde_json::Value::String("105".to_string())
    ])
        .await.expect("Failed to create user");
    
    server.database().users().add_permission("frank", Permission::Clone)
        .expect("Failed to add permission");
    
    // Step 1: Remove Clone permission from frank
    println!("\nStep 1: Removing Clone permission from frank...");
    let response = client.rpc_call("user/remove_permission", vec![
        serde_json::Value::String("frank".to_string()),
        serde_json::Value::String("Clone".to_string())
    ])
        .await
        .expect("Failed to remove permission");
    
    response.assert_success("Remove permission");
    println!("✅ Permission removed");
    
    // Step 2: Verify permission was removed
    println!("\nStep 2: Verifying permission removal...");
    let frank = server.database().users().get_user("frank")
        .expect("Failed to get user")
        .expect("User not found");
    
    assert!(!frank.has_permission(&Permission::Clone), "User should not have Clone permission");
    println!("✅ User does not have Clone permission");
    
    println!("\n✅ Test passed: Remove permission success");
}

#[tokio::test]
async fn test_generate_api_key_self() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: User generates their own API key (self-service)");
    
    // Generate API key for Wizard (self)
    println!("\nStep 1: Generating API key...");
    let response = client.rpc_call("user/generate_api_key", vec![])
        .await
        .expect("Failed to generate API key");
    
    let api_key = response.require_result_str("Generate API key");
    assert!(!api_key.contains("Error"), "Should not get error: {}", api_key);
    assert!(api_key.len() > 20, "API key should be a reasonable length");
    println!("✅ API key generated: {} (length: {})", api_key, api_key.len());
    
    // Verify the key was added to the user
    println!("\nStep 2: Verifying API key was added...");
    let wizard = server.get_wizard_user().expect("Failed to get Wizard user");
    assert!(wizard.authorized_keys.contains(&api_key.to_string()), "User should have the new API key");
    println!("✅ API key verified in user's authorized keys");
    
    println!("\n✅ Test passed: Generate API key (self-service) success");
}

#[tokio::test]
async fn test_generate_api_key_other_user() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Generate API key for another user (requires ManageApiKeys)");
    
    // Setup
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server.database().users().add_permission(&wizard_user.id, Permission::ManageApiKeys)
        .expect("Failed to add ManageApiKeys permission");
    
    client.rpc_call("user/create", vec![
        serde_json::Value::String("grace".to_string()),
        serde_json::Value::String("grace@example.com".to_string()),
        serde_json::Value::String("106".to_string())
    ])
        .await.expect("Failed to create user");
    
    // Generate API key for grace
    println!("\nStep 1: Generating API key for grace...");
    let response = client.rpc_call("user/generate_api_key", vec![
        serde_json::Value::String("grace".to_string())
    ])
        .await
        .expect("Failed to generate API key");
    
    let api_key = response.require_result_str("Generate API key");
    assert!(!api_key.contains("Error"), "Should not get error: {}", api_key);
    println!("✅ API key generated for grace");
    
    // Verify the key was added
    let grace = server.database().users().get_user("grace")
        .expect("Failed to get user")
        .expect("User not found");
    assert!(grace.authorized_keys.contains(&api_key.to_string()), "User should have the new API key");
    println!("✅ API key verified");
    
    println!("\n✅ Test passed: Generate API key for other user success");
}

#[tokio::test]
async fn test_delete_api_key_self() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: User deletes their own API key");
    
    // Generate an API key first
    let response = client.rpc_call("user/generate_api_key", vec![])
        .await
        .expect("Failed to generate API key");
    let api_key = response.require_result_str("Generate API key").to_string();
    
    // Delete the API key
    println!("\nStep 1: Deleting API key...");
    let response = client.rpc_call("user/delete_api_key", vec![
        serde_json::Value::String(api_key.clone())
    ])
        .await
        .expect("Failed to delete API key");
    
    response.assert_success("Delete API key");
    println!("✅ API key deleted");
    
    // Verify the key was removed
    println!("\nStep 2: Verifying API key removal...");
    let wizard = server.get_wizard_user().expect("Failed to get Wizard user");
    assert!(!wizard.authorized_keys.contains(&api_key), "User should not have the API key");
    println!("✅ API key removed from user");
    
    println!("\n✅ Test passed: Delete API key (self) success");
}

#[tokio::test]
async fn test_delete_api_key_other_user() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: Delete API key from another user (requires ManageApiKeys)");
    
    // Setup
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server.database().users().add_permission(&wizard_user.id, Permission::ManageApiKeys)
        .expect("Failed to add ManageApiKeys permission");
    
    client.rpc_call("user/create", vec![
        serde_json::Value::String("henry".to_string()),
        serde_json::Value::String("henry@example.com".to_string()),
        serde_json::Value::String("107".to_string())
    ])
        .await.expect("Failed to create user");
    
    // Generate API key for henry
    let api_key = server.database().users().generate_api_key("henry")
        .expect("Failed to generate API key");
    
    // Delete the API key
    println!("\nStep 1: Deleting API key from henry...");
    let response = client.rpc_call("user/delete_api_key", vec![
        serde_json::Value::String(api_key.clone()),
        serde_json::Value::String("henry".to_string())
    ])
        .await
        .expect("Failed to delete API key");
    
    response.assert_success("Delete API key");
    println!("✅ API key deleted from henry");
    
    // Verify the key was removed
    let henry = server.database().users().get_user("henry")
        .expect("Failed to get user")
        .expect("User not found");
    assert!(!henry.authorized_keys.contains(&api_key), "User should not have the API key");
    println!("✅ API key removed");
    
    println!("\n✅ Test passed: Delete API key from other user success");
}

#[tokio::test]
async fn test_list_users() {
    let server = TestServer::start().await.expect("Failed to start test server");
    let client = server.client();
    
    println!("Test: List all users");
    
    // Setup
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server.database().users().add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    server.database().users().add_permission(&wizard_user.id, Permission::ManagePermissions)
        .expect("Failed to add ManagePermissions permission");
    
    // Create a few users
    client.rpc_call("user/create", vec![
        serde_json::Value::String("iris".to_string()),
        serde_json::Value::String("iris@example.com".to_string()),
        serde_json::Value::String("108".to_string())
    ])
        .await.expect("Failed to create user");
    client.rpc_call("user/create", vec![
        serde_json::Value::String("jack".to_string()),
        serde_json::Value::String("jack@example.com".to_string()),
        serde_json::Value::String("109".to_string())
    ])
        .await.expect("Failed to create user");
    
    // List users
    println!("\nStep 1: Listing users...");
    let response = client.rpc_call("user/list", vec![])
        .await
        .expect("Failed to list users");
    
    // Response should be a list
    let users_list = response["result"].as_array()
        .expect("Result should be an array");
    
    // Should have at least 4 users (Everyone, Wizard, iris, jack)
    assert!(users_list.len() >= 4, "Should have at least 4 users, got {}", users_list.len());
    println!("✅ Listed {} users", users_list.len());
    
    println!("\n✅ Test passed: List users success");
}

#[tokio::test]
async fn test_external_user_in_clone() {
    // This test requires two test servers, which is complex to set up
    // For now, we'll just test that the external user info can be stored/retrieved
    let server = TestServer::start().await.expect("Failed to start test server");
    
    println!("Test: External user configuration storage");
    
    // Store external user info
    println!("\nStep 1: Storing external user credentials...");
    server.database().index().set_external_user_api_key("test-api-key-123")
        .expect("Failed to set external user API key");
    server.database().index().set_external_user_id("external-user-id")
        .expect("Failed to set external user ID");
    println!("✅ External user credentials stored");
    
    // Retrieve and verify
    println!("\nStep 2: Retrieving external user credentials...");
    let api_key = server.database().index().get_external_user_api_key()
        .expect("Failed to get external user API key")
        .expect("External user API key not found");
    let user_id = server.database().index().get_external_user_id()
        .expect("Failed to get external user ID")
        .expect("External user ID not found");
    
    assert_eq!(api_key, "test-api-key-123");
    assert_eq!(user_id, "external-user-id");
    println!("✅ External user credentials retrieved: user_id={}, api_key={}", user_id, api_key);
    
    println!("\n✅ Test passed: External user configuration works");
}

// Note: test_external_user_stat_validation would require setting up a second server
// and making actual HTTP requests between them. This is left as a future enhancement.

