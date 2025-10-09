use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::net::TcpListener;
use moor_vcs_worker::{create_registry_with_config, Config, DatabaseRef};
use moor_vcs_worker::router::create_http_router;
use moor_vcs_worker::providers::objects::ObjectsProvider;
use moor_vcs_worker::providers::refs::RefsProvider;
use moor_vcs_worker::providers::index::IndexProvider;
use moor_vcs_worker::types::ChangeStatus;
use serde_json::json;
use sha2::{Sha256, Digest};

/// Helper struct to manage test server lifecycle
struct TestServer {
    port: u16,
    temp_dir: TempDir,
    database: DatabaseRef,
    _shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl TestServer {
    /// Start a new test server with a temporary database
    async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        // Create temporary directory for database
        let temp_dir = TempDir::new()?;
        
        // Create config with test database path
        let config = Config::with_db_path(temp_dir.path().to_path_buf());
        
        // Create operation registry and get database reference
        let (registry, database) = create_registry_with_config(config)?;
        let registry = Arc::new(registry);
        
        // Find an available port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        
        // Create router
        let router = create_http_router(registry);
        
        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        
        // Spawn server in background
        tokio::spawn(async move {
            let server = axum::serve(listener, router.into_make_service());
            
            tokio::select! {
                result = server => {
                    if let Err(e) = result {
                        eprintln!("Server error: {}", e);
                    }
                }
                _ = &mut shutdown_rx => {
                    // Shutdown requested
                }
            }
        });
        
        // Wait for server to be ready
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(Self {
            port,
            temp_dir,
            database,
            _shutdown_tx: shutdown_tx,
        })
    }
    
    /// Get the base URL for API requests
    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
    
    /// Get the database path
    fn db_path(&self) -> std::path::PathBuf {
        self.temp_dir.path().to_path_buf()
    }
    
    /// Get the database reference for direct state inspection
    fn database(&self) -> &DatabaseRef {
        &self.database
    }
    
