use tracing::{info, error};
use moor_var::{Var, v_str};
use crate::git_ops::GitRepository;
use crate::meta_config::MetaConfig;
use crate::config::Config;
use moor_common::tasks::WorkerError;
use std::path::PathBuf;

/// Handles meta file operations for MOO objects
pub struct MetaHandler {
    config: Config,
}

impl MetaHandler {
    /// Create a new MetaHandler
    pub fn new(config: Config) -> Self {
        Self { config }
    }
    
    /// Update ignored properties for a specific object
    pub fn update_ignored_properties(
        &self,
        repo: &GitRepository,
        object_name: String,
        properties: Vec<String>,
    ) -> Result<Vec<Var>, WorkerError> {
        info!("Updating ignored properties for object: {}", object_name);
        
        let meta_path = self.get_meta_path(&object_name);
        let meta_full_path = repo.work_dir().join(&meta_path);
        
        // Load existing meta config or create new one
        let mut meta_config = match MetaConfig::from_file(&meta_full_path) {
            Ok(config) => {
                info!("Loaded existing meta config from: {:?}", meta_full_path);
                config
            }
            Err(_) => {
                info!("Creating new meta config at: {:?}", meta_full_path);
                MetaConfig::new()
            }
        };
        
        // Update ignored properties
        for property in properties {
            meta_config.ignore_property(property);
        }
        
        // Save the updated config
        match meta_config.to_file(&meta_full_path) {
            Ok(_) => {
                info!("Successfully updated ignored properties for object: {}", object_name);
                Ok(vec![v_str(&format!("Updated ignored properties for object: {}", object_name))])
            }
            Err(e) => {
                error!("Failed to save meta config: {}", e);
                Err(WorkerError::RequestError(format!("Failed to save meta config: {}", e)))
            }
        }
    }
    
    /// Update ignored verbs for a specific object
    pub fn update_ignored_verbs(
        &self,
        repo: &GitRepository,
        object_name: String,
        verbs: Vec<String>,
    ) -> Result<Vec<Var>, WorkerError> {
        info!("Updating ignored verbs for object: {}", object_name);
        
        let meta_path = self.get_meta_path(&object_name);
        let meta_full_path = repo.work_dir().join(&meta_path);
        
        // Load existing meta config or create new one
        let mut meta_config = match MetaConfig::from_file(&meta_full_path) {
            Ok(config) => {
                info!("Loaded existing meta config from: {:?}", meta_full_path);
                config
            }
            Err(_) => {
                info!("Creating new meta config at: {:?}", meta_full_path);
                MetaConfig::new()
            }
        };
        
        // Update ignored verbs
        for verb in verbs {
            meta_config.ignore_verb(verb);
        }
        
        // Save the updated config
        match meta_config.to_file(&meta_full_path) {
            Ok(_) => {
                info!("Successfully updated ignored verbs for object: {}", object_name);
                Ok(vec![v_str(&format!("Updated ignored verbs for object: {}", object_name))])
            }
            Err(e) => {
                error!("Failed to save meta config: {}", e);
                Err(WorkerError::RequestError(format!("Failed to save meta config: {}", e)))
            }
        }
    }
    
    /// Get the meta file path for an object
    fn get_meta_path(&self, object_name: &str) -> PathBuf {
        let mut path = PathBuf::from(self.config.meta_directory());
        path.push(object_name);
        path.set_extension("meta");
        path
    }
}
