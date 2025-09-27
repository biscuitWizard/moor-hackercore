use tracing::{info, error};
use git2::Repository;
use crate::config::Config;
use super::commit_ops::CommitOps;
use super::remote_ops::RemoteOps;

/// Pull operations for git repositories
pub struct PullOps;

impl PullOps {
    /// Pull with rebase strategy
    pub fn pull_with_rebase(
        repo: &Repository,
        config: &Config,
        dry_run: bool,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        info!("Starting pull operation with rebase strategy (dry_run: {})", dry_run);
        
        if dry_run {
            return Self::pull_dry_run(repo, config);
        }
        
        // First, fetch the latest changes from remote
        let ssh_key_path = config.ssh_key_path();
        let keys_dir = config.keys_directory();
        
        match RemoteOps::fetch_remote(repo, ssh_key_path.as_ref().map(|s| s.as_str()), &keys_dir) {
            Ok(_) => {
                info!("Successfully fetched from remote");
            }
            Err(e) => {
                error!("Failed to fetch from remote: {}", e);
                return Err(format!("Failed to fetch from remote: {}", e).into());
            }
        }
        
        // Get the current branch and upstream
        let current_branch = match RemoteOps::get_current_branch(repo)? {
            Some(branch) => branch,
            None => {
                error!("No current branch found");
                return Err("No current branch found".into());
            }
        };
        
        let upstream_branch = format!("origin/{}", current_branch);
        info!("Current branch: {}, upstream: {}", current_branch, upstream_branch);
        
        // Check if there are any commits to pull
        match CommitOps::get_commits_ahead_behind(repo, &current_branch, &upstream_branch) {
            Ok((_ahead, behind)) => {
                info!("Branch is {} commits behind upstream", behind);
                
                if behind == 0 {
                    info!("No commits to pull");
                    return Ok(vec!["No commits to pull".to_string()]);
                }
                
                // Get commits that will be pulled
                let commits_to_pull = match CommitOps::get_commits_between(repo, "HEAD", &upstream_branch) {
                    Ok(commits) => commits,
                    Err(e) => {
                        error!("Failed to get commits between HEAD and {}: {}", upstream_branch, e);
                        return Err(format!("Failed to get commits: {}", e).into());
                    }
                };
                
                // Perform the actual rebase
                match CommitOps::rebase_onto(repo, &upstream_branch) {
                    Ok(_) => {
                        info!("Successfully completed rebase");
                        Ok(commits_to_pull.iter().map(|c| c.id.clone()).collect())
                    }
                    Err(e) => {
                        error!("Failed to complete rebase: {}", e);
                        Err(format!("Failed to complete rebase: {}", e).into())
                    }
                }
            }
            Err(e) => {
                error!("Failed to check commits ahead/behind: {}", e);
                Err(format!("Failed to check commits ahead/behind: {}", e).into())
            }
        }
    }
    
    /// Dry run for pull operation - returns what would be modified
    pub fn pull_dry_run(
        repo: &Repository,
        config: &Config,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        info!("Performing pull dry run");
        
        // Fetch to get latest remote info
        let ssh_key_path = config.ssh_key_path();
        let keys_dir = config.keys_directory();
        
        match RemoteOps::fetch_remote(repo, ssh_key_path.as_ref().map(|s| s.as_str()), &keys_dir) {
            Ok(_) => {
                info!("Successfully fetched from remote for dry run");
            }
            Err(e) => {
                error!("Failed to fetch from remote for dry run: {}", e);
                return Err(format!("Failed to fetch from remote: {}", e).into());
            }
        }
        
        let current_branch = match RemoteOps::get_current_branch(repo)? {
            Some(branch) => branch,
            None => {
                error!("No current branch found");
                return Err("No current branch found".into());
            }
        };
        
        let upstream_branch = format!("origin/{}", current_branch);
        
        match CommitOps::get_commits_ahead_behind(repo, &current_branch, &upstream_branch) {
            Ok((_ahead, behind)) => {
                if behind == 0 {
                    return Ok(vec!["No commits to pull".to_string()]);
                }
                
                // Get commits that would be pulled
                let commits_to_pull = match CommitOps::get_commits_between(repo, "HEAD", &upstream_branch) {
                    Ok(commits) => commits,
                    Err(e) => {
                        error!("Failed to get commits between HEAD and {}: {}", upstream_branch, e);
                        return Err(format!("Failed to get commits: {}", e).into());
                    }
                };
                
                Ok(commits_to_pull.iter().map(|c| c.id.clone()).collect())
            }
            Err(e) => {
                error!("Failed to check commits ahead/behind: {}", e);
                Err(format!("Failed to check commits ahead/behind: {}", e).into())
            }
        }
    }
}
