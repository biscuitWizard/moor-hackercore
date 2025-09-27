use super::*;
use crate::git::operations::file_ops::FileOps;
use git2::Repository;

#[test]
fn test_add_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create a test file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    
    let result = FileOps::add_file(&repo, work_dir, &test_file);
    assert!(result.is_ok());
    
    // Verify the file was added to the index
    let index = repo.index().unwrap();
    assert!(index.get_path(std::path::Path::new("test.txt"), 0).is_some());
}

#[test]
fn test_add_file_relative_path() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create a test file in a subdirectory
    let subdir = work_dir.join("subdir");
    std::fs::create_dir_all(&subdir).unwrap();
    let test_file = subdir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    
    let result = FileOps::add_file(&repo, work_dir, &test_file);
    assert!(result.is_ok());
    
    // Verify the file was added to the index
    let index = repo.index().unwrap();
    assert!(index.get_path(std::path::Path::new("subdir/test.txt"), 0).is_some());
}

#[test]
fn test_add_file_outside_repository() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create a file outside the repository
    let outside_file = temp_dir.path().parent().unwrap().join("outside.txt");
    create_test_file(&outside_file, "Hello, world!").unwrap();
    
    let result = FileOps::add_file(&repo, work_dir, &outside_file);
    assert!(result.is_err());
}

#[test]
fn test_remove_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and add a test file
    let test_file = work_dir.join("test.txt");
    create_test_file(&test_file, "Hello, world!").unwrap();
    FileOps::add_file(&repo, work_dir, &test_file).unwrap();
    
    // Remove the file
    let result = FileOps::remove_file(&repo, work_dir, &test_file);
    assert!(result.is_ok());
    
    // Verify the file was removed from the index
    let index = repo.index().unwrap();
    assert!(index.get_path(std::path::Path::new("test.txt"), 0).is_none());
    
    // Verify the file was removed from the filesystem
    assert!(!test_file.exists());
}

#[test]
fn test_remove_nonexistent_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Try to remove a file that doesn't exist
    let nonexistent_file = work_dir.join("nonexistent.txt");
    let result = FileOps::remove_file(&repo, work_dir, &nonexistent_file);
    assert!(result.is_ok()); // Should succeed even if file doesn't exist
}

#[test]
fn test_add_all_changes() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create multiple test files
    let file1 = work_dir.join("file1.txt");
    let file2 = work_dir.join("file2.txt");
    create_test_file(&file1, "Content 1").unwrap();
    create_test_file(&file2, "Content 2").unwrap();
    
    let result = FileOps::add_all_changes(&repo);
    assert!(result.is_ok());
    
    // Verify both files were added to the index
    let index = repo.index().unwrap();
    assert!(index.get_path(std::path::Path::new("file1.txt"), 0).is_some());
    assert!(index.get_path(std::path::Path::new("file2.txt"), 0).is_some());
}

#[test]
fn test_write_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let work_dir = temp_dir.path();
    
    let test_file = std::path::Path::new("test.txt");
    let content = "Hello, world!";
    
    let result = FileOps::write_file(work_dir, test_file, content);
    assert!(result.is_ok());
    
    // Verify the file was created with correct content
    let full_path = work_dir.join(test_file);
    assert!(full_path.exists());
    let file_content = std::fs::read_to_string(&full_path).unwrap();
    assert_eq!(file_content, content);
}

#[test]
fn test_write_file_with_subdirectories() {
    let temp_dir = tempfile::tempdir().unwrap();
    let work_dir = temp_dir.path();
    
    let test_file = std::path::Path::new("subdir/nested/test.txt");
    let content = "Nested content";
    
    let result = FileOps::write_file(work_dir, test_file, content);
    assert!(result.is_ok());
    
    // Verify the file was created with correct content
    let full_path = work_dir.join(test_file);
    assert!(full_path.exists());
    let file_content = std::fs::read_to_string(&full_path).unwrap();
    assert_eq!(file_content, content);
}

#[test]
fn test_read_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let work_dir = temp_dir.path();
    
    let test_file = std::path::Path::new("test.txt");
    let content = "Hello, world!";
    
    // Create the file first
    create_test_file(&work_dir.join(test_file), content).unwrap();
    
    let result = FileOps::read_file(work_dir, test_file);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), content);
}

#[test]
fn test_read_nonexistent_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let work_dir = temp_dir.path();
    
    let nonexistent_file = std::path::Path::new("nonexistent.txt");
    let result = FileOps::read_file(work_dir, nonexistent_file);
    assert!(result.is_err());
}

#[test]
fn test_file_exists() {
    let temp_dir = tempfile::tempdir().unwrap();
    let work_dir = temp_dir.path();
    
    let test_file = std::path::Path::new("test.txt");
    let nonexistent_file = std::path::Path::new("nonexistent.txt");
    
    // Create the file
    create_test_file(&work_dir.join(test_file), "content").unwrap();
    
    assert!(FileOps::file_exists(work_dir, test_file));
    assert!(!FileOps::file_exists(work_dir, nonexistent_file));
}

#[test]
fn test_rename_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    // Create and add a test file
    let old_file = work_dir.join("old.txt");
    let new_file = work_dir.join("new.txt");
    create_test_file(&old_file, "Hello, world!").unwrap();
    FileOps::add_file(&repo, work_dir, &old_file).unwrap();
    
    let result = FileOps::rename_file(&repo, work_dir, &old_file, &new_file);
    assert!(result.is_ok());
    
    // Verify the old file doesn't exist
    assert!(!old_file.exists());
    
    // Verify the new file exists with correct content
    assert!(new_file.exists());
    let content = std::fs::read_to_string(&new_file).unwrap();
    assert_eq!(content, "Hello, world!");
    
    // Verify the index was updated
    let index = repo.index().unwrap();
    assert!(index.get_path(std::path::Path::new("old.txt"), 0).is_none());
    assert!(index.get_path(std::path::Path::new("new.txt"), 0).is_some());
}

#[test]
fn test_rename_nonexistent_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let work_dir = temp_dir.path();
    
    let old_file = work_dir.join("nonexistent.txt");
    let new_file = work_dir.join("new.txt");
    
    let result = FileOps::rename_file(&repo, work_dir, &old_file, &new_file);
    assert!(result.is_err());
}

#[test]
fn test_meta_path() {
    let moo_file = std::path::Path::new("objects/player.moo");
    let meta_path = crate::utils::PathUtils::meta_path(moo_file);
    
    assert_eq!(meta_path, std::path::Path::new("objects/player.meta"));
}

#[test]
fn test_meta_path_no_extension() {
    let moo_file = std::path::Path::new("objects/player");
    let meta_path = crate::utils::PathUtils::meta_path(moo_file);
    
    assert_eq!(meta_path, std::path::Path::new("objects/player.meta"));
}

#[test]
fn test_meta_path_different_extension() {
    let other_file = std::path::Path::new("objects/player.txt");
    let meta_path = crate::utils::PathUtils::meta_path(other_file);
    
    assert_eq!(meta_path, std::path::Path::new("objects/player.txt.meta"));
}
