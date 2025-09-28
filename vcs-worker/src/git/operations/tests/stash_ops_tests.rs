use tempfile::tempdir;
use crate::git::GitRepository;
use crate::git::operations::stash_ops::{StashOps, StashedObject, StashOperation};
use crate::vcs::object_handler::ObjectHandler;
use crate::vcs::tests::create_test_config;
use moor_var::Obj;

fn create_test_moo_object(name: &str, oid: u32) -> String {
    format!(
        "object #{} with\n\
        name \"{}\"\n\
        verb @test_verb():\n\
        endverb\n\
        endobject\n",
        oid, name
    )
}

#[test]
fn test_stash_modified_file() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Create a test MOO object
    let test_object_content = create_test_moo_object("test_object", 42);
    let test_object_path = objects_dir.join("test_object.moo");
    std::fs::write(&test_object_path, &test_object_content).unwrap();
    
    // Add the file to git and commit it
    repo.add_file(&test_object_path).unwrap();
    
    // Create a commit to establish a baseline
    use crate::git::operations::commit_ops::CommitOps;
    CommitOps::create_commit(repo.repo(), "Initial commit", "test", "test@example.com").unwrap();
    
    // Modify the file
    let modified_content = create_test_moo_object("test_object", 43);
    std::fs::write(&test_object_path, &modified_content).unwrap();
    
    // Stash the changes
    let stashed_objects = StashOps::stash_changes(&repo, &object_handler).unwrap();
    assert_eq!(stashed_objects.len(), 1);
    
    // Verify the stashed object
    assert_eq!(stashed_objects[0].original_filename, "test_object");
    assert!(matches!(stashed_objects[0].operation, StashOperation::Modified));
    assert!(stashed_objects[0].object_def.is_some());
    
    // Verify the object definition has the modified content
    if let Some(ref object_def) = stashed_objects[0].object_def {
        assert_eq!(object_def.oid, Obj::mk_id(43));
    }
}

#[test]
fn test_stash_new_file() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Create a new test MOO object (untracked)
    let test_object_content = create_test_moo_object("new_object", 100);
    let test_object_path = objects_dir.join("new_object.moo");
    std::fs::write(&test_object_path, &test_object_content).unwrap();
    
    // Stash the changes
    let stashed_objects = StashOps::stash_changes(&repo, &object_handler).unwrap();
    assert_eq!(stashed_objects.len(), 1);
    
    // Verify the stashed object
    assert_eq!(stashed_objects[0].original_filename, "new_object");
    assert!(matches!(stashed_objects[0].operation, StashOperation::Modified));
    assert!(stashed_objects[0].object_def.is_some());
}

#[test]
fn test_replay_modified_file() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Create a test object definition
    let test_object_def = object_handler.parse_object_dump(&create_test_moo_object("test_object", 42)).unwrap();
    
    // Create a stashed object
    let stashed_object = StashedObject {
        object_def: Some(test_object_def),
        original_filename: "test_object".to_string(),
        operation: StashOperation::Modified,
    };
    
    // Replay the stashed changes
    let result = StashOps::replay_stashed_changes(&repo, &object_handler, vec![stashed_object]);
    assert!(result.is_ok());
    
    // Verify the file was written
    let test_object_path = objects_dir.join("test_object.moo");
    assert!(test_object_path.exists());
    
    // Verify the content
    let content = std::fs::read_to_string(&test_object_path).unwrap();
    assert!(content.contains("object #42"));
}

#[test]
fn test_replay_deleted_file() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Create a test file
    let test_object_content = create_test_moo_object("test_object", 42);
    let test_object_path = objects_dir.join("test_object.moo");
    std::fs::write(&test_object_path, &test_object_content).unwrap();
    
    // Add to git
    repo.add_file(&test_object_path).unwrap();
    
    // Verify file exists
    assert!(test_object_path.exists());
    
    // Create a stashed object for deletion
    let stashed_object = StashedObject {
        object_def: None,
        original_filename: "test_object".to_string(),
        operation: StashOperation::Deleted,
    };
    
    // Replay the stashed changes (should delete the file)
    let result = StashOps::replay_stashed_changes(&repo, &object_handler, vec![stashed_object]);
    assert!(result.is_ok());
    
    // Verify the file was deleted
    assert!(!test_object_path.exists());
}

