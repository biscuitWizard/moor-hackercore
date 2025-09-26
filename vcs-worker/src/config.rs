use std::env;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Configuration for the VCS Worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// URL to clone the repository from instead of initializing an empty one
    /// If not set, will initialize an empty repository
    pub repository_url: Option<String>,
    
    /// Path where the git repository should be located
    /// Defaults to "/game"
    pub repository_path: PathBuf,
    
    /// Subdirectory within the repository where MOO and meta files should be stored
    /// Defaults to "objects"
    pub objects_directory: String,
    
    /// Whether to enable debug logging
    /// Defaults to false
    pub debug: bool,
    
    /// Git user name for commits
    /// Defaults to "vcs-worker"
    pub git_user_name: String,
    
    /// Git user email for commits
    /// Defaults to "vcs-worker@system"
    pub git_user_email: String,
    
    /// Path to SSH private key for git authentication
    /// If not set, will use default SSH key discovery
    pub ssh_key_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repository_url: None,
            repository_path: PathBuf::from("/game"),
            objects_directory: "objects".to_string(),
            debug: false,
            git_user_name: "vcs-worker".to_string(),
            git_user_email: "vcs-worker@system".to_string(),
            ssh_key_path: None,
        }
    }
}

impl Config {
    /// Create a new Config instance by reading from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Read repository URL from environment
        if let Ok(repo_url) = env::var("VCS_REPOSITORY_URL") {
            if !repo_url.trim().is_empty() {
                config.repository_url = Some(repo_url.trim().to_string());
                info!("Repository URL configured from VCS_REPOSITORY_URL: {}", repo_url);
            } else {
                warn!("VCS_REPOSITORY_URL is empty, will initialize empty repository");
            }
        } else {
            info!("VCS_REPOSITORY_URL not set, will initialize empty repository");
        }
        
        // Read repository path from environment
        if let Ok(repo_path) = env::var("VCS_REPOSITORY_PATH") {
            if !repo_path.trim().is_empty() {
                config.repository_path = PathBuf::from(repo_path.trim());
                info!("Repository path configured from VCS_REPOSITORY_PATH: {:?}", config.repository_path);
            } else {
                warn!("VCS_REPOSITORY_PATH is empty, using default: {:?}", config.repository_path);
            }
        } else {
            info!("VCS_REPOSITORY_PATH not set, using default: {:?}", config.repository_path);
        }
        
        // Read objects directory from environment
        if let Ok(objects_dir) = env::var("VCS_OBJECTS_DIRECTORY") {
            if !objects_dir.trim().is_empty() {
                config.objects_directory = objects_dir.trim().to_string();
                info!("Objects directory configured from VCS_OBJECTS_DIRECTORY: {}", config.objects_directory);
            } else {
                warn!("VCS_OBJECTS_DIRECTORY is empty, using default: {}", config.objects_directory);
            }
        } else {
            info!("VCS_OBJECTS_DIRECTORY not set, using default: {}", config.objects_directory);
        }
        
        // Read debug flag from environment
        if let Ok(debug_str) = env::var("VCS_DEBUG") {
            config.debug = debug_str.to_lowercase() == "true" || debug_str == "1";
            info!("Debug logging configured from VCS_DEBUG: {}", config.debug);
        } else {
            info!("VCS_DEBUG not set, debug logging disabled");
        }
        
        // Read git user name from environment
        if let Ok(git_user_name) = env::var("VCS_GIT_USER_NAME") {
            if !git_user_name.trim().is_empty() {
                config.git_user_name = git_user_name.trim().to_string();
                info!("Git user name configured from VCS_GIT_USER_NAME: {}", config.git_user_name);
            } else {
                warn!("VCS_GIT_USER_NAME is empty, using default: {}", config.git_user_name);
            }
        } else {
            info!("VCS_GIT_USER_NAME not set, using default: {}", config.git_user_name);
        }
        
        // Read git user email from environment
        if let Ok(git_user_email) = env::var("VCS_GIT_USER_EMAIL") {
            if !git_user_email.trim().is_empty() {
                config.git_user_email = git_user_email.trim().to_string();
                info!("Git user email configured from VCS_GIT_USER_EMAIL: {}", config.git_user_email);
            } else {
                warn!("VCS_GIT_USER_EMAIL is empty, using default: {}", config.git_user_email);
            }
        } else {
            info!("VCS_GIT_USER_EMAIL not set, using default: {}", config.git_user_email);
        }
        
        // Read SSH key path from environment
        if let Ok(ssh_key_path) = env::var("VCS_SSH_KEY_PATH") {
            if !ssh_key_path.trim().is_empty() {
                config.ssh_key_path = Some(ssh_key_path.trim().to_string());
                info!("SSH key path configured from VCS_SSH_KEY_PATH: {}", ssh_key_path);
            } else {
                warn!("VCS_SSH_KEY_PATH is empty, will use default SSH key discovery");
            }
        } else {
            info!("VCS_SSH_KEY_PATH not set, will use default SSH key discovery");
        }
        
