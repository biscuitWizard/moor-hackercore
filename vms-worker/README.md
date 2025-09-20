# VMS Worker - Git-based Version Control for MOO Objects

## Overview

The VMS Worker is a Rust-based wrapper around git that provides version control capabilities for MOO object files. It manages MOO object dumps, applies filtering based on meta configurations, and maintains a git repository for source control.

## Architecture
+
### Key Features

- **Object Dump Processing**: Accepts MOO object dumps and parses them into structured data using moor's proven infrastructure
- **Meta File Management**: Automatically creates and manages `.meta` files for configuration
- **Property/Verb Filtering**: Strips out ignored properties and verbs based on meta configuration
- **Git Integration**: Full git repository management with commit capabilities in `/game` directory
- **Automatic Initialization**: Automatically initializes git repository at `/game` on startup

## Usage

### Operations

1. **Add/Update Object**
   ```rust
   VcsOperation::AddOrUpdateObject { 
       object_dump: String, 
       filename: String,
   }
   ```

2. **Delete Object**
   ```rust
   VcsOperation::DeleteObject { 
       filename: String,
   }
   ```

3. **Create Commit**
   ```rust
   VcsOperation::Commit { 
       message: String,
       author_name: String,
       author_email: String,
   }
   ```

4. **Get Status**
   ```rust
   VcsOperation::Status
   ```

### Meta Configuration Format

```yaml
# .meta file example
ignored_properties:
  - "temp_property"
  - "debug_flag"

ignored_verbs:
  - "debug_verb"
  - "test_verb"

metadata:
  description: "Core system object"
  version: "1.0"
```

## Implementation Details

### MOO Object Processing

The system uses moor's proven ObjectDefinition infrastructure to handle MOO object dumps:

```
object #1
  name: "Test Object"
  parent: #0
  location: #-1
  owner: #2
  readable: true
  
  property "test_prop" (owner: #2, flags: "r") = "test_value";

  verb "test_verb" (this none this) owner: #2 flags: "rxd"
    return "test";
  endverb

endobject
```

This leverages the same parsing and compilation infrastructure used by the moor server itself.

### Filtering Process

1. Parse the MOO object dump using moor's ObjectDefinition
2. Load corresponding `.meta` file from `/game` directory (create default if missing)
3. Apply filtering to remove ignored properties and verbs
4. Write filtered object to `/game` directory
5. Add to git repository
6. Create commit if requested

All operations work within the `/game` directory which is mounted in the Docker container.

### Git Integration

- Uses `git2` crate for git operations
- Automatically initializes git repository at `/game` on startup
- Maintains working directory structure in `/game`
- Handles file staging and commits
- Provides status information
- Works with Docker volume mount at `/game`

## File Structure

```
vms-worker/
├── src/
│   ├── lib.rs              # Library exports
│   ├── main.rs             # CLI/RPC interface
│   ├── meta_config.rs      # Meta configuration management
│   ├── git_ops.rs          # Git operations wrapper
│   └── vcs_operations.rs   # High-level VCS operations with objdef integration
├── Cargo.toml              # Dependencies and configuration
└── README.md               # This file
```

## Dependencies

- `moor-objdef`: MOO object definition parsing and dumping
- `moor-compiler`: MOO compilation infrastructure
- `moor-common`: Common moor types and utilities
- `moor-var`: MOO variable types
- `git2`: Git operations
- `serde` + `serde_yaml`: YAML configuration
- `clap`: Command-line interface

## Integration

The VMS Worker integrates with the existing MOO RPC system:

- Registers as a worker with type "vms-worker"
- Accepts RPC requests with operation parameters
- Returns operation results as RPC responses
- Maintains persistent git repository state

## Future Enhancements

- Branch management
- Merge conflict resolution
- Object dependency tracking
- Automated testing integration
- Web interface for repository browsing
