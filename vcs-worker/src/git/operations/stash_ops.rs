use tracing::{info, error};
use crate::git::GitRepository;
use crate::utils::PathUtils;
use crate::vcs::object_handler::ObjectHandler;
use moor_common::tasks::WorkerError;

/// Structure to hold both object definition and original filename for stashing
pub struct StashedObject {
    pub object_def: moor_compiler::ObjectDefinition,
    pub original_filename: String,
}

/// Stash operations for git repositories
pub struct StashOps;

impl StashOps {
    /// Stash current changes using ObjDef models
    pub fn stash_changes(
        repo: &GitRepository,
        object_handler: &ObjectHandler,
    ) -> Result<Vec<StashedObject>, WorkerError> {
        info!("Stashing current changes using ObjDef models");
        
        let mut stashed_objects = Vec::new();
        let objects_dir = object_handler.config.objects_directory();
        let objects_path = repo.work_dir().join(objects_dir);
        
        // Find all .moo files that have changes
        let moo_files = match object_handler.find_moo_files(&objects_path) {
            Ok(files) => files,
            Err(e) => {
                error!("Failed to find .moo files: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to find .moo files: {}", e)));
            }
        };
        
        // Load each object that has changes
        for file_path in moo_files {
            if let Some(object_name) = PathUtils::extract_object_name_from_path(file_path.to_str().unwrap_or("")) {
                // Convert absolute path to relative path for git status checking
                let relative_path = match file_path.strip_prefix(repo.work_dir()) {
                    Ok(rel_path) => rel_path,
                    Err(_) => {
                        error!("Failed to convert absolute path to relative: {}", file_path.display());
                        continue;
                    }
                };
                
                // Check if this file has changes
                if repo.file_has_changes(&relative_path) {
                    match repo.read_file(&file_path) {
                        Ok(content) => {
                            match object_handler.parse_object_dump(&content) {
                                Ok(object_def) => {
                                    info!("Stashing object: {} (filename: {})", object_name, file_path.file_name().unwrap_or_default().to_string_lossy());
                                    stashed_objects.push(StashedObject {
                                        object_def,
                                        original_filename: object_name.clone(),
                                    });
                                }
                                Err(e) => {
                                    error!("Failed to parse object dump for {}: {}", object_name, e);
                                    // Continue with other objects
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to read file {}: {}", file_path.display(), e);
                            // Continue with other files
                        }
                    }
                }
            }
        }
        
        info!("Stashed {} objects", stashed_objects.len());
        Ok(stashed_objects)
    }
    
    /// Replay stashed changes after pull
    pub fn replay_stashed_changes(
        repo: &GitRepository,
        object_handler: &ObjectHandler,
        stashed_objects: Vec<StashedObject>,
    ) -> Result<(), WorkerError> {
        info!("Replaying {} stashed objects", stashed_objects.len());
        
        for mut stashed_obj in stashed_objects {
            // Load meta configuration using the original filename
            let meta_full_path = PathUtils::object_meta_path(repo.work_dir(), &object_handler.config, &stashed_obj.original_filename);
            let meta_config = match object_handler.load_or_create_meta_config(&meta_full_path) {
                Ok(config) => config,
                Err(e) => {
                    error!("Failed to load meta config for {}: {}", stashed_obj.original_filename, e);
                    continue;
                }
            };
            
            // Apply meta configuration filtering
            object_handler.apply_meta_config(&mut stashed_obj.object_def, &meta_config);
            
            // Convert back to dump format
            match object_handler.to_dump(&stashed_obj.object_def) {
                Ok(filtered_dump) => {
                    // Write the filtered object using the original filename
                    let object_path = PathUtils::object_path(repo.work_dir(), &object_handler.config, &stashed_obj.original_filename);
                    
                    if let Err(e) = repo.write_file(&object_path, &filtered_dump) {
                        error!("Failed to write object {}: {}", stashed_obj.original_filename, e);
                        continue;
                    }
                    
                    // Add to git
                    if let Err(e) = repo.add_file(&object_path) {
                        error!("Failed to add object {} to git: {}", stashed_obj.original_filename, e);
                        continue;
                    }
                    
                    info!("Replayed object: {} (filename: {})", stashed_obj.object_def.name, stashed_obj.original_filename);
                }
                Err(e) => {
                    error!("Failed to convert object {} to dump: {}", stashed_obj.original_filename, e);
                    continue;
                }
            }
        }
        
        info!("Successfully replayed all stashed changes");
        Ok(())
    }
}
