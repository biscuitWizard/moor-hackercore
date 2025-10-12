# VCS Worker Architecture

## Overview

The VCS Worker is a version control system designed specifically for MOO (Multi-User Object Oriented) environments. It provides Git-like functionality for managing MOO object definitions, enabling collaborative development, code review, and change tracking.

## Design Philosophy

### RPC Over REST

Unlike traditional RESTful APIs, the VCS Worker implements an **RPC (Remote Procedure Call)** architecture:

- **Single endpoint pattern**: Operations are invoked by name through a consistent request structure
- **Uniform interface**: All requests follow the same format: `{"operation": "name", "args": [...]}`
- **MOO-friendly**: Designed to work seamlessly with MOO's `worker_request()` function
- **Type-safe operations**: Each operation is a strongly-typed Rust struct implementing the `Operation` trait

### Changelist-Based Workflow

The system uses a changelist model similar to Perforce rather than Git's branch model:

- **Changes as units of work**: Each feature/fix is developed in a separate change
- **Serial development**: Only one change is active (Local) at a time
- **Easy context switching**: Switch between changes without losing work
- **Clear state transitions**: Changes move through well-defined states (Local → Review → Merged)

## Core Components

### 1. Database Layer (`database/`)

The foundational storage layer managing all persistent data.

```
database/
├── Database          - Main database coordinator
├── ObjectsTree       - Content-addressable object storage (SHA256-keyed)
├── Refs              - Object name → SHA256 mappings with versioning
├── Index             - Change tracking and history management
└── Workspace         - Non-active change storage
```

#### Objects Tree
- **Content-addressable storage**: Objects stored by SHA256 hash of content
- **Deduplication**: Identical objects share storage
- **Two types**: MOO objects (objdef format) and meta objects (YAML)
- **Immutable**: Once stored, objects never change
- **Backend**: Fjall embedded database

#### Refs (References)
- **Name resolution**: Maps `(object_name, version, type)` tuples to SHA256 hashes
- **Versioning**: Each object has a version number that increments on *submitted* changes (not within a local change)
- **Multi-type support**: Handles MOO objects and meta objects separately
- **Fast lookups**: Enables quick resolution of current object state
- **Critical invariant**: Refs MUST stay in sync with the names in Change tracking lists

**Key Constraint**: When an object's name changes in a Change's tracking lists (added_objects, modified_objects), the corresponding ref must be updated from old_name → new_name, otherwise "version N not found" errors occur when resolving the object.

#### Index
- **Change ordering**: Maintains chronological list of changes (changelist)
- **Current state**: Tracks which change is active (top of index)
- **History compilation**: Computes current repository state from change sequence
- **Source tracking**: Records remote repository URL if cloned

#### Workspace
- **Non-active changes**: Stores Idle and Review changes
- **State filtering**: Can query changes by status
- **Change parking**: Temporary storage when switching contexts

### 2. Providers Layer (`providers/`)

Abstraction layer providing high-level interfaces to database functionality.

```
providers/
├── ObjectsProvider    - Object storage and retrieval
├── RefsProvider       - Name resolution and versioning
├── IndexProvider      - Change and history management
├── WorkspaceProvider  - Workspace change management
└── UserProvider       - User authentication and permissions
```

Each provider:
- Encapsulates database operations for a specific domain
- Provides transaction-like semantics
- Handles error mapping and validation
- Implements business logic rules

### 3. Operations Layer (`operations/`)

The public API surface implementing all user-facing functionality.

```
operations/
├── object/      - Object CRUD operations
├── change/      - Change lifecycle management
├── index/       - Index and history queries
├── workspace/   - Workspace management
├── meta/        - Object filtering configuration
└── user/        - User information
```

#### Operation Trait

All operations implement a common trait:

```rust
pub trait Operation: Send + Sync {
    fn name(&self) -> &'static str;           // Operation identifier
    fn description(&self) -> &'static str;    // Short description
    fn philosophy(&self) -> &'static str;     // Purpose and workflow context
    fn parameters(&self) -> Vec<OperationParameter>;  // Parameter specs
    fn examples(&self) -> Vec<OperationExample>;      // Usage examples
    fn routes(&self) -> Vec<OperationRoute>;  // HTTP endpoints
    fn execute(&self, args: Vec<String>, user: &User) -> Var;  // Core logic
}
```

