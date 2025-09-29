use sled::Tree;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use tokio::sync::mpsc;

use super::{ProviderError, ProviderResult};

/// Represents the current state of the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub current_change: Option<String>, // Change ID of current working change
    pub metadata: RepositoryMetadata,
}

/// Repository-level metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_timestamp: u64,
    pub last_modified: u64,
}

/// Provider trait for repository metadata management
pub trait RepositoryProvider: Send + Sync {
    /// Get the current repository state
    fn get_repository(&self) -> ProviderResult<Repository>;
    
    /// Update the repository state
    fn set_repository(&self, repository: &Repository) -> ProviderResult<()>;
    
    /// Update repository metadata
    #[allow(dead_code)]
    fn update_metadata(&self, metadata: &RepositoryMetadata) -> ProviderResult<()>;
    
    /// Set the current change
    #[allow(dead_code)]
    fn set_current_change(&self, change_id: Option<String>) -> ProviderResult<()>;
    
    /// Get the current change ID
    #[allow(dead_code)]
    fn get_current_change_id(&self) -> ProviderResult<Option<String>>;
    
    /// Update the last modified timestamp
    fn touch(&self) -> ProviderResult<()>;
}

/// Implementation of RepositoryProvider using Sled
pub struct RepositoryProviderImpl {
    repository_tree: Tree,
    flush_sender: mpsc::UnboundedSender<()>,
}

impl RepositoryProviderImpl {
    /// Create a new repository provider
    pub fn new(repository_tree: Tree, flush_sender: mpsc::UnboundedSender<()>) -> Self {
        Self { repository_tree, flush_sender }
    }
}

impl RepositoryProvider for RepositoryProviderImpl {
    fn get_repository(&self) -> ProviderResult<Repository> {
        match self.repository_tree.get("state".as_bytes())? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let repository: Repository = serde_json::from_str(&json)
                    .map_err(|e| ProviderError::SerializationError(format!("JSON parse error: {e}")))?;
                Ok(repository)
            }
            None => {
                // Return default repository state
                Ok(Repository { 
                    current_change: None,
                    metadata: RepositoryMetadata {
                        name: None,
                        description: None,
                        created_timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                        last_modified: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    },
                })
            }
        }
    }

    fn set_repository(&self, repository: &Repository) -> ProviderResult<()> {
        let json = serde_json::to_string(repository)
            .map_err(|e| ProviderError::SerializationError(format!("JSON serialization error: {e}")))?;
        self.repository_tree.insert("state".as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if self.flush_sender.send(()).is_err() {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Updated repository state");
        Ok(())
    }

    fn update_metadata(&self, metadata: &RepositoryMetadata) -> ProviderResult<()> {
        let mut repository = self.get_repository()?;
        repository.metadata = metadata.clone();
        repository.metadata.last_modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.set_repository(&repository)
    }

    fn set_current_change(&self, change_id: Option<String>) -> ProviderResult<()> {
        let mut repository = self.get_repository()?;
        repository.current_change = change_id.clone();
        self.touch()?; // Update last modified timestamp
        self.set_repository(&repository)?;
        
        if let Some(id) = change_id {
            info!("Set change '{}' as current", id);
        } else {
            info!("Cleared current change");
        }
        
        Ok(())
    }

    fn get_current_change_id(&self) -> ProviderResult<Option<String>> {
        let repository = self.get_repository()?;
        Ok(repository.current_change)
    }

    fn touch(&self) -> ProviderResult<()> {
        let mut repository = self.get_repository()?;
        repository.metadata.last_modified = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.set_repository(&repository)
    }
}
