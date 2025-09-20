use std::path::PathBuf;
use std::collections::HashMap;
use tracing::{info, error};
use moor_compiler::{ObjectDefinition, CompileOptions};
use moor_objdef::{ObjectDefinitionLoader, dump_object};
use moor_common::model::loader::LoaderInterface;

/// VCS operation types
#[derive(Debug, Clone)]
pub enum VcsOperation {    
    /// Add or update a MOO object file
    AddOrUpdateObject { 
        object_dump: String, 
        object_name: String,
    },
    
    /// Delete a tracked MOO object file
    DeleteObject { 
        object_name: String,
    },
    
    /// Create a commit with current changes
    Commit { 
        message: String,
        author_name: String,
        author_email: String,
    },
    
    /// Get repository status
    Status,
}

/// Result of a VCS operation
#[derive(Debug, Clone)]
pub enum VcsResult {
    /// Operation completed successfully
    Success { message: String },
    
    /// Operation completed with additional data
    SuccessWithData { message: String, data: Vec<String> },
    
    /// Operation failed
    Error { message: String },
}

/// Process VCS operations
pub struct VcsProcessor {
    git_repo: Option<crate::git_ops::GitRepository>,
}

impl VcsProcessor {
    pub fn new() -> Self {
        let mut processor = Self { git_repo: None };
        processor.initialize_repository();
        processor
    }
    
    /// Initialize the git repository in /game directory
    pub fn initialize_repository(&mut self) {
        let game_dir = PathBuf::from("/game");
        
        // Try to open existing repository first
        match crate::git_ops::GitRepository::open(&game_dir) {
            Ok(repo) => {
                info!("Opened existing git repository at /game");
                self.git_repo = Some(repo);
            }
            Err(_) => {
                // If no existing repo, try to initialize one
                match crate::git_ops::GitRepository::init(&game_dir) {
                    Ok(repo) => {
                        info!("Initialized new git repository at /game");
                        self.git_repo = Some(repo);
                    }
                    Err(e) => {
                        error!("Failed to initialize git repository at /game: {}", e);
                        // Continue without git repo - operations will fail gracefully
                    }
                }
            }
        }
    }
    
    /// Process a VCS operation
    pub fn process_operation(&mut self, operation: VcsOperation) -> VcsResult {
        match operation {            
            VcsOperation::AddOrUpdateObject { object_dump, object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.add_object(repo, object_dump, object_name)
                } else {
                    VcsResult::Error { 
                        message: "Git repository not available at /game".to_string() 
                    }
                }
            }
            
            VcsOperation::DeleteObject { object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.delete_object(repo, object_name, None)
                } else {
                    VcsResult::Error { 
                        message: "Git repository not available at /game".to_string() 
                    }
                }
            }
            
            VcsOperation::Commit { message, author_name, author_email } => {
                if let Some(ref repo) = self.git_repo {
                    self.create_commit(repo, message, author_name, author_email)
                } else {
                    VcsResult::Error { 
                        message: "Git repository not available at /game".to_string() 
                    }
                }
            }
            
