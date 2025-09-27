use tracing::{error, info};
use moor_var::{Var, v_str, v_map, v_list, v_int};
use crate::git::GitRepository;
use crate::config::Config;
use moor_common::tasks::WorkerError;

/// Handles repository status and commit information operations
pub struct StatusHandler;

impl StatusHandler {    
    /// Get comprehensive repository status including credentials
    pub fn get_repository_status(&self, repo: &GitRepository, config: &Config) -> Result<Vec<Var>, WorkerError> {
        info!("StatusHandler: Getting repository status");
        match self.collect_repository_status_vars(repo, config) {
            Ok(status_pairs) => {
                info!("StatusHandler: Successfully collected {} status pairs", status_pairs.len());
                Ok(vec![v_map(&status_pairs)])
            }
            Err(e) => {
                error!("StatusHandler: Failed to get repository status: {}", e);
                Err(WorkerError::RequestError(format!("Failed to get repository status: {}", e)))
            }
        }
    }
    
    /// Collect comprehensive repository status information as Var pairs
    fn collect_repository_status_vars(&self, repo: &GitRepository, config: &Config) -> Result<Vec<(Var, Var)>, Box<dyn std::error::Error>> {
        info!("StatusHandler: Collecting repository status variables");
        let mut status_pairs = Vec::new();
        
        // Get current branch
        info!("StatusHandler: Getting current branch");
        match repo.get_current_branch() {
            Ok(Some(branch)) => {
                status_pairs.push((v_str("current_branch"), v_str(branch.as_str())));
            }
            Ok(None) => {
                status_pairs.push((v_str("current_branch"), v_str("(detached HEAD)")));
            }
            Err(e) => {
                error!("Failed to get current branch: {}", e);
                status_pairs.push((v_str("current_branch"), v_str("(error)")));
            }
        }
        
        // Get upstream information
        info!("StatusHandler: Getting upstream info");
        match repo.get_upstream_info() {
            Ok(Some(upstream)) => {
                status_pairs.push((v_str("upstream"), v_str(upstream.as_str())));
            }
            Ok(None) => {
                status_pairs.push((v_str("upstream"), v_str("(none)")));
            }
            Err(e) => {
                error!("Failed to get upstream info: {}", e);
                status_pairs.push((v_str("upstream"), v_str("(error)")));
            }
        }
        
        // Get last commit information
        info!("StatusHandler: Getting last commit info");
        match repo.get_last_commit_info() {
            Ok(Some(commit)) => {
                status_pairs.push((v_str("last_commit_id"), v_str(&commit.id)));
                status_pairs.push((v_str("last_commit_full_id"), v_str(&commit.full_id)));
                status_pairs.push((v_str("last_commit_message"), v_str(&commit.message)));
                status_pairs.push((v_str("last_commit_author"), v_str(&commit.author)));
                status_pairs.push((v_str("last_commit_datetime"), v_int(commit.datetime)));
            }
            Ok(None) => {
                status_pairs.push((v_str("last_commit_id"), v_str("(no commits yet)")));
                status_pairs.push((v_str("last_commit_full_id"), v_str("(no commits yet)")));
                status_pairs.push((v_str("last_commit_message"), v_str("(no commits yet)")));
                status_pairs.push((v_str("last_commit_author"), v_str("(no commits yet)")));
                status_pairs.push((v_str("last_commit_datetime"), v_int(0)));
            }
            Err(e) => {
                error!("Failed to get last commit info: {}", e);
                status_pairs.push((v_str("last_commit_id"), v_str("(error)")));
                status_pairs.push((v_str("last_commit_full_id"), v_str("(error)")));
                status_pairs.push((v_str("last_commit_message"), v_str("(error)")));
                status_pairs.push((v_str("last_commit_author"), v_str("(error)")));
                status_pairs.push((v_str("last_commit_datetime"), v_int(0)));
            }
        }
        
        // Get current changes
        info!("StatusHandler: Getting repository status");
        match repo.status() {
            Ok(changes) => {
                let change_list = if changes.is_empty() {
                    v_list(&[])
                } else {
                    let change_vars: Vec<Var> = changes
                        .iter()
                        .map(|change_line| {
                            // Parse the change line to extract type and file
                            // Format is typically "Type: filepath" (e.g., "Added: src/main.rs")
                            if let Some(colon_pos) = change_line.find(':') {
                                let change_type = &change_line[..colon_pos];
                                let file_path = &change_line[colon_pos + 1..].trim();
                                v_list(&[v_str(change_type), v_str(file_path)])
                            } else {
                                // Fallback if format is unexpected
                                v_list(&[v_str("unknown"), v_str(change_line)])
                            }
                        })
                        .collect();
                    v_list(&change_vars)
                };
                status_pairs.push((v_str("changes"), change_list));
            }
            Err(e) => {
                error!("Failed to get status: {}", e);
                status_pairs.push((v_str("changes"), v_list(&[v_str("error"), v_str(&format!("Failed to get status: {}", e))])));
            }
        }
        
        // Add credential information
        self.add_credential_status(&mut status_pairs, config);
        
        Ok(status_pairs)
    }
    
    /// Add credential status information to status pairs
    fn add_credential_status(&self, status_pairs: &mut Vec<(Var, Var)>, config: &Config) {
        // Git user information
        status_pairs.push((v_str("git_user_name"), v_str(config.git_user_name())));
        status_pairs.push((v_str("git_user_email"), v_str(config.git_user_email())));
        
        // SSH key information
        if let Some(key_path) = config.ssh_key_path() {
            status_pairs.push((v_str("ssh_key_path"), v_str(&key_path)));
        } else {
            status_pairs.push((v_str("ssh_key_path"), v_str("(not configured)")));
        }
        
        // Keys directory information
        let keys_dir = config.keys_directory();
        if keys_dir.exists() {
            let mut key_files = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&keys_dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.starts_with("id_") {
                            key_files.push(file_name.to_string());
                        }
                    }
                }
            }
            if !key_files.is_empty() {
                status_pairs.push((v_str("keys_directory"), v_str(&format!("{:?} (contains: {})", keys_dir, key_files.join(", ")))));
            } else {
                status_pairs.push((v_str("keys_directory"), v_str(&format!("{:?} (empty)", keys_dir))));
            }
        } else {
            status_pairs.push((v_str("keys_directory"), v_str(&format!("{:?} (does not exist)", keys_dir))));
        }
    }
}