#[test]
fn test_stash_no_changes() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Stash when there are no changes
    let stashed_objects = StashOps::stash_changes(&repo, &object_handler).unwrap();
    assert_eq!(stashed_objects.len(), 0);
}

#[test]
fn test_stash_multiple_files() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Create multiple test files
    let file1_content = create_test_moo_object("object1", 1);
    let file1_path = objects_dir.join("object1.moo");
    std::fs::write(&file1_path, &file1_content).unwrap();
    repo.add_file(&file1_path).unwrap();
    
    let file2_content = create_test_moo_object("object2", 2);
    let file2_path = objects_dir.join("object2.moo");
    std::fs::write(&file2_path, &file2_content).unwrap();
    repo.add_file(&file2_path).unwrap();
    
    // Create a commit to establish a baseline
    use crate::git::operations::commit_ops::CommitOps;
    CommitOps::create_commit(repo.repo(), "Initial commit", "test", "test@example.com").unwrap();
    
    // Modify both files
    let modified_content1 = create_test_moo_object("object1", 11);
    let modified_content2 = create_test_moo_object("object2", 22);
    std::fs::write(&file1_path, &modified_content1).unwrap();
    std::fs::write(&file2_path, &modified_content2).unwrap();
    
    // Stash the changes
    let stashed_objects = StashOps::stash_changes(&repo, &object_handler).unwrap();
    assert_eq!(stashed_objects.len(), 2);
    
    // Verify both objects are stashed
    let mut filenames: Vec<String> = stashed_objects.iter()
        .map(|obj| obj.original_filename.clone())
        .collect();
    filenames.sort();
    
    assert_eq!(filenames, vec!["object1", "object2"]);
    
    // Verify both are Modified operations
    for stashed_obj in &stashed_objects {
        assert!(matches!(stashed_obj.operation, StashOperation::Modified));
    }
}

#[test]
fn test_stash_renamed_file() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Create a test file
    let test_object_content = create_test_moo_object("old_object", 42);
    let old_file_path = objects_dir.join("old_object.moo");
    std::fs::write(&old_file_path, &test_object_content).unwrap();
    
    // Add to git and commit
    repo.add_file(&old_file_path).unwrap();
    use crate::git::operations::commit_ops::CommitOps;
    CommitOps::create_commit(repo.repo(), "Initial commit", "test", "test@example.com").unwrap();
    
    // Rename the file
    let new_file_path = objects_dir.join("new_object.moo");
    std::fs::rename(&old_file_path, &new_file_path).unwrap();
    
    // Add the rename to git (git sees this as a delete + add)
    repo.add_file(&new_file_path).unwrap();
    
    // Stash the changes
    let stashed_objects = StashOps::stash_changes(&repo, &object_handler).unwrap();
    
    // Should detect this as a rename
    assert_eq!(stashed_objects.len(), 1);
    
    // Verify the file is stashed as Renamed
    let renamed_stashed = &stashed_objects[0];
    assert_eq!(renamed_stashed.original_filename, "new_object");
    assert!(matches!(&renamed_stashed.operation, StashOperation::Renamed { old_name, new_name } if old_name == "old_object" && new_name == "new_object"));
    assert!(renamed_stashed.object_def.is_some());
}