This trait enables:
- **Self-documenting APIs**: Operations describe themselves
- **Dynamic routing**: Routes generated from operation definitions
- **Swagger generation**: Documentation auto-generated from metadata
- **Uniform execution**: All operations called the same way

#### Operation Registry

Central registry managing all available operations:

```rust
pub struct OperationRegistry {
    operations: HashMap<String, Box<dyn Operation>>,
    user_provider: Option<Arc<dyn UserProvider>>,
}
```

- **Dynamic discovery**: Operations registered at startup
- **Name-based dispatch**: Routes requests to appropriate operation
- **User context injection**: Provides user information to operations
- **HTTP and RPC support**: Single implementation serves both interfaces

### 4. Router Layer (`router.rs`)

HTTP server and API gateway.

```
Router
├── Base redirect (/)          → /swagger-ui
├── Swagger UI (/swagger-ui)   → Interactive documentation
├── OpenAPI spec (/api-docs)   → Machine-readable API definition
├── RPC endpoint (/rpc)        → Generic operation executor
└── Named routes (/api/*)      → Direct operation endpoints
```

#### Dynamic Route Generation

Routes are automatically generated from operation definitions:

```rust
for (route, op_name) in registry.get_all_routes() {
    // Generate Axum route handler
    // Extract operation parameters
    // Build OpenAPI documentation
    // Add to router
}
```

#### OpenAPI/Swagger Integration

- **Auto-generated specs**: Created from operation metadata
- **Category-based organization**: Operations grouped by folder structure
- **Rich documentation**: Includes philosophy, parameters, examples
- **Request body schemas**: Proper parameter documentation and examples
- **Try it out**: Interactive testing from browser

### 5. Types Layer (`types.rs`)

Common data structures used throughout the system.

#### Core Types

- **Change**: Represents a changelist with metadata and object lists
- **ObjectInfo**: Identifies an object (type, name, version)
- **ChangeStatus**: Enum defining change states (Local, Idle, Review, Merged)
- **ObjectDiffModel**: Represents differences between states
- **User**: Authentication and permission information

#### Local Changes: Collapsed Diffs (Critical Concept)

**Local changes are NOT historical records - they are collapsed diffs from the previous state.**

Key implications:

1. **No intermediate history**: If you modify an object multiple times in the same local change, only the final state matters. The intermediate states never existed in any committed form.

2. **Version numbers don't increment**: Within a local change, modifying an object multiple times keeps the same version number. The version only increments when the change is submitted/approved.

3. **Renames collapse**: If you rename A→B→C within a local change, only A→C is recorded. The intermediate name B never existed in committed history, so:
   - Only ONE ref exists (pointing from C to the SHA256)
   - No ref for B is created or maintained
   - The change records A→C in renamed_objects

4. **Refs are replaced in-place**: When you modify an object already modified in the local change, the old SHA256 is replaced (and deleted if orphaned), keeping the same version number.

5. **Critical constraint**: Operations that update object names in tracking lists (added_objects, modified_objects) MUST also update the refs from old_name to new_name, maintaining the invariant that refs and tracking lists stay synchronized.

**Example**: 
```
# In a single local change:
1. Modify object "foo" (version 2)
2. Rename "foo" to "bar"
3. Modify "bar" again

Result:
- One entry in modified_objects: {name: "bar", version: 2}
- One entry in renamed_objects: {from: "foo", to: "bar"}
- One ref: ("bar", 2) → SHA256_final
- NO ref for "foo" version 2 (deleted during rename)
- Version stayed at 2 throughout (replaced in-place)
```

## Data Flow

### Creating and Modifying Objects

