use std::path::Path;
use std::fs;
use tracing::info;
use git2::Repository;

/// File operations for git repositories
pub struct FileOps;

impl FileOps {
    /// Add a file to the git index
    pub fn add_file(repo: &Repository, work_dir: &Path, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut index = repo.index()?;
        
        // Convert to relative path from work directory
        let rel_path = path.strip_prefix(work_dir)
            .map_err(|_| "File path is not within repository")?;
        
        index.add_path(rel_path)?;
        index.write()?;
        
        info!("Added file to git index: {:?}", rel_path);
        Ok(())
    }
    
    /// Remove a file from the git index and working directory
    pub fn remove_file(repo: &Repository, work_dir: &Path, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let mut index = repo.index()?;
        
        // Convert to relative path from work directory
        let rel_path = path.strip_prefix(work_dir)
            .map_err(|_| "File path is not within repository")?;
        
        // Remove from index
        index.remove_path(rel_path)?;
        index.write()?;
        
        // Remove from working directory if it exists
        if path.exists() {
            fs::remove_file(path)?;
        }
        
        info!("Removed file from git: {:?}", rel_path);
        Ok(())
    }
    
    /// Add all changes (untracked, modified, deleted) to the git index
    pub fn add_all_changes(repo: &Repository) -> Result<(), Box<dyn std::error::Error>> {
        info!("Adding all changes to git index");
        
        let mut index = repo.index()?;
        
        // Get status of all files
        let mut status_options = git2::StatusOptions::new();
        status_options.include_ignored(false);
        status_options.include_untracked(true);
        
        let statuses = repo.statuses(Some(&mut status_options))?;
        
        let mut added_count = 0;
        let mut removed_count = 0;
        
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("unknown");
            
            if status.is_wt_new() || status.is_wt_modified() {
                // Add untracked or modified files
                index.add_path(std::path::Path::new(path))?;
                added_count += 1;
                info!("Added to index: {}", path);
            } else if status.is_wt_deleted() {
                // Remove deleted files from index
                index.remove_path(std::path::Path::new(path))?;
                removed_count += 1;
                info!("Removed from index: {}", path);
            }
        }
        
        // Write the updated index
        index.write()?;
        
        info!("Added {} files and removed {} files from git index", added_count, removed_count);
        Ok(())
    }
    
    /// Write content to a file in the working directory
    pub fn write_file(work_dir: &Path, path: &Path, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let full_path = work_dir.join(path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&full_path, content)?;
        info!("Wrote file: {:?}", path);
        Ok(())
    }
    
    /// Read content from a file in the working directory
    pub fn read_file(work_dir: &Path, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        let full_path = work_dir.join(path);
        
        let content = fs::read_to_string(&full_path)?;
        Ok(content)
    }
    
    /// Check if a file exists in the working directory
    pub fn file_exists(work_dir: &Path, path: &Path) -> bool {
        let full_path = work_dir.join(path);
        full_path.exists()
    }
    
    /// Rename a file in the git index and working directory
    pub fn rename_file(repo: &Repository, work_dir: &Path, old_path: &Path, new_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Convert to relative paths from work directory
        let old_rel_path = old_path.strip_prefix(work_dir)
            .map_err(|_| "Old file path is not within repository")?;
        let new_rel_path = new_path.strip_prefix(work_dir)
            .map_err(|_| "New file path is not within repository")?;
        
        // Check if source file exists
        if !old_path.exists() {
            return Err("Source file does not exist".into());
        }
        
        // Create parent directories for new path if they don't exist
        if let Some(parent) = new_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Use git's proper rename functionality
        let mut index = repo.index()?;
        
        // Check if the old file is tracked in git
        if let Some(old_entry) = index.get_path(old_rel_path, 0) {
            // File is tracked, perform a proper git rename
            // First, move the file in the filesystem
            fs::rename(old_path, new_path)?;
            info!("Moved file in filesystem: {:?} -> {:?}", old_path, new_path);
            
            // Add the new file to the index with the same content hash as the old file
            let new_entry = git2::IndexEntry {
                ctime: old_entry.ctime,
                mtime: old_entry.mtime,
                dev: old_entry.dev,
                ino: old_entry.ino,
                mode: old_entry.mode,
                uid: old_entry.uid,
                gid: old_entry.gid,
                file_size: old_entry.file_size,
                id: old_entry.id,
                flags: old_entry.flags,
                flags_extended: old_entry.flags_extended,
                path: new_rel_path.to_string_lossy().to_string().into(),
            };
            
            // Remove the old entry and add the new one
            index.remove_path(old_rel_path)?;
            index.add(&new_entry)?;
            
            // Write the updated index
            index.write()?;
            
            info!("Renamed tracked file in git: {:?} -> {:?}", old_rel_path, new_rel_path);
        } else {
            // File is not tracked, just move it and add to index
            fs::rename(old_path, new_path)?;
            index.add_path(new_rel_path)?;
            index.write()?;
            
            info!("Moved untracked file: {:?} -> {:?}", old_path, new_path);
        }
        
        Ok(())
    }
    
}
