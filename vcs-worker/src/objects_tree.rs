use std::sync::Arc;
use std::time::Duration;
use sled::{Db, Tree};
use moor_compiler::{compile_object_definitions, ObjFileContext, CompileOptions, ObjectDefinition};
use tracing::{info, warn, error};
use tokio::sync::mpsc;
use tokio::time::sleep;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};

use crate::config::Config;

#[derive(Debug, thiserror::Error)]
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
    #[error("Multiple objects found: expected 1, got {0}")]
    MultipleObjects(usize),
}

/// Represents a file rename operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamedObject {
    pub from: String,
    pub to: String,
}

/// Represents a change in the version control system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: String,
    pub timestamp: u64, // Linux UTC epoch
    pub added_objects: Vec<String>,
    pub modified_objects: Vec<String>,
    pub deleted_objects: Vec<String>,
    pub renamed_objects: Vec<RenamedObject>,
    pub version_overrides: Vec<ObjectVersionOverride>, // Object name + version -> SHA256 mappings
}

/// Represents the current state of the repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub current_change: Option<String>, // Change ID of current working change
}

/// Represents a reference to an object with its current version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRef {
    pub object_name: String,
    pub version: u64, // Monotonic version number
    pub sha256_key: String, // SHA256 hash of the object dump
}

/// Represents an object version override in a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectVersionOverride {
    pub object_name: String,
    pub version: u64,
    pub sha256_key: String,
}

/// Represents the current working state (HEAD) which references object versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Head {
    pub refs: Vec<HeadRef>, // List of object_name + version pairs
}

/// Represents a reference in the HEAD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadRef {
    pub object_name: String,
    pub version: u64,
}

/// Database for storing MOO object definitions, changes, repository state, refs, and HEAD
pub struct Database {
    db: Db,
    objects_tree: Tree,        // SHA256 -> Object dump content
    refs_tree: Tree,          // object_name -> ObjectRef (latest version)
    head_tree: Tree,          // HEAD state (list of refs)
    changes_tree: Tree,       // changes
    repository_tree: Tree,    // repository metadata
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
        let objects_tree = db.open_tree("objects")?;      // SHA256 -> Object dump
        let refs_tree = db.open_tree("refs")?;           // object_name -> ObjectRef  
        let head_tree = db.open_tree("head")?;           // HEAD state
        let changes_tree = db.open_tree("changes")?;     // changes
        let repository_tree = db.open_tree("repository")?; // repository metadata
        
        let object_count = objects_tree.len();
        let change_count = changes_tree.len();
        let ref_count = refs_tree.len();
        info!("Database initialized with {} objects", object_count);
        info!("Changes tree initialized with {} changes", change_count);
        info!("Refs tree initialized with {} refs", ref_count);
        
        // List existing objects for debugging
        if object_count > 0 {
            info!("Existing objects in database:");
            for result in objects_tree.iter() {
                let (key, _) = result?;
                if let Ok(name) = String::from_utf8(key.to_vec()) {
                    info!("  - {}", name);
                }
            }
        }
        
        // Create channel for background flushing
        let (flush_sender, mut flush_receiver) = mpsc::unbounded_channel();
        let db_clone = db.clone();
        