```
User Request (MOOCode or HTTP)
    ↓
Router → Operation Registry
    ↓
Operation.execute()
    ↓
1. Get/create local change (IndexProvider)
2. Parse and validate object (ObjectsProvider)
3. Apply meta filtering (RefsProvider)
4. Generate SHA256 hash
5. Store object content (ObjectsTree)
6. Update ref (Refs: name → SHA256, version++ if first mod, else reuse version)
   - First modification in change: version increments
   - Subsequent modifications in same change: version stays same, ref replaced
7. Track in change (Index: added/modified lists)
   - If already tracked: update in-place, delete orphaned old SHA256
   - If not tracked: add to appropriate list
    ↓
Return success
```

### Retrieving Objects

```
User Request
    ↓
Router → Operation
    ↓
1. Resolve current state (IndexProvider)
   - Walk changelist history
   - Apply renames and deletes
   - Find current name
2. Get ref (RefsProvider: name → SHA256)
3. Retrieve content (ObjectsTree: SHA256 → content)
4. Apply meta filtering (remove ignored properties/verbs)
    ↓
Return object definition
```

### Change Workflow

```
1. Create Change
   - Generate UUID
   - Set status = Local
   - Store in Index
   - Add to top of changelist

2. Modify Objects
   - Objects tracked in change (collapsed state)
   - Added/modified/deleted/renamed lists show FINAL state only
   - Refs updated (version increments once per object, then replaced in-place)
   - Content stored by SHA256 (old SHAs deleted if orphaned)
   - Multiple modifications to same object collapse into single entry
   - Rename chains collapse (A→B→C becomes A→C)

3. Check Status
   - Build ObjectDiffModel
   - Compare current state to pre-change state
   - Return summary of modifications

4. Submit Change
   IF remote repository:
      - Status → Review
      - Move to Workspace
      - Remove from Index top
      - Send to remote for approval
   ELSE:
      - Status → Merged
      - Append to changelist history
      - Keep in Index

5. Approve Change (Review only)
   - Status → Merged
   - Move from Workspace to Index
   - Append to changelist history
   - Now part of permanent record
```

### Context Switching

```
Switch Operation:
1. Save current Local change
   - Build undo diff (ObjectDiffModel)
   - Status → Idle
   - Move to Workspace
   - Remove from Index

2. Load target change
   - Get from Workspace
   - Build apply diff (ObjectDiffModel)
   - Status → Local
   - Move to Index top

3. Return merged diff
   - First undo current
   - Then apply target
   - Caller updates MOO database accordingly
```

## Key Algorithms

### Object Resolution with History

When resolving an object's current state:

```
function resolve_object_current_state(object_name):
    name_map = {}  // Maps final_name → original_name
    
    // Walk changelist chronologically
    for change in changelist:
        for added in change.added_objects:
            name_map[added.name] = added.name
        
        for deleted in change.deleted_objects:
            // Find what name maps to deleted object
            original = find_original(name_map, deleted.name)
            if original:
                delete name_map[original]
        
        for renamed in change.renamed_objects:
            // Update mapping to track rename
            original = find_original(name_map, renamed.from)
            if original:
                name_map[original] = renamed.to
    
    // Check if object_name exists in final state
    if object_name in name_map or is_added_later(object_name):
        return get_ref(object_name)
    else:
        return None  // Object deleted or never existed
```

This algorithm handles:
- Objects added then renamed
- Objects renamed then deleted
- Multiple renames of the same object
- Objects deleted then recreated with same name

### SHA256 Deduplication

Objects are stored content-addressably:

```
function store_object(object_dump):
    sha256 = hash(object_dump)
    
    if exists(sha256):
        // Content already stored, just update ref
        return sha256
    
    // New content, store it
    objects_tree.put(sha256, object_dump)
    return sha256
```

Benefits:
- Identical objects share storage (space efficient)
- Natural deduplication across repository
- Content integrity verification
- Garbage collection of unreferenced content

### Meta Filtering

Objects can have ignored properties and verbs:

```
function apply_meta_filtering(object_def, object_name):
    meta = get_meta(object_name)
    if not meta:
        return object_def
    
    // Parse object definition
    obj = parse_objdef(object_def)
    
    // Filter properties
    obj.property_definitions = filter(
        p -> not in meta.ignored_properties
    )
    obj.property_overrides = filter(
        p -> not in meta.ignored_properties
    )
    
    // Filter verbs
    obj.verbs = filter(
        v -> none of v.names in meta.ignored_verbs
    )
    
    return serialize(obj)
```

