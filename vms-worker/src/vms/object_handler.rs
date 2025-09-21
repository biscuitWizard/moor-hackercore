use std::path::PathBuf;
use std::collections::HashMap;
use tracing::{info, error};
use moor_compiler::{ObjectDefinition, CompileOptions};
use moor_objdef::dump_object;
use moor_var::{Var, v_str};
use crate::config::Config;
use crate::git_ops::GitRepository;
use crate::meta_config::MetaConfig;
use moor_common::tasks::WorkerError;

/// Handles MOO object parsing, filtering, and file operations
pub struct ObjectHandler {
    config: Config,
}

impl ObjectHandler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Get the path for a .meta file corresponding to a .moo file in the objects directory
    pub fn meta_path(&self, moo_path: &str) -> PathBuf {
        let objects_dir = self.config.objects_directory();
        let mut meta_path = PathBuf::from(objects_dir).join(moo_path);
        
        // Replace .moo extension with .meta
        if let Some(ext) = meta_path.extension() {
            if ext == "moo" {
                meta_path.set_extension("meta");
            }
        } else {
            meta_path.set_extension("meta");
        }
        
        meta_path
    }

    /// Load meta configuration from file, creating it with default values if it doesn't exist
    pub fn load_or_create_meta_config(&self, meta_path: &std::path::Path) -> Result<MetaConfig, Box<dyn std::error::Error>> {
        info!("Loading or creating meta config at: {:?}", meta_path);
        
        // Try to load existing meta config
        match MetaConfig::from_file(meta_path) {
            Ok(config) => {
                info!("Successfully loaded existing meta config from: {:?}", meta_path);
                Ok(config)
            },
            Err(e) => {
                info!("Meta config file doesn't exist or can't be loaded: {}, creating new one", e);
                
                // If file doesn't exist or can't be loaded, create a new default config
                let default_config = MetaConfig::new();
                
                // Create the directory if it doesn't exist
                if let Some(parent) = meta_path.parent() {
                    info!("Creating parent directory: {:?}", parent);
                    std::fs::create_dir_all(parent)?;
                }
                
                // Write the default config to file
                info!("Writing default meta config to: {:?}", meta_path);
                default_config.to_file(meta_path)?;
                
                info!("Successfully created new meta config file: {:?}", meta_path);
                Ok(default_config)
            }
        }
    }

    /// Add or update a MOO object file
    pub fn add_object(
        &self, 
        repo: &GitRepository, 
        object_dump: String, 
        object_name: String,
    ) -> Result<Vec<Var>, WorkerError> {
        // Parse the MOO object using objdef's ObjectDefinitionLoader
        let mut object_def = match self.parse_object_dump(&object_dump) {
            Ok(obj) => obj,
            Err(e) => {
                error!("Failed to parse MOO object dump: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to parse MOO object dump: {}", e)));
            }
        };
        
        // Load or create meta configuration
        let meta_relative_path = self.meta_path(&object_name);
        let meta_full_path = repo.work_dir().join(&meta_relative_path);
        let meta_config = match self.load_or_create_meta_config(&meta_full_path) {
            Ok(config) => {
                info!("Successfully loaded/created meta config at: {:?}", meta_full_path);
                config
            },
            Err(e) => {
                error!("Failed to load or create meta config at {:?}: {}", meta_full_path, e);
                return Err(WorkerError::RequestError(format!("Failed to load or create meta config at {:?}: {}", meta_full_path, e)));
            }
        };
        
        // Apply meta configuration to filter out ignored properties/verbs
        self.apply_meta_config(&mut object_def, &meta_config);
        
        // Write the filtered object to file
        let filtered_dump = match self.to_dump(&object_def) {
            Ok(dump) => dump,
            Err(e) => {
                error!("Failed to convert object to dump format: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to convert object to dump format: {}", e)));
            }
        };
        
        // Create the objects directory path
        let objects_dir = self.config.objects_directory();
        let mut object_full_path = repo.work_dir().join(objects_dir).join(&object_name);
        if let Some(ext) = object_full_path.extension() {
            if ext == "moo" {
                object_full_path.set_extension("moo");
            }
        } else {
            object_full_path.set_extension("moo");
        }

        if let Err(e) = repo.write_file(&object_full_path, &filtered_dump) {
            error!("Failed to write MOO object file: {}", e);
            return Err(WorkerError::RequestError(format!("Failed to write MOO object file: {}", e)));
        }
        
        // Add files to git
        if let Err(e) = repo.add_file(&object_full_path) {
            error!("Failed to add MOO file to git: {}", e);
            return Err(WorkerError::RequestError(format!("Failed to add MOO file to git: {}", e)));
        }
        
        info!("Attempting to add meta file to git: {:?}", meta_full_path);
        if let Err(e) = repo.add_file(&meta_full_path) {
            error!("Failed to add meta file to git at {:?}: {}", meta_full_path, e);
            return Err(WorkerError::RequestError(format!("Failed to add meta file to git at {:?}: {}", meta_full_path, e)));
        }
        
        Ok(vec![v_str(&format!("Added object {} to staging area", object_name))])
    }
    
    /// Delete a tracked MOO object file
    pub fn delete_object(
        &self, 
        repo: &GitRepository, 
        object_name: String,
        _commit_message: Option<String>,
    ) -> Result<Vec<Var>, WorkerError> {
        let meta_path = self.meta_path(&object_name);
        
        // Remove files from git
        let objects_dir = self.config.objects_directory();
        let object_full_path = repo.work_dir().join(objects_dir).join(&object_name);
        if let Err(e) = repo.remove_file(&object_full_path) {
            error!("Failed to remove MOO file from git: {}", e);
            return Err(WorkerError::RequestError(format!("Failed to remove MOO file from git: {}", e)));
        }
        
        let meta_full_path = repo.work_dir().join(&meta_path);
        if repo.file_exists(&meta_full_path) {
            if let Err(e) = repo.remove_file(&meta_full_path) {
                error!("Failed to remove meta file from git: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to remove meta file from git: {}", e)));
            }
        }
        
        Ok(vec![v_str(&format!("Deleted object {} from staging area", object_name))])
    }
    
    /// Parse a MOO object dump string into an ObjectDefinition using objdef
    fn parse_object_dump(&self, dump: &str) -> Result<ObjectDefinition, Box<dyn std::error::Error>> {
        // Use the compiler directly to parse the object definition
        use moor_compiler::{compile_object_definitions, ObjFileContext};
        
        let mut context = ObjFileContext::new();
        let compiled_defs = compile_object_definitions(
            dump,
            &CompileOptions::default(),
            &mut context,
        )?;
        
        // Ensure we got exactly one object
        if compiled_defs.len() != 1 {
            return Err(format!("Expected single object definition, but got {}", compiled_defs.len()).into());
        }
        
        Ok(compiled_defs.into_iter().next().unwrap())
    }
    
    /// Apply meta configuration filtering to an ObjectDefinition
    fn apply_meta_config(&self, object_def: &mut ObjectDefinition, config: &MetaConfig) {
        // Filter property definitions
        object_def.property_definitions.retain(|prop| {
            !config.is_property_ignored(&prop.name.to_string())
        });
        
        // Filter property overrides
        object_def.property_overrides.retain(|prop| {
            !config.is_property_ignored(&prop.name.to_string())
        });
        
        // Filter verbs
        object_def.verbs.retain(|verb| {
            let verb_name = verb.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ");
            !config.is_verb_ignored(&verb_name)
        });
    }
    
    /// Convert ObjectDefinition back to MOO dump format using objdef
    fn to_dump(&self, object_def: &ObjectDefinition) -> Result<String, Box<dyn std::error::Error>> {
        // Create a simple index for object names
        let mut index_names = HashMap::new();
        index_names.insert(object_def.oid, object_def.name.clone());
        
        // Use the existing dump_object function from objdef
        let lines = dump_object(&index_names, object_def)?;
        Ok(lines.join("\n"))
    }
}
