use super::*;
use crate::git::channel::GitChannel;
use std::thread;

#[test]
fn test_git_channel_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    
    let result = GitChannel::new_with_config(config);
    assert!(result.is_ok());
    
    let channel = result.unwrap();
    
    // Test that we can call methods on the channel
    let result = channel.file_exists("nonexistent.txt".into());
    assert!(!result);
}

#[test]
fn test_git_channel_file_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Test write_file
    let result = channel.write_file("test.txt".into(), "Hello, world!".to_string());
    assert!(result.is_ok());
    
    // Test read_file
    let result = channel.read_file("test.txt".into());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, world!");
    
    // Test file_exists
    let exists = channel.file_exists("test.txt".into());
    assert!(exists);
    
    let not_exists = channel.file_exists("nonexistent.txt".into());
    assert!(!not_exists);
}

#[test]
fn test_git_channel_add_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create a file
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    
    // Add file to git
    let result = channel.add_file("test.txt".into());
    assert!(result.is_ok());
}

#[test]
fn test_git_channel_remove_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create and add a file
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    channel.add_file("test.txt".into()).unwrap();
    
    // Remove file from git
    let result = channel.remove_file("test.txt".into());
    assert!(result.is_ok());
    
    // Verify file was removed
    let exists = channel.file_exists("test.txt".into());
    assert!(!exists);
}

#[test]
fn test_git_channel_rename_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create and add a file
    channel.write_file("old.txt".into(), "Hello, world!".to_string()).unwrap();
    channel.add_file("old.txt".into()).unwrap();
    
    // Rename file
    let result = channel.rename_file("old.txt".into(), "new.txt".into());
    assert!(result.is_ok());
    
    // Verify old file doesn't exist
    let old_exists = channel.file_exists("old.txt".into());
    assert!(!old_exists);
    
    // Verify new file exists with correct content
    let new_exists = channel.file_exists("new.txt".into());
    assert!(new_exists);
    
    let content = channel.read_file("new.txt".into()).unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_git_channel_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create and add a file
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    channel.add_file("test.txt".into()).unwrap();
    
    // Create commit
    let result = channel.commit(
        "Test commit".to_string(),
        "test-user".to_string(),
        "test@example.com".to_string(),
    );
    assert!(result.is_ok());
}

#[test]
fn test_git_channel_get_commits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create multiple commits
    for i in 1..=3 {
        channel.write_file(format!("file{}.txt", i).into(), format!("Content {}", i)).unwrap();
        channel.add_file(format!("file{}.txt", i).into()).unwrap();
        channel.commit(
            format!("Commit {}", i),
            "test-user".to_string(),
            "test@example.com".to_string(),
        ).unwrap();
    }
    
    // Get commits
    let result = channel.get_commits(None, None);
    assert!(result.is_ok());
    
    let commits = result.unwrap();
    assert_eq!(commits.len(), 3);
    
    // Verify commit order (most recent first)
    assert_eq!(commits[0].message, "Commit 3");
    assert_eq!(commits[1].message, "Commit 2");
    assert_eq!(commits[2].message, "Commit 1");
}

#[test]
fn test_git_channel_get_last_commit_info() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Initially, there should be no commit info
    let result = channel.get_last_commit_info();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create a commit
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    channel.add_file("test.txt".into()).unwrap();
    channel.commit(
        "Test commit".to_string(),
        "test-user".to_string(),
        "test@example.com".to_string(),
    ).unwrap();
    
    // Now there should be commit info
    let result = channel.get_last_commit_info();
    assert!(result.is_ok());
    
    let commit_info = result.unwrap().unwrap();
    assert_eq!(commit_info.message, "Test commit");
    assert_eq!(commit_info.author, "test-user");
    assert!(!commit_info.id.is_empty());
    assert!(!commit_info.full_id.is_empty());
}

#[test]
fn test_git_channel_has_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Initially, there should be no changes
    let result = channel.has_changes();
    assert!(result.is_ok());
    assert!(!result.unwrap());
    
    // Create an untracked file
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    
    // Should detect changes
    let result = channel.has_changes();
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_git_channel_get_status() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create an untracked file
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    
    let result = channel.get_status();
    assert!(result.is_ok());
    
    let status = result.unwrap();
    assert_eq!(status.len(), 1);
    assert!(status[0].contains("Added: test.txt"));
}