Use cases:
- Exclude auto-generated properties (`.last_modified`)
- Ignore debug verbs
- Skip environment-specific settings

### Object Renaming (Critical: Refs Must Stay in Sync)

When renaming an object within a local change, refs must be updated to maintain the invariant that refs and tracking lists stay synchronized:

```
function rename_object_in_local_change(from_name, to_name):
    change = get_local_change()
    
    // Case 1: Object in added_objects or modified_objects
    if object_in_tracking_list(change, from_name):
        obj_version = get_version_from_tracking_list(change, from_name)
        
        // CRITICAL: Update refs
        sha256 = get_ref(from_name, obj_version)
        update_ref(to_name, obj_version, sha256)  // Create new ref
        delete_ref(from_name, obj_version)         // Delete old ref
        
        // Update tracking list name
        update_tracking_list_name(change, from_name, to_name)
        
        // Handle rename chaining (if A→B exists, update to A→to_name)
        if rename_exists_to(change, from_name):
            update_rename_chain(change, to_name)
        else if has_previous_versions(from_name):
            add_to_renamed_objects(change, from_name, to_name)
    
    // Case 2: Object exists in committed history only
    else:
        // Add to renamed_objects list for history tracking
        add_to_renamed_objects(change, from_name, to_name)
    
    // Case 3: Rename-back (A→B→A cancellation)
    if is_rename_back(change, from_name, to_name):
        remove_from_renamed_objects(change)
        update_refs_back(from_name, to_name)  // Same ref update as Case 1
```

**Critical Rules**:
1. When name changes in tracking list → refs MUST be updated
2. Within a local change, only ONE ref exists per object (current name)
3. Rename chains collapse: A→B→C stores only A→C
4. Rename-back cancels: A→B→A removes the rename entry entirely
5. Old refs are deleted immediately (no intermediate refs in local changes)

**Failure Mode**: If refs are not updated when names change in tracking lists, other operations (like `change/status`) will fail with "version N not found" errors when trying to resolve the object by its new name.

## Concurrency Model

### Single-Threaded Operations

- **Tokio runtime**: Async I/O for network operations
- **Database locks**: Fjall handles internal concurrency
- **Sequential changes**: Only one Local change at a time
- **User isolation**: Each user has independent context

### Thread Safety

All shared state is wrapped in `Arc`:
```rust
Arc<Database>
Arc<OperationRegistry>
Arc<dyn UserProvider>
```

Operations are `Send + Sync`, allowing them to be called from any thread.

## Storage Backend

### Fjall Embedded Database

- **Log-structured merge tree (LSM)**: Optimized for write-heavy workloads
- **LZ4 compression**: Reduces disk usage for text-heavy MOO objects
- **ACID transactions**: Ensures consistency
- **Embedded**: No separate database server needed

### Directory Structure

```
vcs-data/
├── objects/          - Content-addressed object storage
├── refs/             - Name → SHA256 mappings
├── index/            - Changelist and history
└── workspace/        - Saved changes
```

## Security Model

### User Authentication

- **Provider-based**: User information injected by UserProvider
- **Permission system**: Fine-grained capability checks
  - `SubmitChanges`: Can create and submit changes
  - `ApproveChanges`: Can approve submitted changes (privileged)
  - `Clone`: Can clone/export repository

### Operation Authorization

```rust
if !user.has_permission(&Permission::ApproveChanges) {
    return error("Insufficient permissions");
}
```

Each operation checks required permissions before executing.

## API Integration

### From MOOCode

```moo
// Simple call
result = worker_request("vcs", {"operation/name", arg1, arg2});

// With error handling
result = worker_request("vcs", {"object/update", obj_name, lines});
if (is_err(result))
    player:tell("Error: ", result[2]);
else
    player:tell("Success: ", result);
endif
```

### From HTTP/REST