    /// Calculate SHA256 hash for object content (matches what the system does)
    fn calculate_sha256(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Load a .moo file from the test resources directory
fn load_moo_file(filename: &str) -> String {
    let resources_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("resources");
    let file_path = resources_dir.join(filename);
    
    std::fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {}", file_path.display(), e))
}

/// Convert .moo file content to a vector of lines (for API compatibility)
fn moo_to_lines(content: &str) -> Vec<String> {
    content.lines().map(|s| s.to_string()).collect()
}

/// Helper function to make HTTP requests
async fn make_request(
    method: &str,
    url: &str,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let request = match method {
        "GET" => client.get(url),
        "POST" => {
            let mut req = client.post(url);
            if let Some(json_body) = body {
                req = req.json(&json_body);
            }
            req
        }
        _ => panic!("Unsupported HTTP method: {}", method),
    };
    
    let response = request.send().await?;
    let status = response.status();
    let text = response.text().await?;
    
    if !status.is_success() {
        return Err(format!("HTTP error {}: {}", status, text).into());
    }
    
    let json: serde_json::Value = serde_json::from_str(&text)?;
    Ok(json)
}

#[tokio::test]
async fn test_object_update_workflow() {
    // Start test server
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test server started at: {}", base_url);
    println!("Database path: {:?}", server.db_path());
    
    // Step 1: Verify no active change initially
    println!("\nStep 1: Verifying no active change in database initially...");
    
    let top_change = server.database().index().get_top_change()
        .expect("Failed to get top change");
    
    assert!(
        top_change.is_none(),
        "Expected no top change initially, but found: {:?}",
        top_change
    );
    
    println!("✅ Confirmed: No active change in database initially");
    
    // Step 2: Create an object update
    println!("\nStep 2: Creating object update for test object...");
    let object_name = "test_object";
    let object_dump = load_moo_file("test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    let update_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(update_request),
    )
    .await
    .expect("Failed to update object");
    
    println!("Update response: {}", serde_json::to_string_pretty(&update_response).unwrap());
    
    // Verify the update was successful via API
    assert!(
        update_response["success"].as_bool().unwrap_or(false),
        "Object update should succeed, got: {}",
        update_response
    );
    
    // Now verify the internal state directly
    // The API joins the lines with "\n", so we need to match that for hash calculation
    let object_dump = object_content.join("\n");
    let sha256_hash = TestServer::calculate_sha256(&object_dump);
    
    println!("Calculated SHA256: {}", sha256_hash);
    
    // Check that the object exists in the objects provider with this hash
    let stored_object = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object from provider");
    
    assert!(
        stored_object.is_some(),
        "Object with hash {} should exist in objects provider",
        sha256_hash
    );
    
    println!("✅ Confirmed: Object exists in objects provider with correct hash");
    
    // Check that the ref was created
    let object_ref = server.database().refs().get_ref(object_name, None)
        .expect("Failed to get object ref");
    
    assert!(
        object_ref.is_some(),
        "Object ref for '{}' should exist",
        object_name
    );
    
    assert_eq!(
        object_ref.unwrap(),
        sha256_hash,
        "Object ref should point to the correct SHA256 hash"
    );
    
    println!("✅ Confirmed: Object ref points to correct hash");
    
    // Step 3: Verify change tracks the object
    println!("\nStep 3: Verifying change tracks the object...");
    
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change after creating object");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    // Verify the change has our object
    assert_eq!(
        change.status,
        ChangeStatus::Local,
        "Change should be in Local status"
    );
    
    let object_in_change = change.added_objects.iter()
        .any(|obj| obj.name == object_name);
    
    assert!(
        object_in_change,
        "Object '{}' should be in added_objects list, found: {:?}",
        object_name,
        change.added_objects
    );
    
    println!("✅ Confirmed: Change exists with our object in added_objects list");
    
    // Step 4: Verify stored content matches submission exactly
    println!("\nStep 4: Verifying stored content matches submission...");
    
    let stored_content = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object")
        .expect("Object should exist");
    
    assert_eq!(
        stored_content,
        object_dump,
        "Stored content should match exactly what was submitted"
    );
    
    println!("✅ Confirmed: Stored content matches submission exactly");
    
    println!("\n✅ Test completed successfully!");
    
    // Cleanup happens automatically when TestServer is dropped
}

#[tokio::test]
async fn test_database_persistence_and_change_tracking() {
    // Start test server
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    let db_path = server.db_path();
    
    println!("Test server started at: {}", base_url);
    println!("Database path: {:?}", db_path);
    
    // Step 1: Create multiple object updates
    println!("\nStep 1: Creating multiple object updates...");
    
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        let object_dump = load_moo_file(&format!("test_object_{}.moo", i));
        let object_content = moo_to_lines(&object_dump);
        
        let update_request = json!({
            "operation": "object/update",
            "args": [
                object_name,
                serde_json::to_string(&object_content).unwrap()
            ]
        });
        
        let update_response = make_request(
            "POST",
            &format!("{}/rpc", base_url),
            Some(update_request),
        )
        .await
        .expect("Failed to update object");
        
        assert!(
            update_response["success"].as_bool().unwrap_or(false),
            "Object update {} should succeed",
            i
        );
        
        // The API joins the lines with "\n", so calculate hash on that
        let object_dump = object_content.join("\n");
        let sha256_hash = TestServer::calculate_sha256(&object_dump);
        
        // Verify the object exists in the database
        let stored = server.database().objects().get(&sha256_hash)
            .expect("Failed to query objects provider");
        assert!(stored.is_some(), "Object {} should be stored", i);
        
        // Verify the ref exists
        let ref_hash = server.database().refs().get_ref(&object_name, None)
            .expect("Failed to query refs provider");
        assert_eq!(ref_hash, Some(sha256_hash.clone()), "Ref should point to correct hash");
        
        println!("Created test_object_{} with hash {}", i, sha256_hash);
    }
    
    // Step 2: Verify the database was built and contains the changes
    println!("\nStep 2: Verifying database persistence...");
    
    // Check that the database directory was created
    assert!(db_path.exists(), "Database directory should exist");
    
    // The fjall database should have created subdirectories
    let db_contents: Vec<_> = std::fs::read_dir(&db_path)
        .expect("Failed to read database directory")
        .filter_map(|entry| entry.ok())
        .collect();
    
    assert!(
        !db_contents.is_empty(),
        "Database directory should contain files"
    );
    
    println!("Database contains {} entries", db_contents.len());
    
    // Verify all 3 objects are in the objects provider
    let objects_count = server.database().objects().count();
    assert!(
        objects_count >= 3,
        "Objects provider should contain at least 3 objects, found {}",
        objects_count
    );
    
    println!("✅ Confirmed: {} objects stored in objects provider", objects_count);
    
    // Step 3: Verify the change is at the top of the index
    println!("\nStep 3: Verifying change is tracked in index...");
    
    // Check database state directly
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    println!("Top change: {} (status: {:?})", top_change_id, change.status);
    
    // Verify all 3 objects are in the change
    assert_eq!(
        change.added_objects.len(),
        3,
        "Change should have 3 added objects, found: {:?}",
        change.added_objects
    );
    
    // Verify each object is in the change
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        let found = change.added_objects.iter()
            .any(|obj| obj.name == object_name);
        assert!(found, "Object {} should be in change", object_name);
    }
    
