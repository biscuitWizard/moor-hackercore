use tempfile::TempDir;
use std::fs;
use std::thread;
use std::time::Duration;

mod common;
use common::*;

/// Helper to set up a git repository for testing
fn setup_git_repo() -> TempDir {
    let git_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Initialize git repo
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

/// Helper to configure git backup via environment variables
fn set_git_backup_env(repo_path: &str, token: Option<&str>) {
    unsafe {
        std::env::set_var("VCS_GIT_BACKUP_REPO", repo_path);
        if let Some(t) = token {
            std::env::set_var("VCS_GIT_BACKUP_TOKEN", t);
        } else {
            std::env::remove_var("VCS_GIT_BACKUP_TOKEN");
        }
    }
}

/// Helper to clean up git backup env vars
fn clear_git_backup_env() {
    unsafe {
        std::env::remove_var("VCS_GIT_BACKUP_REPO");
        std::env::remove_var("VCS_GIT_BACKUP_TOKEN");
    }
}

#[test]
fn test_config_git_backup_fields() {
    let temp_db = TempDir::new().unwrap();
    
    // Test with git backup configured
    unsafe {
        std::env::set_var("VCS_DB_PATH", temp_db.path().to_str().unwrap());
        std::env::set_var("VCS_GIT_BACKUP_REPO", "https://github.com/test/repo.git");
        std::env::set_var("VCS_GIT_BACKUP_TOKEN", "test_token_123");
    }
    
    let config = moor_vcs_worker::Config::new();
    
    assert_eq!(
        config.git_backup_repo,
        Some("https://github.com/test/repo.git".to_string())
    );
    assert_eq!(config.git_backup_token, Some("test_token_123".to_string()));
    
    // Test with git backup not configured
    unsafe {
        std::env::remove_var("VCS_GIT_BACKUP_REPO");
        std::env::remove_var("VCS_GIT_BACKUP_TOKEN");
    }
    
    let config2 = moor_vcs_worker::Config::new();
    
    assert_eq!(config2.git_backup_repo, None);
    assert_eq!(config2.git_backup_token, None);
    
    // Clean up
    clear_git_backup_env();
}

#[tokio::test]
async fn test_git_backup_with_local_repo() {
    let git_dir = setup_git_repo();
    
    // Set up git backup environment
    set_git_backup_env(git_dir.path().to_str().unwrap(), None);
    
    let server = TestServer::start().await.expect("Failed to start server");
    let client = server.client();
    
    // Create a test object
    let objdef = vec![
        "object #1",
        "  name: \"Test Object\"",
        "  parent: #0",
        "  owner: #1",
        "  location: #0",
        "",
        "  property test_prop (owner: #1, flags: \"rw\") = \"test value\";",
        "",
        "  verb test (this none this) owner: #1 flags: \"rx\"",
        "    return \"hello\";",
        "  endverb",
        "endobject",
    ].join("\n");
    
    let response = client
        .rpc_call("object/update", vec![
            serde_json::Value::String("#1".to_string()),
            serde_json::Value::String(objdef),
        ])
        .await
        .expect("Failed to update object");
    
    response.assert_success("object/update");
    
    // Submit the change (which triggers git backup)
    let submit_response = client
        .rpc_call("change/submit", vec![])
        .await
        .expect("Failed to submit change");
    
    submit_response.assert_success("change/submit");
    
    // Wait for the background thread to complete
    thread::sleep(Duration::from_secs(3));
    
    // Check that the .moo file was created in the git repo (filename is sanitized object name)
    let expected_file = git_dir.path().join("#1.moo");
    assert!(
        expected_file.exists(),
        "Expected git backup file to exist at {:?}",
        expected_file
    );
    
    // Check file content
    let content = fs::read_to_string(&expected_file).unwrap();
    assert!(content.contains("object #1"));
    assert!(content.contains("Test Object"));
    assert!(content.contains("test_prop"));
    
    // Check that a git commit was made
    let log_output = std::process::Command::new("git")
        .current_dir(git_dir.path())
        .args(&["log", "--oneline"])
        .output()
        .unwrap();
    
    let log = String::from_utf8_lossy(&log_output.stdout);
    assert!(log.contains("VCS backup:"), "Expected commit message not found in git log");
    
    // Clean up
    clear_git_backup_env();
}

#[tokio::test]
async fn test_git_backup_with_meta_filtering() {
    let git_dir = setup_git_repo();
    set_git_backup_env(git_dir.path().to_str().unwrap(), None);
    
    let server = TestServer::start().await.expect("Failed to start server");
    let client = server.client();
    
    // Create an object
    let objdef = vec![
        "object #2",
        "  name: \"Test Object\"",
        "  parent: #0",
        "  owner: #1",
        "  location: #0",
        "",
        "  property visible_prop (owner: #1, flags: \"rw\") = \"visible\";",
        "  property ignored_prop (owner: #1, flags: \"rw\") = \"ignored\";",
        "",
        "  verb visible_verb (this none this) owner: #1 flags: \"rx\"",
        "    return \"visible\";",
        "  endverb",
        "",
        "  verb ignored_verb (this none this) owner: #1 flags: \"rx\"",
        "    return \"ignored\";",
        "  endverb",
        "endobject",
    ].join("\n");
    
    client
        .rpc_call("object/update", vec![
            serde_json::Value::String("#2".to_string()),
            serde_json::Value::String(objdef),
        ])
        .await
        .expect("Failed to update object");
    
    // Add meta filtering
    client
        .rpc_call("meta/add_ignored_property", vec![
            serde_json::Value::String("#2".to_string()),
            serde_json::Value::String("ignored_prop".to_string()),
        ])
        .await
        .expect("Failed to add ignored property");
    
    client
        .rpc_call("meta/add_ignored_verb", vec![
            serde_json::Value::String("#2".to_string()),
            serde_json::Value::String("ignored_verb".to_string()),
        ])
        .await
        .expect("Failed to add ignored verb");
    
    // Submit the change (triggers backup)
    client
        .rpc_call("change/submit", vec![])
        .await
        .expect("Failed to submit change");
    
    // Wait for backup to complete
    thread::sleep(Duration::from_secs(3));
    
    // Check that the file was created with meta filtering applied
    let expected_file = git_dir.path().join("#2.moo");
    assert!(expected_file.exists());
    
    let content = fs::read_to_string(&expected_file).unwrap();
    
    // Visible items should be present
    assert!(content.contains("visible_prop"));
    assert!(content.contains("visible_verb"));
    
    // Ignored items should NOT be present
    assert!(!content.contains("ignored_prop"));
    assert!(!content.contains("ignored_verb"));
    
    clear_git_backup_env();
}

#[tokio::test]
async fn test_git_backup_cleanup_old_files() {
    let git_dir = setup_git_repo();
    
    // Create a stale .moo file that should be cleaned up
    let stale_file = git_dir.path().join("stale_object.moo");
    fs::write(&stale_file, "object #999\nendobject").unwrap();
    
    set_git_backup_env(git_dir.path().to_str().unwrap(), None);
    
    let server = TestServer::start().await.expect("Failed to start server");
    let client = server.client();
    
    // Create a real object
    let objdef = vec![
        "object #3",
        "  name: \"Real Object\"",
        "  parent: #0",
        "  owner: #1",
        "  location: #0",
        "endobject",
    ].join("\n");
    
    client
        .rpc_call("object/update", vec![
            serde_json::Value::String("#3".to_string()),
            serde_json::Value::String(objdef),
        ])
        .await
        .expect("Failed to update object");
    
    // Submit (triggers backup and cleanup)
    client
        .rpc_call("change/submit", vec![])
        .await
        .expect("Failed to submit change");
    
    // Wait for backup
    thread::sleep(Duration::from_secs(3));
    
    // Check that the real file exists
    let real_file = git_dir.path().join("#3.moo");
    assert!(real_file.exists());
    
    // Check that the stale file was removed
    assert!(!stale_file.exists(), "Stale file should have been removed");
    
    clear_git_backup_env();
}

#[tokio::test]
async fn test_git_backup_disabled_by_default() {
    // Ensure git backup env vars are not set
    clear_git_backup_env();
    
    let server = TestServer::start().await.expect("Failed to start server");
    let client = server.client();
    
    // Create and submit a change
    let objdef = vec![
        "object #5",
        "  name: \"Test\"",
        "  parent: #0",
        "  owner: #1",
        "  location: #0",
        "endobject",
    ].join("\n");
    
    client
        .rpc_call("object/update", vec![
            serde_json::Value::String("#5".to_string()),
            serde_json::Value::String(objdef),
        ])
        .await
        .expect("Failed to update object");
    
    let result = client
        .rpc_call("change/submit", vec![])
        .await
        .expect("Failed to submit change");
    
    // Should succeed even without git backup configured
    result.assert_success("change/submit");
}

#[tokio::test]
async fn test_git_backup_with_special_characters_in_object_name() {
    let git_dir = setup_git_repo();
    set_git_backup_env(git_dir.path().to_str().unwrap(), None);
    
    let server = TestServer::start().await.expect("Failed to start server");
    let client = server.client();
    
    // Create an object with special characters in the name
    let objdef = vec![
        "object #100",
        "  name: \"Player Object\"",
        "  parent: #0",
        "  owner: #1",
        "  location: #0",
        "endobject",
    ].join("\n");
    
    let update_response = client
        .rpc_call("object/update", vec![
            serde_json::Value::String("$player".to_string()),
            serde_json::Value::String(objdef),
        ])
        .await
        .expect("Failed to update object");
    
    update_response.assert_success("object/update");
    
    client
        .rpc_call("change/submit", vec![])
        .await
        .expect("Failed to submit change");
    
    // Wait for backup
    thread::sleep(Duration::from_secs(3));
    
    // Check that a file was created with sanitized name
    let expected_file = git_dir.path().join("player.moo");
    assert!(
        expected_file.exists(),
        "Expected sanitized filename 'player.moo' to exist"
    );
    
    clear_git_backup_env();
}

#[test]
fn test_filename_sanitization() {
    use moor_vcs_worker::git_backup::sanitize_filename;
    
    assert_eq!(sanitize_filename("simple"), "simple");
    assert_eq!(sanitize_filename("$player"), "player");
    assert_eq!(sanitize_filename("obj/with/slashes"), "obj_with_slashes");
    assert_eq!(sanitize_filename("obj*with*stars"), "obj_with_stars");
    assert_eq!(sanitize_filename("$room:utilities"), "room_utilities");
}

#[test]
fn test_token_injection() {
    use moor_vcs_worker::git_backup::inject_token_into_url;
    
    assert_eq!(
        inject_token_into_url("https://github.com/user/repo.git", "mytoken"),
        "https://mytoken@github.com/user/repo.git"
    );
    
    assert_eq!(
        inject_token_into_url("http://example.com/repo.git", "token123"),
        "http://token123@example.com/repo.git"
    );
    
    // Non-HTTP URLs remain unchanged
    assert_eq!(
        inject_token_into_url("git@github.com:user/repo.git", "token"),
        "git@github.com:user/repo.git"
    );
}
