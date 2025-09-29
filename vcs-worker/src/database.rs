use std::sync::Arc;
use std::time::Duration;
use fjall::{Config as FjallConfig, Keyspace, PersistMode};
use tracing::{info, warn, error};
use tokio::sync::mpsc;
use tokio::time::sleep;
use crate::config::Config;
use crate::providers::{
    ObjectsProvider, ObjectsProviderImpl,
    RefsProviderImpl,
    IndexProviderImpl,
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
    
    #[allow(dead_code)]
    flush_sender: mpsc::UnboundedSender<()>,
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
        
        // Create channel for background flushing
        let (flush_sender, mut flush_receiver) = mpsc::unbounded_channel();
        
        // Initialize providers
        let objects_provider = Arc::new(ObjectsProviderImpl::new(objects_tree.clone(), flush_sender.clone()));
        let refs_provider = Arc::new(RefsProviderImpl::new(refs_tree.clone(), flush_sender.clone()));
        let index_provider = Arc::new(IndexProviderImpl::new(index_tree.clone(), changes_tree.clone(), flush_sender.clone()));
        
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
            flush_sender,
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


}

/// Shared reference to the database for use across the application
pub type DatabaseRef = Arc<Database>;