#[test]
fn test_git_channel_get_current_branch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Initially, there should be no current branch
    let result = channel.get_current_branch();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create initial commit to establish a branch
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    channel.add_file("test.txt".into()).unwrap();
    channel.commit(
        "Initial commit".to_string(),
        "test-user".to_string(),
        "test@example.com".to_string(),
    ).unwrap();
    
    // Now there should be a current branch
    let result = channel.get_current_branch();
    assert!(result.is_ok());
    
    let branch = result.unwrap();
    assert!(branch.is_some());
    assert_eq!(branch.unwrap(), "main");
}

#[test]
fn test_git_channel_get_upstream_info() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Initially, there should be no upstream
    let result = channel.get_upstream_info();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_git_channel_configure_git_user() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Configure git user
    let result = channel.configure_git_user();
    assert!(result.is_ok());
}

#[test]
fn test_git_channel_reset_working_tree() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create and commit a file
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    channel.add_file("test.txt".into()).unwrap();
    channel.commit(
        "Initial commit".to_string(),
        "test-user".to_string(),
        "test@example.com".to_string(),
    ).unwrap();
    
    // Modify the file
    channel.write_file("test.txt".into(), "Hello, modified world!".to_string()).unwrap();
    
    // Reset working tree
    let result = channel.reset_working_tree();
    assert!(result.is_ok());
    
    // Verify file was reset to committed state
    let content = channel.read_file("test.txt".into()).unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_git_channel_rollback_last_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create initial commit
    channel.write_file("test.txt".into(), "Hello, world!".to_string()).unwrap();
    channel.add_file("test.txt".into()).unwrap();
    channel.commit(
        "Initial commit".to_string(),
        "test-user".to_string(),
        "test@example.com".to_string(),
    ).unwrap();
    
    // Create second commit
    channel.write_file("test2.txt".into(), "Hello, world 2!".to_string()).unwrap();
    channel.add_file("test2.txt".into()).unwrap();
    channel.commit(
        "Second commit".to_string(),
        "test-user".to_string(),
        "test@example.com".to_string(),
    ).unwrap();
    
    // Rollback the last commit
    let result = channel.rollback_last_commit();
    assert!(result.is_ok());
    
    // Verify we're back to the first commit
    let commits = channel.get_commits(Some(1), None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].message, "Initial commit");
    
    // Verify the second file is still in the working directory but not committed
    assert!(channel.file_exists("test2.txt".into()));
}

#[test]
fn test_git_channel_add_all_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Create multiple files
    channel.write_file("file1.txt".into(), "Content 1".to_string()).unwrap();
    channel.write_file("file2.txt".into(), "Content 2".to_string()).unwrap();
    
    // Add all changes
    let result = channel.add_all_changes();
    assert!(result.is_ok());
}

#[test]
fn test_git_channel_shutdown() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Test that we can call methods before shutdown
    let result = channel.file_exists("nonexistent.txt".into());
    assert!(!result);
    
    // Shutdown the channel
    let result = channel.shutdown();
    assert!(result.is_ok());
}

#[test]
fn test_git_channel_concurrent_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Test concurrent file operations
    let channel_clone = std::sync::Arc::new(channel);
    let mut handles = vec![];
    
    for i in 0..5 {
        let channel_clone = channel_clone.clone();
        let handle = thread::spawn(move || {
            let filename = format!("file{}.txt", i);
            let content = format!("Content {}", i);
            
            // Write file
            channel_clone.write_file(filename.clone().into(), content.clone()).unwrap();
            
            // Read file
            let read_content = channel_clone.read_file(filename.clone().into()).unwrap();
            assert_eq!(read_content, content);
            
            // Check file exists
            assert!(channel_clone.file_exists(filename.clone().into()));
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify all files exist
    for i in 0..5 {
        let filename = format!("file{}.txt", i);
        assert!(channel_clone.file_exists(filename.into()));
    }
}

#[test]
fn test_git_channel_error_handling() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let channel = GitChannel::new_with_config(config).unwrap();
    
    // Test reading non-existent file
    let result = channel.read_file("nonexistent.txt".into());
    assert!(result.is_err());
    
    // Test removing non-existent file
    let result = channel.remove_file("nonexistent.txt".into());
    assert!(result.is_ok()); // Should succeed even if file doesn't exist
    
    // Test renaming non-existent file
    let result = channel.rename_file("nonexistent.txt".into(), "new.txt".into());
    assert!(result.is_err());
}
