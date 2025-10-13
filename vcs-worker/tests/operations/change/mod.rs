//! Integration tests for change operations
//!
//! This module is organized by operation type:
//! - create_tests: Tests for creating changes
//! - abandon_tests: Tests for abandoning changes and cleanup
//! - approve_tests: Tests for approving changes (merge to main history)
//! - stash_tests: Tests for stashing changes to workspace
//! - submit_tests: Tests for submitting changes (remote vs local behavior)

mod abandon_tests;
mod approve_tests;
mod create_tests;
mod stash_tests;
mod submit_tests;

