use super::*;
use crate::git::repository::GitRepository;

#[test]
fn test_git_repository_init() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    
    let result = GitRepository::init(temp_dir.path(), config);
    assert!(result.is_ok());
    
    let repo = result.unwrap();
    assert_eq!(repo.work_dir(), temp_dir.path());
    
    // Verify .git directory was created
    assert!(temp_dir.path().join(".git").exists());
    
    // Verify .gitignore was created with keys/ entry
    let gitignore_path = temp_dir.path().join(".gitignore");
    assert!(gitignore_path.exists());
    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("keys/"));
}

#[test]
fn test_git_repository_open() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Initialize repository first
    let _repo1 = GitRepository::init(temp_dir.path(), config.clone()).unwrap();
    
    // Open existing repository
    let result = GitRepository::open(temp_dir.path(), config);
    assert!(result.is_ok());
    
    let repo = result.unwrap();
    assert_eq!(repo.work_dir(), temp_dir.path());
}

#[test]
fn test_git_repository_open_nonexistent() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    
    // Try to open non-existent repository
    let result = GitRepository::open(temp_dir.path(), config);
    assert!(result.is_err());
}

#[test]
fn test_git_repository_file_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Test write_file
    let result = repo.write_file("test.txt", "Hello, world!");
    assert!(result.is_ok());
    
    // Test read_file
    let result = repo.read_file("test.txt");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, world!");
    
    // Test file_exists
    assert!(repo.file_exists("test.txt"));
    assert!(!repo.file_exists("nonexistent.txt"));
}

#[test]
fn test_git_repository_add_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create a file
    repo.write_file("test.txt", "Hello, world!").unwrap();
    
    // Add file to git
    let result = repo.add_file("test.txt");
    assert!(result.is_ok());
    
    // Verify file was added to index
    let index = repo.repo().index().unwrap();
    assert!(index.get_path(std::path::Path::new("test.txt"), 0).is_some());
}

#[test]
fn test_git_repository_remove_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create and add a file
    repo.write_file("test.txt", "Hello, world!").unwrap();
    repo.add_file("test.txt").unwrap();
    
    // Remove file from git
    let result = repo.remove_file("test.txt");
    assert!(result.is_ok());
    
    // Verify file was removed from index and filesystem
    let index = repo.repo().index().unwrap();
    assert!(index.get_path(std::path::Path::new("test.txt"), 0).is_none());
    assert!(!repo.file_exists("test.txt"));
}

#[test]
fn test_git_repository_rename_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create and add a file
    repo.write_file("old.txt", "Hello, world!").unwrap();
    repo.add_file("old.txt").unwrap();
    
    // Rename file
    let result = repo.rename_file("old.txt", "new.txt");
    assert!(result.is_ok());
    
    // Verify old file doesn't exist
    assert!(!repo.file_exists("old.txt"));
    
    // Verify new file exists with correct content
    assert!(repo.file_exists("new.txt"));
    let content = repo.read_file("new.txt").unwrap();
    assert_eq!(content, "Hello, world!");
    
    // Verify index was updated
    let index = repo.repo().index().unwrap();
    assert!(index.get_path(std::path::Path::new("old.txt"), 0).is_none());
    assert!(index.get_path(std::path::Path::new("new.txt"), 0).is_some());
}

#[test]
fn test_git_repository_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create and add a file
    repo.write_file("test.txt", "Hello, world!").unwrap();
    repo.add_file("test.txt").unwrap();
    
    // Create commit
    let result = repo.commit("Test commit", "test-user", "test@example.com");
    assert!(result.is_ok());
    
    let commit = result.unwrap();
    assert_eq!(commit.message().unwrap(), "Test commit");
    assert_eq!(commit.author().name().unwrap(), "test-user");
    assert_eq!(commit.author().email().unwrap(), "test@example.com");
}

#[test]
fn test_git_repository_get_commits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create multiple commits
    for i in 1..=3 {
        repo.write_file(&format!("file{}.txt", i), &format!("Content {}", i)).unwrap();
        repo.add_file(&format!("file{}.txt", i)).unwrap();
        repo.commit(&format!("Commit {}", i), "test-user", "test@example.com").unwrap();
    }
    
    // Get commits
    let result = repo.get_commits(None, None);
    assert!(result.is_ok());
    
    let commits = result.unwrap();
    assert_eq!(commits.len(), 3);
    
    // Verify commit order (most recent first)
    assert_eq!(commits[0].message, "Commit 3");
    assert_eq!(commits[1].message, "Commit 2");
    assert_eq!(commits[2].message, "Commit 1");
}

