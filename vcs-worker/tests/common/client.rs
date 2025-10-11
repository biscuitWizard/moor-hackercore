//! High-level test client that abstracts common API operations
//!
//! This module provides a fluent API for test operations, reducing boilerplate
//! and making tests more readable.

#![allow(dead_code)]

use serde_json::Value;
use crate::common::*;

/// High-level test client for VCS worker operations
pub struct VcsTestClient {
    base_url: String,
    database: Option<DatabaseRef>,
}

impl VcsTestClient {
    /// Create a client with database reference for direct operations
    pub fn with_database(server: &TestServer) -> Self {
        Self {
            base_url: server.base_url(),
            database: Some(server.database().clone()),
        }
    }
}

impl VcsTestClient {
    /// Create a new test client for the given server
    pub fn new(server: &TestServer) -> Self {
        Self {
            base_url: server.base_url(),
            database: Some(server.database().clone()),
        }
    }
    
    // ==================== Object Operations ====================
    
    /// Update an object with the given name and content
    pub async fn object_update(
        &self,
        name: &str,
        content: Vec<String>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("object/update", vec![
            Value::String(name.to_string()),
            Value::String(serde_json::to_string(&content)?),
        ]).await
    }
    
    /// Update an object from a .moo file
    pub async fn object_update_from_file(
        &self,
        name: &str,
        filename: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let content = load_moo_file(filename);
        let lines = moo_to_lines(&content);
        self.object_update(name, lines).await
    }
    
