/// Test to demonstrate multi-instance isolation
/// 
/// This test verifies that multiple VCS worker instances can run simultaneously
/// without conflicts, even when both use git backup features.

use tempfile::TempDir;
use std::thread;
use std::time::Duration;

mod common;
use common::*;

/// Helper to set up a git repository for testing
fn setup_git_repo() -> TempDir {
    let git_dir = TempDir::new().expect("Failed to create temp dir");
    
    std::process::Command::new("git")
        .current_dir(git_dir.path())
        .args(&["init"])
        .output()
        .expect("Failed to init git repo");
    
    std::process::Command::new("git")
        .current_dir(git_dir.path())
        .args(&["config", "user.email", "test@test.com"])
        .output()
        .expect("Failed to set git user email");
    
    std::process::Command::new("git")
        .current_dir(git_dir.path())
        .args(&["config", "user.name", "Test User"])
        .output()
        .expect("Failed to set git user name");
    
    git_dir
}

#[tokio::test]
async fn test_two_instances_with_git_backup_isolated() {
    println!("Test: Two VCS worker instances with git backup should run simultaneously without conflicts");
    
    // Set up git repos for both instances
    let git_dir_1 = setup_git_repo();
    let git_dir_2 = setup_git_repo();
    let temp_db_1 = TempDir::new().unwrap();
    let temp_db_2 = TempDir::new().unwrap();
    
    println!("\n✅ Created isolated git repositories and databases");
    
    // Create first server with git backup
    let config_1 = moor_vcs_worker::Config::with_db_path(temp_db_1.path().to_path_buf())
        .with_git_backup(git_dir_1.path().to_str().unwrap().to_string(), None);
    
    let server_1 = TestServer::start_with_config(config_1)
        .await
        .expect("Failed to start server 1");
    let client_1 = server_1.client();
    
    println!("✅ Server 1 started on port {}", server_1.base_url());
    println!("   - DB path: {:?}", temp_db_1.path());
    println!("   - Git path: {:?}", git_dir_1.path());
    println!("   - Git work dir: {:?}", server_1.git_work_dir());
    
    // Create second server with git backup
    let config_2 = moor_vcs_worker::Config::with_db_path(temp_db_2.path().to_path_buf())
        .with_git_backup(git_dir_2.path().to_str().unwrap().to_string(), None);
    
    let server_2 = TestServer::start_with_config(config_2)
        .await
        .expect("Failed to start server 2");
    let client_2 = server_2.client();
    
    println!("✅ Server 2 started on port {}", server_2.base_url());
    println!("   - DB path: {:?}", temp_db_2.path());
    println!("   - Git path: {:?}", git_dir_2.path());
    println!("   - Git work dir: {:?}", server_2.git_work_dir());
    
    // Verify git work dirs are different
    assert_ne!(
        server_1.git_work_dir().unwrap(),
        server_2.git_work_dir().unwrap(),
        "Git work directories should be different"
    );
    println!("\n✅ Verified git work directories are isolated");
    
    // Create objects in both servers simultaneously
    println!("\n📝 Creating objects in both servers...");
    
    let objdef_1 = vec![
        "object #1",
        "  name: \"Server 1 Object\"",
        "  parent: #0",
        "  owner: #1",
        "  location: #0",
        "endobject",
    ].join("\n");
    
    let objdef_2 = vec![
        "object #2",
        "  name: \"Server 2 Object\"",
        "  parent: #0",
        "  owner: #1",
        "  location: #0",
        "endobject",
    ].join("\n");
    
    // Update objects concurrently
    let response_1 = client_1
        .rpc_call("object/update", vec![
            serde_json::Value::String("#1".to_string()),
            serde_json::Value::String(objdef_1),
        ])
        .await
        .expect("Failed to update object on server 1");
    response_1.assert_success("Server 1 object/update");
    
    let response_2 = client_2
        .rpc_call("object/update", vec![
            serde_json::Value::String("#2".to_string()),
            serde_json::Value::String(objdef_2),
        ])
        .await
        .expect("Failed to update object on server 2");
    response_2.assert_success("Server 2 object/update");
    
    println!("✅ Objects created on both servers");
    
    // Submit changes (triggers git backup on both)
    println!("\n📤 Submitting changes (triggers git backup)...");
    
    let submit_1 = client_1
        .rpc_call("change/submit", vec![])
        .await
        .expect("Failed to submit on server 1");
    submit_1.assert_success("Server 1 submit");
    
    let submit_2 = client_2
        .rpc_call("change/submit", vec![])
        .await
        .expect("Failed to submit on server 2");
    submit_2.assert_success("Server 2 submit");
    
    println!("✅ Changes submitted on both servers");
    
    // Wait for git backups to complete
    thread::sleep(Duration::from_secs(4));
    
    // Verify both git repos have their respective files
    println!("\n🔍 Verifying git backup files...");
    
    let file_1 = git_dir_1.path().join("#1.moo");
    let file_2 = git_dir_2.path().join("#2.moo");
    
    assert!(file_1.exists(), "Server 1 git backup file should exist");
    assert!(file_2.exists(), "Server 2 git backup file should exist");
    
    // Verify content
    let content_1 = std::fs::read_to_string(&file_1).unwrap();
    let content_2 = std::fs::read_to_string(&file_2).unwrap();
    
    assert!(content_1.contains("Server 1 Object"), "File 1 should contain Server 1 Object");
    assert!(content_2.contains("Server 2 Object"), "File 2 should contain Server 2 Object");
    
    // Verify they don't have each other's files
    let wrong_file_1 = git_dir_1.path().join("#2.moo");
    let wrong_file_2 = git_dir_2.path().join("#1.moo");
    
    assert!(!wrong_file_1.exists(), "Server 1 should not have Server 2's file");
    assert!(!wrong_file_2.exists(), "Server 2 should not have Server 1's file");
    
    println!("✅ Git backup files verified - complete isolation");
    
    println!("\n✅ Test passed: Two instances with git backup ran simultaneously without conflicts");
}

#[tokio::test]
async fn test_three_instances_without_git_backup() {
    println!("Test: Three VCS worker instances should run simultaneously without git backup");
    
    // Create three servers without git backup
    let server_1 = TestServer::start().await.expect("Failed to start server 1");
    let server_2 = TestServer::start().await.expect("Failed to start server 2");
    let server_3 = TestServer::start().await.expect("Failed to start server 3");
    
    println!("✅ Server 1: {}", server_1.base_url());
    println!("✅ Server 2: {}", server_2.base_url());
    println!("✅ Server 3: {}", server_3.base_url());
    
    let client_1 = server_1.client();
    let client_2 = server_2.client();
    let client_3 = server_3.client();
    
    // Create objects on all three servers
    for (i, client) in [client_1, client_2, client_3].iter().enumerate() {
        let objdef = format!(
            "object #{}\n  name: \"Object {}\"\n  parent: #0\n  owner: #1\n  location: #0\nendobject",
            i + 1, i + 1
        );
        
        let response = client
            .rpc_call("object/update", vec![
                serde_json::Value::String(format!("#{}", i + 1)),
                serde_json::Value::String(objdef),
            ])
            .await
            .expect("Failed to update object");
        response.assert_success(&format!("Server {} object/update", i + 1));
    }
    
    println!("✅ All three servers processed objects independently");
    println!("✅ Test passed: Three instances ran without conflicts");
}

