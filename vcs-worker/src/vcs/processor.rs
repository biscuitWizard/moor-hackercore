use tracing::{info, error};
use moor_var::{Var, v_str};
use crate::config::Config;
use crate::git_ops::GitRepository;
use super::types::VcsOperation;
use super::repository_manager::RepositoryManager;
use super::object_handler::ObjectHandler;
use super::status_handler::StatusHandler;
use super::meta_handler::MetaHandler;
use moor_common::tasks::WorkerError;

/// Process VCS operations
pub struct VcsProcessor {
    git_repo: Option<GitRepository>,
    config: Config,
    object_handler: ObjectHandler,
    status_handler: StatusHandler,
    meta_handler: MetaHandler,
}

impl VcsProcessor {
    pub fn new() -> Self {
        let config = Config::from_env();
        let mut processor = Self { 
            git_repo: None,
            config: config.clone(),
            object_handler: ObjectHandler::new(config.clone()),
            status_handler: StatusHandler,
            meta_handler: MetaHandler::new(config),
        };
        processor.initialize_repository();
        processor
    }
    
    pub fn with_config(config: Config) -> Self {
        let mut processor = Self { 
            git_repo: None,
            config: config.clone(),
            object_handler: ObjectHandler::new(config.clone()),
            status_handler: StatusHandler,
            meta_handler: MetaHandler::new(config),
        };
        processor.initialize_repository();
        processor
    }
    
    /// Initialize the git repository using configuration
    pub fn initialize_repository(&mut self) {
        let repository_manager = RepositoryManager::new(self.config.clone());
        self.git_repo = repository_manager.initialize_repository();
    }
    
