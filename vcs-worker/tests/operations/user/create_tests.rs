//! Tests for user creation operations

use crate::common::*;
use moor_vcs_worker::types::Permission;

#[tokio::test]
async fn test_create_user_success() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let client = server.client();

    println!("Test: Create user with CreateUser permission");

    // Step 1: Grant CreateUser permission to Wizard
    println!("\nStep 1: Granting CreateUser permission to Wizard...");
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");
    server
        .database()
        .users()
        .add_permission(&wizard_user.id, Permission::CreateUser)
        .expect("Failed to add CreateUser permission");
    println!("✅ Wizard has CreateUser permission");

    // Step 2: Create a new user
    println!("\nStep 2: Creating new user 'alice'...");
    let response = client
        .rpc_call(
            "user/create",
            vec![
                serde_json::Value::String("alice".to_string()),
                serde_json::Value::String("alice@example.com".to_string()),
                serde_json::Value::String("100".to_string()),
            ],
        )
        .await
        .expect("Failed to create user");

    response.assert_success("Create user");
    println!("✅ User created successfully");

    // Step 3: Verify user exists
    println!("\nStep 3: Verifying user exists...");
    let alice = server
        .database()
        .users()
        .get_user("alice")
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
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    println!("Test: Create user without CreateUser permission should fail");

    // Step 1: Create a regular user (using database directly)
    println!("\nStep 1: Creating regular user without CreateUser permission...");
    let alice = moor_vcs_worker::types::User::new(
        "alice".to_string(),
        "alice@example.com".to_string(),
        moor_var::Obj::mk_id(101),
    );
    
    server
        .database()
        .users()
        .create_user("alice".to_string(), "alice@example.com".to_string(), moor_var::Obj::mk_id(101))
        .expect("Failed to create alice");
    println!("✅ Created user 'alice' without CreateUser permission");

    // Step 2: Try to create another user as alice (who doesn't have CreateUser permission)
    println!("\nStep 2: Testing permission check directly...");
    
    // Get the UserCreateOperation
    use moor_vcs_worker::operations::Operation;
    let user_create_op = moor_vcs_worker::operations::UserCreateOperation::new(
        server.database().users().clone()
    );
    
    // Execute the operation as alice (who doesn't have CreateUser permission)
    let result = user_create_op.execute(
        vec![
            "bob".to_string(),
            "bob@example.com".to_string(),
            "102".to_string(),
        ],
        &alice,
    );
    
    // Check that the result is an error about permissions
    let result_str = if let Some(s) = result.as_string() {
        s.to_string()
    } else if let Some(e) = result.as_error() {
        e.msg.as_ref().map(|s| s.to_string()).unwrap_or_else(|| format!("{:?}", e))
    } else {
        format!("{:?}", result)
    };
    
    assert!(
        result_str.contains("Error") || result_str.contains("permission"),
        "Expected permission error, got: {}",
        result_str
    );
    
    // Verify bob was not created
    let bob = server.database().users().get_user("bob").expect("Failed to query user");
    assert!(bob.is_none(), "User 'bob' should not have been created");

    println!("✅ Test passed: Create user correctly requires permission");
}

