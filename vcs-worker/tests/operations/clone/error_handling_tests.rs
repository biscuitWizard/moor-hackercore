//! Tests for clone error handling and edge cases

use crate::common::*;

#[tokio::test]
async fn test_clone_import_with_invalid_url_format() {
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");
    let target_client = target_server.client();

    println!("Test: Clone import with invalid URL format should fail gracefully");

    let invalid_urls = vec![
        "not-a-url",
        "ftp://invalid-protocol.com",
        "http://",
        "",
        "   ",
    ];

    for invalid_url in invalid_urls {
        println!("\nTesting invalid URL: '{}'", invalid_url);
        let response = target_client
            .clone_import(invalid_url)
            .await
            .expect("Request should complete");

        // Should fail with error
        let result_str = response.get_result_str().unwrap_or("");
        assert!(
            result_str.contains("Error")
                || result_str.contains("failed")
                || result_str.contains("invalid"),
            "Should indicate error for '{}', got: {}",
            invalid_url,
            result_str
        );
        println!("✅ Failed appropriately: {}", result_str);
    }

    println!("\n✅ Test passed: Clone import handles invalid URLs gracefully");
}

#[tokio::test]
async fn test_clone_import_with_unreachable_url() {
    let target_server = TestServer::start()
        .await
        .expect("Failed to start target server");
    let target_client = target_server.client();

    println!("Test: Clone import with unreachable URL should handle network errors gracefully");

    // Use a URL that should be unreachable
    let unreachable_url = "http://localhost:99999/api/clone";

    println!(
        "\nAttempting to clone from unreachable URL: {}",
        unreachable_url
    );
    let response = target_client
        .clone_import(unreachable_url)
        .await
        .expect("Request should complete");

    // Should fail with network error
    let result_str = response.get_result_str().unwrap_or("");
    assert!(
        result_str.contains("Error")
            || result_str.contains("failed")
            || result_str.contains("connection"),
        "Should indicate network error, got: {}",
        result_str
    );
    println!("✅ Network error handled gracefully: {}", result_str);

    println!("\n✅ Test passed: Clone handles unreachable URLs gracefully");
}

