use std::env;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// Configuration for the VMS Worker
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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repository_url: None,
            repository_path: PathBuf::from("/game"),
            objects_directory: "objects".to_string(),
            debug: false,
        }
    }
}

impl Config {
    /// Create a new Config instance by reading from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Read repository URL from environment
        if let Ok(repo_url) = env::var("VMS_REPOSITORY_URL") {
            if !repo_url.trim().is_empty() {
                config.repository_url = Some(repo_url.trim().to_string());
                info!("Repository URL configured from VMS_REPOSITORY_URL: {}", repo_url);
            } else {
                warn!("VMS_REPOSITORY_URL is empty, will initialize empty repository");
            }
        } else {
            info!("VMS_REPOSITORY_URL not set, will initialize empty repository");
        }
        
        // Read repository path from environment
        if let Ok(repo_path) = env::var("VMS_REPOSITORY_PATH") {
            if !repo_path.trim().is_empty() {
                config.repository_path = PathBuf::from(repo_path.trim());
                info!("Repository path configured from VMS_REPOSITORY_PATH: {:?}", config.repository_path);
            } else {
                warn!("VMS_REPOSITORY_PATH is empty, using default: {:?}", config.repository_path);
            }
        } else {
            info!("VMS_REPOSITORY_PATH not set, using default: {:?}", config.repository_path);
        }
        
        // Read objects directory from environment
        if let Ok(objects_dir) = env::var("VMS_OBJECTS_DIRECTORY") {
            if !objects_dir.trim().is_empty() {
                config.objects_directory = objects_dir.trim().to_string();
                info!("Objects directory configured from VMS_OBJECTS_DIRECTORY: {}", config.objects_directory);
            } else {
                warn!("VMS_OBJECTS_DIRECTORY is empty, using default: {}", config.objects_directory);
            }
        } else {
            info!("VMS_OBJECTS_DIRECTORY not set, using default: {}", config.objects_directory);
        }
        
        // Read debug flag from environment
        if let Ok(debug_str) = env::var("VMS_DEBUG") {
            config.debug = debug_str.to_lowercase() == "true" || debug_str == "1";
            info!("Debug logging configured from VMS_DEBUG: {}", config.debug);
        } else {
            info!("VMS_DEBUG not set, debug logging disabled");
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
        assert!(!config.should_clone_repository());
    }
    
    #[test]
    fn test_config_from_env() {
        // Set environment variables
        env::set_var("VMS_REPOSITORY_URL", "https://github.com/example/repo.git");
        env::set_var("VMS_REPOSITORY_PATH", "/custom/path");
        env::set_var("VMS_OBJECTS_DIRECTORY", "custom_objects");
        env::set_var("VMS_DEBUG", "true");
        
        let config = Config::from_env();
        
        assert_eq!(config.repository_url, Some("https://github.com/example/repo.git".to_string()));
        assert_eq!(config.repository_path, PathBuf::from("/custom/path"));
        assert_eq!(config.objects_directory, "custom_objects");
        assert!(config.debug);
        assert!(config.should_clone_repository());
        
        // Clean up
        env::remove_var("VMS_REPOSITORY_URL");
        env::remove_var("VMS_REPOSITORY_PATH");
        env::remove_var("VMS_OBJECTS_DIRECTORY");
        env::remove_var("VMS_DEBUG");
    }
    
    #[test]
    fn test_config_from_env_empty_values() {
        // Set empty environment variables
        env::set_var("VMS_REPOSITORY_URL", "");
        env::set_var("VMS_REPOSITORY_PATH", "");
        env::set_var("VMS_OBJECTS_DIRECTORY", "");
        env::set_var("VMS_DEBUG", "false");
        
        let config = Config::from_env();
        
        assert!(config.repository_url.is_none());
        assert_eq!(config.repository_path, PathBuf::from("/game")); // Should use default
        assert_eq!(config.objects_directory, "objects"); // Should use default
        assert!(!config.debug);
        assert!(!config.should_clone_repository());
        
        // Clean up
        env::remove_var("VMS_REPOSITORY_URL");
        env::remove_var("VMS_REPOSITORY_PATH");
        env::remove_var("VMS_OBJECTS_DIRECTORY");
        env::remove_var("VMS_DEBUG");
    }
}