    println!("✅ Confirmed: All 3 objects are in the top change");
    
    // Step 4: Verify the DB contains the object updates in top of changes
    println!("\nStep 4: Verifying DB contains object updates at top of changes...");
    
    // Check that it's a Local change (top of changes)
    assert_eq!(
        change.status,
        ChangeStatus::Local,
        "Top change should be Local status"
    );
    
    // Verify the change order - our change should be at the top
    let all_changes = server.database().index().get_change_order()
        .expect("Failed to get change order");
    
    if !all_changes.is_empty() {
        assert_eq!(
            all_changes[all_changes.len() - 1],
            top_change_id,
            "Our change should be at the top (end) of the change order"
        );
        println!("✅ Confirmed: Change is at the top of the change order");
    }
    
    // Verify we can retrieve each object by its ref
    for i in 1..=3 {
        let object_name = format!("test_object_{}", i);
        let ref_hash = server.database().refs().get_ref(&object_name, None)
            .expect("Failed to get ref");
        assert!(ref_hash.is_some(), "Ref for {} should exist", object_name);
        
        let content = server.database().objects().get(&ref_hash.unwrap())
            .expect("Failed to get object");
        assert!(content.is_some(), "Object content for {} should exist", object_name);
        assert!(
            content.unwrap().contains(&format!("Test Object {}", i)),
            "Object {} should contain correct name",
            i
        );
    }
    
    println!("✅ Confirmed: All objects retrievable and contain correct content");
    
    println!("\n✅ Test completed successfully!");
    
    // Cleanup happens automatically when TestServer is dropped
}

#[tokio::test]
async fn test_object_update_and_retrieval() {
    // Start test server
    let server = TestServer::start().await.expect("Failed to start test server");
    let base_url = server.base_url();
    
    println!("Test server started at: {}", base_url);
    
    // Create an object with specific content
    let object_name = "detailed_test_object";
    let object_dump = load_moo_file("detailed_test_object.moo");
    let object_content = moo_to_lines(&object_dump);
    
    let update_request = json!({
        "operation": "object/update",
        "args": [
            object_name,
            serde_json::to_string(&object_content).unwrap()
        ]
    });
    
    let update_response = make_request(
        "POST",
        &format!("{}/rpc", base_url),
        Some(update_request),
    )
    .await
    .expect("Failed to update object");
    
    assert!(
        update_response["success"].as_bool().unwrap_or(false),
        "Object update should succeed"
    );
    
    // Verify directly from database
    // The API joins the lines with "\n", so calculate hash on that
    let object_dump = object_content.join("\n");
    let sha256_hash = TestServer::calculate_sha256(&object_dump);
    
    println!("Calculated SHA256: {}", sha256_hash);
    
    // Check that the object exists in the objects provider
    let stored_content = server.database().objects().get(&sha256_hash)
        .expect("Failed to get object from provider")
        .expect("Object should exist in objects provider");
    
    // Verify exact content match
    assert_eq!(
        stored_content,
        object_dump,
        "Stored content should match exactly"
    );
    
    println!("✅ Stored content matches exactly");
    
    // Verify the ref points to the correct hash
    let ref_hash = server.database().refs().get_ref(object_name, None)
        .expect("Failed to get ref")
        .expect("Ref should exist");
    
    assert_eq!(
        ref_hash,
        sha256_hash,
        "Ref should point to the correct SHA256 hash"
    );
    
    println!("✅ Ref points to correct hash");
    
    // Verify specific fields in the content
    assert!(
        stored_content.contains("Detailed Test Object"),
        "Content should contain object name"
    );
    assert!(
        stored_content.contains("#12345"),
        "Content should contain object ID"
    );
    assert!(
        stored_content.contains("readable: true"),
        "Content should contain readable flag"
    );
    
    println!("✅ All content fields verified");
    
    // Verify the object is in a change
    let top_change_id = server.database().index().get_top_change()
        .expect("Failed to get top change")
        .expect("Should have a top change");
    
    let change = server.database().index().get_change(&top_change_id)
        .expect("Failed to get change")
        .expect("Change should exist");
    
    let object_in_change = change.added_objects.iter()
        .any(|obj| obj.name == object_name);
    
    assert!(
        object_in_change,
        "Object should be tracked in the change"
    );
    
    println!("✅ Object is tracked in change");
    
    println!("\n✅ Content verification test passed!");
}

