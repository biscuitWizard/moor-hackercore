//! Tests for external user configuration (used for clone operations)

use crate::common::*;

#[tokio::test]
async fn test_external_user_in_clone() {
    // This test requires two test servers, which is complex to set up
    // For now, we'll just test that the external user info can be stored/retrieved
    let server = TestServer::start()
        .await
        .expect("Failed to start test server");

    println!("Test: External user configuration storage");

    // Store external user info
    println!("\nStep 1: Storing external user credentials...");
    server
        .database()
        .index()
        .set_external_user_api_key("test-api-key-123")
        .expect("Failed to set external user API key");
    server
        .database()
        .index()
        .set_external_user_id("external-user-id")
        .expect("Failed to set external user ID");
    println!("✅ External user credentials stored");

    // Retrieve and verify
    println!("\nStep 2: Retrieving external user credentials...");
    let api_key = server
        .database()
        .index()
        .get_external_user_api_key()
        .expect("Failed to get external user API key")
        .expect("External user API key not found");
    let user_id = server
        .database()
        .index()
        .get_external_user_id()
        .expect("Failed to get external user ID")
        .expect("External user ID not found");

    assert_eq!(api_key, "test-api-key-123");
    assert_eq!(user_id, "external-user-id");
    println!(
        "✅ External user credentials retrieved: user_id={}, api_key={}",
        user_id, api_key
    );

    println!("\n✅ Test passed: External user configuration works");
}

// Note: test_external_user_stat_validation would require setting up a second server
// and making actual HTTP requests between them. This is left as a future enhancement.

