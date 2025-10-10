//! Common test utilities and harness for vcs-worker integration tests
//!
//! This module provides:
//! - TestServer: Manages test server lifecycle with temporary database
//! - Helper functions for loading test resources
//! - Common assertions and utilities

use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;
use sha2::{Sha256, Digest};

use moor_vcs_worker::{create_registry_with_config, Config, DatabaseRef};
use moor_vcs_worker::router::create_http_router;

// Import provider traits so their methods are available
pub use moor_vcs_worker::providers::objects::ObjectsProvider;
pub use moor_vcs_worker::providers::refs::RefsProvider;
pub use moor_vcs_worker::providers::index::IndexProvider;
pub use moor_vcs_worker::providers::user::UserProvider;
pub use moor_vcs_worker::types::User;

/// Test server managing lifecycle of HTTP server and database
pub struct TestServer {
    port: u16,
    temp_dir: TempDir,
    database: DatabaseRef,
    _shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    /// Start a new test server with a temporary database
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        // Create temporary directory for database
        let temp_dir = TempDir::new()?;
        
        // Create config with test database path
        let config = Config::with_db_path(temp_dir.path().to_path_buf());
        
        // Create operation registry and get database reference
        let (registry, database) = create_registry_with_config(config)?;
        let registry = Arc::new(registry);
        
        // Find an available port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        
        // Create router
        let router = create_http_router(registry);
        
        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        
        // Spawn server in background
        tokio::spawn(async move {
            let server = axum::serve(listener, router.into_make_service());
            
            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        eprintln!("Server error: {}", e);
                    }
                }
                _ = &mut shutdown_rx => {
                    // Shutdown requested
                }
            }
        });
        
        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(Self {
            port,
            temp_dir,
            database,
            _shutdown_tx: shutdown_tx,
        })
    }
    
    /// Get the base URL for API requests
    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
    
    /// Get the database path
    pub fn db_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().to_path_buf()
    }
    
    /// Get the database reference for direct state inspection
    pub fn database(&self) -> &DatabaseRef {
        &self.database
    }
    
    /// Get the Wizard user (system admin with all permissions)
    #[allow(dead_code)]
    pub fn get_wizard_user(&self) -> Result<User, Box<dyn std::error::Error>> {
        Ok(self.database.users().get_wizard_user()?)
    }
    
    /// Get the configured wizard API key for authentication tests
    #[allow(dead_code)]
    pub fn get_wizard_api_key(&self) -> String {
        // This matches the default key from config.rs
        "wizard-default-key-change-in-production".to_string()
    }
    
    /// Calculate SHA256 hash for object content (matches what the system does)
    pub fn calculate_sha256(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Load a .moo file from the test resources directory
pub fn load_moo_file(filename: &str) -> String {
    let resources_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resources");
    let file_path = resources_dir.join(filename);
    
    std::fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", file_path.display(), e))
}

/// Convert .moo file content to a vector of lines (for API compatibility)
pub fn moo_to_lines(content: &str) -> Vec<String> {
    content.lines().map(|s| s.to_string()).collect()
}

/// Helper function to make HTTP requests
pub async fn make_request(
    method: &str,
    url: &str,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let request = match method {
        "GET" => client.get(url),
        "POST" => {
            let mut req = client.post(url);
            if let Some(json_body) = body {
                req = req.json(&json_body);
            }
            req
        }
        _ => panic!("Unsupported HTTP method: {}", method),
    };
    
    let response = request.send().await?;
    let status = response.status();
    let text = response.text().await?;
    
    if !status.is_success() {
        return Err(format!("HTTP error {}: {}", status, text).into());
    }
    
    let json: serde_json::Value = serde_json::from_str(&text)?;
    Ok(json)
}

// Re-export commonly used types for tests
pub use serde_json::json;

