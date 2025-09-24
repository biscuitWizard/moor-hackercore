use std::path::PathBuf;
use std::collections::{HashMap, HashSet};
use tracing::{info, error};
use moor_compiler::{ObjectDefinition, CompileOptions};
use moor_objdef::dump_object;
use moor_var::{Var, v_str, v_map};
use crate::config::Config;
use crate::git_ops::GitRepository;
use crate::meta_config::MetaConfig;
use moor_common::tasks::WorkerError;

/// Information about a .moo file
#[derive(Debug, Clone)]
struct FileInfo {
    filename: String,
    byte_size: usize,
}

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

    /// List all .moo objects with dependency ordering
    pub fn list_objects(&self, repo: &GitRepository) -> Result<Vec<Var>, WorkerError> {
        let objects_dir = self.config.objects_directory();
        let objects_path = repo.work_dir().join(objects_dir);
        
        // Find all .moo files
        let moo_files = match self.find_moo_files(&objects_path) {
            Ok(files) => files,
            Err(e) => {
                error!("Failed to find .moo files: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to find .moo files: {}", e)));
            }
        };
        
        if moo_files.is_empty() {
            return Ok(vec![v_str("No .moo files found")]);
        }
        
        // Parse all .moo files and collect object definitions
        let mut objects = Vec::new();
        for file_path in &moo_files {
            match self.parse_moo_file(repo, file_path) {
                Ok(mut obj_defs) => objects.append(&mut obj_defs),
                Err(e) => {
                    error!("Failed to parse {}: {}", file_path.display(), e);
                    // Continue with other files instead of failing completely
                }
            }
        }
        
        if objects.is_empty() {
            return Ok(vec![v_str("No valid object definitions found in .moo files")]);
        }
        
        // Sort objects by dependency chain (reversed)
        let sorted_objects = match self.sort_by_dependencies(objects) {
            Ok(mut sorted) => {
                sorted.reverse(); // Reverse the dependency order
                sorted
            },
            Err(e) => {
                error!("Failed to sort objects by dependencies: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to sort objects by dependencies: {}", e)));
            }
        };
        
        // Convert to result format
        let mut result = Vec::new();
        for (obj_def, file_info) in sorted_objects.iter().zip(self.get_file_info_for_objects(&moo_files, &sorted_objects)) {
            let obj_info = v_map(&[
                (v_str("oid"), v_str(&obj_def.oid.to_string())),
                (v_str("name"), v_str(&obj_def.name)),
                (v_str("parent"), v_str(&obj_def.parent.to_string())),
                (v_str("owner"), v_str(&obj_def.owner.to_string())),
                (v_str("location"), v_str(&obj_def.location.to_string())),
                (v_str("verb_count"), v_str(&obj_def.verbs.len().to_string())),
                (v_str("property_def_count"), v_str(&obj_def.property_definitions.len().to_string())),
                (v_str("property_override_count"), v_str(&obj_def.property_overrides.len().to_string())),
                (v_str("filename"), v_str(&file_info.filename)),
                (v_str("byte_size"), v_str(&file_info.byte_size.to_string())),
            ]);
            result.push(obj_info);
        }
        
        info!("Listed {} objects in reverse dependency order", result.len());
        Ok(result)
    }

    /// Get the full dump contents for specified object names (returns just content strings)
    pub fn get_objects(&self, repo: &GitRepository, object_names: Vec<String>) -> Result<Vec<Var>, WorkerError> {
        info!("GetObjects called with {} object names: {:?}", object_names.len(), object_names);
        
        if object_names.is_empty() {
            info!("No object names provided, returning empty result");
            return Ok(vec![v_str("No object names provided")]);
        }

        let objects_dir = self.config.objects_directory();
        let objects_path = repo.work_dir().join(objects_dir);
        info!("Looking for object files in directory: {:?}", objects_path);

        let mut results = Vec::new();
        let mut found_count = 0;

        // Process each requested object name
        for (index, object_name) in object_names.iter().enumerate() {
            info!("Processing object #{}: '{}'", index + 1, object_name);
            
            // Map object name to filename by adding .moo extension
            let filename = format!("{}.moo", object_name);
            let file_path = objects_path.join(&filename);
            
            info!("Looking for file: {:?}", file_path);
            
            // Check if file exists and read its content
            if repo.file_exists(&file_path) {
                match repo.read_file(&file_path) {
                    Ok(content) => {
                        info!("Successfully read file '{}', {} bytes", filename, content.len());
                        results.push(v_str(&content));
                        found_count += 1;
                    }
                    Err(e) => {
                        error!("Failed to read file '{}': {}", filename, e);
                        results.push(v_str("")); // Add empty string for failed read
                    }
                }
            } else {
                info!("File '{}' not found", filename);
                results.push(v_str("")); // Add empty string for missing file
            }
        }

        info!("GetObjects completed: Retrieved {} files out of {} requested", found_count, object_names.len());
        
        Ok(results)
    }

    /// Find all .moo files in the objects directory
    fn find_moo_files(&self, objects_path: &std::path::Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
        use std::fs;
        
        if !objects_path.exists() {
            return Ok(Vec::new());
        }
        
        let mut moo_files = Vec::new();
        
        fn collect_moo_files(dir: &std::path::Path, files: &mut Vec<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.is_dir() {
                    // Recursively search subdirectories
                    collect_moo_files(&path, files)?;
                } else if path.extension().and_then(|s| s.to_str()) == Some("moo") {
                    files.push(path);
                }
            }
            Ok(())
        }
        
        collect_moo_files(objects_path, &mut moo_files)?;
        moo_files.sort(); // Sort for consistent ordering
        Ok(moo_files)
    }

    /// Parse a single .moo file and return object definitions
    fn parse_moo_file(&self, repo: &GitRepository, file_path: &std::path::Path) -> Result<Vec<ObjectDefinition>, Box<dyn std::error::Error>> {
        // Read the file content
        let content = repo.read_file(file_path)?;
        
        // Use the existing parse function from object_handler
        use moor_compiler::{compile_object_definitions, ObjFileContext};
        
        let mut context = ObjFileContext::new();
        let compiled_defs = compile_object_definitions(
            &content,
            &CompileOptions::default(),
            &mut context,
        )?;
        
        Ok(compiled_defs)
    }

    /// Get file information for objects (filename and byte size)
    fn get_file_info_for_objects(&self, moo_files: &[PathBuf], objects: &[ObjectDefinition]) -> Vec<FileInfo> {
        let mut file_info_map = HashMap::new();
        
        // Build a map of object names to file info by parsing each file
        for file_path in moo_files {
            if let Ok(obj_defs) = self.parse_moo_file_internal(file_path) {
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let filename = file_path.file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    
                    // Remove .moo extension from filename
                    let filename_without_ext = if filename.ends_with(".moo") {
                        filename.trim_end_matches(".moo").to_string()
                    } else {
                        filename
                    };
                    
                    let byte_size = content.len();
                    
                    for obj_def in obj_defs {
                        file_info_map.insert(obj_def.oid, FileInfo {
                            filename: filename_without_ext.clone(),
                            byte_size,
                        });
                    }
                }
            }
        }
        
        // Return file info for each object in the same order
        objects.iter()
            .map(|obj| file_info_map.get(&obj.oid).cloned().unwrap_or_else(|| FileInfo {
                filename: "unknown".to_string(),
                byte_size: 0,
            }))
            .collect()
    }

    /// Internal parse function that doesn't require a GitRepository (for file info)
    fn parse_moo_file_internal(&self, file_path: &std::path::Path) -> Result<Vec<ObjectDefinition>, Box<dyn std::error::Error>> {
        // Read the file content directly from filesystem
        let content = std::fs::read_to_string(file_path)?;
        
        // Use the existing parse function
        use moor_compiler::{compile_object_definitions, ObjFileContext};
        
        let mut context = ObjFileContext::new();
        let compiled_defs = compile_object_definitions(
            &content,
            &CompileOptions::default(),
            &mut context,
        )?;
        
        Ok(compiled_defs)
    }

    /// Sort objects by dependency chain (parents before children)
    fn sort_by_dependencies(&self, objects: Vec<ObjectDefinition>) -> Result<Vec<ObjectDefinition>, Box<dyn std::error::Error>> {
        // Create adjacency list for dependencies
        let mut dependencies: HashMap<moor_var::Obj, Vec<moor_var::Obj>> = HashMap::new();
        let mut all_objects = HashSet::new();
        
        // First pass: collect all object IDs
        for obj in &objects {
            all_objects.insert(obj.oid);
            dependencies.insert(obj.oid, Vec::new());
        }
        
        // Build dependency graph
        for obj in &objects {
            let parent = obj.parent;
            if parent != obj.oid && all_objects.contains(&parent) {
                dependencies.entry(parent).or_insert_with(Vec::new).push(obj.oid);
            }
        }
        
        // Create index map for quick lookup
        let obj_index: HashMap<moor_var::Obj, usize> = objects
            .iter()
            .enumerate()
            .map(|(i, obj)| (obj.oid, i))
            .collect();
        
        // Topological sort using DFS
        let mut visited = HashSet::new();
        let mut temp_visited = HashSet::new();
        let mut sorted_indices = Vec::new();
        
        fn visit(
            obj_id: moor_var::Obj,
            dependencies: &HashMap<moor_var::Obj, Vec<moor_var::Obj>>,
            visited: &mut HashSet<moor_var::Obj>,
            temp_visited: &mut HashSet<moor_var::Obj>,
            sorted_indices: &mut Vec<usize>,
            obj_index: &HashMap<moor_var::Obj, usize>,
        ) -> Result<(), String> {
            if temp_visited.contains(&obj_id) {
                return Err(format!("Circular dependency detected involving object {}", obj_id));
            }
            
            if visited.contains(&obj_id) {
                return Ok(());
            }
            
            temp_visited.insert(obj_id);
            
            // Visit dependencies first (children)
            if let Some(children) = dependencies.get(&obj_id) {
                for &child_id in children {
                    visit(child_id, dependencies, visited, temp_visited, sorted_indices, obj_index)?;
                }
            }
            
            temp_visited.remove(&obj_id);
            visited.insert(obj_id);
            
            // Add this object to result (parents come before children)
            if let Some(&index) = obj_index.get(&obj_id) {
                sorted_indices.push(index);
            }
            
            Ok(())
        }
        
        // Visit all objects
        for &obj_id in &all_objects {
            if !visited.contains(&obj_id) {
                visit(obj_id, &dependencies, &mut visited, &mut temp_visited, &mut sorted_indices, &obj_index)?;
            }
        }
        
        // Reorder objects according to sorted indices
        let mut result = Vec::new();
        for index in sorted_indices {
            if let Some(obj) = objects.get(index) {
                result.push(obj.oid);
            }
        }
        
        // Alternative approach: sort the objects in place by creating a new vector with the right order
        let mut final_objects = Vec::new();
        let mut remaining_objects = objects;
        
        for obj_id in result {
            if let Some(pos) = remaining_objects.iter().position(|obj| obj.oid == obj_id) {
                final_objects.push(remaining_objects.remove(pos));
            }
        }
        
        // Add any remaining objects that weren't in the dependency chain
        final_objects.extend(remaining_objects);
        
        Ok(final_objects)
    }
}
