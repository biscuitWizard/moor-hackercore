use std::env;
use std::path::PathBuf;

/// Configuration for the VCS worker
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the fjall database directory
    pub db_path: PathBuf,
    /// API key for the default Wizard user with all permissions
    pub wizard_api_key: String,
    /// Name of the game/world
    pub game_name: String,
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        let db_path = Self::get_db_path();
        let wizard_api_key = Self::get_wizard_api_key();
        let game_name = Self::get_game_name();
        tracing::info!("VCS database path: {:?}", db_path);
        tracing::info!("Wizard API key configured: {}", if wizard_api_key.is_empty() { "default" } else { "from env" });
        tracing::info!("Game name: {}", game_name);
        Self { db_path, wizard_api_key, game_name }
    }
    
    /// Create a new config with an explicit database path (useful for testing)
    #[allow(dead_code)]
    pub fn with_db_path(db_path: PathBuf) -> Self {
        let wizard_api_key = Self::get_wizard_api_key();
        let game_name = Self::get_game_name();
        tracing::info!("VCS database path (explicit): {:?}", db_path);
        tracing::info!("Wizard API key configured: {}", if wizard_api_key.is_empty() { "default" } else { "from env" });
        tracing::info!("Game name: {}", game_name);
        Self { db_path, wizard_api_key, game_name }
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

    /// Get the game name from environment or use default
    fn get_game_name() -> String {
        env::var("VCS_GAME_NAME")
            .unwrap_or_else(|_| "Unknown Game".to_string())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