            VcsOperation::Status => {
                if let Some(ref repo) = self.git_repo {
                    self.get_status(repo)
                } else {
                    VcsResult::Error { 
                        message: "Git repository not available at /game".to_string() 
                    }
                }
            }
        }
    }
    
    /// Load meta configuration from file, creating it with default values if it doesn't exist
    fn load_or_create_meta_config(&self, meta_path: &std::path::Path) -> Result<crate::meta_config::MetaConfig, Box<dyn std::error::Error>> {
        info!("Loading or creating meta config at: {:?}", meta_path);
        
        // Try to load existing meta config
        match crate::meta_config::MetaConfig::from_file(meta_path) {
            Ok(config) => {
                info!("Successfully loaded existing meta config from: {:?}", meta_path);
                Ok(config)
            },
            Err(e) => {
                info!("Meta config file doesn't exist or can't be loaded: {}, creating new one", e);
                
                // If file doesn't exist or can't be loaded, create a new default config
                let default_config = crate::meta_config::MetaConfig::new();
                
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
    fn add_object(
        &self, 
        repo: &crate::git_ops::GitRepository, 
        object_dump: String, 
        object_name: String,
    ) -> VcsResult {
        // Parse the MOO object using objdef's ObjectDefinitionLoader
        let mut object_def = match self.parse_object_dump(&object_dump) {
            Ok(obj) => obj,
            Err(e) => {
                error!("Failed to parse MOO object dump: {}", e);
                return VcsResult::Error { 
                    message: format!("Failed to parse MOO object dump: {}", e) 
                };
            }
        };
        
        // Load or create meta configuration
        let meta_relative_path = repo.meta_path(&object_name);
        let meta_full_path = repo.work_dir().join(&meta_relative_path);
        let meta_config = match self.load_or_create_meta_config(&meta_full_path) {
            Ok(config) => {
                info!("Successfully loaded/created meta config at: {:?}", meta_full_path);
                config
            },
            Err(e) => {
                error!("Failed to load or create meta config at {:?}: {}", meta_full_path, e);
                return VcsResult::Error { 
                    message: format!("Failed to load or create meta config at {:?}: {}", meta_full_path, e) 
                };
            }
        };
        
        // Apply meta configuration to filter out ignored properties/verbs
        self.apply_meta_config(&mut object_def, &meta_config);
        
        // Write the filtered object to file
        let filtered_dump = match self.to_dump(&object_def) {
            Ok(dump) => dump,
            Err(e) => {
                error!("Failed to convert object to dump format: {}", e);
                return VcsResult::Error { 
                    message: format!("Failed to convert object to dump format: {}", e) 
                };
            }
        };
        
        let mut object_full_path = repo.work_dir().join(&object_name);
        if let Some(ext) = object_full_path.extension() {
            if ext == "moo" {
                object_full_path.set_extension("moo");
            }
        } else {
            object_full_path.set_extension("moo");
        }

        if let Err(e) = repo.write_file(&object_full_path, &filtered_dump) {
            error!("Failed to write MOO object file: {}", e);
            return VcsResult::Error { 
                message: format!("Failed to write MOO object file: {}", e) 
            };
        }
        
        // Add files to git
        if let Err(e) = repo.add_file(&object_full_path) {
            error!("Failed to add MOO file to git: {}", e);
            return VcsResult::Error { 
                message: format!("Failed to add MOO file to git: {}", e) 
            };
        }
        
        info!("Attempting to add meta file to git: {:?}", meta_full_path);
        if let Err(e) = repo.add_file(&meta_full_path) {
            error!("Failed to add meta file to git at {:?}: {}", meta_full_path, e);
            return VcsResult::Error { 
                message: format!("Failed to add meta file to git at {:?}: {}", meta_full_path, e) 
            };
        }
        
        VcsResult::Success { 
            message: format!("Added object {} to staging area", object_name) 
        }
    }
    
    /// Delete a tracked MOO object file
    fn delete_object(
        &self, 
        repo: &crate::git_ops::GitRepository, 
        object_name: String,
        _commit_message: Option<String>,
    ) -> VcsResult {
        let meta_path = repo.meta_path(&object_name);
        
        // Remove files from git
        let object_full_path = repo.work_dir().join(&object_name);
        if let Err(e) = repo.remove_file(&object_full_path) {
            error!("Failed to remove MOO file from git: {}", e);
            return VcsResult::Error { 
                message: format!("Failed to remove MOO file from git: {}", e) 
            };
        }
        
        let meta_full_path = repo.work_dir().join(&meta_path);
        if repo.file_exists(&meta_full_path) {
            if let Err(e) = repo.remove_file(&meta_full_path) {
                error!("Failed to remove meta file from git: {}", e);
                return VcsResult::Error { 
                    message: format!("Failed to remove meta file from git: {}", e) 
                };
            }
        }
        
        VcsResult::Success { 
            message: format!("Deleted object {} from staging area", object_name) 
        }
    }
    
    /// Create a commit with current changes
    fn create_commit(
        &self, 
        repo: &crate::git_ops::GitRepository, 
        message: String,
        author_name: String,
        author_email: String,
    ) -> VcsResult {
        match repo.commit(&message, &author_name, &author_email) {
            Ok(_) => {
                info!("Created commit: {}", message);
                VcsResult::Success { 
                    message: format!("Created commit: {}", message) 
                }
            }
            Err(e) => {
                error!("Failed to create commit: {}", e);
                VcsResult::Error { 
                    message: format!("Failed to create commit: {}", e) 
                }
            }
        }
    }
    
    /// Get repository status
    fn get_status(&self, repo: &crate::git_ops::GitRepository) -> VcsResult {
        match repo.status() {
            Ok(status_lines) => {
                VcsResult::SuccessWithData { 
                    message: "Repository status".to_string(),
                    data: status_lines,
                }
            }
            Err(e) => {
                error!("Failed to get repository status: {}", e);
                VcsResult::Error { 
                    message: format!("Failed to get repository status: {}", e) 
                }
            }
        }
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
    fn apply_meta_config(&self, object_def: &mut ObjectDefinition, config: &crate::meta_config::MetaConfig) {
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
