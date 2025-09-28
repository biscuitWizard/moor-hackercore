use tracing::{info, error};
use crate::git::GitRepository;
use crate::utils::PathUtils;
use crate::vcs::object_handler::ObjectHandler;
use crate::git::operations::commit_ops::CommitOps;
use moor_common::tasks::WorkerError;

/// Structure to hold both object definition and original filename for stashing
pub struct StashedObject {
    pub object_def: Option<moor_compiler::ObjectDefinition>,
    pub original_filename: String,
    pub operation: StashOperation,
}

/// Type of operation that was stashed
#[derive(Debug, Clone)]
pub enum StashOperation {
    /// File was modified or is new
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed from old_name to new_name
    Renamed { old_name: String, new_name: String },
}

/// Stash operations for git repositories
pub struct StashOps;

impl StashOps {
    /// Stash current changes using ObjDef models with proper rename detection
    pub fn stash_changes(
        repo: &GitRepository,
        object_handler: &ObjectHandler,
    ) -> Result<Vec<StashedObject>, WorkerError> {
        info!("Stashing current changes using ObjDef models with rename detection");
        
        let mut stashed_objects = Vec::new();
        let objects_dir = object_handler.config.objects_directory();
        
        // Get git status to find all changed files (including deleted ones)
        let mut status_options = git2::StatusOptions::new();
        status_options.include_ignored(false);
        status_options.include_untracked(true);
        
        let statuses = match repo.repo().statuses(Some(&mut status_options)) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get git status: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to get git status: {}", e)));
            }
        };
        
        // First pass: collect all status entries and categorize them
        let mut added_files = Vec::new();
        let mut deleted_files = Vec::new();
        let mut modified_files = Vec::new();
        
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                // Only process .moo files in the objects directory
                if path.ends_with(".moo") && path.starts_with(objects_dir.as_str()) {
                    let status = entry.status();
                    
                    if let Some(object_name) = PathUtils::extract_object_name_from_path(path) {
                        if status.is_wt_new() || status.is_index_new() {
                            added_files.push((object_name, path.to_string()));
                        } else if status.is_wt_deleted() || status.is_index_deleted() {
                            deleted_files.push((object_name, path.to_string()));
                        } else if status.is_wt_modified() || status.is_index_modified() {
                            modified_files.push((object_name, path.to_string()));
                        }
                        // Note: We ignore git2's renamed status and detect renames ourselves
                    }
                }
            }
        }
        
        // Second pass: detect renames by comparing first lines (like StatusOps)
        let mut matched_deleted = std::collections::HashSet::new();
        let mut matched_added = std::collections::HashSet::new();
        
        for (added_object_name, added_path) in &added_files {
            // Read first line of added file
            let added_full_path = repo.work_dir().join(added_path);
            let added_first_line = match repo.read_file(&added_full_path) {
                Ok(content) => content.lines().next().unwrap_or("").to_string(),
                Err(_) => continue, // Skip if we can't read the file
            };
            
            // Look for a matching deleted file with the same first line
            for (deleted_object_name, deleted_path) in &deleted_files {
                if matched_deleted.contains(deleted_object_name) {
                    continue; // Already matched
                }
                
                // Read first line of deleted file from git history
                let deleted_first_line = match Self::get_file_first_line_from_history(repo, deleted_path) {
                    Ok(line) => line,
                    Err(_) => continue, // Skip if we can't read from history
                };
                
                if added_first_line == deleted_first_line && !added_first_line.is_empty() {
                    // Found a match! This is a rename
                    info!("Detected rename: {} -> {}", deleted_object_name, added_object_name);
                    
                    // Parse the new file content
                    if let Ok(content) = repo.read_file(&added_full_path) {
                        if let Ok(object_def) = object_handler.parse_object_dump(&content) {
                            stashed_objects.push(StashedObject {
                                object_def: Some(object_def),
                                original_filename: added_object_name.clone(),
                                operation: StashOperation::Renamed {
                                    old_name: deleted_object_name.clone(),
                                    new_name: added_object_name.clone(),
                                },
                            });
                            matched_deleted.insert(deleted_object_name.clone());
                            matched_added.insert(added_object_name.clone());
                            break;
                        }
                    }
                }
            }
        }
        
        // Third pass: handle remaining files (not part of renames)
        for (object_name, path) in &added_files {
            if !matched_added.contains(object_name) {
                // This is a new file (not a rename)
                let full_path = repo.work_dir().join(path);
                if let Ok(content) = repo.read_file(&full_path) {
                    if let Ok(object_def) = object_handler.parse_object_dump(&content) {
                        info!("Stashing new object: {}", object_name);
                        stashed_objects.push(StashedObject {
                            object_def: Some(object_def),
                            original_filename: object_name.clone(),
                            operation: StashOperation::Modified,
                        });
                    }
                }
            }
        }
        
        for (object_name, _path) in &deleted_files {
            if !matched_deleted.contains(object_name) {
                // This is a deleted file (not a rename)
                info!("Stashing deleted object: {}", object_name);
                stashed_objects.push(StashedObject {
                    object_def: None,
                    original_filename: object_name.clone(),
                    operation: StashOperation::Deleted,
                });
            }
        }
        
        for (object_name, path) in &modified_files {
            // This is a modified file
            let full_path = repo.work_dir().join(path);
            if let Ok(content) = repo.read_file(&full_path) {
                if let Ok(object_def) = object_handler.parse_object_dump(&content) {
                    info!("Stashing modified object: {}", object_name);
                    stashed_objects.push(StashedObject {
                        object_def: Some(object_def),
                        original_filename: object_name.clone(),
                        operation: StashOperation::Modified,
                    });
                }
            }
        }
        
        info!("Stashed {} objects (including {} renames)", stashed_objects.len(), matched_added.len());
        Ok(stashed_objects)
    }
    
    /// Replay stashed changes after pull
    pub fn replay_stashed_changes(
        repo: &GitRepository,
        object_handler: &ObjectHandler,
        stashed_objects: Vec<StashedObject>,
    ) -> Result<(), WorkerError> {
        info!("Replaying {} stashed objects", stashed_objects.len());
        
        for stashed_obj in stashed_objects {
            match stashed_obj.operation {
                StashOperation::Deleted => {
                    // Re-delete the file
                    let object_path = PathUtils::object_path(repo.work_dir(), &object_handler.config, &stashed_obj.original_filename);
                    
                    if object_path.exists() {
                        if let Err(e) = std::fs::remove_file(&object_path) {
                            error!("Failed to delete file {}: {}", stashed_obj.original_filename, e);
                            continue;
                        }
                        
                        // Remove from git index
                        if let Err(e) = repo.remove_file(&object_path) {
                            error!("Failed to remove file {} from git: {}", stashed_obj.original_filename, e);
                            continue;
                        }
                        
                        info!("Replayed deletion: {}", stashed_obj.original_filename);
                    }
                }
                StashOperation::Modified => {
                    // Handle modified/new files
                    if let Some(mut object_def) = stashed_obj.object_def {
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
                        object_handler.apply_meta_config(&mut object_def, &meta_config);
                        
                        // Convert back to dump format
                        match object_handler.to_dump(&object_def) {
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
                                
                                info!("Replayed object: {} (filename: {})", object_def.name, stashed_obj.original_filename);
                            }
                            Err(e) => {
                                error!("Failed to convert object {} to dump: {}", stashed_obj.original_filename, e);
                                continue;
                            }
                        }
                    }
                }
                StashOperation::Renamed { old_name, new_name } => {
                    // Handle renamed files: restore the old filename and delete the new one
                    if let Some(mut object_def) = stashed_obj.object_def {
                        // Load meta configuration using the old filename (the original filename)
                        let meta_full_path = PathUtils::object_meta_path(repo.work_dir(), &object_handler.config, &old_name);
                        let meta_config = match object_handler.load_or_create_meta_config(&meta_full_path) {
                            Ok(config) => config,
                            Err(e) => {
                                error!("Failed to load meta config for {}: {}", old_name, e);
                                continue;
                            }
                        };
                        
                        // Apply meta configuration filtering
                        object_handler.apply_meta_config(&mut object_def, &meta_config);
                        
                        // Convert back to dump format
                        match object_handler.to_dump(&object_def) {
                            Ok(filtered_dump) => {
                                // Delete the new file if it exists
                                let new_object_path = PathUtils::object_path(repo.work_dir(), &object_handler.config, &new_name);
                                if new_object_path.exists() {
                                    if let Err(e) = std::fs::remove_file(&new_object_path) {
                                        error!("Failed to delete new file {}: {}", new_name, e);
                                    } else {
                                        if let Err(e) = repo.remove_file(&new_object_path) {
                                            error!("Failed to remove new file {} from git: {}", new_name, e);
                                        }
                                    }
                                }
                                
                                // Write the filtered object using the old filename
                                let old_object_path = PathUtils::object_path(repo.work_dir(), &object_handler.config, &old_name);
                                if let Err(e) = repo.write_file(&old_object_path, &filtered_dump) {
                                    error!("Failed to write object {}: {}", old_name, e);
                                    continue;
                                }
                                
                                // Add to git
                                if let Err(e) = repo.add_file(&old_object_path) {
                                    error!("Failed to add object {} to git: {}", old_name, e);
                                    continue;
                                }
                                
                                info!("Replayed rename: {} -> {} (restored to {})", new_name, old_name, old_name);
                            }
                            Err(e) => {
                                error!("Failed to convert object {} to dump: {}", old_name, e);
                                continue;
                            }
                        }
                    }
                }
            }
        }
        
        info!("Successfully replayed all stashed changes");
        Ok(())
    }
    
    /// Get the first line of a file from git history (for deleted files)
    /// This is used to detect renames by comparing content
    fn get_file_first_line_from_history(repo: &GitRepository, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Get the HEAD commit to read the file from the last committed state
        let head_commit = CommitOps::get_head_commit(repo.repo())?;
        let tree = head_commit.tree()?;
        
        // Find the file in the tree
        if let Ok(entry) = tree.get_path(std::path::Path::new(path)) {
            // Get the blob (file content) from the tree entry
            let blob = repo.repo().find_blob(entry.id())?;
            let content = String::from_utf8_lossy(blob.content());
            
            // Return the first line
            Ok(content.lines().next().unwrap_or("").to_string())
        } else {
            Err("File not found in git history".into())
        }
    }
}
