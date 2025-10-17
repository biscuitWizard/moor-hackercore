//! Integration tests for vcs-worker operations
//!
//! This module organizes tests by operation type, mirroring the src/operations structure:
//! - object: Object CRUD operations (create, update, get, delete, list, rename, hints)
//! - change: Change management operations (create, abandon, approve, stash, submit)
//! - user: User management operations (create, delete, permissions, API keys, etc.)
//! - clone: Clone operations (export, import, authentication, error handling)
//! - index: Index operations (list, calc delta, update)
//! - workspace: Workspace operations
//! - meta: Meta operations (add/remove/clear ignored properties and verbs)
//! - change_switch_tests: Tests for change/switch operation

mod blake3_hash_tests;
mod change;
mod change_status_tests;
mod change_switch_tests;
mod clone;
mod index_operations;
mod index_update_tests;
mod meta;
mod object;
mod object_diff_operation_tests;
mod system_status_tests;
mod test_wizard_user;
mod user;
mod workspace_approve_tests;
mod workspace_operations;

// Future test modules:
