use std::path::Path;
use std::fs;
use tracing::{info, warn};
use git2::{Repository, Signature, CertificateCheckStatus, cert::Cert};

/// Utility functions for git operations
pub struct GitUtils;

impl GitUtils {
    /// Configure git user name and email in the repository
    pub fn configure_git_user(repo: &Repository, user_name: &str, user_email: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = repo.config()?;
        
        config.set_str("user.name", user_name)?;
        config.set_str("user.email", user_email)?;
        
        info!("Configured git user: {} <{}>", user_name, user_email);
        Ok(())
    }
    
    /// Ensure the keys directory is in .gitignore for security
    pub fn ensure_keys_gitignore(work_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let gitignore_path = work_dir.join(".gitignore");
        let keys_entry = "keys/\n";
        
        // Read existing .gitignore content
        let existing_content = if gitignore_path.exists() {
            fs::read_to_string(&gitignore_path)?
        } else {
            String::new()
        };
        
        // Check if keys/ is already ignored
        if existing_content.lines().any(|line| line.trim() == "keys/") {
            info!("Keys directory already in .gitignore");
            return Ok(());
        }
        
        // Add keys/ to .gitignore
        let new_content = if existing_content.is_empty() {
            keys_entry.to_string()
        } else if existing_content.ends_with('\n') {
            format!("{}{}", existing_content, keys_entry)
        } else {
            format!("{}\n{}", existing_content, keys_entry)
        };
        
        fs::write(&gitignore_path, new_content)?;
        info!("Added keys/ to .gitignore for security");
        
        Ok(())
    }
    
    /// Get SSH credentials for authentication
    pub fn get_ssh_credentials(
        repo: &Repository,
        url: Option<&str>,
        username_from_url: Option<&str>,
        _allowed_types: git2::CredentialType,
        ssh_key_path: Option<&str>,
        keys_dir: &Path,
    ) -> Result<git2::Cred, git2::Error> {
        info!("Attempting SSH authentication for URL: {:?}", url);
        
        // Try configured SSH key first
        if let Some(ssh_key_path) = ssh_key_path {
            info!("Trying configured SSH key: {}", ssh_key_path);
            if let Ok(cred) = git2::Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                std::path::Path::new(ssh_key_path),
                None,
            ) {
                info!("Successfully authenticated with configured SSH key: {}", ssh_key_path);
                return Ok(cred);
            } else {
                warn!("Failed to authenticate with configured SSH key: {}", ssh_key_path);
            }
        }
        
        // Try keys directory
        info!("Checking keys directory: {:?}", keys_dir);
        let default_keys = [
            keys_dir.join("id_rsa"),
            keys_dir.join("id_ed25519"),
            keys_dir.join("id_ecdsa"),
        ];
        
        for key_path in &default_keys {
            if key_path.exists() {
                info!("Trying SSH key from keys directory: {:?}", key_path);
                if let Ok(cred) = git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    key_path,
                    None,
                ) {
                    info!("Successfully authenticated with key from keys directory: {:?}", key_path);
                    return Ok(cred);
                } else {
                    warn!("Failed to authenticate with key from keys directory: {:?}", key_path);
                }
            }
        }
        
        // Try default SSH key locations in container
        let home_keys = [
            "/root/.ssh/id_rsa",
            "/root/.ssh/id_ed25519",
            "/root/.ssh/id_ecdsa",
        ];
        
        for key_path in &home_keys {
            if std::path::Path::new(key_path).exists() {
                info!("Trying default SSH key: {}", key_path);
                if let Ok(cred) = git2::Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    std::path::Path::new(key_path),
                    None,
                ) {
                    info!("Successfully authenticated with default SSH key: {}", key_path);
                    return Ok(cred);
                } else {
                    warn!("Failed to authenticate with default SSH key: {}", key_path);
                }
            }
        }
        
        // Try git credential helper
        if let Ok(cred) = git2::Cred::credential_helper(&repo.config()?, url.unwrap_or(""), url) {
            info!("Successfully authenticated with git credential helper");
            return Ok(cred);
        } else {
            warn!("Git credential helper authentication failed");
        }
        
        // Fall back to default credential helper
        warn!("No SSH authentication available, trying default credentials");
        git2::Cred::default()
    }
    
    /// Create a signature for commits
    pub fn create_signature(user_name: &str, user_email: &str) -> Result<Signature<'static>, Box<dyn std::error::Error>> {
        Ok(Signature::now(user_name, user_email)?)
    }
    
    /// Create certificate check callback that always accepts
    pub fn create_certificate_check_callback() -> impl Fn(&Cert<'_>, &str) -> Result<CertificateCheckStatus, git2::Error> {
        |_cert, _host| Ok(CertificateCheckStatus::CertificateOk)
    }
}
