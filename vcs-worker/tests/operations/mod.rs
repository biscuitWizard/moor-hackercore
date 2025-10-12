//! Integration tests for vcs-worker operations
//!
//! This module organizes tests by operation type, mirroring the src/operations structure:
//! - object: Object CRUD operations (create, update, get, delete, list, rename)
//! - change: Change management operations (create, status, submit, etc.)
//! - index: Index operations (list, calc delta, update)
//! - workspace: Workspace operations
//! - meta: Meta operations (add/remove/clear ignored properties and verbs)
//! - change_switch_tests: Tests for change/switch operation

mod blake3_hash_tests;
mod change_operations;
mod change_switch_tests;
mod clone_tests;
mod index_operations;
mod index_update_tests;
mod meta;
mod object;
mod system_status_tests;
mod test_wizard_user;
mod user_management_tests;
mod workspace_approve_tests;
mod workspace_operations;

// Future test modules:
