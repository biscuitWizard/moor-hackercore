use super::*;
use crate::git::operations::remote_ops::RemoteOps;
use git2::Repository;

#[test]
fn test_get_current_branch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    
    // Initially, there should be no current branch (unborn)
    let result = RemoteOps::get_current_branch(&repo);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create initial commit to establish a branch
    let work_dir = temp_dir.path();
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Now there should be a current branch
    let result = RemoteOps::get_current_branch(&repo);
    assert!(result.is_ok());
    
    let branch = result.unwrap();
    assert!(branch.is_some());
    assert_eq!(branch.unwrap(), "main");
}

#[test]
fn test_get_upstream_info() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    
    // Initially, there should be no upstream
    let result = RemoteOps::get_upstream_info(&repo);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create initial commit to establish a branch
    let work_dir = temp_dir.path();
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Still no upstream since we haven't set one
    let result = RemoteOps::get_upstream_info(&repo);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_push_no_remote() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    
    // Try to push without a remote configured
    let result = RemoteOps::push(&repo, None, &keys_dir);
    assert!(result.is_err());
}

#[test]
fn test_fetch_remote_no_remote() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    
    // Try to fetch without a remote configured
    let result = RemoteOps::fetch_remote(&repo, None, &keys_dir);
    assert!(result.is_err());
}

#[test]
fn test_test_ssh_connection_no_remote() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    
    // Try to test SSH connection without a remote configured
    let result = RemoteOps::test_ssh_connection(&repo, None, &keys_dir);
    assert!(result.is_err());
}

#[test]
fn test_push_with_remote() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    
    // Add a remote (this will fail to connect, but we can test the function)
    let remote_url = "git@example.com:test/repo.git";
    repo.remote("origin", remote_url).unwrap();
    
    // Create initial commit
    let work_dir = temp_dir.path();
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Try to push (will fail due to network, but function should be called)
    let result = RemoteOps::push(&repo, None, &keys_dir);
    // This will fail due to network connectivity, but that's expected in tests
    assert!(result.is_err());
}

#[test]
fn test_fetch_remote_with_remote() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    
    // Add a remote
    let remote_url = "git@example.com:test/repo.git";
    repo.remote("origin", remote_url).unwrap();
    
    // Try to fetch (will fail due to network, but function should be called)
    let result = RemoteOps::fetch_remote(&repo, None, &keys_dir);
    // This will fail due to network connectivity, but that's expected in tests
    assert!(result.is_err());
}

#[test]
fn test_test_ssh_connection_with_remote() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    
    // Add a remote
    let remote_url = "git@example.com:test/repo.git";
    repo.remote("origin", remote_url).unwrap();
    
    // Try to test SSH connection (will fail due to network, but function should be called)
    let result = RemoteOps::test_ssh_connection(&repo, None, &keys_dir);
    // This will fail due to network connectivity, but that's expected in tests
    assert!(result.is_err());
}

#[test]
fn test_push_with_ssh_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    std::fs::create_dir_all(&keys_dir).unwrap();
    
    // Create a test SSH key
    let ssh_key_path = keys_dir.join("id_rsa");
    create_test_ssh_key(&ssh_key_path).unwrap();
    
    // Add a remote
    let remote_url = "git@example.com:test/repo.git";
    repo.remote("origin", remote_url).unwrap();
    
    // Create initial commit
    let work_dir = temp_dir.path();
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Try to push with SSH key (will fail due to network, but function should be called)
    let result = RemoteOps::push(&repo, Some(&ssh_key_path.to_string_lossy()), &keys_dir);
    // This will fail due to network connectivity, but that's expected in tests
    assert!(result.is_err());
}

#[test]
fn test_get_current_branch_after_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    
    // Create initial commit
    let work_dir = temp_dir.path();
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Now there should be a current branch
    let result = RemoteOps::get_current_branch(&repo);
    assert!(result.is_ok());
    
    let branch = result.unwrap();
    assert!(branch.is_some());
    assert_eq!(branch.unwrap(), "main");
}

#[test]
fn test_get_current_branch_detached_head() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    
    // Create initial commit
    let work_dir = temp_dir.path();
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    let commit = crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Detach HEAD
    repo.set_head_detached(commit.id()).unwrap();
    
    // Now there should be no current branch (detached HEAD)
    let result = RemoteOps::get_current_branch(&repo);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}
