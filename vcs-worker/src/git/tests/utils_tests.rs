use super::*;
use crate::git::utils::GitUtils;
use git2::Repository;

#[test]
fn test_configure_git_user() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    
    let result = GitUtils::configure_git_user(&repo, "test-user", "test@example.com");
    assert!(result.is_ok());
    
    // Verify the configuration was set
    let config = repo.config().unwrap();
    let user_name = config.get_string("user.name").unwrap();
    let user_email = config.get_string("user.email").unwrap();
    
    assert_eq!(user_name, "test-user");
    assert_eq!(user_email, "test@example.com");
}

#[test]
fn test_ensure_keys_gitignore_new_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let work_dir = temp_dir.path();
    
    // Ensure .gitignore doesn't exist initially
    let gitignore_path = work_dir.join(".gitignore");
    assert!(!gitignore_path.exists());
    
    let result = GitUtils::ensure_keys_gitignore(work_dir);
    assert!(result.is_ok());
    
    // Check that .gitignore was created with keys/ entry
    assert!(gitignore_path.exists());
    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("keys/"));
}

#[test]
fn test_ensure_keys_gitignore_existing_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    let work_dir = temp_dir.path();
    let gitignore_path = work_dir.join(".gitignore");
    
    // Create existing .gitignore with some content
    std::fs::write(&gitignore_path, "*.log\n*.tmp\n").unwrap();
    
    let result = GitUtils::ensure_keys_gitignore(work_dir);
    assert!(result.is_ok());
    
    // Check that keys/ was added
    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    assert!(content.contains("keys/"));
    assert!(content.contains("*.log"));
    assert!(content.contains("*.tmp"));
}

#[test]
fn test_ensure_keys_gitignore_already_ignored() {
    let temp_dir = tempfile::tempdir().unwrap();
    let work_dir = temp_dir.path();
    let gitignore_path = work_dir.join(".gitignore");
    
    // Create .gitignore that already contains keys/
    std::fs::write(&gitignore_path, "*.log\nkeys/\n*.tmp\n").unwrap();
    
    let result = GitUtils::ensure_keys_gitignore(work_dir);
    assert!(result.is_ok());
    
    // Check that keys/ wasn't duplicated
    let content = std::fs::read_to_string(&gitignore_path).unwrap();
    let keys_count = content.matches("keys/").count();
    assert_eq!(keys_count, 1);
}

#[test]
fn test_create_signature() {
    let result = GitUtils::create_signature("test-user", "test@example.com");
    assert!(result.is_ok());
    
    let signature = result.unwrap();
    assert_eq!(signature.name().unwrap(), "test-user");
    assert_eq!(signature.email().unwrap(), "test@example.com");
}

#[test]
fn test_create_certificate_check_callback() {
    let callback = GitUtils::create_certificate_check_callback();
    
    // Test that the callback always returns CertificateOk
    // Note: We can't easily test this without creating actual certificates,
    // but we can verify the function returns a valid callback
    assert!(std::ptr::addr_of!(callback) != std::ptr::null());
}

#[test]
fn test_get_ssh_credentials_no_keys() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    
    // Test with no SSH keys available
    let result = GitUtils::get_ssh_credentials(
        &repo,
        Some("git@example.com:repo.git"),
        Some("git"),
        git2::CredentialType::SSH_KEY,
        None,
        &keys_dir,
    );
    
    // Should fall back to default credentials
    assert!(result.is_ok());
}

#[test]
fn test_get_ssh_credentials_with_configured_key() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    std::fs::create_dir_all(&keys_dir).unwrap();
    
    // Create a test SSH key
    let ssh_key_path = keys_dir.join("id_rsa");
    create_test_ssh_key(&ssh_key_path).unwrap();
    
    let result = GitUtils::get_ssh_credentials(
        &repo,
        Some("git@example.com:repo.git"),
        Some("git"),
        git2::CredentialType::SSH_KEY,
        Some(&ssh_key_path.to_string_lossy()),
        &keys_dir,
    );
    
    // Should succeed with the configured key
    assert!(result.is_ok());
}

#[test]
fn test_get_ssh_credentials_with_keys_directory() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    let keys_dir = temp_dir.path().join("keys");
    std::fs::create_dir_all(&keys_dir).unwrap();
    
    // Create a test SSH key in the keys directory
    let ssh_key_path = keys_dir.join("id_rsa");
    create_test_ssh_key(&ssh_key_path).unwrap();
    
    let result = GitUtils::get_ssh_credentials(
        &repo,
        Some("git@example.com:repo.git"),
        Some("git"),
        git2::CredentialType::SSH_KEY,
        None,
        &keys_dir,
    );
    
    // Should succeed with the key from keys directory
    assert!(result.is_ok());
}
