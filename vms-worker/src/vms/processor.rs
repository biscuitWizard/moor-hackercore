use tracing::{info, error};
use moor_var::{Var, v_str, v_map};
use crate::config::Config;
use crate::git_ops::GitRepository;
use super::types::VmsOperation;
use super::repository_manager::RepositoryManager;
use super::object_handler::ObjectHandler;
use super::status_handler::StatusHandler;
use moor_common::tasks::WorkerError;

/// Process VMS operations
pub struct VmsProcessor {
    git_repo: Option<GitRepository>,
    config: Config,
    object_handler: ObjectHandler,
    status_handler: StatusHandler,
}

impl VmsProcessor {
    pub fn new() -> Self {
        let config = Config::from_env();
        let mut processor = Self { 
            git_repo: None,
            config: config.clone(),
            object_handler: ObjectHandler::new(config.clone()),
            status_handler: StatusHandler,
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
        };
        processor.initialize_repository();
        processor
    }
    
    /// Initialize the git repository using configuration
    pub fn initialize_repository(&mut self) {
        let repository_manager = RepositoryManager::new(self.config.clone());
        self.git_repo = repository_manager.initialize_repository();
    }
    
    /// Process a VMS operation
    pub fn process_operation(&mut self, operation: VmsOperation) -> Result<Vec<Var>, WorkerError> {
        match operation {            
            VmsOperation::AddOrUpdateObject { object_dump, object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.add_object(repo, object_dump, object_name)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VmsOperation::DeleteObject { object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.delete_object(repo, object_name, None)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VmsOperation::Commit { message, author_name, author_email } => {
                if let Some(ref repo) = self.git_repo {
                    self.create_commit(repo, message, author_name, author_email)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VmsOperation::Status => {
                if let Some(ref repo) = self.git_repo {
                    self.status_handler.get_repository_status(repo)
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
                Ok(vec![v_str(&format!("Created commit: {}", message))])
            }
            Err(e) => {
                error!("Failed to create commit: {}", e);
                Err(WorkerError::RequestError(format!("Failed to create commit: {}", e)))
            }
        }
    }
}
