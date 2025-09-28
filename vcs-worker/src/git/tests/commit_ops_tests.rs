use super::*;
use crate::git::operations::commit_ops::CommitOps;
use git2::Repository;

#[test]
fn test_create_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and add a test file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    
    let result = CommitOps::create_commit(
        &repo,
        "Test commit",
        "test-user",
        "test@example.com",
    );
    
    assert!(result.is_ok());
    
    let commit = result.unwrap();
    assert_eq!(commit.message().unwrap(), "Test commit");
    assert_eq!(commit.author().name().unwrap(), "test-user");
    assert_eq!(commit.author().email().unwrap(), "test@example.com");
}

#[test]
fn test_create_commit_with_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create multiple test files
    let file1 = work_dir.join("file1.txt");
    let file2 = work_dir.join("file2.txt");
    create_test_file(&file1, "Content 1").unwrap();
    create_test_file(&file2, "Content 2").unwrap();
    
    // Add all changes
    crate::git::operations::file_ops::FileOps::add_all_changes(&repo).unwrap();
    
    let result = CommitOps::create_commit(
        &repo,
        "Multiple files commit",
        "test-user",
        "test@example.com",
    );
    
    assert!(result.is_ok());
    
    let commit = result.unwrap();
    assert_eq!(commit.message().unwrap(), "Multiple files commit");
    
    // Verify the commit has the expected files
    let tree = commit.tree().unwrap();
    assert!(tree.get_path(std::path::Path::new("file1.txt")).is_ok());
    assert!(tree.get_path(std::path::Path::new("file2.txt")).is_ok());
}

#[test]
fn test_get_head_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Initially, there should be no HEAD commit
    let result = CommitOps::get_head_commit(&repo);
    assert!(result.is_err());
    
    // Create and commit a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    
    CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Now there should be a HEAD commit
    let result = CommitOps::get_head_commit(&repo);
    assert!(result.is_ok());
    
    let commit = result.unwrap();
    assert_eq!(commit.message().unwrap(), "Initial commit");
}

#[test]
fn test_get_commits() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create multiple commits
    for i in 1..=3 {
        let test_file = work_dir.join(format!("file{}.txt", i));
        create_test_file(&test_file, &format!("Content {}", i)).unwrap();
        crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
        
        CommitOps::create_commit(
            &repo,
            &format!("Commit {}", i),
            "test-user",
            "test@example.com",
        ).unwrap();
    }
    
    // Get all commits
    let result = CommitOps::get_commits(&repo, None, None);
    assert!(result.is_ok());
    
    let commits = result.unwrap();
    assert_eq!(commits.len(), 3);
    
    // Verify commit order (most recent first)
    assert_eq!(commits[0].message, "Commit 3");
    assert_eq!(commits[1].message, "Commit 2");
    assert_eq!(commits[2].message, "Commit 1");
}

#[test]
fn test_get_commits_with_limit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create multiple commits
    for i in 1..=5 {
        let test_file = work_dir.join(format!("file{}.txt", i));
        create_test_file(&test_file, &format!("Content {}", i)).unwrap();
        crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
        
        CommitOps::create_commit(
            &repo,
            &format!("Commit {}", i),
            "test-user",
            "test@example.com",
        ).unwrap();
    }
    
    // Get only 2 commits
    let result = CommitOps::get_commits(&repo, Some(2), None);
    assert!(result.is_ok());
    
    let commits = result.unwrap();
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].message, "Commit 5");
    assert_eq!(commits[1].message, "Commit 4");
}

#[test]
fn test_get_commits_with_offset() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create multiple commits
    for i in 1..=5 {
        let test_file = work_dir.join(format!("file{}.txt", i));
        create_test_file(&test_file, &format!("Content {}", i)).unwrap();
        crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
        
        CommitOps::create_commit(
            &repo,
            &format!("Commit {}", i),
            "test-user",
            "test@example.com",
        ).unwrap();
    }
    
    // Get commits with offset
    let result = CommitOps::get_commits(&repo, Some(2), Some(1));
    assert!(result.is_ok());
    
    let commits = result.unwrap();
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].message, "Commit 4");
    assert_eq!(commits[1].message, "Commit 3");
}

#[test]
fn test_get_last_commit_info() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Initially, there should be no commit info
    let result = CommitOps::get_last_commit_info(&repo);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Create a commit
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    
    CommitOps::create_commit(&repo, "Test commit", "test-user", "test@example.com").unwrap();
    
    // Now there should be commit info
    let result = CommitOps::get_last_commit_info(&repo);
    assert!(result.is_ok());
    
    let commit_info = result.unwrap().unwrap();
    assert_eq!(commit_info.message, "Test commit");
    assert_eq!(commit_info.author, "test-user");
    assert!(!commit_info.id.is_empty());
    assert!(!commit_info.full_id.is_empty());
}