```bash
curl -X POST http://localhost:9998/api/object/update \
  -H "Content-Type: application/json" \
  -d '{
    "operation": "object/update",
    "args": ["$player", ["obj $player", "parent #1", "..."]]
  }'
```

### Response Format

Both interfaces return consistent responses:
- **MOOCode**: Native MOO `Var` types (strings, lists, maps, errors)
- **HTTP**: JSON with `{"result": ..., "success": true, "operation": "..."}`

## Extension Points

### Adding New Operations

1. Create operation struct implementing `Operation` trait
2. Register in `create_default_registry()`
3. Routes and documentation auto-generated
4. Immediately available via RPC and HTTP

Example:
```rust
pub struct MyOperation {
    database: DatabaseRef,
}

impl Operation for MyOperation {
    fn name(&self) -> &'static str { "my/operation" }
    fn description(&self) -> &'static str { "Does something" }
    fn philosophy(&self) -> &'static str { "Why this exists" }
    fn parameters(&self) -> Vec<OperationParameter> { vec![...] }
    fn examples(&self) -> Vec<OperationExample> { vec![...] }
    fn routes(&self) -> Vec<OperationRoute> { vec![...] }
    fn execute(&self, args: Vec<String>, user: &User) -> Var {
        // Implementation
    }
}
```

### Custom Storage Backends

Providers are trait-based, allowing alternative implementations:
```rust
pub trait ObjectsProvider: Send + Sync {
    fn store(&self, sha256: &str, content: &str) -> Result<()>;
    fn get(&self, sha256: &str) -> Result<Option<String>>;
    // ...
}
```

Could implement providers backed by:
- PostgreSQL
- S3
- Git repositories
- Custom databases

## Performance Considerations

### Optimizations

1. **Content deduplication**: Identical objects stored once
2. **Lazy loading**: Objects loaded only when accessed
3. **Index caching**: Changelist state computed once per request
4. **Async I/O**: Non-blocking network operations
5. **Efficient serialization**: Binary formats where appropriate

### Scalability

Current architecture targets:
- **Single MOO instance**: Not horizontally scaled
- **Thousands of objects**: Content-addressable storage scales well
- **Hundreds of changes**: Changelist walks are O(n) but cached
- **Multiple concurrent users**: Thread-safe shared state

For larger deployments, consider:
- PostgreSQL backend for refs and index
- Object storage (S3) for content
- Redis for caching compiled state

## Testing Strategy

### Unit Tests

Each operation has isolated tests:
```rust
#[tokio::test]
async fn test_object_update() {
    let (registry, _db) = create_test_registry();
    // Test operation execution
}
```

### Integration Tests

Full workflow tests:
```rust
#[tokio::test]
async fn test_complete_change_workflow() {
    // Create change
    // Modify objects
    // Check status
    // Submit
    // Verify merged
}
```

### Test Fixtures

- **Temporary databases**: Each test gets clean state
- **Mock users**: Configurable permissions
- **Sample objects**: Realistic MOO object definitions

## Future Enhancements

### Potential Improvements

1. **Branching**: Support for long-lived development branches
2. **Merge conflict resolution**: Tools for handling concurrent modifications
3. **Diff visualization**: Rich text/HTML diff output
4. **Change dependencies**: Express that one change requires another
5. **Webhooks**: Notifications on change events
6. **Git bridge**: Bidirectional sync with Git repositories
7. **Web UI**: Rich browser-based interface beyond Swagger
8. **Change review**: Comments, annotations, approval workflows

### Migration Path

The architecture supports gradual enhancement:
- New operations added without breaking existing ones
- Storage backend swappable via provider traits
- API remains stable even with internal changes
- Backward-compatible extensions to data formats

## Conclusion

The VCS Worker architecture balances:
- **Simplicity**: Clear separation of concerns, straightforward data flow
- **Extensibility**: Trait-based design enables customization
- **MOO integration**: First-class support for MOO-specific needs
- **Modern practices**: Leverages Rust, async I/O, OpenAPI

The changelist model provides familiar Git-like workflows while accommodating MOO's unique requirements. The RPC architecture simplifies both implementation and usage, making version control accessible from within the MOO environment itself.

