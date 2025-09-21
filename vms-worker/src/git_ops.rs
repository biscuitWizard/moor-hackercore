use git2::{Repository, Signature, Commit};
use std::path::{Path, PathBuf};
use std::fs;
use tracing::info;

/// Git repository wrapper for VMS operations
pub struct GitRepository {
    repo: Repository,
    work_dir: PathBuf,
}

impl GitRepository {
    /// Initialize a new git repository in the given directory
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        
        let repo = Repository::init(path)?;
        let work_dir = path.to_path_buf();
        
        info!("Initialized git repository at {:?}", work_dir);
        
        Ok(GitRepository { repo, work_dir })
    }
    
    /// Open an existing git repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let repo = Repository::open(path)?;
        let work_dir = path.to_path_buf();
        
        info!("Opened git repository at {:?}", work_dir);
        
        Ok(GitRepository { repo, work_dir })
    }
    
    /// Get the working directory path
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }
    
    /// Add a file to the git index
    pub fn add_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let mut index = self.repo.index()?;
        let path = path.as_ref();
        
        // Convert to relative path from work directory
        let rel_path = path.strip_prefix(&self.work_dir)
            .map_err(|_| "File path is not within repository")?;
        
        index.add_path(rel_path)?;
        index.write()?;
        
        info!("Added file to git index: {:?}", rel_path);
        Ok(())
    }
    
    /// Remove a file from the git index and working directory
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let mut index = self.repo.index()?;
        let path = path.as_ref();
        
        // Convert to relative path from work directory
        let rel_path = path.strip_prefix(&self.work_dir)
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
    
    /// Create a commit with the given message
    pub fn commit(&self, message: &str, author_name: &str, author_email: &str) -> Result<Commit, Box<dyn std::error::Error>> {
        let signature = Signature::now(author_name, author_email)?;
        
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let parent_commit = self.get_head_commit().ok();
        
        let commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_commit.iter().collect::<Vec<_>>(),
        )?;
        
        let commit = self.repo.find_commit(commit_id)?;
        
        info!("Created commit: {}", commit_id);
        Ok(commit)
    }
    
    /// Get the current HEAD commit
    fn get_head_commit(&self) -> Result<Commit, Box<dyn std::error::Error>> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit)
    }
    
    /// Check if the repository has any changes
    pub fn has_changes(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let mut index = self.repo.index()?;
        let head_commit = self.get_head_commit().ok();
        
        if let Some(head) = head_commit {
            let head_tree = head.tree()?;
            let diff = self.repo.diff_tree_to_index(Some(&head_tree), Some(&mut index), None)?;
            Ok(diff.deltas().len() > 0)
        } else {
            // No commits yet, check if index has any entries
            Ok(index.len() > 0)
        }
    }
    
    /// Get the status of files in the repository
    pub fn status(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut status_options = git2::StatusOptions::new();
        status_options.include_ignored(false);
        status_options.include_untracked(true);
        
        let statuses = self.repo.statuses(Some(&mut status_options))?;
        let mut result = Vec::new();
        
        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("unknown");
            
            let status_str = if status.is_index_new() {
                "Added"
            } else if status.is_index_modified() {
                "Modified"
            } else if status.is_wt_new() {
                "Untracked"
            } else if status.is_wt_modified() {
                "Modified"
            } else if status.is_wt_deleted() {
                "Deleted"
            } else {
                "Unknown"
            };
            
            result.push(format!("{}: {}", status_str, path));
        }
        
        Ok(result)
    }
    
    /// Write content to a file in the working directory
    pub fn write_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let full_path = self.work_dir.join(path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&full_path, content)?;
        info!("Wrote file: {:?}", path);
        Ok(())
    }
    
    /// Read content from a file in the working directory
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<String, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let full_path = self.work_dir.join(path);
        
        let content = fs::read_to_string(&full_path)?;
        Ok(content)
    }
    
    /// Check if a file exists in the working directory
    pub fn file_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();
        let full_path = self.work_dir.join(path);
        full_path.exists()
    }
    
    /// Get the path for a .meta file corresponding to a .moo file
    pub fn meta_path<P: AsRef<Path>>(&self, moo_path: P) -> PathBuf {
        let moo_path = moo_path.as_ref();
        let mut meta_path = moo_path.to_path_buf();
        
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
    
    /// Get the current branch name
    pub fn get_current_branch(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match self.repo.head() {
            Ok(head) => {
                if let Some(branch_name) = head.shorthand() {
                    Ok(Some(branch_name.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                // Handle unborn branch case
                if e.code() == git2::ErrorCode::UnbornBranch {
                    // For unborn branches, try to get the symbolic reference
                    match self.repo.head_detached() {
                        Ok(_) => Ok(None), // Detached HEAD
                        Err(_) => {
                            // Try to get the symbolic reference name
                            match self.repo.references_glob("refs/heads/*") {
                                Ok(mut refs) => {
                                    if let Some(reference) = refs.next() {
                                        if let Ok(reference) = reference {
                                            if let Some(name) = reference.name() {
                                                if let Some(branch_name) = name.strip_prefix("refs/heads/") {
                                                    return Ok(Some(branch_name.to_string()));
                                                }
                                            }
                                        }
                                    }
                                    Ok(None)
                                }
                                Err(_) => Ok(None),
                            }
                        }
                    }
                } else {
                    Err(e.into())
                }
            }
        }
    }
    
    /// Get upstream information for the current branch
    pub fn get_upstream_info(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match self.repo.head() {
            Ok(head) => {
                if let Some(branch_name) = head.shorthand() {
                    if let Ok(branch) = self.repo.find_branch(branch_name, git2::BranchType::Local) {
                        if let Ok(upstream) = branch.upstream() {
                            if let Some(upstream_name) = upstream.name()? {
                                return Ok(Some(upstream_name.to_string()));
                            }
                        }
                    }
                }
                Ok(None)
            }
            Err(e) => {
                // Handle unborn branch case
                if e.code() == git2::ErrorCode::UnbornBranch {
                    Ok(None) // No upstream for unborn branches
                } else {
                    Err(e.into())
                }
            }
        }
    }
    
    /// Get information about the last commit
    pub fn get_last_commit_info(&self) -> Result<Option<crate::vms::types::CommitInfo>, Box<dyn std::error::Error>> {
        match self.get_head_commit() {
            Ok(commit) => {
                let id = commit.id().to_string();
                let short_id = &id[..8]; // First 8 characters
                let datetime = commit.time();
                let timestamp = chrono::DateTime::from_timestamp(datetime.seconds(), 0)
                    .unwrap_or_else(|| chrono::Utc::now())
                    .format("%Y-%m-%d %H:%M:%S UTC")
                    .to_string();
                let message = commit.message().unwrap_or("No message").to_string();
                let author = commit.author().name().unwrap_or("Unknown").to_string();
                
                Ok(Some(crate::vms::types::CommitInfo {
                    id: short_id.to_string(),
                    full_id: id,
                    datetime: timestamp,
                    message: message.trim().to_string(),
                    author,
                }))
            }
            Err(_) => Ok(None), // No commits yet
        }
    }

    
}
