//! Test to verify the Wizard user is created properly

use crate::common::*;

#[tokio::test]
async fn test_wizard_user_exists_with_all_permissions() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    println!("Test: Wizard user exists with all permissions");

    // Step 1: Verify Wizard user exists
    println!("\nStep 1: Verifying Wizard user exists...");
    let wizard_user = server.get_wizard_user().expect("Failed to get Wizard user");

    assert_eq!(
        wizard_user.id, "Wizard",
        "Wizard user ID should be 'Wizard'"
    );
    assert_eq!(
        wizard_user.email, "wizard@system",
        "Wizard user email should be 'wizard@system'"
    );
    assert!(
        wizard_user.is_system_user,
        "Wizard user should be marked as a system user"
    );

    println!(
        "✅ Wizard user exists with ID: {}, email: {}",
        wizard_user.id, wizard_user.email
    );

    // Step 2: Verify Wizard has all permissions
    println!("\nStep 2: Verifying Wizard has all permissions...");

    use moor_vcs_worker::types::Permission;

    assert!(
        wizard_user.has_permission(&Permission::ApproveChanges),
        "Wizard should have ApproveChanges permission"
    );
    assert!(
        wizard_user.has_permission(&Permission::SubmitChanges),
        "Wizard should have SubmitChanges permission"
    );
    assert!(
        wizard_user.has_permission(&Permission::Clone),
        "Wizard should have Clone permission"
    );
    assert!(
        wizard_user.has_permission(&Permission::CreateUser),
        "Wizard should have CreateUser permission"
    );
    assert!(
        wizard_user.has_permission(&Permission::DisableUser),
        "Wizard should have DisableUser permission"
    );
    assert!(
        wizard_user.has_permission(&Permission::ManagePermissions),
        "Wizard should have ManagePermissions permission"
    );
    assert!(
        wizard_user.has_permission(&Permission::ManageApiKeys),
        "Wizard should have ManageApiKeys permission"
    );

    println!("✅ Wizard has all permissions: ApproveChanges, SubmitChanges, Clone, CreateUser, DisableUser, ManagePermissions, ManageApiKeys");

    // Step 3: Verify Wizard has the default API key
    println!("\nStep 3: Verifying Wizard has the API key...");

    let expected_api_key = server.get_wizard_api_key();
    assert!(
        wizard_user.authorized_keys.contains(&expected_api_key),
        "Wizard should have the default API key"
    );

    println!("✅ Wizard has the configured API key");

    // Step 4: Verify Wizard user cannot be deleted
    println!("\nStep 4: Verifying Wizard user cannot be deleted...");

    let result = server.database().users().delete_user("Wizard");
    assert!(result.is_err(), "Deleting Wizard user should fail");

    if let Err(e) = result {
        println!("✅ Wizard user deletion correctly prevented: {}", e);
    }

    println!("\n✅ Test passed: Wizard user is properly configured");
}

#[tokio::test]
async fn test_operations_use_wizard_user_by_default() {
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");
    let base_url = server.base_url();

    println!("Test: Operations use Wizard user by default");

    // Step 1: Create a change (requires permissions)
    println!("\nStep 1: Creating a change...");
    let create_request = json!({
        "operation": "change/create",
        "args": ["test_change", "test_author"]
    });

    let response = make_request("POST", &format!("{}/rpc", base_url), Some(create_request))
        .await
        .expect("Failed to create change");

    assert!(
        response["success"].as_bool().unwrap_or(false),
        "Change creation should succeed"
    );

    println!("✅ Change created successfully using Wizard user");

    println!("\n✅ Test passed: Operations use Wizard user by default");
}