#[test]
fn test_get_commits_ahead_behind() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create initial commit
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Create a branch and make commits
    let branch_commit = repo.head().unwrap().peel_to_commit().unwrap();
    let _branch = repo.branch("feature", &branch_commit, false).unwrap();
    repo.set_head("refs/heads/feature").unwrap();
    
    // Make another commit on the branch
    let test_file2 = work_dir.join("test2.txt");
    create_test_file(&test_file2, "Hello, world 2!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file2).unwrap();
    CommitOps::create_commit(&repo, "Feature commit", "test-user", "test@example.com").unwrap();
    
    // Test ahead/behind calculation
    let result = CommitOps::get_commits_ahead_behind(&repo, "feature", "refs/heads/main");
    assert!(result.is_ok());
    
    let (ahead, behind) = result.unwrap();
    assert_eq!(ahead, 1); // feature is 1 commit ahead of main
    assert_eq!(behind, 0); // feature is 0 commits behind main
}

#[test]
fn test_get_commits_between() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create multiple commits
    for i in 1..=3 {
        let test_file = work_dir.join(format!("file{}.txt", i));
        create_test_file(&test_file, &format!("Content {}", i)).unwrap();
        crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
        
        CommitOps::create_commit(
            &repo,
            &format!("Commit {}", i),
            "test-user",
            "test@example.com",
        ).unwrap();
    }
    
    // Get commits between HEAD~2 and HEAD
    let result = CommitOps::get_commits_between(&repo, "HEAD~2", "HEAD");
    assert!(result.is_ok());
    
    let commits = result.unwrap();
    assert_eq!(commits.len(), 2);
    assert_eq!(commits[0].message, "Commit 3");
    assert_eq!(commits[1].message, "Commit 2");
}

#[test]
fn test_get_commit_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create initial commit
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    let _first_commit = CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Modify the file and create another commit
    std::fs::write(&test_file, "Hello, modified world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    let second_commit = CommitOps::create_commit(&repo, "Modified commit", "test-user", "test@example.com").unwrap();
    
    // Get changes in the second commit
    let result = CommitOps::get_commit_changes(&repo, &second_commit.id().to_string());
    assert!(result.is_ok());
    
    let changes = result.unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "test.txt");
    assert!(matches!(changes[0].status, crate::vcs::types::ChangeStatus::Modified));
}

#[test]
fn test_get_file_content_at_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create a commit with a file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    let commit = CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Get file content at that commit
    let result = CommitOps::get_file_content_at_commit(&repo, &commit.id().to_string(), "test.txt");
    assert!(result.is_ok());
    
    let content = result.unwrap();
    assert_eq!(content, "Hello, world!");
}

#[test]
fn test_rollback_last_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create initial commit
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Create second commit
    let test_file2 = work_dir.join("test2.txt");
    create_test_file(&test_file2, "Hello, world 2!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file2).unwrap();
    CommitOps::create_commit(&repo, "Second commit", "test-user", "test@example.com").unwrap();
    
    // Rollback the last commit
    let result = CommitOps::rollback_last_commit(&repo);
    assert!(result.is_ok());
    
    // Verify we're back to the first commit
    let head_commit = CommitOps::get_head_commit(&repo).unwrap();
    assert_eq!(head_commit.message().unwrap(), "Initial commit");
    
    // Verify the second file is still in the working directory but not committed
    assert!(test_file2.exists());
    let index = repo.index().unwrap();
    assert!(index.get_path(std::path::Path::new("test2.txt"), 0).is_some());
}

#[test]
fn test_rollback_first_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create initial commit
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Rollback the first (and only) commit
    let result = CommitOps::rollback_last_commit(&repo);
    assert!(result.is_ok());
    
    // Verify there are no commits now
    let result = CommitOps::get_head_commit(&repo);
    assert!(result.is_err());
    
    // Verify the file is still in the working directory but not committed
    assert!(test_file.exists());
    let index = repo.index().unwrap();
    assert!(index.get_path(std::path::Path::new("test.txt"), 0).is_some());
}

#[test]
fn test_create_commit_with_no_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    
    // Try to create a commit without any changes
    let result = CommitOps::create_commit(
        &repo,
        "Empty commit",
        "test-user",
        "test@example.com",
    );
    
    // Should fail with appropriate error message
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No changes to commit"));
    assert!(error.to_string().contains("Repository is clean"));
}

#[test]
fn test_create_commit_with_no_changes_after_initial_commit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create initial commit
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    crate::git::operations::file_ops::FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    CommitOps::create_commit(&repo, "Initial commit", "test-user", "test@example.com").unwrap();
    
    // Try to create another commit without any changes
    let result = CommitOps::create_commit(
        &repo,
        "Empty commit",
        "test-user",
        "test@example.com",
    );
    
    // Should fail with appropriate error message
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No changes to commit"));
    assert!(error.to_string().contains("Repository is clean"));
}
