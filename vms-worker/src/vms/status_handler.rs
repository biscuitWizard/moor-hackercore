use std::collections::HashMap;
use tracing::error;
use moor_var::{Var, v_str, v_map};
use crate::git_ops::GitRepository;
use moor_common::tasks::WorkerError;

/// Handles repository status and commit information operations
pub struct StatusHandler;

impl StatusHandler {    
    /// Get comprehensive repository status
    pub fn get_repository_status(&self, repo: &GitRepository) -> Result<Vec<Var>, WorkerError> {
        match self.collect_repository_status_map(repo) {
            Ok(status_map) => {
                // Convert HashMap<String, String> to Vec<(Var, Var)> for v_map
                let map_pairs: Vec<(Var, Var)> = status_map
                    .iter()
                    .map(|(k, v)| (v_str(k), v_str(v)))
                    .collect();
                Ok(vec![v_map(&map_pairs)])
            }
            Err(e) => {
                error!("Failed to get repository status: {}", e);
                Err(WorkerError::RequestError(format!("Failed to get repository status: {}", e)))
            }
        }
    }
    
    /// Collect comprehensive repository status information as a map
    fn collect_repository_status_map(&self, repo: &GitRepository) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut status_map = HashMap::new();
        
        // Get current branch
        match repo.get_current_branch() {
            Ok(Some(branch)) => {
                status_map.insert("current_branch".to_string(), branch);
            }
            Ok(None) => {
                status_map.insert("current_branch".to_string(), "(detached HEAD)".to_string());
            }
            Err(e) => {
                error!("Failed to get current branch: {}", e);
                status_map.insert("current_branch".to_string(), "(error)".to_string());
            }
        }
        
        // Get upstream information
        match repo.get_upstream_info() {
            Ok(Some(upstream)) => {
                status_map.insert("upstream".to_string(), upstream);
            }
            Ok(None) => {
                status_map.insert("upstream".to_string(), "(none)".to_string());
            }
            Err(e) => {
                error!("Failed to get upstream info: {}", e);
                status_map.insert("upstream".to_string(), "(error)".to_string());
            }
        }
        
        // Get last commit information
        match repo.get_last_commit_info() {
            Ok(Some(commit)) => {
                status_map.insert("last_commit_id".to_string(), commit.id);
                status_map.insert("last_commit_full_id".to_string(), commit.full_id);
                status_map.insert("last_commit_message".to_string(), commit.message);
                status_map.insert("last_commit_author".to_string(), commit.author);
                status_map.insert("last_commit_datetime".to_string(), commit.datetime);
            }
            Ok(None) => {
                status_map.insert("last_commit_id".to_string(), "(no commits yet)".to_string());
                status_map.insert("last_commit_full_id".to_string(), "(no commits yet)".to_string());
                status_map.insert("last_commit_message".to_string(), "(no commits yet)".to_string());
                status_map.insert("last_commit_author".to_string(), "(no commits yet)".to_string());
                status_map.insert("last_commit_datetime".to_string(), "(no commits yet)".to_string());
            }
            Err(e) => {
                error!("Failed to get last commit info: {}", e);
                status_map.insert("last_commit_id".to_string(), "(error)".to_string());
                status_map.insert("last_commit_full_id".to_string(), "(error)".to_string());
                status_map.insert("last_commit_message".to_string(), "(error)".to_string());
                status_map.insert("last_commit_author".to_string(), "(error)".to_string());
                status_map.insert("last_commit_datetime".to_string(), "(error)".to_string());
            }
        }
        
        // Get current changes
        match repo.status() {
            Ok(changes) => {
                if changes.is_empty() {
                    status_map.insert("changes".to_string(), "(working tree clean)".to_string());
                } else {
                    status_map.insert("changes".to_string(), changes.join("\n"));
                }
            }
            Err(e) => {
                error!("Failed to get status: {}", e);
                status_map.insert("changes".to_string(), "(error)".to_string());
            }
        }
        
        Ok(status_map)
    }
}