    /// Process a VCS operation
    pub fn process_operation(&mut self, operation: VcsOperation) -> Result<Vec<Var>, WorkerError> {
        match operation {            
            VcsOperation::AddOrUpdateObject { object_dump, object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.add_object(repo, object_dump, object_name)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::DeleteObject { object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.delete_object(repo, object_name, None)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::RenameObject { old_name, new_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.rename_object(repo, old_name, new_name)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::Commit { message, author_name, author_email } => {
                if let Some(ref repo) = self.git_repo {
                    self.create_commit(repo, message, author_name, author_email)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::Status => {
                if let Some(ref repo) = self.git_repo {
                    self.status_handler.get_repository_status(repo, &self.config)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::ListObjects => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.list_objects(repo)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::GetObjects { object_names } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.get_objects(repo, object_names)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::GetCommits { limit, offset } => {
                if let Some(ref repo) = self.git_repo {
                    self.get_commits(repo, limit, offset)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            // Credential management operations
            VcsOperation::SetSshKey { key_content, key_name } => {
                info!("Setting SSH key: {} ({} bytes)", key_name, key_content.len());
                
                let keys_dir = self.config.keys_directory();
                
                // Create keys directory if it doesn't exist
                if !keys_dir.exists() {
                    info!("Creating keys directory: {:?}", keys_dir);
                    if let Err(e) = std::fs::create_dir_all(&keys_dir) {
                        error!("Failed to create keys directory: {}", e);
                        return Err(WorkerError::RequestError(format!("Failed to create keys directory: {}", e)));
                    }
                }
                
                let key_path = keys_dir.join(&key_name);
                info!("Writing SSH key to: {:?}", key_path);
                
                // Write the key content
                if let Err(e) = std::fs::write(&key_path, key_content) {
                    error!("Failed to write SSH key to {:?}: {}", key_path, e);
                    return Err(WorkerError::RequestError(format!("Failed to write SSH key: {}", e)));
                }
                
                // Set restrictive permissions
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&key_path).map_err(|e| WorkerError::RequestError(format!("Failed to get metadata: {}", e)))?.permissions();
                perms.set_mode(0o600);
                std::fs::set_permissions(&key_path, perms).map_err(|e| WorkerError::RequestError(format!("Failed to set permissions: {}", e)))?;
                
                // Update config to use this key
                if let Err(e) = self.config.update_ssh_key(key_path.to_string_lossy().to_string()) {
                    error!("Failed to update SSH key configuration: {}", e);
                    return Err(WorkerError::RequestError(format!("Failed to update SSH key configuration: {}", e)));
                }
                
                info!("SSH key set successfully: {} at {:?}", key_name, key_path);
                Ok(vec![v_str(&format!("SSH key set successfully: {}", key_name))])
            }
            
            VcsOperation::ClearSshKey => {
                info!("Clearing SSH key configuration");
                self.config.clear_ssh_key();
                
                // Also clear the keys directory
                let keys_dir = self.config.keys_directory();
                if keys_dir.exists() {
                    info!("Clearing keys directory: {:?}", keys_dir);
                    if let Err(e) = std::fs::remove_dir_all(&keys_dir) {
                        error!("Failed to clear keys directory {:?}: {}", keys_dir, e);
                        // Don't fail the operation, just log the error
                    } else {
                        info!("Keys directory cleared successfully");
                    }
                }
                
                info!("SSH key configuration cleared");
                Ok(vec![v_str("SSH key configuration cleared")])
            }
            
            VcsOperation::SetGitUser { name, email } => {
                info!("Setting git user: {} <{}>", name, email);
                match self.config.set_git_user(name, email) {
                    Ok(_) => {
                        // Reconfigure git user in the repository
                        if let Some(ref repo) = self.git_repo {
                            if let Err(e) = repo.configure_git_user() {
                                error!("Failed to reconfigure git user in repository: {}", e);
                            } else {
                                info!("Git user reconfigured in repository successfully");
                            }
                        }
                        info!("Git user updated successfully");
                        Ok(vec![v_str("Git user updated successfully")])
                    }
                    Err(e) => {
                        error!("Failed to update git user: {}", e);
                        Err(WorkerError::RequestError(format!("Failed to update git user: {}", e)))
                    }
                }
            }
            
            
            VcsOperation::TestSshConnection => {
                info!("Testing SSH connection to remote repository");
                if let Some(ref repo) = self.git_repo {
                    match repo.test_ssh_connection() {
                        Ok(_) => {
                            info!("SSH connection test successful");
                            Ok(vec![v_str("SSH connection test successful")])
                        },
                        Err(e) => {
                            error!("SSH connection test failed: {}", e);
                            Err(WorkerError::RequestError(format!("SSH connection test failed: {}", e)))
                        }
                    }
                } else {
                    error!("Git repository not available for SSH test");
                    Err(WorkerError::RequestError("Git repository not available".to_string()))
                }
            }
            
            VcsOperation::UpdateIgnoredProperties { object_name, properties } => {
                if let Some(ref repo) = self.git_repo {
                    self.meta_handler.update_ignored_properties(repo, object_name, properties)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::UpdateIgnoredVerbs { object_name, verbs } => {
                if let Some(ref repo) = self.git_repo {
                    self.meta_handler.update_ignored_verbs(repo, object_name, verbs)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
        }
    }
    
    /// Create a commit with current changes
    fn create_commit(
        &self, 
        repo: &GitRepository, 
        message: String,
        author_name: String,
        author_email: String,
    ) -> Result<Vec<Var>, WorkerError> {
        match repo.commit(&message, &author_name, &author_email) {
            Ok(_) => {
                info!("Created commit: {}", message);
                
                // Now push the commit to the remote
                match repo.push() {
                    Ok(_) => {
                        info!("Successfully pushed commit to remote");
                        Ok(vec![v_str(&format!("Created and pushed commit: {}", message))])
                    }
                    Err(e) => {
                        error!("Failed to push commit: {}", e);
                        // Return success for commit but note the push failure
                        Ok(vec![v_str(&format!("Created commit: {} (but push failed: {})", message, e))])
                    }
                }
            }
            Err(e) => {
                error!("Failed to create commit: {}", e);
                Err(WorkerError::RequestError(format!("Failed to create commit: {}", e)))
            }
        }
    }
    
    /// Get paginated list of commits
    fn get_commits(
        &self, 
        repo: &GitRepository, 
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Var>, WorkerError> {
        match repo.get_commits(limit, offset) {
            Ok(commits) => {
                use moor_var::v_map;
                
                let mut result = Vec::new();
                for commit in commits {
                    let commit_info = v_map(&[
                        (v_str("id"), v_str(&commit.id)),
                        (v_str("full_id"), v_str(&commit.full_id)),
                        (v_str("datetime"), v_str(&commit.datetime.to_string())),
                        (v_str("message"), v_str(&commit.message)),
                        (v_str("author"), v_str(&commit.author)),
                    ]);
                    result.push(commit_info);
                }
                
                info!("Retrieved {} commits", result.len());
                Ok(result)
            }
            Err(e) => {
                error!("Failed to get commits: {}", e);
                Err(WorkerError::RequestError(format!("Failed to get commits: {}", e)))
            }
        }
    }
    
}