    /// Get an object by name
    pub async fn object_get(&self, name: &str) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("object/get", vec![Value::String(name.to_string())]).await
    }
    
    /// Delete an object by name
    pub async fn object_delete(&self, name: &str) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("object/delete", vec![Value::String(name.to_string())]).await
    }
    
    /// Rename an object
    pub async fn object_rename(
        &self,
        old_name: &str,
        new_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("object/rename", vec![
            Value::String(old_name.to_string()),
            Value::String(new_name.to_string()),
        ]).await
    }
    
    /// List all objects with optional filter
    pub async fn object_list(&self, filter: Option<&str>) -> Result<Value, Box<dyn std::error::Error>> {
        let args = match filter {
            Some(f) => vec![Value::String(f.to_string())],
            None => vec![],
        };
        self.rpc_call("object/list", args).await
    }
    
    // ==================== Change Operations ====================
    
    /// Create a new change
    pub async fn change_create(
        &self,
        name: &str,
        author: &str,
        description: Option<&str>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut args = vec![
            Value::String(name.to_string()),
            Value::String(author.to_string()),
        ];
        
        if let Some(desc) = description {
            args.push(Value::String(desc.to_string()));
        }
        
        self.rpc_call("change/create", args).await
    }
    
    /// Get change status
    pub async fn change_status(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("change/status", vec![]).await
    }
    
    /// Submit/commit a change
    pub async fn change_submit(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("change/submit", vec![]).await
    }
    
    /// Abandon the current change
    pub async fn change_abandon(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("change/abandon", vec![]).await
    }
    
    /// Stash the current change to workspace
    pub async fn change_stash(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("change/stash", vec![]).await
    }
    
    /// Switch to a different change by ID
    pub async fn change_switch(&self, change_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("change/switch", vec![Value::String(change_id.to_string())]).await
    }
    
    /// Approve a change by ID
    pub async fn change_approve(&self, change_id: &str) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("change/approve", vec![Value::String(change_id.to_string())]).await
    }
    
    // ==================== Meta Operations ====================
    
    /// Add an ignored property to an object's meta
    pub async fn meta_add_ignored_property(
        &self,
        object_name: &str,
        property_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("meta/add_ignored_property", vec![
            Value::String(object_name.to_string()),
            Value::String(property_name.to_string()),
        ]).await
    }
    
    /// Add an ignored verb to an object's meta
    pub async fn meta_add_ignored_verb(
        &self,
        object_name: &str,
        verb_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("meta/add_ignored_verb", vec![
            Value::String(object_name.to_string()),
            Value::String(verb_name.to_string()),
        ]).await
    }
    
    /// Remove an ignored property from an object's meta
    pub async fn meta_remove_ignored_property(
        &self,
        object_name: &str,
        property_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("meta/remove_ignored_property", vec![
            Value::String(object_name.to_string()),
            Value::String(property_name.to_string()),
        ]).await
    }
    
    /// Remove an ignored verb from an object's meta
    pub async fn meta_remove_ignored_verb(
        &self,
        object_name: &str,
        verb_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("meta/remove_ignored_verb", vec![
            Value::String(object_name.to_string()),
            Value::String(verb_name.to_string()),
        ]).await
    }
    
    /// Clear all ignored properties from an object's meta
    pub async fn meta_clear_ignored_properties(
        &self,
        object_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("meta/clear_ignored_properties", vec![
            Value::String(object_name.to_string()),
        ]).await
    }
    
    /// Clear all ignored verbs from an object's meta
    pub async fn meta_clear_ignored_verbs(
        &self,
        object_name: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("meta/clear_ignored_verbs", vec![
            Value::String(object_name.to_string()),
        ]).await
    }
    
    // ==================== Workspace Operations ====================
    
    /// List workspace changes with optional status filter
    pub async fn workspace_list(
        &self,
        status_filter: Option<&str>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let args = match status_filter {
            Some(s) => vec![Value::String(s.to_string())],
            None => vec![],
        };
        self.rpc_call("workspace/list", args).await
    }
    
    /// Submit a change to workspace
    pub async fn workspace_submit(
        &self,
        change_json: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("workspace/submit", vec![Value::String(change_json.to_string())]).await
    }
    
    // ==================== Index Operations ====================
    
    /// List changes in the index with optional pagination
    pub async fn index_list(
        &self,
        limit: Option<usize>,
        page: Option<usize>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let mut args = vec![];
        
        if let Some(lim) = limit {
            args.push(Value::String(lim.to_string()));
            
            if let Some(pg) = page {
                args.push(Value::String(pg.to_string()));
            }
        }
        
        self.rpc_call("index/list", args).await
    }
    
    /// Calculate delta from a specific change ID
    pub async fn index_calc_delta(
        &self,
        change_id: &str,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        // Use RPC call with change_id as argument
        self.rpc_call("index/calc_delta", vec![Value::String(change_id.to_string())]).await
    }
    
    /// Update index from remote source
    pub async fn index_update(&self) -> Result<Value, Box<dyn std::error::Error>> {
        // If we have a database reference, call the async version directly
        if let Some(ref db) = self.database {
            let update_op = moor_vcs_worker::operations::IndexUpdateOperation::new(db.clone());
            match update_op.update_async().await {
                Ok(result_var) => {
                    // Convert Var to string result
                    let result_str = if let Some(s) = result_var.as_string() {
                        s.to_string()
                    } else {
                        format!("{:?}", result_var)
                    };
                    
                    let success = !result_str.starts_with("Error:");
                    return Ok(json!({
                        "success": success,
                        "result": result_str,
                        "operation": "index/update"
                    }));
                }
                Err(e) => {
                    return Ok(json!({
                        "success": false,
                        "result": format!("Error: {}", e),
                        "operation": "index/update"
                    }));
                }
            }
        }
        
        // Fallback to HTTP request
        self.rpc_call("index/update", vec![]).await
    }
    
    // ==================== Clone Operations ====================
    
    /// Export the current repository state (clone export)
    pub async fn clone_export(&self) -> Result<Value, Box<dyn std::error::Error>> {
        self.rpc_call("clone", vec![]).await
    }
    
    /// Import repository state from a remote URL (clone import)
    pub async fn clone_import(&self, url: &str) -> Result<Value, Box<dyn std::error::Error>> {
        // If we have a database reference, call the async version directly
        if let Some(ref db) = self.database {
            let clone_op = moor_vcs_worker::operations::CloneOperation::new(db.clone());
            match clone_op.import_from_url_async(url).await {
                Ok(result) => {
                    return Ok(json!({
                        "success": true,
                        "result": result,
                        "operation": "clone"
                    }));
                }
                Err(e) => {
                    return Ok(json!({
                        "success": false,
                        "result": format!("Error: {}", e),
                        "operation": "clone"
                    }));
                }
            }
        }
        
        // Fallback to HTTP request
        self.rpc_call("clone", vec![Value::String(url.to_string())]).await
    }
    
    // ==================== Low-level RPC ====================
    
    /// Make a raw RPC call with the given operation and arguments
    pub async fn rpc_call(
        &self,
        operation: &str,
        args: Vec<Value>,
    ) -> Result<Value, Box<dyn std::error::Error>> {
        let request = json!({
            "operation": operation,
            "args": args
        });
        
        make_request("POST", &format!("{}/rpc", self.base_url), Some(request)).await
    }
}

