use std::env;
use std::path::PathBuf;

/// Configuration for the VCS worker
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the fjall database directory
    pub db_path: PathBuf,
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        let db_path = Self::get_db_path();
        tracing::info!("VCS database path: {:?}", db_path);
        Self { db_path }
    }
    
    /// Create a new config with an explicit database path (useful for testing)
    #[allow(dead_code)]
    pub fn with_db_path(db_path: PathBuf) -> Self {
        tracing::info!("VCS database path (explicit): {:?}", db_path);
        Self { db_path }
    }

    /// Get the database path from environment or use default
    fn get_db_path() -> PathBuf {
        env::var("VCS_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Default to /game directory relative to current working directory
                let mut path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                path.push("game");
                path
            })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
