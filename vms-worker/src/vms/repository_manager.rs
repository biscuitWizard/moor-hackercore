use std::path::PathBuf;
use tracing::{info, error, warn};
use crate::config::Config;
use crate::git_ops::GitRepository;

/// Manages git repository initialization and ownership
pub struct RepositoryManager {
    config: Config,
}

impl RepositoryManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Initialize the git repository using configuration
    pub fn initialize_repository(&self) -> Option<GitRepository> {
        let repo_path = self.config.repository_path();
        
        // Check if the path exists and contains a git repository
        if repo_path.exists() && repo_path.join(".git").exists() {
            // Chown the repository to current user first, before trying to open it
            self.chown_repository_to_current_user(&repo_path);
            
            // Try to open existing repository
            match GitRepository::open(&repo_path) {
                Ok(repo) => {
                    info!("Opened existing git repository at {:?}", repo_path);
                    return Some(repo);
                }
                Err(e) => {
                    warn!("Found .git directory at {:?} but failed to open as repository: {}", repo_path, e);
                    // Don't try to clone/init if there's already a .git directory
                    // This prevents clearing existing repositories
                    return None;
                }
            }
        }
        
        // Only attempt to clone or initialize if no existing repository was found
        // If we have a repository URL configured, try to clone it
        if let Some(repo_url) = self.config.repository_url() {
            info!("Attempting to clone repository from: {}", repo_url);
            match self.clone_repository(repo_url, &repo_path) {
                Ok(repo) => {
                    info!("Successfully cloned repository from {} to {:?}", repo_url, repo_path);
                    // Chown the repository to current user
                    self.chown_repository_to_current_user(&repo_path);
                    return Some(repo);
                }
                Err(e) => {
                    error!("Failed to clone repository from {}: {}", repo_url, e);
                    warn!("Falling back to initializing empty repository");
                }
            }
        }
        
        // If no URL or clone failed, initialize an empty repository
        match GitRepository::init(&repo_path) {
            Ok(repo) => {
                info!("Initialized new empty git repository at {:?}", repo_path);
                // Chown the repository to current user
                self.chown_repository_to_current_user(&repo_path);
                Some(repo)
            }
            Err(e) => {
                error!("Failed to initialize git repository at {:?}: {}", repo_path, e);
                // Continue without git repo - operations will fail gracefully
                None
            }
        }
    }
    
    /// Clone a repository from a URL
    fn clone_repository(&self, url: &str, path: &std::path::Path) -> Result<GitRepository, Box<dyn std::error::Error>> {
        use git2::build::RepoBuilder;
        use std::fs;
        
        // Check if the directory exists and contains a valid git repository
        if path.exists() {
            // Check if there's a .git directory first
            if path.join(".git").exists() {
                // Try to open as a git repository
                match GitRepository::open(path) {
                    Ok(_) => {
                        info!("Directory {:?} already contains a valid git repository, skipping clone", path);
                        return GitRepository::open(path);
                    }
                    Err(e) => {
                        warn!("Directory {:?} contains .git but is not a valid repository: {}", path, e);
                        // Don't remove directories that contain .git - this could be a corrupted repo
                        return Err(format!("Directory {:?} contains .git but is not a valid repository: {}", path, e).into());
                    }
                }
            } else {
                // Directory exists but has no .git directory, safe to remove
                info!("Removing existing non-git directory at {:?} before cloning", path);
                fs::remove_dir_all(path)?;
            }
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Clone the repository
        info!("Cloning repository from {} to {:?}", url, path);
        let repo = RepoBuilder::new()
            .clone(url, path)?;
        
        // Create our GitRepository wrapper
        let git_repo = GitRepository::open(path)?;
        
        Ok(git_repo)
    }
    
    /// Change ownership of the repository to the current user
    fn chown_repository_to_current_user(&self, repo_path: &std::path::Path) {
        use std::process::Command;
        
        // Get current user UID and GID
        let current_uid = unsafe { libc::getuid() };
        let current_gid = unsafe { libc::getgid() };
        
        info!("Chowning repository at {:?} to UID:{} GID:{}", repo_path, current_uid, current_gid);
        
        // Use chown command to recursively change ownership
        let output = Command::new("chown")
            .arg("-R")
            .arg(format!("{}:{}", current_uid, current_gid))
            .arg(repo_path)
            .output();
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Successfully chowned repository at {:?} to current user", repo_path);
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to chown repository at {:?}: {}", repo_path, error_msg);
                }
            }
            Err(e) => {
                warn!("Failed to execute chown command for repository at {:?}: {}", repo_path, e);
            }
        }
    }
}
