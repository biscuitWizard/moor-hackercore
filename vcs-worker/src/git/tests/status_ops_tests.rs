use super::*;
use crate::git::operations::status_ops::StatusOps;
use git2::Repository;

#[test]
fn test_has_changes_no_commits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    
    // Initially, there should be no changes (no commits yet)
    let result = StatusOps::has_changes(&repo);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_has_changes_with_untracked_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create an untracked file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    
    // Should detect changes (untracked files)
    let result = StatusOps::has_changes(&repo);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_has_changes_after_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Should have no changes after commit
    let result = StatusOps::has_changes(&repo);
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[test]
fn test_has_changes_after_modification() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Modify the file
    std::fs::write(&test_file, "Hello, modified world!").unwrap();
    
    // Should detect changes
    let result = StatusOps::has_changes(&repo);
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_get_status_no_commits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create an untracked file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    
    let result = StatusOps::get_status(&repo, work_dir);
    assert!(result.is_ok());
    
    let status = result.unwrap();
    assert_eq!(status.len(), 1);
    assert!(status[0].contains("Added: test.txt"));
}

#[test]
fn test_get_status_after_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    let result = StatusOps::get_status(&repo, work_dir);
    assert!(result.is_ok());
    
    let status = result.unwrap();
    assert!(status.is_empty()); // No changes after commit
}

#[test]
fn test_get_status_after_modification() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Modify the file
    std::fs::write(&test_file, "Hello, modified world!").unwrap();
    
    let result = StatusOps::get_status(&repo, work_dir);
    assert!(result.is_ok());
    
    let status = result.unwrap();
    assert_eq!(status.len(), 1);
    assert!(status[0].contains("Modified: test.txt"));
}

#[test]
fn test_get_status_after_deletion() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Delete the file
    std::fs::remove_file(&test_file).unwrap();
    
    let result = StatusOps::get_status(&repo, work_dir);
    assert!(result.is_ok());
    
    let status = result.unwrap();
    assert_eq!(status.len(), 1);
    assert!(status[0].contains("Deleted: test.txt"));
}

#[test]
fn test_get_status_multiple_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Make multiple changes
    std::fs::write(&test_file, "Hello, modified world!").unwrap(); // Modify
    let new_file = work_dir.join("new.txt");
    create_test_file(&new_file, "New content").unwrap(); // Add
    let to_delete = work_dir.join("to_delete.txt");
    create_test_file(&to_delete, "To delete").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &to_delete).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Add file", "test-user", "test@example.com").unwrap();
    std::fs::remove_file(&to_delete).unwrap(); // Delete
    
    let result = StatusOps::get_status(&repo, work_dir);
    assert!(result.is_ok());
    
    let status = result.unwrap();
    assert_eq!(status.len(), 3); // Modified, Added, Deleted
    
    // Check that all expected changes are present
    let status_str = status.join(" ");
    assert!(status_str.contains("Modified: test.txt"));
    assert!(status_str.contains("Added: new.txt"));
    assert!(status_str.contains("Deleted: to_delete.txt"));
}

#[test]
fn test_reset_working_tree_no_commits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create some untracked files
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    
    let result = StatusOps::reset_working_tree(&repo, work_dir);
    assert!(result.is_ok());
    
    // Verify untracked files were removed
    assert!(!test_file.exists());
}

#[test]
fn test_reset_working_tree_with_commits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Modify the file
    std::fs::write(&test_file, "Hello, modified world!").unwrap();
    
    // Reset working tree
    let result = StatusOps::reset_working_tree(&repo, work_dir);
    assert!(result.is_ok());
    
    // Verify file was reset to committed state
    let content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_reset_working_tree_with_untracked_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Create untracked files
    let untracked_file = work_dir.join("untracked.txt");
    create_test_file(&untracked_file, "Untracked content").unwrap();
    
    let untracked_dir = work_dir.join("untracked_dir");
    std::fs::create_dir_all(&untracked_dir).unwrap();
    let untracked_subfile = untracked_dir.join("subfile.txt");
    create_test_file(&untracked_subfile, "Subfile content").unwrap();
    
    // Reset working tree
    let result = StatusOps::reset_working_tree(&repo, work_dir);
    assert!(result.is_ok());
    
    // Verify untracked files were removed
    assert!(!untracked_file.exists());
    assert!(!untracked_dir.exists());
    
    // Verify committed file still exists
    assert!(test_file.exists());
}

#[test]
fn test_reset_working_tree_clean() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    crate::git::operations::commit_ops::CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Reset working tree (should be no-op since it's clean)
    let result = StatusOps::reset_working_tree(&repo, work_dir);
    assert!(result.is_ok());
    
    // Verify file still exists and content is unchanged
    assert!(test_file.exists());
    let content = std::fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "Hello, world!");
}
