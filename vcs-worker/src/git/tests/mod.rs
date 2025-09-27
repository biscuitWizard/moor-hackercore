pub mod config_tests;
pub mod utils_tests;
pub mod file_ops_tests;
pub mod commit_ops_tests;
pub mod remote_ops_tests;
pub mod status_ops_tests;
pub mod repository_tests;
pub mod channel_tests;

use std::path::PathBuf;
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

/// Helper function to create a test configuration with SSH key
pub fn create_test_config_with_ssh(temp_dir: &TempDir, ssh_key_path: PathBuf) -> Config {
    Config {
        repository_url: None,
        repository_path: temp_dir.path().to_path_buf(),
        objects_directory: "objects".to_string(),
        debug: false,
        git_user_name: "test-user".to_string(),
        git_user_email: "test@example.com".to_string(),
        ssh_key_path: Some(ssh_key_path.to_string_lossy().to_string()),
    }
}

/// Helper function to create a test file
pub fn create_test_file(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(path, content)
}

/// Helper function to create a test SSH key file
pub fn create_test_ssh_key(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path.parent().unwrap())?;
    // Create a dummy SSH key content
    let key_content = "-----BEGIN OPENSSH PRIVATE KEY-----\n\
                      b3BlbnNzaC1rZXktdjEAAAAABG5vbmUAAAAEbm9uZQAAAAAAAAABAAAAFwAAAAdzc2gtcn\n\
                      NhAAAAAwEAAQAAAQEA1234567890abcdefghijklmnopqrstuvwxyz\n\
                      -----END OPENSSH PRIVATE KEY-----";
    std::fs::write(path, key_content)?;
    
    // Set restrictive permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }
    
    Ok(())
}