        // Spawn background flush task
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
                                warn!("Periodic flush failed: {}", e);
                            }
                            last_flush = std::time::Instant::now();
                        }
                    }
                }
            }
        });
        
        Ok(Self {
            db,
            objects_tree,
            refs_tree,
            head_tree,
            changes_tree,
            repository_tree,
            flush_sender,
        })
    }

    /// Parse a MOO object dump string into an ObjectDefinition using objdef
    pub fn parse_object_dump(&self, dump: &str) -> Result<ObjectDefinition, ObjectsTreeError> {
        // Use the compiler directly to parse the object definition
        let mut context = ObjFileContext::new();
        let compiled_defs = compile_object_definitions(
            dump,
            &CompileOptions::default(),
            &mut context,
        )?;
        
        // Ensure we got exactly one object
        if compiled_defs.len() != 1 {
            return Err(ObjectsTreeError::MultipleObjects(compiled_defs.len()));
        }
        
        Ok(compiled_defs.into_iter().next().unwrap())
    }

    /// Retrieve object by SHA256 key (the primary way to get objects)
    pub fn get_object_by_sha256(&self, sha256_key: &str) -> Result<Option<String>, ObjectsTreeError> {
        match self.objects_tree.get(sha256_key.as_bytes())? {
            Some(data) => {
                let stored_data = String::from_utf8(data.to_vec())?;
                Ok(Some(stored_data))
            }
            None => Ok(None),
        }
    }

    /// Store object by SHA256 key (the primary way to store objects)
    pub fn store_object_by_sha256(&self, sha256_key: &str, dump: &str) -> Result<(), ObjectsTreeError> {
        self.objects_tree.insert(sha256_key.as_bytes(), dump.as_bytes())?;
        
        // Request background flush
        if let Err(_) = self.flush_sender.send(()) {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Stored object with SHA256 key '{}' in database ({} bytes)", sha256_key, dump.len());
        Ok(())
    }

    /// Get the number of objects in the database
    pub fn count(&self) -> usize {
        self.objects_tree.len()
    }

    /// Request an immediate flush (non-blocking)
    pub fn request_flush(&self) {
        if let Err(_) = self.flush_sender.send(()) {
            warn!("Failed to request flush - channel closed");
        }
    }
    
    /// Flush the database to disk (blocking - use sparingly)
    pub fn flush(&self) -> Result<(), ObjectsTreeError> {
        self.db.flush()?;
        Ok(())
    }

    /// Get the current repository state
    pub fn get_repository(&self) -> Result<Repository, ObjectsTreeError> {
        match self.repository_tree.get("state".as_bytes())? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let repository: Repository = serde_json::from_str(&json)
                    .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON parse error: {}", e)))?;
                Ok(repository)
            }
            None => {
                // Return default repository state (no current change)
                Ok(Repository { current_change: None })
            }
        }
    }

    /// Update the repository state
    pub fn set_repository(&self, repository: &Repository) -> Result<(), ObjectsTreeError> {
        let json = serde_json::to_string(repository)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON serialization error: {}", e)))?;
        self.repository_tree.insert("state".as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if let Err(_) = self.flush_sender.send(()) {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Updated repository state");
        Ok(())
    }

    /// Store a change in the database
    pub fn store_change(&self, change: &Change) -> Result<(), ObjectsTreeError> {
        let json = serde_json::to_string(change)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON serialization error: {}", e)))?;
        self.changes_tree.insert(change.id.as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if let Err(_) = self.flush_sender.send(()) {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Stored change '{}' ({}) in database", change.name, change.id);
        Ok(())
    }

    /// Get a change by ID
    pub fn get_change(&self, change_id: &str) -> Result<Option<Change>, ObjectsTreeError> {
        match self.changes_tree.get(change_id.as_bytes())? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let change: Change = serde_json::from_str(&json)
                    .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON parse error: {}", e)))?;
                Ok(Some(change))
            }
            None => Ok(None),
        }
    }

    /// Get the current change if one is active
    pub fn get_current_change(&self) -> Result<Option<Change>, ObjectsTreeError> {
        let repository = self.get_repository()?;
        if let Some(change_id) = repository.current_change {
            self.get_change(&change_id)
        } else {
            Ok(None)
        }
    }

    /// Create a blank change automatically
    pub fn create_blank_change(&self) -> Result<Change, ObjectsTreeError> {
        let change = Change {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(), // Blank name
            description: None,
            author: "system".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            added_objects: Vec::new(),
            modified_objects: Vec::new(),
            deleted_objects: Vec::new(),
            renamed_objects: Vec::new(),
            version_overrides: Vec::new(),
        };
        
        // Store the change
        self.store_change(&change)?;
        
        // Set it as the current change
        let mut repository = self.get_repository()?;
        repository.current_change = Some(change.id.clone());
        self.set_repository(&repository)?;
        
        info!("Created and set blank change '{}' as current", change.id);
        Ok(change)
    }

    /// Set the current change
    pub fn set_current_change(&self, change_id: Option<String>) -> Result<(), ObjectsTreeError> {
        let mut repository = self.get_repository()?;
        repository.current_change = change_id.clone();
        self.set_repository(&repository)?;
        
        if let Some(id) = change_id {
            info!("Set change '{}' as current", id);
        } else {
            info!("Cleared current change");
        }
        
        Ok(())
    }

    /// Update an existing change
    pub fn update_change(&self, change: &Change) -> Result<(), ObjectsTreeError> {
        self.store_change(change)
    }

    /// Generate SHA256 hash from object dump
    pub fn generate_sha256_hash(&self, dump: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(dump.as_bytes());
        format!("{:x}", hasher.finalize())
    }


    /// Get the current reference for an object name
    pub fn get_object_ref(&self, object_name: &str) -> Result<Option<ObjectRef>, ObjectsTreeError> {
        match self.refs_tree.get(object_name.as_bytes())? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let object_ref: ObjectRef = serde_json::from_str(&json)
                    .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON parse error: {}", e)))?;
                Ok(Some(object_ref))
            }
            None => Ok(None),
        }
    }

    /// Update or create an object reference
    pub fn update_object_ref(&self, object_ref: &ObjectRef) -> Result<(), ObjectsTreeError> {
        let json = serde_json::to_string(object_ref)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON serialization error: {}", e)))?;
        self.refs_tree.insert(object_ref.object_name.as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if let Err(_) = self.flush_sender.send(()) {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Updated ref for object '{}' version {}", object_ref.object_name, object_ref.version);
        Ok(())
    }

    /// Get the next version number for an object
    pub fn get_next_version(&self, object_name: &str) -> Result<u64, ObjectsTreeError> {
        match self.get_object_ref(object_name)? {
            Some(ref_) => Ok(ref_.version + 1),
            None => Ok(1), // First version
        }
    }

    /// Get the current HEAD state
    pub fn get_head(&self) -> Result<Head, ObjectsTreeError> {
        match self.head_tree.get("state".as_bytes())? {
            Some(data) => {
                let json = String::from_utf8(data.to_vec())?;
                let head: Head = serde_json::from_str(&json)
                    .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON parse error: {}", e)))?;
                Ok(head)
            }
            None => {
                // Return default HEAD (empty)
                Ok(Head { refs: Vec::new() })
            }
        }
    }

    /// Update the HEAD state
    pub fn set_head(&self, head: &Head) -> Result<(), ObjectsTreeError> {
        let json = serde_json::to_string(head)
            .map_err(|e| ObjectsTreeError::SerializationError(format!("JSON serialization error: {}", e)))?;
        self.head_tree.insert("state".as_bytes(), json.as_bytes())?;
        
        // Request background flush
        if let Err(_) = self.flush_sender.send(()) {
            warn!("Failed to request background flush - channel closed");
        }
        
        info!("Updated HEAD state with {} refs", head.refs.len());
        Ok(())
    }

    /// Resolve object by name through HEAD state, respecting current change overrides
    pub fn resolve_object_through_head(&self, object_name: &str) -> Result<Option<String>, ObjectsTreeError> {
        info!("Resolving object '{}' through HEAD", object_name);
        
        // First check current change for overrides
        if let Some(current_change) = self.get_current_change()? {
            // Check if deleted
            if current_change.deleted_objects.contains(&object_name.to_string()) {
                return Ok(None);
            }
            
            // Check for renamed object
            if let Some(renamed) = current_change.renamed_objects.iter()
                .find(|r| r.from == object_name) {
                info!("Object '{}' has been renamed to '{}' in current change", object_name, renamed.to);
                return self.resolve_object_through_head(&renamed.to);
            }
            
            // Check for version override
            if let Some(version_override) = current_change.version_overrides.iter()
                .find(|vo| vo.object_name == object_name) {
                info!("Found version override for object '{}' in current change", object_name);
                return self.get_object_by_sha256(&version_override.sha256_key);
            }
        }
        
        // No override found, resolve through HEAD
        let head = self.get_head()?;
        match head.refs.iter().find(|head_ref| head_ref.object_name == object_name) {
            Some(head_ref) => {
                info!("Found object '{}' in HEAD at version {}", object_name, head_ref.version);
                // Verify the reference exists and get the SHA256
                match self.get_object_ref(object_name)? {
                    Some(object_ref) => {
                        if object_ref.version == head_ref.version {
                            self.get_object_by_sha256(&object_ref.sha256_key)
                        } else {
                            error!("Version mismatch: HEAD has version {}, but ref has version {}", 
                                   head_ref.version, object_ref.version);
                            Err(ObjectsTreeError::SerializationError(
                                "Version mismatch between HEAD and refs".to_string()
                            ))
                        }
                    }
                    None => {
                        error!("Object '{}' not found in refs but present in HEAD", object_name);
                        Err(ObjectsTreeError::SerializationError(
                            "Object reference not found".to_string()
                        ))
                    }
                }
            }
            None => {
                info!("Object '{}' not found in HEAD", object_name);
                Ok(None)
            }
        }
    }

    /// Add or update a reference in the HEAD
    pub fn update_head_ref(&self, object_name: &str, version: u64) -> Result<(), ObjectsTreeError> {
        let mut head = self.get_head()?;
        
        // Check if ref already exists and update, or add new
        if let Some(head_ref) = head.refs.iter_mut().find(|hr| hr.object_name == object_name) {
            head_ref.version = version;
            info!("Updated HEAD ref for object '{}' to version {}", object_name, version);
        } else {
            head.refs.push(HeadRef {
                object_name: object_name.to_string(),
                version,
            });
            info!("Added HEAD ref for object '{}' at version {}", object_name, version);
        }
        
        self.set_head(&head)
    }

    /// Remove a reference from HEAD
    pub fn remove_head_ref(&self, object_name: &str) -> Result<(), ObjectsTreeError> {
        let mut head = self.get_head()?;
        head.refs.retain(|hr| hr.object_name != object_name);
        self.set_head(&head)?;
        info!("Removed HEAD ref for object '{}'", object_name);
        Ok(())
    }
}

/// Shared reference to the database for use across the application
pub type DatabaseRef = Arc<Database>;
