pub mod workflow_tests;

use tempfile::TempDir;
use crate::config::Config;

/// Helper function to create a test configuration
pub fn create_test_config(temp_dir: &TempDir) -> Config {
    Config {
        repository_url: None,
        repository_path: temp_dir.path().to_path_buf(),
        objects_directory: "objects".to_string(),
        debug: false,
        git_user_name: "test-user".to_string(),
        git_user_email: "test@example.com".to_string(),
        ssh_key_path: None,
    }
}
