use git2::{Repository, Signature, Commit, CertificateCheckStatus};
use std::path::{Path, PathBuf};
use std::fs;
use tracing::{info, warn, error};
use crate::config::Config;

/// Git repository wrapper for VMS operations
pub struct GitRepository {
    repo: Repository,
    work_dir: PathBuf,
    config: Config,
}

impl GitRepository {
    /// Initialize a new git repository in the given directory
    pub fn init<P: AsRef<Path>>(path: P, config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        
        // Create directory if it doesn't exist
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        
        let repo = Repository::init(path)?;
        let work_dir = path.to_path_buf();
        
        info!("Initialized git repository at {:?}", work_dir);
        
        let git_repo = GitRepository { repo, work_dir, config };
        
        // Configure git user name and email
        git_repo.configure_git_user()?;
        
        // Ensure keys directory is in .gitignore
        git_repo.ensure_keys_gitignore()?;
        
        Ok(git_repo)
    }
    
    /// Open an existing git repository
    pub fn open<P: AsRef<Path>>(path: P, config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let repo = Repository::open(path)?;
        let work_dir = path.to_path_buf();
        
        info!("Opened git repository at {:?}", work_dir);
        
        let git_repo = GitRepository { repo, work_dir, config };
        
        // Configure git user name and email
        git_repo.configure_git_user()?;
        
        // Ensure keys directory is in .gitignore
        git_repo.ensure_keys_gitignore()?;
        
        Ok(git_repo)
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
    
    /// Add all changes (untracked, modified, deleted) to the git index
    pub fn add_all_changes(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Adding all changes to git index");
        
        let mut index = self.repo.index()?;
        
        // Get status of all files
        let mut status_options = git2::StatusOptions::new();
        status_options.include_ignored(false);
        status_options.include_untracked(true);
        
        let statuses = self.repo.statuses(Some(&mut status_options))?;
        
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
    
    /// Create a commit with the given message
    pub fn commit(&self, message: &str, author_name: &str, author_email: &str) -> Result<Commit, Box<dyn std::error::Error>> {
        let signature = Signature::now(author_name, author_email)?;
        
        // Add all changes to the index before committing
        self.add_all_changes()?;
        
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
            
            let status_str = if status.is_wt_new() || status.is_index_new() {
                "Added"
            } else if status.is_index_modified() || status.is_wt_modified() {
                "Modified"
            } else if status.is_wt_deleted() || status.is_index_deleted() {
                "Deleted"
            } else if status.is_wt_renamed() || status.is_index_renamed() {
                "Renamed"
            } else if status.is_ignored() {
                continue;
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
    
    /// Rename a file in the git index and working directory
    pub fn rename_file<P: AsRef<Path>>(&self, old_path: P, new_path: P) -> Result<(), Box<dyn std::error::Error>> {
        let old_path = old_path.as_ref();
        let new_path = new_path.as_ref();
        
        // Convert to relative paths from work directory
        let old_rel_path = old_path.strip_prefix(&self.work_dir)
            .map_err(|_| "Old file path is not within repository")?;
        let new_rel_path = new_path.strip_prefix(&self.work_dir)
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
        // The key is to use git's index manipulation to preserve the rename relationship
        let mut index = self.repo.index()?;
        
        // Check if the old file is tracked in git
        if let Some(old_entry) = index.get_path(old_rel_path, 0) {
            // File is tracked, perform a proper git rename
            // First, move the file in the filesystem
            fs::rename(old_path, new_path)?;
            info!("Moved file in filesystem: {:?} -> {:?}", old_path, new_path);
            
            // Add the new file to the index with the same content hash as the old file
            // This preserves the relationship for git's rename detection
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
                let timestamp = datetime.seconds();
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

    /// Configure git user name and email in the repository
    pub fn configure_git_user(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = self.repo.config()?;
        
        config.set_str("user.name", self.config.git_user_name())?;
        config.set_str("user.email", self.config.git_user_email())?;
        
        info!("Configured git user: {} <{}>", self.config.git_user_name(), self.config.git_user_email());
        Ok(())
    }
    
    /// Ensure the keys directory is in .gitignore for security
    fn ensure_keys_gitignore(&self) -> Result<(), Box<dyn std::error::Error>> {
        let gitignore_path = self.work_dir.join(".gitignore");
        let keys_entry = "keys/\n";
        
        // Read existing .gitignore content
        let existing_content = if gitignore_path.exists() {
            fs::read_to_string(&gitignore_path)?
        } else {
            String::new()
        };
        
        // Check if keys/ is already ignored
        if existing_content.lines().any(|line| line.trim() == "keys/") {
            info!("Keys directory already in .gitignore");
            return Ok(());
        }
        
        // Add keys/ to .gitignore
        let new_content = if existing_content.is_empty() {
            keys_entry.to_string()
        } else if existing_content.ends_with('\n') {
            format!("{}{}", existing_content, keys_entry)
        } else {
            format!("{}\n{}", existing_content, keys_entry)
        };
        
        fs::write(&gitignore_path, new_content)?;
        info!("Added keys/ to .gitignore for security");
        
        // Add .gitignore to git index
        self.add_file(".gitignore")?;
        
        Ok(())
    }
    
    /// Configure SSH to handle host key verification for the given host
    fn configure_ssh_for_host(&self, hostname: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Configuring SSH for host: {}", hostname);
        
        // Create .ssh directory if it doesn't exist
        let ssh_dir = std::path::Path::new("/root/.ssh");
        if !ssh_dir.exists() {
            info!("Creating SSH directory: {:?}", ssh_dir);
            std::fs::create_dir_all(ssh_dir)?;
        }
        
        // Set proper permissions on .ssh directory
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(ssh_dir)?.permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(ssh_dir, perms)?;
        
        // Create SSH config to disable host key checking for this specific host
        let ssh_config_path = ssh_dir.join("config");
        let mut existing_config = String::new();
        
        // Read existing config if it exists
        if ssh_config_path.exists() {
            existing_config = std::fs::read_to_string(&ssh_config_path)?;
        }
        
        // Check if hostname already exists in config
        let hostname_exists = existing_config.lines()
            .any(|line| line.trim().starts_with(&format!("Host {}", hostname)));
        
        if !hostname_exists {
            // Append new host configuration
            let mut new_config = existing_config;
            if !new_config.is_empty() && !new_config.ends_with('\n') {
                new_config.push('\n');
            }
            new_config.push_str(&format!(
                "Host {}\n\
                 \tStrictHostKeyChecking no\n\
                 \tUserKnownHostsFile /dev/null\n\
                 \tLogLevel ERROR\n",
                hostname
            ));
            
            std::fs::write(&ssh_config_path, new_config)?;
            
            // Set proper permissions on SSH config
            let mut perms = std::fs::metadata(&ssh_config_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&ssh_config_path, perms)?;
            
            info!("Added SSH configuration for {}", hostname);
        } else {
            info!("SSH configuration for {} already exists", hostname);
        }
        
        // Set environment variable for git to use our SSH config
        unsafe {
            std::env::set_var("GIT_SSH_COMMAND", "ssh -F /root/.ssh/config");
        }
        
        Ok(())
    }
    fn get_ssh_credentials(&self, url: Option<&str>, username_from_url: Option<&str>, _allowed_types: git2::CredentialType) -> Result<git2::Cred, git2::Error> {
        info!("Attempting SSH authentication for URL: {:?}", url);
        
        // Try configured SSH key first
        if let Some(ssh_key_path) = self.config.ssh_key_path() {
            info!("Trying configured SSH key: {}", ssh_key_path);
            if let Ok(cred) = git2::Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                std::path::Path::new(ssh_key_path),
                None,
            ) {
                info!("Successfully authenticated with configured SSH key: {}", ssh_key_path);
                return Ok(cred);
            } else {
                warn!("Failed to authenticate with configured SSH key: {}", ssh_key_path);
            }
        }
        
        // Try keys directory
        let keys_dir = self.config.keys_directory();
        info!("Checking keys directory: {:?}", keys_dir);
        let default_keys = [
            keys_dir.join("id_rsa"),
            keys_dir.join("id_ed25519"),
            keys_dir.join("id_ecdsa"),
        ];
        
        for key_path in &default_keys {
            if key_path.exists() {
                info!("Trying SSH key from keys directory: {:?}", key_path);
                if let Ok(cred) = git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    key_path,
                    None,
                ) {
                    info!("Successfully authenticated with key from keys directory: {:?}", key_path);
                    return Ok(cred);
                } else {
                    warn!("Failed to authenticate with key from keys directory: {:?}", key_path);
                }
            }
        }
        
        // Try default SSH key locations in container
        let home_keys = [
            "/root/.ssh/id_rsa",
            "/root/.ssh/id_ed25519",
            "/root/.ssh/id_ecdsa",
        ];
        
        for key_path in &home_keys {
            if std::path::Path::new(key_path).exists() {
                info!("Trying default SSH key: {}", key_path);
                if let Ok(cred) = git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    std::path::Path::new(key_path),
                    None,
                ) {
                    info!("Successfully authenticated with default SSH key: {}", key_path);
                    return Ok(cred);
                } else {
                    warn!("Failed to authenticate with default SSH key: {}", key_path);
                }
            }
        }
        
        // Try git credential helper
        if let Ok(cred) = git2::Cred::credential_helper(&self.repo.config()?, url.unwrap_or(""), url) {
            info!("Successfully authenticated with git credential helper");
            return Ok(cred);
        } else {
            warn!("Git credential helper authentication failed");
        }
        
        // Fall back to default credential helper
        warn!("No SSH authentication available, trying default credentials");
        git2::Cred::default()
    }

    /// Push commits to the remote repository
    pub fn push(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting git push operation");
        
        // Get the current branch name
        let branch_name = match self.get_current_branch()? {
            Some(name) => {
                info!("Current branch: {}", name);
                name
            },
            None => {
                error!("No current branch found");
                return Err("No current branch found".into());
            }
        };

        // Get the upstream remote name (default to "origin")
        let remote_name = "origin";
        info!("Using remote: {}", remote_name);
        
        // Find the remote
        let mut remote = self.repo.find_remote(remote_name)?;
        info!("Found remote: {}", remote_name);
        
        // Push the current branch to its upstream
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        let refspecs = [refspec.as_str()];
        info!("Pushing refspec: {}", refspec);
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            info!("Git requesting credentials for URL: {:?}, username: {:?}, allowed_types: {:?}", url, username_from_url, allowed_types);
            self.get_ssh_credentials(Some(url), username_from_url, allowed_types)
        });
        
        // Certificate (host key) check callback
        callbacks.certificate_check(|_cert, _host| {
            Ok(CertificateCheckStatus::CertificateOk)
        });
        
        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        remote.push(&refspecs, Some(&mut push_options))?;
        
        info!("Successfully pushed branch '{}' to remote '{}'", branch_name, remote_name);
        Ok(())
    }
    
    /// Test SSH connection to remote
    pub fn test_ssh_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing SSH connection to remote repository");
        
        let mut remote = self.repo.find_remote("origin")?;
        info!("Found remote 'origin' for SSH test");
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            info!("SSH test requesting credentials for URL: {:?}, username: {:?}, allowed_types: {:?}", url, username_from_url, allowed_types);
            self.get_ssh_credentials(Some(url), username_from_url, allowed_types)
        });
        
        // Certificate (host key) check callback
        callbacks.certificate_check(|_cert, _host| {
            Ok(CertificateCheckStatus::CertificateOk)
        });
        
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(&["main"], Some(&mut fetch_options), None)?;
        
        info!("SSH connection test successful");
        Ok(())
    }
    
    /// Get paginated list of commits
    pub fn get_commits(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<crate::vms::types::CommitInfo>, Box<dyn std::error::Error>> {
        let limit = limit.unwrap_or(5); // Default to 5 commits
        let offset = offset.unwrap_or(0); // Default to no offset
        
        info!("Getting {} commits starting from offset {}", limit, offset);
        
        // Get the HEAD reference
        let head = self.repo.head()?;
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(head.target().unwrap())?;
        revwalk.set_sorting(git2::Sort::TIME)?;
        
        let mut commits = Vec::new();
        let mut count = 0;
        let mut skipped = 0;
        
        for commit_id in revwalk {
            let commit_id = commit_id?;
            
            // Skip commits until we reach the offset
            if skipped < offset {
                skipped += 1;
                continue;
            }
            
            // Stop if we've reached the limit
            if count >= limit {
                break;
            }
            
            let commit = self.repo.find_commit(commit_id)?;
            let id = commit.id().to_string();
            let short_id = &id[..8]; // First 8 characters
            let datetime = commit.time();
            let timestamp = datetime.seconds();
            let message = commit.message().unwrap_or("No message").to_string();
            let author = commit.author().name().unwrap_or("Unknown").to_string();
            
            commits.push(crate::vms::types::CommitInfo {
                id: short_id.to_string(),
                full_id: id,
                datetime: timestamp,
                message: message.trim().to_string(),
                author,
            });
            
            count += 1;
        }
        
        info!("Retrieved {} commits (skipped {}, total requested: {})", count, skipped, limit);
        Ok(commits)
    }
    
}