// ==================== Response Helpers ====================

/// Extension trait for working with RPC responses
pub trait ResponseExt {
    /// Check if the response indicates success
    fn is_success(&self) -> bool;
    
    /// Get the result string from the response
    fn get_result_str(&self) -> Option<&str>;
    
    /// Assert that the response is successful, panic with message if not
    fn assert_success(&self, context: &str) -> &Self;
    
    /// Assert that the response failed
    fn assert_failure(&self, context: &str) -> &Self;
    
    /// Get result string or panic
    fn require_result_str(&self, context: &str) -> &str;
}

impl ResponseExt for Value {
    fn is_success(&self) -> bool {
        self["success"].as_bool().unwrap_or(false)
    }
    
    fn get_result_str(&self) -> Option<&str> {
        self["result"].as_str()
    }
    
    fn assert_success(&self, context: &str) -> &Self {
        assert!(
            self.is_success(),
            "{} should succeed, got: {}",
            context,
            self
        );
        self
    }
    
    fn assert_failure(&self, context: &str) -> &Self {
        assert!(
            !self.is_success(),
            "{} should fail, got: {}",
            context,
            self
        );
        self
    }
    
    fn require_result_str(&self, context: &str) -> &str {
        self.get_result_str()
            .unwrap_or_else(|| panic!("{}: response has no result string: {}", context, self))
    }
}

// ==================== Database Assertion Helpers ====================

/// Helper for making common database assertions
pub struct DbAssertions<'a> {
    db: &'a DatabaseRef,
}

impl<'a> DbAssertions<'a> {
    pub fn new(db: &'a DatabaseRef) -> Self {
        Self { db }
    }
    
    /// Assert that an object ref exists
    pub fn assert_ref_exists(
        &self,
        object_type: moor_vcs_worker::types::VcsObjectType,
        name: &str,
    ) -> String {
        let ref_hash = self.db.refs()
            .get_ref(object_type, name, None)
            .expect("Failed to query ref")
            .unwrap_or_else(|| panic!("Ref for '{}' should exist", name));
        ref_hash
    }
    
    /// Assert that an object ref does not exist
    pub fn assert_ref_not_exists(
        &self,
        object_type: moor_vcs_worker::types::VcsObjectType,
        name: &str,
    ) {
        let ref_hash = self.db.refs()
            .get_ref(object_type, name, None)
            .expect("Failed to query ref");
        assert!(ref_hash.is_none(), "Ref for '{}' should not exist", name);
    }
    
    /// Assert that a SHA256 hash exists in objects
    pub fn assert_sha256_exists(&self, sha256: &str) {
        let exists = self.db.objects()
            .get(sha256)
            .expect("Failed to query objects")
            .is_some();
        assert!(exists, "SHA256 '{}' should exist in objects", sha256);
    }
    
    /// Assert that a SHA256 hash does not exist in objects
    pub fn assert_sha256_not_exists(&self, sha256: &str) {
        let exists = self.db.objects()
            .get(sha256)
            .expect("Failed to query objects")
            .is_some();
        assert!(!exists, "SHA256 '{}' should not exist in objects", sha256);
    }
    
    /// Get the top change or panic if none exists
    pub fn require_top_change(&self) -> (String, moor_vcs_worker::types::Change) {
        let change_id = self.db.index()
            .get_top_change()
            .expect("Failed to get top change")
            .expect("Should have a top change");
        
        let change = self.db.index()
            .get_change(&change_id)
            .expect("Failed to get change")
            .expect("Change should exist");
        
        (change_id, change)
    }
    
    /// Assert that there is no top change
    pub fn assert_no_top_change(&self) {
        let top_change = self.db.index()
            .get_top_change()
            .expect("Failed to get top change");
        assert!(top_change.is_none(), "Should have no top change");
    }
    
    /// Assert that an object is in the added_objects list of the top change
    pub fn assert_object_in_top_change(&self, object_name: &str) {
        let (_, change) = self.require_top_change();
        let found = change.added_objects.iter().any(|obj| obj.name == object_name);
        assert!(found, "Object '{}' should be in top change's added_objects", object_name);
    }
}

// Re-export DatabaseRef for convenience
pub use moor_vcs_worker::DatabaseRef;

