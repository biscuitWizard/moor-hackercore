use std::env;
use std::path::PathBuf;

/// Configuration for the VCS worker
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the fjall database directory
    pub db_path: PathBuf,
    /// API key for the default Wizard user with all permissions
    pub wizard_api_key: String,
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        let db_path = Self::get_db_path();
        let wizard_api_key = Self::get_wizard_api_key();
        tracing::info!("VCS database path: {:?}", db_path);
        tracing::info!("Wizard API key configured: {}", if wizard_api_key.is_empty() { "default" } else { "from env" });
        Self { db_path, wizard_api_key }
    }
    
    /// Create a new config with an explicit database path (useful for testing)
    #[allow(dead_code)]
    pub fn with_db_path(db_path: PathBuf) -> Self {
        let wizard_api_key = Self::get_wizard_api_key();
        tracing::info!("VCS database path (explicit): {:?}", db_path);
        tracing::info!("Wizard API key configured: {}", if wizard_api_key.is_empty() { "default" } else { "from env" });
        Self { db_path, wizard_api_key }
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

    /// Get the wizard API key from environment or use default
    fn get_wizard_api_key() -> String {
        env::var("VCS_WIZARD_API_KEY")
            .unwrap_or_else(|_| "wizard-default-key-change-in-production".to_string())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
