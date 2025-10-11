use std::sync::Arc;
use std::time::Duration;
use fjall::{Config as FjallConfig, Keyspace, PersistMode};
use tracing::{info, warn, error};
use tokio::sync::mpsc;
use tokio::time::sleep;
use crate::config::Config;
use crate::providers::{
    objects::ObjectsProvider,
    ObjectsProviderImpl,
    refs::RefsProvider,
    RefsProviderImpl,
    index::IndexProvider,
    IndexProviderImpl,
    workspace::WorkspaceProvider,
    UserProviderImpl,
    WorkspaceProviderImpl,
};

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum ObjectsTreeError {
    #[error("Fjall database error: {0}")]
    FjallError(#[from] fjall::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Compilation error: {0}")]
    CompilationError(#[from] moor_compiler::ObjDefParseError),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}


/// Database coordinator that aggregates providers for different subsystems
pub struct Database {
    #[allow(dead_code)]
    keyspace: Keyspace,
    
    // Provider instances
    objects_provider: Arc<ObjectsProviderImpl>,
    refs_provider: Arc<RefsProviderImpl>,
    index_provider: Arc<IndexProviderImpl>,
    user_provider: Arc<UserProviderImpl>,
    workspace_provider: Arc<WorkspaceProviderImpl>,
    
    #[allow(dead_code)]
    flush_sender: mpsc::UnboundedSender<()>,
    
    // Store the database path for partition size calculations
    #[allow(dead_code)]
    db_path: std::path::PathBuf,
    
    // Store the game name
    game_name: String,
}

impl Database {
    /// Create a new database with the given config
    pub fn new(config: &Config) -> Result<Self, ObjectsTreeError> {
        info!("Opening fjall database at: {:?}", config.db_path);
        
        // Create the database directory if it doesn't exist
        if let Some(parent) = config.db_path.parent() {
            std::fs::create_dir_all(parent)?;
            info!("Created database directory: {:?}", parent);
        }
        
        let keyspace = FjallConfig::new(&config.db_path).open()?;
        let objects_tree = keyspace.open_partition("objects", fjall::PartitionCreateOptions::default())?;
        let refs_tree = keyspace.open_partition("refs", fjall::PartitionCreateOptions::default())?;
        let index_tree = keyspace.open_partition("index", fjall::PartitionCreateOptions::default())?;
        let changes_tree = keyspace.open_partition("changes", fjall::PartitionCreateOptions::default())?;
        let workspace_tree = keyspace.open_partition("workspace", fjall::PartitionCreateOptions::default())?;
        let users_tree = keyspace.open_partition("users", fjall::PartitionCreateOptions::default())?;
        
        // Create channel for background flushing
        let (flush_sender, mut flush_receiver) = mpsc::unbounded_channel();
        
        // Initialize providers
        let objects_provider = Arc::new(ObjectsProviderImpl::new(objects_tree.clone(), flush_sender.clone()));
        let refs_provider = Arc::new(RefsProviderImpl::new(refs_tree.clone(), flush_sender.clone()));
        let index_provider = Arc::new(IndexProviderImpl::new(index_tree.clone(), changes_tree.clone(), flush_sender.clone()));
        let user_provider = Arc::new(UserProviderImpl::new(users_tree.clone(), flush_sender.clone()));
        let workspace_provider = Arc::new(WorkspaceProviderImpl::new(workspace_tree.clone(), flush_sender.clone()));
        
        info!("Database initialized with {} objects", objects_provider.count());
        info!("Changes tree initialized with {} changes", changes_tree.len().unwrap_or(0));
        info!("Workspace tree initialized with {} workspace changes", workspace_tree.len().unwrap_or(0));
        info!("Refs tree initialized with {} refs", refs_tree.len().unwrap_or(0));
        
        // List existing objects for debugging
        let object_count = objects_provider.count();
        if object_count > 0 {
            info!("Existing objects in database:");
            for result in objects_tree.iter() {
                let (key, _) = result?;
                if let Ok(name) = String::from_utf8(key.to_vec()) {
                    info!("  - {}", name);
                }
            }


        }
        
        // Spawn background flush task
        let keyspace_clone = keyspace.clone();
        tokio::spawn(async move {
            let mut last_flush = std::time::Instant::now();
            let flush_interval = Duration::from_secs(5); // Flush every 5 seconds
            
            loop {
                tokio::select! {
                    _ = flush_receiver.recv() => {
                        // Immediate flush requested
                        if let Err(e) = keyspace_clone.persist(PersistMode::SyncAll) {
                            warn!("Background flush failed: {}", e);
                        } else {
                            info!("Background flush completed");
                        }
                        last_flush = std::time::Instant::now();
                    }
                    _ = sleep(flush_interval) => {
                        // Periodic flush
                        if last_flush.elapsed() >= flush_interval {
                            if let Err(e) = keyspace_clone.persist(PersistMode::SyncAll) {
                                warn!("Periodic background flush failed: {}", e);
                            } else {
                                info!("Periodic background flush completed");
                            }
                            last_flush = std::time::Instant::now();
                        }
                    }
                }
            }
        });
        
        Ok(Self {
            keyspace,
            objects_provider,
            refs_provider,
            index_provider,
            user_provider,
            workspace_provider,
            flush_sender,
            db_path: config.db_path.clone(),
            game_name: config.game_name.clone(),
        })
    }

    /// Get direct access to the objects provider
    pub fn objects(&self) -> &Arc<ObjectsProviderImpl> {
        &self.objects_provider
    }

    /// Get direct access to the refs provider
    pub fn refs(&self) -> &Arc<RefsProviderImpl> {
        &self.refs_provider
    }

    /// Get direct access to the index provider
    pub fn index(&self) -> &Arc<IndexProviderImpl> {
        &self.index_provider
    }

    /// Get direct access to the user provider
    pub fn users(&self) -> &Arc<UserProviderImpl> {
        &self.user_provider
    }

    /// Get direct access to the workspace provider
    pub fn workspace(&self) -> &Arc<WorkspaceProviderImpl> {
        &self.workspace_provider
    }
    
    /// Get the database path
    #[allow(dead_code)]
    pub fn db_path(&self) -> &std::path::Path {
        &self.db_path
    }
    
    /// Get the game name
    pub fn game_name(&self) -> &str {
        &self.game_name
    }
    
    /// Get the data size of a partition by iterating through all entries
    pub fn get_partition_data_size(&self, partition_name: &str) -> u64 {
        match partition_name {
            "objects" => self.objects_provider.get_data_size(),
            "refs" => self.refs_provider.get_data_size(),
            "index" => self.index_provider.get_index_data_size(),
            "changes" => self.index_provider.get_changes_data_size(),
            _ => 0,
        }
    }
    
    /// Resolve a possibly-short hash to a full hash by searching both index and workspace
    /// Returns the full hash if found uniquely, error if ambiguous or not found
    pub fn resolve_change_id(&self, short_or_full: &str) -> Result<String, ObjectsTreeError> {
        // If it's already a full hash (64 chars for Blake3), return it
        if short_or_full.len() == 64 {
            return Ok(short_or_full.to_string());
        }
        
        // Collect all change IDs from both index and workspace
        let mut all_change_ids = Vec::new();
        
        // Get changes from index
        if let Ok(change_order) = self.index_provider.get_change_order() {
            all_change_ids.extend(change_order);
        }
        
        // Get changes from workspace
        if let Ok(workspace_changes) = self.workspace_provider.list_all_workspace_changes() {
            all_change_ids.extend(workspace_changes.into_iter().map(|c| c.id));
        }
        
        // Find matches
        let matches: Vec<String> = all_change_ids
            .into_iter()
            .filter(|hash| hash.starts_with(short_or_full))
            .collect();
        
        match matches.len() {
            0 => Err(ObjectsTreeError::SerializationError(
                format!("Change ID '{short_or_full}' not found")
            )),
            1 => Ok(matches[0].clone()),
            _ => Err(ObjectsTreeError::SerializationError(
                format!("Ambiguous change ID prefix '{short_or_full}' matches multiple changes. Please provide more characters.")
            )),
        }
    }
}

/// Shared reference to the database for use across the application
pub type DatabaseRef = Arc<Database>;