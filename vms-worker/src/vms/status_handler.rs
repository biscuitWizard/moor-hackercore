use tracing::error;
use moor_var::{Var, v_str, v_map, v_list, v_int};
use crate::git_ops::GitRepository;
use moor_common::tasks::WorkerError;

/// Handles repository status and commit information operations
pub struct StatusHandler;

impl StatusHandler {    
    /// Get comprehensive repository status
    pub fn get_repository_status(&self, repo: &GitRepository) -> Result<Vec<Var>, WorkerError> {
        match self.collect_repository_status_vars(repo) {
            Ok(status_pairs) => {
                Ok(vec![v_map(&status_pairs)])
            }
            Err(e) => {
                error!("Failed to get repository status: {}", e);
                Err(WorkerError::RequestError(format!("Failed to get repository status: {}", e)))
            }
        }
    }
    
    /// Collect comprehensive repository status information as Var pairs
    fn collect_repository_status_vars(&self, repo: &GitRepository) -> Result<Vec<(Var, Var)>, Box<dyn std::error::Error>> {
        let mut status_pairs = Vec::new();
        
        // Get current branch
        match repo.get_current_branch() {
            Ok(Some(branch)) => {
                status_pairs.push((v_str("current_branch"), v_str(&branch)));
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
        match repo.get_upstream_info() {
            Ok(Some(upstream)) => {
                status_pairs.push((v_str("upstream"), v_str(&upstream)));
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
        
        Ok(status_pairs)
    }
}
