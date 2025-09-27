use tracing::{info, error};
use moor_var::{Var, v_str};
use crate::git::GitRepository;
use crate::meta_config::MetaConfig;
use crate::config::Config;
use crate::utils::PathUtils;
use moor_common::tasks::WorkerError;

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
    ) -> Result<Var, WorkerError> {
        info!("Updating ignored properties for object: {}", object_name);
        
        let meta_full_path = PathUtils::object_meta_path(repo.work_dir(), &self.config, &object_name);
        
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
                Ok(v_str(&format!("Updated ignored properties for object: {}", object_name)))
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
    ) -> Result<Var, WorkerError> {
        info!("Updating ignored verbs for object: {}", object_name);
        
        let meta_full_path = PathUtils::object_meta_path(repo.work_dir(), &self.config, &object_name);
        
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
                Ok(v_str(&format!("Updated ignored verbs for object: {}", object_name)))
            }
            Err(e) => {
                error!("Failed to save meta config: {}", e);
                Err(WorkerError::RequestError(format!("Failed to save meta config: {}", e)))
            }
        }
    }
}
