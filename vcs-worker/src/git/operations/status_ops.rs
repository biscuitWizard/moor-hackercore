use std::collections::HashSet;
use tracing::{info, error, warn};
use git2::{Repository, ResetType};

/// Status operations for git repositories
pub struct StatusOps;

impl StatusOps {
    /// Check if the repository has any changes
    pub fn has_changes(repo: &Repository) -> Result<bool, Box<dyn std::error::Error>> {
        let mut index = repo.index()?;
        let head_commit = super::commit_ops::CommitOps::get_head_commit(repo).ok();
        
        if let Some(head) = head_commit {
            let head_tree = head.tree()?;
            let diff = repo.diff_tree_to_index(Some(&head_tree), Some(&mut index), None)?;
            Ok(diff.deltas().len() > 0)
        } else {
            // No commits yet, check if index has any entries
            Ok(index.len() > 0)
        }
    }
    
    /// Get the status of files in the repository
    pub fn get_status(repo: &Repository, work_dir: &std::path::Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut status_options = git2::StatusOptions::new();
        status_options.include_ignored(false);
        status_options.include_untracked(true);
        
        let statuses = repo.statuses(Some(&mut status_options))?;
        let mut result = Vec::new();
        
        // First pass: collect all status entries and categorize them
        let mut added_files = Vec::new();
        let mut deleted_files = Vec::new();
        
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("unknown");
            
            if status.is_wt_new() || status.is_index_new() {
                added_files.push((path.to_string(), entry));
            } else if status.is_wt_deleted() || status.is_index_deleted() {
                deleted_files.push((path.to_string(), entry));
            } else if status.is_wt_renamed() || status.is_index_renamed() {
                result.push(format!("Renamed: {}", path));
            } else if status.is_index_modified() || status.is_wt_modified() {
                result.push(format!("Modified: {}", path));
            } else if status.is_ignored() {
                continue;
            } else {
                result.push(format!("Unknown: {}", path));
            }
        }
        
        // Second pass: detect renames by comparing first lines
        let mut matched_deleted = HashSet::new();
        let mut matched_added = HashSet::new();
        
        for (added_path, _) in &added_files {
            // Read first line of added file
            let added_first_line = match super::file_ops::FileOps::read_file(work_dir, std::path::Path::new(added_path)) {
                Ok(content) => content.lines().next().unwrap_or("").to_string(),
                Err(_) => continue, // Skip if we can't read the file
            };
            
            // Look for a matching deleted file with the same first line
            for (deleted_path, _) in &deleted_files {
                if matched_deleted.contains(deleted_path) {
                    continue; // Already matched
                }
                
                // Read first line of deleted file from git history
                let deleted_first_line = match Self::get_file_first_line_from_history(repo, deleted_path) {
                    Ok(line) => line,
                    Err(_) => continue, // Skip if we can't read from history
                };
                
                if added_first_line == deleted_first_line && !added_first_line.is_empty() {
                    // Found a match! This is a rename
                    result.push(format!("Renamed: {} -> {}", deleted_path, added_path));
                    matched_deleted.insert(deleted_path.clone());
                    matched_added.insert(added_path.clone());
                    break;
                }
            }
        }
        
        // Add remaining added files (not matched as renames)
        for (added_path, _) in &added_files {
            if !matched_added.contains(added_path) {
                result.push(format!("Added: {}", added_path));
            }
        }
        
        // Add remaining deleted files (not matched as renames)
        for (deleted_path, _) in &deleted_files {
            if !matched_deleted.contains(deleted_path) {
                result.push(format!("Deleted: {}", deleted_path));
            }
        }
        