        config
    }
    
    /// Get the repository URL if configured
    pub fn repository_url(&self) -> Option<&String> {
        self.repository_url.as_ref()
    }
    
    /// Get the repository path
    pub fn repository_path(&self) -> &PathBuf {
        &self.repository_path
    }
    
    /// Get the objects directory name
    pub fn objects_directory(&self) -> &String {
        &self.objects_directory
    }
    
    /// Check if debug logging is enabled
    pub fn is_debug_enabled(&self) -> bool {
        self.debug
    }
    
    /// Check if a repository URL is configured for cloning
    pub fn should_clone_repository(&self) -> bool {
        self.repository_url.is_some()
    }
    
    /// Get the git user name
    pub fn git_user_name(&self) -> &String {
        &self.git_user_name
    }
    
    /// Get the git user email
    pub fn git_user_email(&self) -> &String {
        &self.git_user_email
    }
    
    /// Get the SSH key path if configured
    pub fn ssh_key_path(&self) -> Option<&String> {
        self.ssh_key_path.as_ref()
    }
    
    /// Update SSH key path with validation
    pub fn update_ssh_key(&mut self, key_path: String) -> Result<(), String> {
        let path = std::path::Path::new(&key_path);
        
        // Check if path exists
        if !path.exists() {
            return Err(format!("SSH key path does not exist: {}", key_path));
        }
        
        // Check if it's a file
        if !path.is_file() {
            return Err(format!("SSH key path is not a file: {}", key_path));
        }
        
        // Check permissions
        if let Ok(metadata) = std::fs::metadata(&path) {
            use std::os::unix::fs::PermissionsExt;
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;
            if mode > 0o600 {
                return Err(format!("SSH key has overly permissive permissions: {:o}", mode));
            }
        }
        
        // Update the key path
        self.ssh_key_path = Some(key_path);
        info!("SSH key path updated successfully");
        Ok(())
    }
    
    /// Set git user name and email
    pub fn set_git_user(&mut self, name: String, email: String) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Git user name cannot be empty".to_string());
        }
        if email.trim().is_empty() {
            return Err("Git user email cannot be empty".to_string());
        }
        
        self.git_user_name = name.trim().to_string();
        self.git_user_email = email.trim().to_string();
        info!("Git user updated: {} <{}>", self.git_user_name, self.git_user_email);
        Ok(())
    }
    
    /// Clear SSH key path
    pub fn clear_ssh_key(&mut self) {
        self.ssh_key_path = None;
        info!("SSH key path cleared");
    }
    
    /// Get keys directory path
    pub fn keys_directory(&self) -> PathBuf {
        self.repository_path.join("keys")
    }
    
    /// Get meta directory path
    pub fn meta_directory(&self) -> PathBuf {
        self.repository_path.join("meta")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.repository_url.is_none());
        assert_eq!(config.repository_path, PathBuf::from("/game"));
        assert_eq!(config.objects_directory, "objects");
        assert!(!config.debug);
        assert_eq!(config.git_user_name, "vcs-worker");
        assert_eq!(config.git_user_email, "vcs-worker@system");
        assert!(config.ssh_key_path.is_none());
        assert!(!config.should_clone_repository());
    }
    
    #[test]
    fn test_config_from_env() {
        // Set environment variables
        unsafe {
            env::set_var("VCS_REPOSITORY_URL", "https://github.com/example/repo.git");
            env::set_var("VCS_REPOSITORY_PATH", "/custom/path");
            env::set_var("VCS_OBJECTS_DIRECTORY", "custom_objects");
            env::set_var("VCS_DEBUG", "true");
            env::set_var("VCS_GIT_USER_NAME", "test-user");
            env::set_var("VCS_GIT_USER_EMAIL", "test@example.com");
            env::set_var("VCS_SSH_KEY_PATH", "/path/to/key");
        }
        
        let config = Config::from_env();
        
        assert_eq!(config.repository_url, Some("https://github.com/example/repo.git".to_string()));
        assert_eq!(config.repository_path, PathBuf::from("/custom/path"));
        assert_eq!(config.objects_directory, "custom_objects");
        assert!(config.debug);
        assert_eq!(config.git_user_name, "test-user");
        assert_eq!(config.git_user_email, "test@example.com");
        assert_eq!(config.ssh_key_path, Some("/path/to/key".to_string()));
        assert!(config.should_clone_repository());
        
        // Clean up
        unsafe {
            env::remove_var("VCS_REPOSITORY_URL");
            env::remove_var("VCS_REPOSITORY_PATH");
            env::remove_var("VCS_OBJECTS_DIRECTORY");
            env::remove_var("VCS_DEBUG");
            env::remove_var("VCS_GIT_USER_NAME");
            env::remove_var("VCS_GIT_USER_EMAIL");
            env::remove_var("VCS_SSH_KEY_PATH");
        }
    }
    
    #[test]
    fn test_config_from_env_empty_values() {
        // Set empty environment variables
        unsafe {
            env::set_var("VCS_REPOSITORY_URL", "");
            env::set_var("VCS_REPOSITORY_PATH", "");
            env::set_var("VCS_OBJECTS_DIRECTORY", "");
            env::set_var("VCS_DEBUG", "false");
            env::set_var("VCS_GIT_USER_NAME", "");
            env::set_var("VCS_GIT_USER_EMAIL", "");
            env::set_var("VCS_SSH_KEY_PATH", "");
        }
        
        let config = Config::from_env();
        
        assert!(config.repository_url.is_none());
        assert_eq!(config.repository_path, PathBuf::from("/game")); // Should use default
        assert_eq!(config.objects_directory, "objects"); // Should use default
        assert!(!config.debug);
        assert_eq!(config.git_user_name, "vcs-worker"); // Should use default
        assert_eq!(config.git_user_email, "vcs-worker@system"); // Should use default
        assert!(config.ssh_key_path.is_none()); // Should use default
        assert!(!config.should_clone_repository());
        
        // Clean up
        unsafe {
            env::remove_var("VCS_REPOSITORY_URL");
            env::remove_var("VCS_REPOSITORY_PATH");
            env::remove_var("VCS_OBJECTS_DIRECTORY");
            env::remove_var("VCS_DEBUG");
            env::remove_var("VCS_GIT_USER_NAME");
            env::remove_var("VCS_GIT_USER_EMAIL");
            env::remove_var("VCS_SSH_KEY_PATH");
        }
    }
}
