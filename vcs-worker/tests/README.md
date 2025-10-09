# VCS Worker Integration Tests

This directory contains integration tests for the vcs-worker service, organized to mirror the project structure.

## Test Architecture

### Directory Structure

```
tests/
├── common/
│   └── mod.rs              # Reusable test harness and utilities
├── operations/
│   ├── mod.rs              # Operations test module
│   └── object.rs           # Object operation tests
├── resources/
│   ├── test_object.moo     # Test fixtures
│   ├── detailed_test_object.moo
│   └── test_object_{1,2,3}.moo
├── test_suite.rs           # Main test entry point
└── README.md               # This file
```

### Design Principles

1. **Mirror Project Structure**: Tests are organized to match `src/` layout
2. **Reusable Harness**: Common test utilities in `common/` module
3. **Direct State Verification**: Tests access database providers directly
4. **Resource-Based Fixtures**: Test data stored in `.moo` files

## Test Harness (`common` module)

The `common` module provides reusable test infrastructure:

### TestServer

Manages the complete lifecycle of a test server with temporary database:

```rust
let server = TestServer::start().await?;

// Access the database directly
let objects = server.database().objects();
let refs = server.database().refs();
let index = server.database().index();

// Make API requests
let response = make_request("POST", &server.base_url(), payload).await?;
```

### Helper Functions

```rust
// Load test fixtures
let content = load_moo_file("test_object.moo");
let lines = moo_to_lines(&content);

// Calculate SHA256 hashes
let hash = TestServer::calculate_sha256(&content);

// Make HTTP requests
let response = make_request("POST", url, Some(json_body)).await?;
```

### Provider Trait Re-exports

The `common` module re-exports provider traits so they're available to all tests:

```rust
use crate::common::*;  // Brings in ObjectsProvider, RefsProvider, IndexProvider traits
```

## Test Organization

### Operations Tests (`operations/`)

Tests for VCS operations, organized by operation type:

#### `object.rs` - Object Operations

- **test_object_create_and_verify**: Full workflow from creation to verification
- **test_object_content_integrity**: Content preservation and field validation
- **test_multiple_objects_persistence**: Multi-object creation and persistence

**Future modules:**
- `change.rs` - Change management operations
- `index.rs` - Index operations  
- `workspace.rs` - Workspace operations

## Test Data and Resources

### Test Resources (`resources/`)

Test fixtures stored as `.moo` files:

- **test_object.moo** - Basic test object
- **detailed_test_object.moo** - Object with additional fields
- **test_object_{1,2,3}.moo** - Multiple objects for batch tests

### MOO Object Format

```
object #9999
  name: "Test Object"
  parent: #1
  location: #2
  owner: #2
endobject
```

### Loading Resources

```rust
// Load .moo file
let object_dump = load_moo_file("test_object.moo");

// Convert to lines for API
let object_content = moo_to_lines(&object_dump);

// Calculate hash (API joins lines with "\n")
let object_dump = object_content.join("\n");
let sha256_hash = TestServer::calculate_sha256(&object_dump);
```

## Running Tests

### Run all tests
```bash
cargo test
```

### Run specific test suite
```bash
cargo test --test test_suite
```

### Run tests in a specific module
```bash
cargo test --test test_suite operations::object
```

### Run a specific test
```bash
cargo test --test test_suite test_object_create_and_verify
```

### Run with output
```bash
cargo test --test test_suite -- --nocapture
```

### Run sequentially
```bash
cargo test --test test_suite -- --test-threads=1
```

## Test Strategy

### Direct Database Verification

Tests verify internal state directly rather than parsing API responses:

```rust
// Calculate expected hash
let sha256_hash = TestServer::calculate_sha256(&object_dump);

// Verify object exists in objects provider
let stored = server.database().objects().get(&sha256_hash)?;
assert!(stored.is_some());

// Verify ref points to correct hash
let ref_hash = server.database().refs().get_ref("object_name", None)?;
assert_eq!(ref_hash, Some(sha256_hash));

// Verify change tracking
let change = server.database().index().get_change(&change_id)?;
assert!(change.added_objects.iter().any(|obj| obj.name == "object_name"));
```

### Benefits

1. **More Reliable**: Direct access to actual state vs parsing strings
2. **Faster**: No JSON serialization/deserialization overhead
3. **Better Errors**: Clear assertion failures on actual values
4. **Type-Safe**: Compiler-checked access to database state
5. **Easier Debug**: Can inspect actual database structures

## Adding New Tests

### 1. Add Test to Existing Module

Edit an existing test file (e.g., `operations/object.rs`):

```rust
#[tokio::test]
async fn test_new_feature() {
    let server = TestServer::start().await.expect("Failed to start");
    
    // Your test code here
    
    // Direct verification
    assert!(server.database().objects().get(&hash)?.is_some());
}
```

### 2. Create New Test Module

Create new file `operations/change.rs`:

```rust
//! Integration tests for change operations

use crate::common::*;
use moor_vcs_worker::types::ChangeStatus;

#[tokio::test]
async fn test_change_create() {
    let server = TestServer::start().await.expect("Failed to start");
    // Test implementation
}
```

Add to `operations/mod.rs`:

```rust
mod object;
mod change;  // Add this line
```

### 3. Add New Test Fixture

Create new `.moo` file in `resources/`:

```bash
cat > tests/resources/my_fixture.moo << 'EOF'
object #5678
  name: "My Test Object"
  parent: #1
endobject
EOF
```

Use in tests:

```rust
let content = load_moo_file("my_fixture.moo");
```

## Test Isolation

Each test:
- Uses a unique temporary database (`TempDir`)
- Starts its own HTTP server on a random port
- Cleans up automatically when dropped
- Can run in parallel with other tests

## Expected Behavior

### Success Criteria

- All tests should pass
- Each test completes in under 2 seconds
- Temporary databases are cleaned up automatically
- No errors or panics during execution
- SHA256 hashes match calculated values
- All refs point to correct hashes
- Changes track objects correctly

### Troubleshooting

**Tests fail to start server:**
- Ensure ports are available for binding
- Check temporary directory is writable

**Database errors:**
- Verify `fjall` dependencies are installed
- Check disk space for temporary files

**Hash mismatches:**
- Verify .moo files don't have trailing whitespace
- Remember API joins lines with `\n`

## Continuous Integration

These tests are designed for CI/CD:
- No external dependencies
- Self-contained with temporary resources
- Fast execution time
- Clear pass/fail indicators
- Parallel execution safe

## Future Enhancements

Planned test additions:
- Change operation tests (create, submit, approve)
- Index operation tests (list, calc delta, update)
- Workspace operation tests (submit, list)
- Router/API endpoint tests
- Provider-level unit tests
- Performance/load tests
- Concurrent modification tests
