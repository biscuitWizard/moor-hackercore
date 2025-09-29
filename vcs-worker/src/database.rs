use std::sync::Arc;
use std::time::Duration;
use sled::Db;
use tracing::{info, warn, error};
use tokio::sync::mpsc;
use tokio::time::sleep;
use crate::config::Config;
use crate::providers::{
    ObjectsProvider, ObjectsProviderImpl,
    RefsProviderImpl,
    HeadProviderImpl,
    ChangesProviderImpl,
    RepositoryProviderImpl,
};

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum ObjectsTreeError {
    #[error("Sled database error: {0}")]
    SledError(#[from] sled::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Compilation error: {0}")]
    CompilationError(#[from] moor_compiler::ObjDefParseError),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

// Re-export types for compatibility
pub use crate::providers::changes::{Change, ObjectVersionOverride};
pub use crate::providers::refs::ObjectRef;

/// Database coordinator that aggregates providers for different subsystems
pub struct Database {
    #[allow(dead_code)]
    db: Db,
    
    // Provider instances
    objects_provider: Arc<ObjectsProviderImpl>,
    refs_provider: Arc<RefsProviderImpl>,
    head_provider: Arc<HeadProviderImpl>,
    changes_provider: Arc<ChangesProviderImpl>,
    repository_provider: Arc<RepositoryProviderImpl>,
    
    #[allow(dead_code)]
    flush_sender: mpsc::UnboundedSender<()>,
}

impl Database {
    /// Create a new database with the given config
    pub fn new(config: &Config) -> Result<Self, ObjectsTreeError> {
        info!("Opening sled database at: {:?}", config.db_path);
        
        // Create the database directory if it doesn't exist
        if let Some(parent) = config.db_path.parent() {
            std::fs::create_dir_all(parent)?;
            info!("Created database directory: {:?}", parent);
        }
        
        let db = sled::open(&config.db_path)?;
        let objects_tree = db.open_tree("objects")?;
        let refs_tree = db.open_tree("refs")?;
        let head_tree = db.open_tree("head")?;
        let changes_tree = db.open_tree("changes")?;
        let repository_tree = db.open_tree("repository")?;
        
        // Create channel for background flushing
        let (flush_sender, mut flush_receiver) = mpsc::unbounded_channel();
        
        // Initialize providers
        let objects_provider = Arc::new(ObjectsProviderImpl::new(objects_tree.clone(), flush_sender.clone()));
        let refs_provider = Arc::new(RefsProviderImpl::new(refs_tree.clone(), flush_sender.clone()));
        let head_provider = Arc::new(HeadProviderImpl::new(head_tree.clone(), flush_sender.clone()));
        let changes_provider = Arc::new(ChangesProviderImpl::new(changes_tree.clone(), flush_sender.clone()));
        let repository_provider = Arc::new(RepositoryProviderImpl::new(repository_tree.clone(), flush_sender.clone()));
        
        info!("Database initialized with {} objects", objects_provider.count());
        info!("Changes tree initialized with {} changes", changes_tree.len());
        info!("Refs tree initialized with {} refs", refs_tree.len());
        
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
        let db_clone = db.clone();
        tokio::spawn(async move {
            let mut last_flush = std::time::Instant::now();
            let flush_interval = Duration::from_secs(5); // Flush every 5 seconds
            
            loop {
                tokio::select! {
                    _ = flush_receiver.recv() => {
                        // Immediate flush requested
                        if let Err(e) = db_clone.flush() {
                            warn!("Background flush failed: {}", e);
                        } else {
                            info!("Background flush completed");
                        }
                        last_flush = std::time::Instant::now();
                    }
                    _ = sleep(flush_interval) => {
                        // Periodic flush
                        if last_flush.elapsed() >= flush_interval {
                            if let Err(e) = db_clone.flush() {
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
            db,
            objects_provider,
            refs_provider,
            head_provider,
            changes_provider,
            repository_provider,
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

    /// Get direct access to the head provider
    pub fn head(&self) -> &Arc<HeadProviderImpl> {
        &self.head_provider
    }

    /// Get direct access to the changes provider
    pub fn changes(&self) -> &Arc<ChangesProviderImpl> {
        &self.changes_provider
    }

    /// Get direct access to the repository provider
    pub fn repository(&self) -> &Arc<RepositoryProviderImpl> {
        &self.repository_provider
    }
}

/// Shared reference to the database for use across the application
pub type DatabaseRef = Arc<Database>;