        Ok(result)
    }
    
    /// Get the first line of a file from git history (for deleted files)
    fn get_file_first_line_from_history(repo: &Repository, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Get the HEAD commit to read the file from the last committed state
        let head_commit = super::commit_ops::CommitOps::get_head_commit(repo)?;
        let tree = head_commit.tree()?;
        
        // Find the file in the tree
        if let Ok(entry) = tree.get_path(std::path::Path::new(path)) {
            // Get the blob (file content) from the tree entry
            let blob = repo.find_blob(entry.id())?;
            let content = String::from_utf8_lossy(blob.content());
            
            // Return the first line
            Ok(content.lines().next().unwrap_or("").to_string())
        } else {
            Err("File not found in git history".into())
        }
    }
    
    /// Reset working tree to HEAD, discarding all changes
    pub fn reset_working_tree(repo: &Repository, work_dir: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting reset working tree operation");
        
        // Check if repository has any commits
        match super::commit_ops::CommitOps::get_head_commit(repo) {
            Ok(head_commit) => {
                let head_oid = head_commit.id();
                info!("Found HEAD commit: {}", head_oid);
                
                // Reset the working tree to match HEAD (hard reset)
                let head_obj = repo.find_object(head_oid, None)?;
                repo.reset(&head_obj, ResetType::Hard, None)?;
                
                info!("Successfully reset working tree to HEAD commit: {}", head_oid);
                Ok(())
            }
            Err(e) => {
                // Handle unborn branch case (no commits yet)
                if let Some(git_err) = e.downcast_ref::<git2::Error>() {
                    if git_err.code() == git2::ErrorCode::UnbornBranch {
                        info!("Repository has no commits yet, clearing working tree and index");
                        
                        // Clear the index
                        let mut index = repo.index()?;
                        index.clear()?;
                        index.write()?;
                        
                        // Remove all untracked files
                        let mut status_options = git2::StatusOptions::new();
                        status_options.include_ignored(false);
                        status_options.include_untracked(true);
                        
                        let statuses = repo.statuses(Some(&mut status_options))?;
                        for entry in statuses.iter() {
                            if let Some(path) = entry.path() {
                                let file_path = work_dir.join(path);
                                if file_path.exists() {
                                    if file_path.is_file() {
                                        std::fs::remove_file(&file_path)?;
                                        info!("Removed untracked file: {}", path);
                                    } else if file_path.is_dir() {
                                        std::fs::remove_dir_all(&file_path)?;
                                        info!("Removed untracked directory: {}", path);
                                    }
                                }
                            }
                        }
                        
                        info!("Successfully cleared working tree (no commits to reset to)");
                        Ok(())
                    } else {
                        error!("Failed to get HEAD commit: {}", e);
                        Err(e)
                    }
                } else {
                    error!("Failed to get HEAD commit: {}", e);
                    Err(e)
                }
            }
        }
    }
    
    /// Reset working tree with verification and detailed status reporting
    pub fn reset_working_tree_with_verification(repo: &Repository, work_dir: &std::path::Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        info!("Starting reset working tree operation with verification");
        
        // Check repository status before reset
        match Self::get_status(repo, work_dir) {
            Ok(changes) => {
                if changes.is_empty() {
                    info!("No changes to discard, working tree is already clean");
                    return Ok(vec!["Working tree is already clean - no changes to discard".to_string()]);
                } else {
                    info!("Found {} changes to discard: {:?}", changes.len(), changes);
                }
            }
            Err(e) => {
                warn!("Could not check repository status before reset: {}", e);
            }
        }
        
        match Self::reset_working_tree(repo, work_dir) {
            Ok(_) => {
                info!("Successfully reset working tree");
                
                // Verify the reset worked
                match Self::get_status(repo, work_dir) {
                    Ok(changes) => {
                        if changes.is_empty() {
                            info!("Reset verification successful - working tree is now clean");
                            Ok(vec!["Working tree reset - all changes discarded".to_string()])
                        } else {
                            warn!("Reset completed but {} changes remain: {:?}", changes.len(), changes);
                            Ok(vec![format!("Working tree reset completed, but {} changes remain", changes.len())])
                        }
                    }
                    Err(e) => {
                        warn!("Could not verify reset status: {}", e);
                        Ok(vec!["Working tree reset completed (verification failed)".to_string()])
                    }
                }
            }
            Err(e) => {
                error!("Failed to reset working tree: {}", e);
                Err(format!("Failed to reset working tree: {}", e).into())
            }
        }
    }
}
