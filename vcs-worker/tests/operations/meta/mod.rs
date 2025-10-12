//! Integration tests for meta operations (add/remove ignored properties and verbs)
//!
//! This module is organized by operation type:
//! - add: Tests for adding ignored properties and verbs
//! - remove: Tests for removing ignored properties and verbs
//! - clear: Tests for clearing ignored properties and verbs
//! - lifecycle: Tests for meta behavior during object rename/delete
//! - filtering_get: Tests for filtering during object get operations
//! - filtering_update: Tests for filtering during object update operations
//! - diff: Tests for diff behavior with ignored properties/verbs

mod add;
mod clear;
mod diff;
mod filtering_get;
mod filtering_update;
mod lifecycle;
mod remove;
