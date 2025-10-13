//! Integration tests for user management operations
//!
//! This module is organized by operation type:
//! - create_tests: Tests for creating users
//! - delete_tests: Tests for deleting users
//! - enable_disable_tests: Tests for enabling/disabling users
//! - permissions_tests: Tests for managing user permissions
//! - api_keys_tests: Tests for generating and deleting API keys
//! - list_tests: Tests for listing users
//! - external_user_tests: Tests for external user configuration (clone operations)

mod api_keys_tests;
mod create_tests;
mod delete_tests;
mod enable_disable_tests;
mod external_user_tests;
mod list_tests;
mod permissions_tests;

