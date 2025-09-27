use tracing::{info, error, warn};
use git2::build::RepoBuilder;
use std::fs;
use std::path::Path;
use libc;
use crate::config::Config;
use crate::git::GitRepository;

/// Repository initialization operations
pub struct InitOps;

impl InitOps {
    /// Initialize the git repository using configuration
    pub fn initialize_repository(repo_path: &Path, config: &Config) -> Result<Option<GitRepository>, Box<dyn std::error::Error>> {
        info!("InitOps: Initializing repository");
        info!("InitOps: Using repository path: {:?}", repo_path);
        
        // Check if the path exists and contains a git repository
        if repo_path.exists() && repo_path.join(".git").exists() {
            info!("InitOps: Found existing .git directory at {:?}", repo_path);
            
            // Chown the repository to current user first to fix permission issues
            Self::chown_repository_to_current_user(repo_path);
            
            // Try to open existing repository
            match GitRepository::open(repo_path, config.clone()) {
                Ok(repo) => {
                    info!("InitOps: Successfully opened existing git repository at {:?}", repo_path);
                    return Ok(Some(repo));
                }
                Err(e) => {
                    warn!("InitOps: Found .git directory at {:?} but failed to open as repository: {}", repo_path, e);
                    // Don't try to clone/init if there's already a .git directory
                    // This prevents clearing existing repositories
                    return Ok(None);
                }
            }
        }
        
        // Only attempt to clone or initialize if no existing repository was found
        // If we have a repository URL configured, try to clone it
        if let Some(repo_url) = config.repository_url() {
            info!("InitOps: Attempting to clone repository from: {}", repo_url);
            match Self::clone_repository(repo_url, repo_path, config) {
                Ok(repo) => {
                    info!("InitOps: Successfully cloned repository from {} to {:?}", repo_url, repo_path);
                    return Ok(Some(repo));
                }
                Err(e) => {
                    error!("InitOps: Failed to clone repository from {}: {}", repo_url, e);
                    warn!("InitOps: Falling back to initializing empty repository");
                }
            }
        }
        
        // If no URL or clone failed, initialize an empty repository
        match GitRepository::init(repo_path, config.clone()) {
            Ok(repo) => {
                info!("InitOps: Initialized new empty git repository at {:?}", repo_path);
                Ok(Some(repo))
            }
            Err(e) => {
                error!("InitOps: Failed to initialize git repository at {:?}: {}", repo_path, e);
                // Continue without git repo - operations will fail gracefully
                Ok(None)
            }
        }
    }
    
    /// Change ownership of repository directory to current user
    pub fn chown_repository_to_current_user(repo_path: &Path) {
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command;
        
        info!("Changing ownership of repository at {:?} to current user", repo_path);
        
        // Get current user ID
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        
        info!("Current user ID: {}, group ID: {}", uid, gid);
        
        // Use chown command to recursively change ownership
        match Command::new("chown")
            .args(&["-R", &format!("{}:{}", uid, gid), repo_path.to_str().unwrap_or("")])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("Successfully changed ownership of repository directory");
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to change ownership of repository directory: {}", error_msg);
                }
            }
            Err(e) => {
                warn!("Failed to execute chown command: {}", e);
            }
        }
        
        // Also try to fix permissions on the directory
        if let Err(e) = std::fs::metadata(repo_path).and_then(|metadata| {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(repo_path, perms)
        }) {
            warn!("Failed to set permissions on repository directory: {}", e);
        }
    }
    
    /// Clone a repository from a URL
    pub fn clone_repository(url: &str, path: &Path, config: &Config) -> Result<GitRepository, Box<dyn std::error::Error>> {
        // Check if the directory exists and contains a valid git repository
        if path.exists() {
            // Check if there's a .git directory first
            if path.join(".git").exists() {
                // Try to open as a git repository
                match GitRepository::open(path, config.clone()) {
                    Ok(_) => {
                        info!("Directory {:?} already contains a valid git repository, skipping clone", path);
                        return GitRepository::open(path, config.clone());
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
        let _repo = RepoBuilder::new()
            .clone(url, path)?;
        
        // Create our GitRepository wrapper
        let git_repo = GitRepository::open(path, config.clone())?;
        
        Ok(git_repo)
    }
}
