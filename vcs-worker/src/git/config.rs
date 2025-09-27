use std::path::PathBuf;
use crate::config::Config;

/// Git-specific configuration and utilities
pub struct GitConfig {
    config: Config,
}

impl GitConfig {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Get the git user name
    pub fn git_user_name(&self) -> &String {
        self.config.git_user_name()
    }
    
    /// Get the git user email
    pub fn git_user_email(&self) -> &String {
        self.config.git_user_email()
    }
    
    /// Get the SSH key path if configured
    pub fn ssh_key_path(&self) -> Option<&String> {
        self.config.ssh_key_path()
    }
    
    /// Get keys directory path
    pub fn keys_directory(&self) -> PathBuf {
        self.config.keys_directory()
    }
    
    /// Get meta directory path
    pub fn meta_directory(&self) -> PathBuf {
        self.config.meta_directory()
    }
}
