//! Integration tests for vcs-worker operations
//!
//! This module organizes tests by operation type, mirroring the src/operations structure:
//! - object: Object CRUD operations (create, update, get, delete)
//! - change: Change management operations (create, status, submit, etc.)
//! - index: Index operations (list, calc delta, update)
//! - workspace: Workspace operations

mod object;
mod object_lifecycle;
mod object_rename_update_integration;
mod object_list;
mod change_operations;
mod test_wizard_user;

// Future test modules:
// mod index;
// mod workspace;