#[test]
fn test_replay_renamed_file() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Create a test file
    let test_object_content = create_test_moo_object("test_object", 42);
    let test_file_path = objects_dir.join("test_object.moo");
    std::fs::write(&test_file_path, &test_object_content).unwrap();
    
    // Add to git and commit
    repo.add_file(&test_file_path).unwrap();
    use crate::git::operations::commit_ops::CommitOps;
    CommitOps::create_commit(repo.repo(), "Initial commit", "test", "test@example.com").unwrap();
    
    // Create a stashed rename manually
    let stashed_objects = vec![
        StashedObject {
            object_def: Some(object_handler.parse_object_dump(&test_object_content).unwrap()),
            original_filename: "renamed_object".to_string(),
            operation: StashOperation::Renamed {
                old_name: "test_object".to_string(),
                new_name: "renamed_object".to_string(),
            },
        },
    ];
    
    // Replay the stashed changes
    let result = StashOps::replay_stashed_changes(&repo, &object_handler, stashed_objects);
    assert!(result.is_ok());
    
    // Verify the old filename was restored
    assert!(test_file_path.exists());
    let file_content = std::fs::read_to_string(&test_file_path).unwrap();
    assert!(file_content.contains("#42"));
    
    // Verify the new filename doesn't exist
    let new_file_path = objects_dir.join("renamed_object.moo");
    assert!(!new_file_path.exists());
}

#[test]
fn test_stash_and_replay_mixed_operations() {
    let temp_dir = tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let object_handler = ObjectHandler::new(config.clone());
    
    // Create a git repository
    let repo = GitRepository::init(temp_dir.path(), config).unwrap();
    
    // Create objects directory
    let objects_dir = temp_dir.path().join(&object_handler.config.objects_directory);
    std::fs::create_dir_all(&objects_dir).unwrap();
    
    // Create two test files
    let file1_content = create_test_moo_object("file1", 1);
    let file1_path = objects_dir.join("file1.moo");
    std::fs::write(&file1_path, &file1_content).unwrap();
    
    let file2_content = create_test_moo_object("file2", 2);
    let file2_path = objects_dir.join("file2.moo");
    std::fs::write(&file2_path, &file2_content).unwrap();
    
    // Add to git and commit
    repo.add_file(&file1_path).unwrap();
    repo.add_file(&file2_path).unwrap();
    use crate::git::operations::commit_ops::CommitOps;
    CommitOps::create_commit(repo.repo(), "Initial commit", "test", "test@example.com").unwrap();
    
    // Modify file1 and delete file2
    let modified_content = create_test_moo_object("file1", 11);
    std::fs::write(&file1_path, &modified_content).unwrap();
    std::fs::remove_file(&file2_path).unwrap();
    
    // Stage the deletion
    repo.remove_file(&file2_path).unwrap();
    
    // Create mixed stashed objects manually (including a rename)
    let stashed_objects = vec![
        StashedObject {
            object_def: Some(object_handler.parse_object_dump(&modified_content).unwrap()),
            original_filename: "file1".to_string(),
            operation: StashOperation::Modified,
        },
        StashedObject {
            object_def: None,
            original_filename: "file2".to_string(),
            operation: StashOperation::Deleted,
        },
        StashedObject {
            object_def: Some(object_handler.parse_object_dump(&file1_content).unwrap()),
            original_filename: "file3".to_string(),
            operation: StashOperation::Renamed {
                old_name: "file1".to_string(),
                new_name: "file3".to_string(),
            },
        },
    ];
    
    // Replay the mixed operations
    let result = StashOps::replay_stashed_changes(&repo, &object_handler, stashed_objects);
    assert!(result.is_ok());
    
    // Verify file1 was restored with modifications
    assert!(file1_path.exists());
    let file1_content = std::fs::read_to_string(&file1_path).unwrap();
    assert!(file1_content.contains("#11"));
    
    // Verify file2 was deleted
    assert!(!file2_path.exists());
    
    // Verify the rename was reverted (file3 should not exist, file1 should exist)
    let file3_path = objects_dir.join("file3.moo");
    assert!(!file3_path.exists());
    assert!(file1_path.exists());
}
