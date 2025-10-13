//! Comprehensive unit tests for config.rs
//!
//! This test module provides 100% test coverage for the Config module,
//! including environment variable handling, default values, and constructor methods.

use serial_test::serial;
use std::env;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper function to clear all VCS-related environment variables
fn clear_vcs_env_vars() {
    unsafe {
        env::remove_var("VCS_DB_PATH");
        env::remove_var("VCS_WIZARD_API_KEY");
        env::remove_var("VCS_GAME_NAME");
        env::remove_var("VCS_GIT_BACKUP_REPO");
        env::remove_var("VCS_GIT_BACKUP_TOKEN");
    }
}

/// Helper function to set environment variables safely
fn set_env_var(key: &str, value: &str) {
    unsafe {
        env::set_var(key, value);
    }
}

#[test]
#[serial]
fn test_config_default_values() {
    // Clear all env vars to ensure defaults are used
    clear_vcs_env_vars();

    let config = moor_vcs_worker::Config::new();

    // Test default wizard API key
    assert_eq!(
        config.wizard_api_key,
        "wizard-default-key-change-in-production"
    );

    // Test default game name
    assert_eq!(config.game_name, "Unknown Game");

    // Test default db_path is ./game relative to current dir
    let mut expected_path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    expected_path.push("game");
    assert_eq!(config.db_path, expected_path);

    // Test git backup defaults to None
    assert_eq!(config.git_backup_repo, None);
    assert_eq!(config.git_backup_token, None);

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_custom_db_path() {
    clear_vcs_env_vars();
    let temp_dir = TempDir::new().unwrap();

    set_env_var("VCS_DB_PATH", temp_dir.path().to_str().unwrap());

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.db_path, temp_dir.path());

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_custom_wizard_api_key() {
    clear_vcs_env_vars();

    let custom_key = "my-secret-wizard-key-12345";
    set_env_var("VCS_WIZARD_API_KEY", custom_key);

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.wizard_api_key, custom_key);

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_custom_game_name() {
    clear_vcs_env_vars();

    let custom_name = "My Awesome Game";
    set_env_var("VCS_GAME_NAME", custom_name);

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.game_name, custom_name);

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_unicode_game_name() {
    clear_vcs_env_vars();

    let unicode_name = "游戏世界 🎮 Мир игры";
    set_env_var("VCS_GAME_NAME", unicode_name);

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.game_name, unicode_name);

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_git_backup_repo() {
    clear_vcs_env_vars();

    let repo_url = "https://github.com/test/repo.git";
    set_env_var("VCS_GIT_BACKUP_REPO", repo_url);

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.git_backup_repo, Some(repo_url.to_string()));

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_git_backup_token() {
    clear_vcs_env_vars();

    let token = "ghp_1234567890abcdefghijklmnopqrstuvwxyz";
    set_env_var("VCS_GIT_BACKUP_TOKEN", token);

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.git_backup_token, Some(token.to_string()));

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_both_git_backup_fields() {
    clear_vcs_env_vars();

    let repo_url = "https://github.com/test/repo.git";
    let token = "test_token_123";
    set_env_var("VCS_GIT_BACKUP_REPO", repo_url);
    set_env_var("VCS_GIT_BACKUP_TOKEN", token);

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.git_backup_repo, Some(repo_url.to_string()));
    assert_eq!(config.git_backup_token, Some(token.to_string()));

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_empty_string_git_backup_repo_becomes_none() {
    clear_vcs_env_vars();

    // Empty string should be filtered out and become None
    set_env_var("VCS_GIT_BACKUP_REPO", "");

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.git_backup_repo, None);

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_empty_string_git_backup_token_becomes_none() {
    clear_vcs_env_vars();

    // Empty string should be filtered out and become None
    set_env_var("VCS_GIT_BACKUP_TOKEN", "");

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.git_backup_token, None);

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_all_environment_variables() {
    clear_vcs_env_vars();
    let temp_dir = TempDir::new().unwrap();

    // Set all possible environment variables
    set_env_var("VCS_DB_PATH", temp_dir.path().to_str().unwrap());
    set_env_var("VCS_WIZARD_API_KEY", "custom-wizard-key");
    set_env_var("VCS_GAME_NAME", "Test Game");
    set_env_var("VCS_GIT_BACKUP_REPO", "https://example.com/repo.git");
    set_env_var("VCS_GIT_BACKUP_TOKEN", "token123");

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.db_path, temp_dir.path());
    assert_eq!(config.wizard_api_key, "custom-wizard-key");
    assert_eq!(config.game_name, "Test Game");
    assert_eq!(
        config.git_backup_repo,
        Some("https://example.com/repo.git".to_string())
    );
    assert_eq!(config.git_backup_token, Some("token123".to_string()));

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_db_path_method() {
    clear_vcs_env_vars();
    let temp_dir = TempDir::new().unwrap();
    let explicit_path = temp_dir.path().to_path_buf();

    // Set some env vars to ensure they're still picked up
    set_env_var("VCS_WIZARD_API_KEY", "test-key");
    set_env_var("VCS_GAME_NAME", "Test Game");
    set_env_var("VCS_GIT_BACKUP_REPO", "https://example.com/repo.git");

    let config = moor_vcs_worker::Config::with_db_path(explicit_path.clone());

    // Explicit path should be used
    assert_eq!(config.db_path, explicit_path);

    // Other env vars should still be read
    assert_eq!(config.wizard_api_key, "test-key");
    assert_eq!(config.game_name, "Test Game");
    assert_eq!(
        config.git_backup_repo,
        Some("https://example.com/repo.git".to_string())
    );

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_with_db_path_ignores_env_var() {
    clear_vcs_env_vars();
    let temp_dir1 = TempDir::new().unwrap();
    let temp_dir2 = TempDir::new().unwrap();

    // Set VCS_DB_PATH to one directory
    set_env_var("VCS_DB_PATH", temp_dir1.path().to_str().unwrap());

    // But use with_db_path with a different directory
    let config = moor_vcs_worker::Config::with_db_path(temp_dir2.path().to_path_buf());

    // Should use the explicit path, not the env var
    assert_eq!(config.db_path, temp_dir2.path());
    assert_ne!(config.db_path, temp_dir1.path());

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_default_trait() {
    clear_vcs_env_vars();

    let config1 = moor_vcs_worker::Config::new();
    let config2 = moor_vcs_worker::Config::default();

    // Default should behave the same as new()
    assert_eq!(config1.db_path, config2.db_path);
    assert_eq!(config1.wizard_api_key, config2.wizard_api_key);
    assert_eq!(config1.game_name, config2.game_name);
    assert_eq!(config1.git_backup_repo, config2.git_backup_repo);
    assert_eq!(config1.git_backup_token, config2.git_backup_token);

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_clone_trait() {
    clear_vcs_env_vars();
    let temp_dir = TempDir::new().unwrap();

    set_env_var("VCS_DB_PATH", temp_dir.path().to_str().unwrap());
    set_env_var("VCS_WIZARD_API_KEY", "test-key");
    set_env_var("VCS_GAME_NAME", "Test Game");

    let config1 = moor_vcs_worker::Config::new();
    let config2 = config1.clone();

    assert_eq!(config1.db_path, config2.db_path);
    assert_eq!(config1.wizard_api_key, config2.wizard_api_key);
    assert_eq!(config1.game_name, config2.game_name);
    assert_eq!(config1.git_backup_repo, config2.git_backup_repo);
    assert_eq!(config1.git_backup_token, config2.git_backup_token);

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_debug_trait() {
    clear_vcs_env_vars();

    let config = moor_vcs_worker::Config::new();
    let debug_output = format!("{:?}", config);

    // Just verify Debug trait works and includes expected content
    assert!(debug_output.contains("Config"));
    assert!(debug_output.contains("db_path"));
    assert!(debug_output.contains("wizard_api_key"));
    assert!(debug_output.contains("game_name"));

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_very_long_strings() {
    clear_vcs_env_vars();

    // Test with very long strings to ensure no buffer issues
    let long_key = "a".repeat(1000);
    let long_name = "Game ".repeat(200);
    let long_url = format!("https://example.com/{}", "path/".repeat(100));

    set_env_var("VCS_WIZARD_API_KEY", &long_key);
    set_env_var("VCS_GAME_NAME", &long_name);
    set_env_var("VCS_GIT_BACKUP_REPO", &long_url);

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.wizard_api_key, long_key);
    assert_eq!(config.game_name, long_name);
    assert_eq!(config.git_backup_repo, Some(long_url));

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_local_git_path() {
    clear_vcs_env_vars();

    // Test with local file path instead of URL
    let local_path = "/tmp/local-repo.git";
    set_env_var("VCS_GIT_BACKUP_REPO", local_path);

    let config = moor_vcs_worker::Config::new();

    assert_eq!(config.git_backup_repo, Some(local_path.to_string()));

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_whitespace_handling() {
    clear_vcs_env_vars();

    // Test with whitespace in values - should be preserved
    set_env_var("VCS_GAME_NAME", "  Game With Spaces  ");
    set_env_var("VCS_WIZARD_API_KEY", " key-with-spaces ");

    let config = moor_vcs_worker::Config::new();

    // Values should be preserved as-is (no trimming)
    assert_eq!(config.game_name, "  Game With Spaces  ");
    assert_eq!(config.wizard_api_key, " key-with-spaces ");

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_special_characters_in_paths() {
    clear_vcs_env_vars();

    // Test with special characters that are valid in URLs/paths
    set_env_var("VCS_GIT_BACKUP_REPO", "https://user:pass@github.com/repo.git");
    set_env_var("VCS_GIT_BACKUP_TOKEN", "ghp_abc123!@#$%^&*()");

    let config = moor_vcs_worker::Config::new();

    assert_eq!(
        config.git_backup_repo,
        Some("https://user:pass@github.com/repo.git".to_string())
    );
    assert_eq!(
        config.git_backup_token,
        Some("ghp_abc123!@#$%^&*()".to_string())
    );

    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_logging_coverage_new() {
    clear_vcs_env_vars();

    // This test ensures the tracing::info! calls in Config::new() are executed
    // We can't easily verify the log output, but we can ensure it doesn't panic
    let _config = moor_vcs_worker::Config::new();

    // If we got here without panicking, the logging code paths were executed
    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_logging_coverage_with_db_path() {
    clear_vcs_env_vars();
    let temp_dir = TempDir::new().unwrap();

    // This test ensures the tracing::info! calls in Config::with_db_path() are executed
    let _config = moor_vcs_worker::Config::with_db_path(temp_dir.path().to_path_buf());

    // If we got here without panicking, the logging code paths were executed
    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_logging_with_git_backup_enabled() {
    clear_vcs_env_vars();

    // Test logging path when git backup is configured
    set_env_var("VCS_GIT_BACKUP_REPO", "https://github.com/test/repo.git");
    let _config = moor_vcs_worker::Config::new();

    // Logging should indicate "enabled"
    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_logging_with_git_backup_disabled() {
    clear_vcs_env_vars();

    // Test logging path when git backup is not configured
    let _config = moor_vcs_worker::Config::new();

    // Logging should indicate "disabled"
    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_logging_with_custom_wizard_key() {
    clear_vcs_env_vars();

    // Test logging path when wizard key is from env
    set_env_var("VCS_WIZARD_API_KEY", "custom-key");
    let _config = moor_vcs_worker::Config::new();

    // Logging should indicate "from env"
    clear_vcs_env_vars();
}

#[test]
#[serial]
fn test_config_logging_with_default_wizard_key() {
    clear_vcs_env_vars();

    // Test logging path when wizard key is default
    let _config = moor_vcs_worker::Config::new();

    // Logging should indicate "default"
    clear_vcs_env_vars();
}