#[test]
fn test_git_repository_get_last_commit_info() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Initially, there should be no commit info
    let result = repo.get_last_commit_info();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create a commit
    repo.write_file("test.txt", "Hello, world!").unwrap();
    repo.add_file("test.txt").unwrap();
    repo.commit("Test commit", "test-user", "test@example.com").unwrap();
    
    // Now there should be commit info
    let result = repo.get_last_commit_info();
    assert!(result.is_ok());
    
    let commit_info = result.unwrap().unwrap();
    assert_eq!(commit_info.message, "Test commit");
    assert_eq!(commit_info.author, "test-user");
    assert!(!commit_info.id.is_empty());
    assert!(!commit_info.full_id.is_empty());
}

#[test]
fn test_git_repository_has_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Initially, there should be no changes
    let result = repo.has_changes();
    assert!(result.is_ok());
    assert!(!result.unwrap());
    
    // Create an untracked file
    repo.write_file("test.txt", "Hello, world!").unwrap();
    
    // Should detect changes
    let result = repo.has_changes();
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_git_repository_status() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create an untracked file
    repo.write_file("test.txt", "Hello, world!").unwrap();
    
    let result = repo.status();
    assert!(result.is_ok());
    
    let status = result.unwrap();
    assert_eq!(status.len(), 1);
    assert!(status[0].contains("Added: test.txt"));
}

#[test]
fn test_git_repository_get_current_branch() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Initially, there should be no current branch
    let result = repo.get_current_branch();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create initial commit to establish a branch
    repo.write_file("test.txt", "Hello, world!").unwrap();
    repo.add_file("test.txt").unwrap();
    repo.commit("Initial commit", "test-user", "test@example.com").unwrap();
    
    // Now there should be a current branch
    let result = repo.get_current_branch();
    assert!(result.is_ok());
    
    let branch = result.unwrap();
    assert!(branch.is_some());
    assert_eq!(branch.unwrap(), "main");
}

#[test]
fn test_git_repository_get_upstream_info() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Initially, there should be no upstream
    let result = repo.get_upstream_info();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_git_repository_meta_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Test meta path generation
    let moo_path = std::path::Path::new("objects/player.moo");
    let meta_path = repo.meta_path(moo_path);
    
    assert_eq!(meta_path, std::path::Path::new("objects/player.meta"));
}

#[test]
fn test_git_repository_configure_git_user() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Configure git user
    let result = repo.configure_git_user();
    assert!(result.is_ok());
    
    // Verify the configuration was set
    let git_config = repo.repo().config().unwrap();
    let user_name = git_config.get_string("user.name").unwrap();
    let user_email = git_config.get_string("user.email").unwrap();
    
    assert_eq!(user_name, "test-user");
    assert_eq!(user_email, "test@example.com");
}

#[test]
fn test_git_repository_reset_working_tree() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create and commit a file
    repo.write_file("test.txt", "Hello, world!").unwrap();
    repo.add_file("test.txt").unwrap();
    repo.commit("Initial commit", "test-user", "test@example.com").unwrap();
    
    // Modify the file
    repo.write_file("test.txt", "Hello, modified world!").unwrap();
    
    // Reset working tree
    let result = repo.reset_working_tree();
    assert!(result.is_ok());
    
    // Verify file was reset to committed state
    let content = repo.read_file("test.txt").unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_git_repository_rollback_last_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create initial commit
    repo.write_file("test.txt", "Hello, world!").unwrap();
    repo.add_file("test.txt").unwrap();
    repo.commit("Initial commit", "test-user", "test@example.com").unwrap();
    
    // Create second commit
    repo.write_file("test2.txt", "Hello, world 2!").unwrap();
    repo.add_file("test2.txt").unwrap();
    repo.commit("Second commit", "test-user", "test@example.com").unwrap();
    
    // Rollback the last commit
    let result = repo.rollback_last_commit();
    assert!(result.is_ok());
    
    // Verify we're back to the first commit
    let commits = repo.get_commits(Some(1), None).unwrap();
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].message, "Initial commit");
    
    // Verify the second file is still in the working directory but not committed
    assert!(repo.file_exists("test2.txt"));
}
