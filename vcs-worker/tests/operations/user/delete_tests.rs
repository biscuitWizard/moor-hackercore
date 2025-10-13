//! Tests for user deletion operations

use crate::common::*;
use moor_vcs_worker::types::Permission;

#[tokio::test]
async fn test_delete_user_success() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Delete user with DeleteUser permission");

    // Step 1: Setup - Create user and grant permissions
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
        .add_permission(&wizard_user.id, Permission::DeleteUser)
        .expect("Failed to add DeleteUser permission");

    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("testuser".to_string()),
                serde_json::Value::String("testuser@example.com".to_string()),
                serde_json::Value::String("200".to_string()),
            ],
        )
        .await
        .expect("Failed to create user")
        .assert_success("Create user");
    println!("✅ User created");

    // Step 2: Delete the user
    println!("\nStep 2: Deleting user...");
    let response = client
        .rpc_call(
            "user/delete",
            vec![serde_json::Value::String("testuser".to_string())],
        )
        .await
        .expect("Failed to delete user");

    response.assert_success("Delete user");
    println!("✅ User deleted");

    // Step 3: Verify user no longer exists
    println!("\nStep 3: Verifying user is deleted...");
    let user = server
        .database()
        .users()
        .get_user("testuser")
        .expect("Failed to query user");

    assert!(user.is_none(), "User should not exist after deletion");
    println!("✅ User confirmed deleted");

    println!("\n✅ Test passed: Delete user success");
}

#[tokio::test]
async fn test_delete_user_no_permission() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Delete user without DeleteUser permission should fail");

    // Step 1: Remove DeleteUser permission from Wizard
    println!("\nStep 1: Removing DeleteUser permission from Wizard...");
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server
        .database()
        .users()
        .remove_permission(&wizard_user.id, &Permission::DeleteUser)
        .expect("Failed to remove DeleteUser permission");
    println!("✅ DeleteUser permission removed");

    // Step 2: Create a user
    println!("\nStep 2: Creating user...");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");

    client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("tempuser".to_string()),
                serde_json::Value::String("tempuser@example.com".to_string()),
                serde_json::Value::String("201".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");
    println!("✅ User created");

    // Step 3: Try to delete without permission
    println!("\nStep 3: Attempting to delete without permission...");
    let response = client
        .rpc_call(
            "user/delete",
            vec![serde_json::Value::String("tempuser".to_string())],
        )
        .await
        .expect("Failed to call delete user");

    // Should get an error about permissions
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("permission"),
        "Expected permission error, got: {}",
        result_str
    );
    println!("✅ Permission check works correctly");

    // Step 4: Verify user still exists
    println!("\nStep 4: Verifying user still exists...");
    let user = server
        .database()
        .users()
        .get_user("tempuser")
        .expect("Failed to query user");

    assert!(user.is_some(), "User should still exist after failed deletion");
    println!("✅ User was not deleted");

    println!("\n✅ Test passed: Delete user correctly requires permission");
}

#[tokio::test]
async fn test_cannot_delete_system_user() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Cannot delete system users (Wizard, Everyone)");

    // Grant DeleteUser permission
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::DeleteUser)
        .expect("Failed to add DeleteUser permission");

    // Try to delete the Wizard (system user)
    println!("\nStep 1: Attempting to delete Wizard...");
    let response = client
        .rpc_call(
            "user/delete",
            vec![serde_json::Value::String("Wizard".to_string())],
        )
        .await
        .expect("Failed to call delete user");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("system user"),
        "Expected system user error, got: {}",
        result_str
    );
    println!("✅ Wizard cannot be deleted");

    // Try to delete Everyone (system user)
    println!("\nStep 2: Attempting to delete Everyone...");
    let response = client
        .rpc_call(
            "user/delete",
            vec![serde_json::Value::String("Everyone".to_string())],
        )
        .await
        .expect("Failed to call delete user");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("system user"),
        "Expected system user error, got: {}",
        result_str
    );
    println!("✅ Everyone cannot be deleted");

    println!("\n✅ Test passed: System users cannot be deleted");
}

#[tokio::test]
async fn test_delete_nonexistent_user() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Delete nonexistent user returns appropriate error");

    // Grant DeleteUser permission
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::DeleteUser)
        .expect("Failed to add DeleteUser permission");

    // Try to delete a user that doesn't exist
    println!("\nStep 1: Attempting to delete nonexistent user...");
    let response = client
        .rpc_call(
            "user/delete",
            vec![serde_json::Value::String("nonexistent_user".to_string())],
        )
        .await
        .expect("Failed to call delete user");

    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error") || result_str.contains("not found"),
        "Expected not found error, got: {}",
        result_str
    );
    println!("✅ Appropriate error for nonexistent user");

    println!("\n✅ Test passed: Delete nonexistent user handled correctly");
}

