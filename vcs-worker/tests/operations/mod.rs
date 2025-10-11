//! Integration tests for vcs-worker operations
//!
//! This module organizes tests by operation type, mirroring the src/operations structure:
//! - object: Object CRUD operations (create, update, get, delete, list, rename)
//! - change: Change management operations (create, status, submit, etc.)
//! - index: Index operations (list, calc delta, update)
//! - workspace: Workspace operations
//! - meta: Meta operations (add/remove/clear ignored properties and verbs)

mod object;
mod change_operations;
mod test_wizard_user;
mod workspace_operations;
mod meta;
mod index_operations;
mod workspace_approve_tests;
mod clone_tests;
mod index_update_tests;

// Future test modules:

