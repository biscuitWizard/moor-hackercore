use tracing::{info, error};
use git2::Repository;
use crate::git::utils::GitUtils;

/// Remote operations for git repositories
pub struct RemoteOps;

impl RemoteOps {
    /// Push commits to the remote repository
    pub fn push(
        repo: &Repository,
        ssh_key_path: Option<&str>,
        keys_dir: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting git push operation");
        
        // Get the current branch name
        let branch_name = match Self::get_current_branch(repo)? {
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
        let mut remote = repo.find_remote(remote_name)?;
        info!("Found remote: {}", remote_name);
        
        // Push the current branch to its upstream
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        let refspecs = [refspec.as_str()];
        info!("Pushing refspec: {}", refspec);
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            info!("Git requesting credentials for URL: {:?}, username: {:?}, allowed_types: {:?}", url, username_from_url, allowed_types);
            GitUtils::get_ssh_credentials(repo, Some(url), username_from_url, allowed_types, ssh_key_path, keys_dir)
        });
        
        // Certificate (host key) check callback
        callbacks.certificate_check(GitUtils::create_certificate_check_callback());
        
        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        remote.push(&refspecs, Some(&mut push_options))?;
        
        info!("Successfully pushed branch '{}' to remote '{}'", branch_name, remote_name);
        Ok(())
    }
    
    /// Test SSH connection to remote
    pub fn test_ssh_connection(
        repo: &Repository,
        ssh_key_path: Option<&str>,
        keys_dir: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Testing SSH connection to remote repository");
        
        let mut remote = repo.find_remote("origin")?;
        info!("Found remote 'origin' for SSH test");
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            info!("SSH test requesting credentials for URL: {:?}, username: {:?}, allowed_types: {:?}", url, username_from_url, allowed_types);
            GitUtils::get_ssh_credentials(repo, Some(url), username_from_url, allowed_types, ssh_key_path, keys_dir)
        });
        
        // Certificate (host key) check callback
        callbacks.certificate_check(GitUtils::create_certificate_check_callback());
        
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(&["main"], Some(&mut fetch_options), None)?;
        
        info!("SSH connection test successful");
        Ok(())
    }
    
    /// Fetch from remote repository
    pub fn fetch_remote(
        repo: &Repository,
        ssh_key_path: Option<&str>,
        keys_dir: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Fetching from remote repository");
        
        let mut remote = repo.find_remote("origin")?;
        
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            info!("Git requesting credentials for fetch URL: {:?}, username: {:?}, allowed_types: {:?}", url, username_from_url, allowed_types);
            GitUtils::get_ssh_credentials(repo, Some(url), username_from_url, allowed_types, ssh_key_path, keys_dir)
        });
        
        // Certificate (host key) check callback
        callbacks.certificate_check(GitUtils::create_certificate_check_callback());
        
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fetch_options), None)?;
        
        info!("Successfully fetched from remote");
        Ok(())
    }
    
    /// Get the current branch name
    pub fn get_current_branch(repo: &Repository) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match repo.head() {
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
                    match repo.head_detached() {
                        Ok(_) => Ok(None), // Detached HEAD
                        Err(_) => {
                            // Try to get the symbolic reference name
                            match repo.references_glob("refs/heads/*") {
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
    pub fn get_upstream_info(repo: &Repository) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match repo.head() {
            Ok(head) => {
                if let Some(branch_name) = head.shorthand() {
                    if let Ok(branch) = repo.find_branch(branch_name, git2::BranchType::Local) {
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
}
