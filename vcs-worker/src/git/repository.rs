use std::path::{Path, PathBuf};
use std::fs;
use tracing::info;
use git2::Repository;
use crate::config::Config;
use crate::vcs::types::CommitInfo;

use super::config::GitConfig;
use super::utils::GitUtils;
use super::operations::*;

/// Git repository wrapper for VCS operations
pub struct GitRepository {
    repo: Repository,
    work_dir: PathBuf,
    git_config: GitConfig,
}

impl GitRepository {
    /// Initialize a new git repository in the given directory
    pub fn init<P: AsRef<Path>>(path: P, config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        
        // Get absolute path to avoid working directory issues
        let absolute_path = if path.exists() {
            path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
        } else {
            path.to_path_buf()
        };
        
        info!("GitRepository::init: Using absolute path: {:?}", absolute_path);
        
        // Create directory if it doesn't exist
        if !absolute_path.exists() {
            fs::create_dir_all(&absolute_path)?;
        }
        
        let repo = Repository::init(&absolute_path)?;
        let work_dir = absolute_path;
        
        info!("Initialized git repository at {:?}", work_dir);
        
        let git_repo = GitRepository { 
            repo, 
            work_dir, 
            git_config: GitConfig::new(config),
        };
        
        // Configure git user name and email
        git_repo.configure_git_user()?;
        
        // Ensure keys directory is in .gitignore
        git_repo.ensure_keys_gitignore()?;
        
        Ok(git_repo)
    }
    
    /// Open an existing git repository
    pub fn open<P: AsRef<Path>>(path: P, config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        info!("GitRepository::open: Attempting to open repository at {:?}", path);
        
        // Get absolute path to avoid working directory issues
        let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        info!("GitRepository::open: Using absolute path: {:?}", absolute_path);
        
        // Check if the path exists
        if !absolute_path.exists() {
            return Err(format!("Repository path does not exist: {:?}", absolute_path).into());
        }
        
        // Check if it's a directory
        if !absolute_path.is_dir() {
            return Err(format!("Repository path is not a directory: {:?}", absolute_path).into());
        }
        
        // Check if .git directory exists
        let git_dir = absolute_path.join(".git");
        if !git_dir.exists() {
            return Err(format!("No .git directory found at {:?}", absolute_path).into());
        }
        
        info!("GitRepository::open: Found .git directory at {:?}", git_dir);
        
        let repo = Repository::open(&absolute_path)?;
        let work_dir = absolute_path;
        
        info!("GitRepository::open: Successfully opened git repository at {:?}", work_dir);
        
        let git_repo = GitRepository { 
            repo, 
            work_dir, 
            git_config: GitConfig::new(config),
        };
        
        // Configure git user name and email
        git_repo.configure_git_user()?;
        
        // Ensure keys directory is in .gitignore
        info!("GitRepository::open: About to call ensure_keys_gitignore");
        git_repo.ensure_keys_gitignore()?;
        info!("GitRepository::open: ensure_keys_gitignore completed successfully");
        
        Ok(git_repo)
    }
    
