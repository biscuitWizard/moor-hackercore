use tracing::{info, error, warn};
use moor_var::{Var, v_str, v_list};
use crate::config::Config;
use crate::git::GitRepository;
use crate::error_utils::ErrorUtils;
use super::types::VcsOperation;
use super::object_handler::ObjectHandler;
use super::status_handler::StatusHandler;
use super::meta_handler::MetaHandler;
use super::workflow_handler::WorkflowHandler;
use moor_common::tasks::WorkerError;

/// Process VCS operations
pub struct VcsProcessor {
    git_repo: Option<GitRepository>,
    config: Config,
    object_handler: ObjectHandler,
    status_handler: StatusHandler,
    meta_handler: MetaHandler,
    workflow_handler: WorkflowHandler,
}

impl VcsProcessor {
    pub fn new() -> Self {
        let config = Config::from_env();
        let mut processor = Self { 
            git_repo: None,
            config: config.clone(),
            object_handler: ObjectHandler::new(config.clone()),
            status_handler: StatusHandler,
            meta_handler: MetaHandler::new(config.clone()),
            workflow_handler: WorkflowHandler::new(config),
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
            meta_handler: MetaHandler::new(config.clone()),
            workflow_handler: WorkflowHandler::new(config),
        };
        processor.initialize_repository();
        processor
    }
    
    /// Initialize the git repository using configuration
    pub fn initialize_repository(&mut self) {
        use crate::git::operations::InitOps;
        
        info!("VcsProcessor: Initializing repository");
        let repo_path = self.config.repository_path();
        info!("VcsProcessor: Using repository path: {:?}", repo_path);
        
        match InitOps::initialize_repository(&repo_path, &self.config) {
            Ok(Some(repo)) => {
                info!("VcsProcessor: Successfully initialized git repository");
                self.git_repo = Some(repo);
            }
            Ok(None) => {
                warn!("VcsProcessor: Failed to initialize git repository - continuing without git");
                self.git_repo = None;
            }
            Err(e) => {
                error!("VcsProcessor: Failed to initialize git repository: {}", e);
                self.git_repo = None;
            }
        }
    }
    
    
    /// Process a VCS operation
    pub fn process_operation(&mut self, operation: VcsOperation) -> Result<Var, WorkerError> {
        info!("VcsProcessor: Processing operation: {:?}", operation);
        match operation {            
            VcsOperation::AddOrUpdateObject { object_dump, object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.add_object(repo, object_dump, object_name)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::DeleteObject { object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.delete_object(repo, object_name, None)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::RenameObject { old_name, new_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.rename_object(repo, old_name, new_name)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::Commit { message, author_name, author_email } => {
                if let Some(ref repo) = self.git_repo {
                    self.workflow_handler.execute_commit_workflow(repo, message, author_name, author_email)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::Status => {
                info!("VcsProcessor: Processing Status operation");
                if let Some(ref repo) = self.git_repo {
                    info!("VcsProcessor: Git repository is available, calling status handler");
                    self.status_handler.get_repository_status(repo, &self.config)
                } else {
                    let repo_path = self.config.repository_path().to_str().unwrap_or("/game");
                    error!("VcsProcessor: Git repository not available at {}", repo_path);
                    Err(ErrorUtils::git_repo_not_available(Some(repo_path)))
                }
            }
            
            VcsOperation::ListObjects => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.list_objects(repo)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::GetObjects { object_names } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.get_objects(repo, object_names)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::GetCommits { limit, offset } => {
                if let Some(ref repo) = self.git_repo {
                    use crate::git::operations::CommitOps;
                    use moor_var::v_map;
                    
                    match CommitOps::get_commits(repo.repo(), limit, offset) {
                        Ok(commits) => {
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
                            Ok(v_list(&result))
                        }
                        Err(e) => {
                            error!("Failed to get commits: {}", e);
                            Err(ErrorUtils::operation_failed("get commits", &e.to_string()))
                        }
                    }
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
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
                Ok(v_str(&format!("SSH key set successfully: {}", key_name)))
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
                Ok(v_str("SSH key configuration cleared"))
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
                        Ok(v_str("Git user updated successfully"))
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
                            Ok(v_str("SSH connection test successful"))
                        },
                        Err(e) => {
                            error!("SSH connection test failed: {}", e);
                            Err(ErrorUtils::operation_failed("SSH connection test", &e.to_string()))
                        }
                    }
                } else {
                    error!("Git repository not available for SSH test");
                    Err(ErrorUtils::git_repo_not_available_ssh())
                }
            }
            
            VcsOperation::UpdateIgnoredProperties { object_name, properties } => {
                if let Some(ref repo) = self.git_repo {
                    self.meta_handler.update_ignored_properties(repo, object_name, properties)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::UpdateIgnoredVerbs { object_name, verbs } => {
                if let Some(ref repo) = self.git_repo {
                    self.meta_handler.update_ignored_verbs(repo, object_name, verbs)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::Pull { dry_run } => {
                if let Some(ref repo) = self.git_repo {
                    self.workflow_handler.execute_pull_workflow(repo, dry_run)
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::Reset => {
                if let Some(ref repo) = self.git_repo {
                    use crate::git::operations::StatusOps;
                    
                    match StatusOps::reset_working_tree_with_verification(repo.repo(), repo.work_dir()) {
                        Ok(messages) => {
                            let vars: Vec<Var> = messages.into_iter().map(|msg| v_str(&msg)).collect();
                            Ok(v_list(&vars))
                        }
                        Err(e) => {
                            error!("Failed to reset working tree: {}", e);
                            Err(ErrorUtils::operation_failed("reset working tree", &e.to_string()))
                        }
                    }
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::Stash => {
                if let Some(ref repo) = self.git_repo {
                    match self.workflow_handler.stash_changes(repo) {
                        Ok(stashed_objects) => {
                            info!("Successfully stashed {} objects", stashed_objects.len());
                            // Store the stashed objects in the processor for later replay
                            // For now, we'll just return a success message
                            Ok(v_str(&format!("Stashed {} objects successfully", stashed_objects.len())))
                        }
                        Err(e) => {
                            error!("Failed to stash changes: {}", e);
                            Err(ErrorUtils::operation_failed("stash changes", &e.to_string()))
                        }
                    }
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
            
            VcsOperation::ReplayStash => {
                if let Some(ref _repo) = self.git_repo {
                    // For now, we'll return an error since we need to implement proper stash storage
                    // In a real implementation, we'd store the stashed objects and retrieve them here
                    Err(ErrorUtils::operation_failed("replay stash", "Stash storage not implemented yet"))
                } else {
                    Err(ErrorUtils::git_repo_not_available(Some(self.config.repository_path().to_str().unwrap_or("/game"))))
                }
            }
        }
    }
    
}
