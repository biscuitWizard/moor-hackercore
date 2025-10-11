//! Integration tests for object operations
//!
//! This module is organized by operation type and scenarios:
//! - crud: Tests for basic object CRUD operations (create, update, get)
//! - list: Tests for object/list operation
//! - lifecycle: Tests for object lifecycle and SHA256 management
//! - rename_update_integration: Tests for complex rename and update interactions
//! - delete_tests: Edge case tests for object/delete operation
//! - get_tests: Edge case tests for object/get operation
//! - rename_edge_cases_tests: Edge case tests for object/rename operation

mod crud;
mod list;
mod lifecycle;
mod rename_update_integration;
mod delete_tests;
mod get_tests;
mod rename_edge_cases_tests;