    /// Get the working directory path
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }
    
    /// Get the underlying git2 repository
    pub fn repo(&self) -> &Repository {
        &self.repo
    }
    
    /// Get the git configuration
    pub fn git_config(&self) -> &GitConfig {
        &self.git_config
    }
    
    // File operations
    pub fn add_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        FileOps::add_file(&self.repo, &self.work_dir, path.as_ref())
    }
    
    pub fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        FileOps::remove_file(&self.repo, &self.work_dir, path.as_ref())
    }
    
    pub fn add_all_changes(&self) -> Result<(), Box<dyn std::error::Error>> {
        FileOps::add_all_changes(&self.repo)
    }
    
    pub fn write_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        FileOps::write_file(&self.work_dir, path.as_ref(), content)
    }
    
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<String, Box<dyn std::error::Error>> {
        FileOps::read_file(&self.work_dir, path.as_ref())
    }
    
    pub fn file_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        FileOps::file_exists(&self.work_dir, path.as_ref())
    }
    
    pub fn rename_file<P: AsRef<Path>>(&self, old_path: P, new_path: P) -> Result<(), Box<dyn std::error::Error>> {
        FileOps::rename_file(&self.repo, &self.work_dir, old_path.as_ref(), new_path.as_ref())
    }
    
    pub fn meta_path<P: AsRef<Path>>(&self, moo_path: P) -> PathBuf {
        FileOps::meta_path(moo_path.as_ref())
    }
    
    // Commit operations
    pub fn commit(&self, message: &str, author_name: &str, author_email: &str) -> Result<git2::Commit, Box<dyn std::error::Error>> {
        CommitOps::create_commit(&self.repo, message, author_name, author_email)
    }
    
    pub fn get_commits(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
        CommitOps::get_commits(&self.repo, limit, offset)
    }
    
    pub fn get_last_commit_info(&self) -> Result<Option<CommitInfo>, Box<dyn std::error::Error>> {
        CommitOps::get_last_commit_info(&self.repo)
    }
    
    pub fn get_commits_ahead_behind(&self, local_branch: &str, remote_branch: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        CommitOps::get_commits_ahead_behind(&self.repo, local_branch, remote_branch)
    }
    
    pub fn get_commits_between(&self, from: &str, to: &str) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
        CommitOps::get_commits_between(&self.repo, from, to)
    }
    
    pub fn get_commit_changes(&self, commit_id: &str) -> Result<Vec<crate::vcs::types::CommitChange>, Box<dyn std::error::Error>> {
        CommitOps::get_commit_changes(&self.repo, commit_id)
    }
    
    pub fn get_file_content_at_commit(&self, commit_id: &str, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        CommitOps::get_file_content_at_commit(&self.repo, commit_id, file_path)
    }
    
    pub fn rebase_onto(&self, upstream_branch: &str) -> Result<(), Box<dyn std::error::Error>> {
        CommitOps::rebase_onto(&self.repo, upstream_branch)
    }
    
    pub fn rollback_last_commit(&self) -> Result<(), Box<dyn std::error::Error>> {
        CommitOps::rollback_last_commit(&self.repo)
    }
    
    // Remote operations
    pub fn push(&self) -> Result<(), Box<dyn std::error::Error>> {
        RemoteOps::push(
            &self.repo,
            self.git_config.ssh_key_path().map(|s| s.as_str()),
            &self.git_config.keys_directory(),
        )
    }
    
    pub fn test_ssh_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        RemoteOps::test_ssh_connection(
            &self.repo,
            self.git_config.ssh_key_path().map(|s| s.as_str()),
            &self.git_config.keys_directory(),
        )
    }
    
    pub fn fetch_remote(&self) -> Result<(), Box<dyn std::error::Error>> {
        RemoteOps::fetch_remote(
            &self.repo,
            self.git_config.ssh_key_path().map(|s| s.as_str()),
            &self.git_config.keys_directory(),
        )
    }
    
    pub fn get_current_branch(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        RemoteOps::get_current_branch(&self.repo)
    }
    
    pub fn get_upstream_info(&self) -> Result<Option<String>, Box<dyn std::error::Error>> {
        RemoteOps::get_upstream_info(&self.repo)
    }
    
    // Status operations
    pub fn has_changes(&self) -> Result<bool, Box<dyn std::error::Error>> {
        StatusOps::has_changes(&self.repo)
    }
    
    pub fn status(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        StatusOps::get_status(&self.repo, &self.work_dir)
    }
    
    pub fn reset_working_tree(&self) -> Result<(), Box<dyn std::error::Error>> {
        StatusOps::reset_working_tree(&self.repo, &self.work_dir)
    }
    
    /// Configure git user name and email in the repository
    pub fn configure_git_user(&self) -> Result<(), Box<dyn std::error::Error>> {
        GitUtils::configure_git_user(
            &self.repo,
            self.git_config.git_user_name(),
            self.git_config.git_user_email(),
        )
    }
    
    /// Ensure the keys directory is in .gitignore for security
    fn ensure_keys_gitignore(&self) -> Result<(), Box<dyn std::error::Error>> {
        GitUtils::ensure_keys_gitignore(&self.work_dir)?;
        
        // Add .gitignore to git index
        info!("GitRepository::ensure_keys_gitignore: About to call add_file with .gitignore");
        self.add_file(".gitignore")?;
        info!("GitRepository::ensure_keys_gitignore: add_file completed successfully");
        
        Ok(())
    }
}
