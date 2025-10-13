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
    /// Git backup repository URL or local path (optional)
    pub git_backup_repo: Option<String>,
    /// Git backup authentication token (optional)
    pub git_backup_token: Option<String>,
    /// Git backup working directory (for temp clones)
    pub git_backup_work_dir: Option<PathBuf>,
}

impl Config {
    /// Create a new config with default values
    pub fn new() -> Self {
        let db_path = Self::get_db_path();
        let wizard_api_key = Self::get_wizard_api_key();
        let game_name = Self::get_game_name();
        let git_backup_repo = Self::get_git_backup_repo();
        let git_backup_token = Self::get_git_backup_token();
        let git_backup_work_dir = Self::get_git_backup_work_dir();
        tracing::info!("VCS database path: {:?}", db_path);
        tracing::info!(
            "Wizard API key configured: {}",
            if wizard_api_key.is_empty() {
                "default"
            } else {
                "from env"
            }
        );
        tracing::info!("Game name: {}", game_name);
        tracing::info!(
            "Git backup configured: {}",
            if git_backup_repo.is_some() {
                "enabled"
            } else {
                "disabled"
            }
        );
        Self {
            db_path,
            wizard_api_key,
            game_name,
            git_backup_repo,
            git_backup_token,
            git_backup_work_dir,
        }
    }

    /// Create a new config with an explicit database path (useful for testing)
    #[allow(dead_code)]
    pub fn with_db_path(db_path: PathBuf) -> Self {
        let wizard_api_key = Self::get_wizard_api_key();
        let game_name = Self::get_game_name();
        let git_backup_repo = Self::get_git_backup_repo();
        let git_backup_token = Self::get_git_backup_token();
        // For testing, don't set a default work dir - let it be auto-generated per-instance
        let git_backup_work_dir = None;
        tracing::info!("VCS database path (explicit): {:?}", db_path);
        tracing::info!(
            "Wizard API key configured: {}",
            if wizard_api_key.is_empty() {
                "default"
            } else {
                "from env"
            }
        );
        tracing::info!("Game name: {}", game_name);
        tracing::info!(
            "Git backup configured: {}",
            if git_backup_repo.is_some() {
                "enabled"
            } else {
                "disabled"
            }
        );
        Self {
            db_path,
            wizard_api_key,
            game_name,
            git_backup_repo,
            git_backup_token,
            git_backup_work_dir,
        }
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
        env::var("VCS_GAME_NAME").unwrap_or_else(|_| "Unknown Game".to_string())
    }

    /// Get the git backup repository from environment (optional)
    fn get_git_backup_repo() -> Option<String> {
        env::var("VCS_GIT_BACKUP_REPO").ok().filter(|s| !s.is_empty())
    }

    /// Get the git backup token from environment (optional)
    fn get_git_backup_token() -> Option<String> {
        env::var("VCS_GIT_BACKUP_TOKEN").ok().filter(|s| !s.is_empty())
    }

    /// Get the git backup working directory from environment (optional)
    fn get_git_backup_work_dir() -> Option<PathBuf> {
        env::var("VCS_GIT_BACKUP_WORK_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                // Default to /tmp/vcs-git-backup for production (backward compatible)
                Some(PathBuf::from("/tmp/vcs-git-backup"))
            })
    }

    /// Builder method to set git backup configuration
    #[allow(dead_code)]
    pub fn with_git_backup(mut self, repo: String, token: Option<String>) -> Self {
        self.git_backup_repo = Some(repo);
        self.git_backup_token = token;
        self
    }

    /// Builder method to set git backup working directory
    #[allow(dead_code)]
    pub fn with_git_work_dir(mut self, path: PathBuf) -> Self {
        self.git_backup_work_dir = Some(path);
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
