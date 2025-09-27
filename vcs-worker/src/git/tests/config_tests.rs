use super::*;
use crate::git::config::GitConfig;

#[test]
fn test_git_config_creation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let git_config = GitConfig::new(config);
    
    assert_eq!(git_config.git_user_name(), "test-user");
    assert_eq!(git_config.git_user_email(), "test@example.com");
    assert!(git_config.ssh_key_path().is_none());
}

#[test]
fn test_git_config_with_ssh_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let ssh_key_path = temp_dir.path().join("test_key");
    create_test_ssh_key(&ssh_key_path).unwrap();
    
    let config = create_test_config_with_ssh(&temp_dir, ssh_key_path.clone());
    let git_config = GitConfig::new(config);
    
    assert_eq!(git_config.git_user_name(), "test-user");
    assert_eq!(git_config.git_user_email(), "test@example.com");
    assert_eq!(git_config.ssh_key_path(), Some(&ssh_key_path.to_string_lossy().to_string()));
}

#[test]
fn test_git_config_keys_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let git_config = GitConfig::new(config);
    
    let expected_keys_dir = temp_dir.path().join("keys");
    assert_eq!(git_config.keys_directory(), expected_keys_dir);
}

#[test]
fn test_git_config_meta_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = create_test_config(&temp_dir);
    let git_config = GitConfig::new(config);
    
    let expected_meta_dir = temp_dir.path().join("meta");
    assert_eq!(git_config.meta_directory(), expected_meta_dir);
}
