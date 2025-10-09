# VCS Worker Integration Tests

This directory contains integration tests for the vcs-worker service.

## Test Suite Overview

The integration test suite validates the complete workflow of the VCS worker using **direct database state inspection** rather than parsing API responses. This provides:

- **Direct verification** of internal state (SHA256 hashes, refs, changes)
- **Faster execution** without JSON parsing overhead
- **More reliable** assertions on actual database state
- **Better debugging** with clear state inspection

Tests validate:

1. **Database persistence** - Verifying objects are stored with correct SHA256 hashes
2. **Reference tracking** - Ensuring refs point to correct object hashes
3. **Change tracking** - Confirming changes contain expected objects
4. **Content integrity** - Validating exact content matches submissions

## Tests Included

### 1. `test_object_update_workflow`

This test validates the core workflow by checking database state directly:
- **Step 1**: Verifies `get_top_change()` returns None initially
- **Step 2**: Creates object, calculates SHA256, verifies it exists in objects provider
- **Step 3**: Checks refs provider points to correct hash
- **Step 4**: Verifies change contains the object in `added_objects` list
- **Step 5**: Confirms stored content matches submission exactly

### 2. `test_database_persistence_and_change_tracking`

This test verifies database and change tracking with direct state inspection:
- **Step 1**: Creates 3 objects, verifies each SHA256 hash exists in objects provider
- **Step 2**: Confirms refs provider has entries for all 3 objects
- **Step 3**: Verifies objects provider count is at least 3
- **Step 4**: Checks top change contains all 3 objects in `added_objects`
- **Step 5**: Validates change is at top of change order
- **Step 6**: Retrieves each object by ref and verifies content

### 3. `test_object_update_and_retrieval`

This test validates content integrity with direct hash verification:
- Creates object with detailed content
- Calculates expected SHA256 hash
- Verifies object exists in objects provider with exact hash
- Confirms ref points to correct hash
- Validates stored content is byte-for-byte identical
- Checks object is tracked in change

## Running the Tests

### Run all integration tests:
```bash
cargo test --test integration_test
```

### Run with output:
```bash
cargo test --test integration_test -- --nocapture
```

### Run a specific test:
```bash
cargo test --test integration_test test_object_update_workflow
```

### Run with verbose output:
```bash
cargo test --test integration_test -- --nocapture --test-threads=1
```

## Test Architecture

### Test Server Setup
Each test creates a temporary database and starts an HTTP server on a random available port. This ensures:
- Tests run in isolation
- No conflicts between concurrent tests
- Automatic cleanup after test completion
- **Direct database access** for state verification

### Temporary Database
Tests use `tempfile::TempDir` to create isolated database instances that are automatically cleaned up when the test completes.

### Direct State Inspection
Tests access database providers directly:
- **Objects Provider**: `server.database().objects().get(sha256_hash)`
- **Refs Provider**: `server.database().refs().get_ref(name, version)`
- **Index Provider**: `server.database().index().get_top_change()`
- **SHA256 Verification**: `TestServer::calculate_sha256(content)`

This "white box" approach validates actual internal state rather than parsing API response strings.

## Test Data

### Test Resources

Test fixtures are stored in the `tests/resources/` directory as `.moo` files:

- **test_object.moo** - Basic test object used in workflow tests
- **detailed_test_object.moo** - Object with additional fields for content verification
- **test_object_1.moo**, **test_object_2.moo**, **test_object_3.moo** - Multiple objects for persistence tests

### Loading Test Resources

Tests use helper functions to load .moo files:

```rust
// Load a .moo file from resources
let object_dump = load_moo_file("test_object.moo");

// Convert to lines for API compatibility
let object_content = moo_to_lines(&object_dump);

// Calculate hash (API joins lines with "\n")
let object_dump = object_content.join("\n");
let sha256_hash = TestServer::calculate_sha256(&object_dump);
```

### MOO Object Format

MOO objects follow this format:
```
object #9999
  name: "Test Object"
  parent: #1
  location: #2
  owner: #2
endobject
```

### Adding New Test Fixtures

1. Create a new `.moo` file in `tests/resources/`
2. Use `load_moo_file("your_file.moo")` in your test
3. The file will be automatically loaded and parsed

## Expected Behavior

### Success Criteria
- All three tests should pass
- Each test should complete in under 2 seconds
- Temporary databases should be cleaned up automatically
- No errors or panics during execution
- SHA256 hashes match calculated values
- All refs point to correct hashes
- Changes track objects correctly

### Direct State Verification Examples

```rust
// Calculate SHA256 hash for content
let sha256_hash = TestServer::calculate_sha256(&object_dump);

// Verify object exists in objects provider
let stored = server.database().objects().get(&sha256_hash)
    .expect("Failed to query")
    .expect("Object should exist");

// Verify ref points to correct hash
let ref_hash = server.database().refs().get_ref("object_name", None)
    .expect("Failed to query");
assert_eq!(ref_hash, Some(sha256_hash));

// Verify change contains object
let change = server.database().index().get_change(&change_id)
    .expect("Failed to get change");
assert!(change.added_objects.iter().any(|obj| obj.name == "object_name"));
```

## Troubleshooting

### Tests Fail to Start Server
- Ensure ports 0-65535 are available for binding
- Check that the temporary directory is writable

### Database Errors
- Verify `fjall` dependencies are installed
- Check disk space for temporary files

### API Timeout
- Increase wait time in `TestServer::start()` if needed
- Check for port conflicts

## Adding New Tests

To add a new integration test:

1. Create a new async test function:
```rust
#[tokio::test]
async fn test_new_feature() {
    let server = TestServer::start().await.expect("Failed to start test server");
    // Your test code here
}
```

2. Use the `make_request` helper for API calls:
```rust
let response = make_request(
    "POST",
    &format!("{}/api/endpoint", server.base_url()),
    Some(json!({"key": "value"})),
).await.expect("Request failed");
```

3. Add appropriate assertions to verify behavior

## Continuous Integration

These tests are designed to run in CI/CD environments:
- No external dependencies required
- Self-contained with temporary resources
- Fast execution time
- Clear pass/fail indicators

## Cleanup

Test cleanup is automatic:
- `TestServer` implements Drop semantics via `TempDir`
- Database files are removed when tests complete
- HTTP servers are shut down via oneshot channels
- No manual cleanup required

