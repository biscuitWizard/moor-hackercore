# VCS Worker Architecture

## Overview

The VCS Worker is a Rust-based service that provides version control functionality for MOO (MUD Object Oriented) game objects. It acts as a bridge between the MOO game engine and Git, enabling collaborative development and version control of game objects.

## Project Structure

```
vcs-worker/
├── src/
│   ├── main.rs              # Entry point and RPC worker loop
│   ├── lib.rs               # Library exports and public API
│   ├── config.rs            # Configuration management
│   ├── meta_config.rs       # MOO object metadata configuration
│   ├── utils.rs             # Utility functions for path operations
│   ├── error_utils.rs       # Common error handling utilities
│   ├── arg_validation.rs    # Argument validation utilities
│   ├── git/                 # Git operations and repository management
│   │   ├── mod.rs           # Git module exports
│   │   ├── repository.rs    # GitRepository wrapper
│   │   ├── channel.rs       # Git channel operations
│   │   ├── utils.rs         # Git utility functions
│   │   ├── operations.rs    # Git operations module exports
│   │   └── operations/      # Individual git operation modules
│   │       ├── init_ops.rs      # Repository initialization
│   │       ├── commit_ops.rs    # Commit operations
│   │       ├── file_ops.rs      # File operations
│   │       ├── remote_ops.rs    # Remote repository operations
│   │       ├── status_ops.rs    # Repository status operations
│   │       └── pull_ops.rs      # Pull operations
│   └── vcs/                 # VCS business logic
│       ├── mod.rs           # VCS module exports
│       ├── types.rs         # VCS operation types and data structures
│       ├── processor.rs     # Main VCS operation processor
│       ├── object_handler.rs    # MOO object file handling
│       ├── status_handler.rs    # Repository status handling
│       ├── meta_handler.rs      # Metadata handling
│       └── workflow_handler.rs  # Complex workflow orchestration
├── Cargo.toml               # Rust project configuration
├── ENVIRONMENT.md           # Environment setup documentation
├── PROTOCOL.md             # RPC protocol documentation
└── ARCHITECTURE.md         # This file
```

## Architecture Principles

### 1. Separation of Concerns

The architecture follows a clear separation of concerns:

- **Git Operations** (`git/operations/`): Low-level git operations
- **VCS Handlers** (`vcs/`): Business logic for MOO object management
- **Processor** (`vcs/processor.rs`): High-level operation routing
- **Configuration** (`config.rs`, `meta_config.rs`): Configuration management

### 2. Modular Design

Each git operation is encapsulated in its own module:
- `init_ops.rs`: Repository initialization and cloning
- `commit_ops.rs`: Commit creation, history, and rollback
- `file_ops.rs`: File add, remove, rename operations
- `remote_ops.rs`: Push, pull, fetch operations
- `status_ops.rs`: Repository status and reset operations
- `pull_ops.rs`: Pull workflow orchestration

### 3. Handler Pattern

VCS operations are handled by specialized handlers:
- `ObjectHandler`: MOO object file operations
- `StatusHandler`: Repository status reporting
- `MetaHandler`: Object metadata management
- `WorkflowHandler`: Complex multi-step operations

## Core Components

### 1. VcsProcessor

The main entry point for VCS operations. It:
- Routes operations to appropriate handlers
- Manages repository initialization
- Handles SSH key and git user configuration
- Provides high-level operation coordination

**Key Methods:**
- `process_operation()`: Main operation dispatcher
- `initialize_repository()`: Repository setup

### 2. Git Operations Layer

Low-level git operations organized by functionality:

#### InitOps
- `initialize_repository()`: Initialize or clone repositories
- `clone_repository()`: Clone from remote URL
- `chown_repository_to_current_user()`: Fix permissions

#### CommitOps
- `create_commit()`: Create commits
- `create_commit_with_push()`: Commit with push workflow
- `get_commits()`: Retrieve commit history
- `rollback_last_commit()`: Undo last commit

#### FileOps
- `add_file()`: Add files to git index
- `remove_file()`: Remove files from git
- `rename_file()`: Rename files in git
- `write_file()`: Write file content

#### RemoteOps
- `push()`: Push commits to remote
- `fetch_remote()`: Fetch from remote
- `test_ssh_connection()`: Test SSH connectivity

#### StatusOps
- `get_status()`: Get repository status
- `reset_working_tree()`: Reset working directory
- `reset_working_tree_with_verification()`: Reset with status reporting

#### PullOps
- `pull_with_rebase()`: Pull with rebase strategy
- `pull_dry_run()`: Analyze pull impact without executing

### 3. VCS Handlers

#### ObjectHandler
Manages MOO object files:
- Parse and serialize MOO object definitions
- Handle object add, update, delete, rename operations
- Apply metadata filtering
- Convert between MOO and git formats

#### StatusHandler
Provides repository status information:
- Format status for MOO consumption
- Handle status reporting
- Manage repository state queries

#### MetaHandler
Manages object metadata:
- Handle ignored properties and verbs
- Manage `.meta` configuration files
- Apply metadata filtering to objects

#### WorkflowHandler
Orchestrates complex operations:
- Pull with detailed change analysis
- Commit workflows with pull-before-commit
- Rebase with automatic conflict resolution
- Object change analysis and reporting

### 4. Configuration Management

#### Config
Main application configuration:
- Repository paths and URLs
- Git user settings
- SSH key configuration
- Debug settings

#### MetaConfig
Per-object metadata configuration:
- Ignored properties and verbs
- Object-specific metadata
- Stored in `.meta` files alongside MOO objects

### 5. Utility Modules

#### ErrorUtils
Common error handling patterns:
- Standardized error messages
- Success message formatting
- Consistent error types

#### ArgValidation
Argument validation utilities:
- Parameter count validation
- Type checking and extraction
- Default value handling

## Data Flow

### 1. Operation Processing Flow

```
RPC Request → VcsProcessor → Handler → Git Operations → Git Repository
     ↓              ↓           ↓            ↓              ↓
MOO Format    Operation    Business      Low-level      File System
             Routing       Logic         Git Ops
```

### 2. Object Management Flow

```
MOO Object → ObjectHandler → Parse → Apply Meta → Git Operations → Repository
     ↓            ↓           ↓         ↓            ↓              ↓
Object Dump   Validation   Structure  Filtering    File Ops    .moo Files
```

### 3. Workflow Operations

```
Workflow Request → WorkflowHandler → Multiple Git Ops → Analysis → Results
       ↓               ↓                ↓              ↓          ↓
   Pull/Commit    Orchestration    Execute Steps   Change      MOO Format
                                    in Sequence    Analysis
```

## Key Features

### 1. MOO Object Integration
- Seamless conversion between MOO object definitions and git files
- Metadata filtering for version control
- Object-specific configuration via `.meta` files

### 2. Git Workflow Support
- Pull-before-commit strategy
- Automatic conflict resolution
- Rebase-based pull operations
- Rollback capabilities

### 3. SSH Authentication
- SSH key management
- Connection testing
- Secure remote operations

### 4. Error Handling
- Comprehensive error reporting
- Graceful degradation
- Detailed logging and tracing
- Standardized error utilities

### 5. RPC Integration
- Asynchronous RPC worker
- MOO-compatible data types
- Request/response handling
- Argument validation utilities

### 6. Code Quality
- Reduced code duplication
- Consistent error handling
- Standardized validation patterns
- Maintainable architecture

## Dependencies

### Core Dependencies
- `git2`: Git operations
- `tokio`: Async runtime
- `tracing`: Logging and tracing
- `serde`: Serialization
- `clap`: Command line parsing

### MOO Integration
- `moor-common`: Common MOO types and utilities
- `moor-var`: MOO variable types
- `moor-compiler`: MOO object definition parsing
- `moor-objdef`: Object definition handling

### RPC Framework
- `rpc-async-client`: RPC client implementation
- `rpc-common`: RPC utilities
- `tmq`: ZeroMQ messaging

## Configuration

### Environment Variables
- `VCS_REPOSITORY_PATH`: Repository location (default: `/game`)
- `VCS_REPOSITORY_URL`: Remote repository URL
- `VCS_OBJECTS_DIRECTORY`: Objects subdirectory (default: `objects`)
- `VCS_DEBUG`: Enable debug logging
- `VCS_GIT_USER_NAME`: Git user name
- `VCS_GIT_USER_EMAIL`: Git user email
- `VCS_SSH_KEY_PATH`: SSH private key path

### Meta Configuration
Each MOO object can have a `.meta` file specifying:
- Properties to ignore in version control
- Verbs to ignore in version control
- Additional metadata

## Security Considerations

### 1. SSH Key Management
- SSH keys stored with restrictive permissions (600)
- Key validation and testing
- Secure key handling

### 2. File System Security
- Repository ownership management
- Permission validation
- Path traversal protection

### 3. Input Validation
- MOO object validation
- Path sanitization
- Operation parameter validation

## Performance Considerations

### 1. Async Operations
- Non-blocking git operations
- Concurrent request handling
- Efficient I/O operations

### 2. Caching
- Repository state caching
- Object definition caching
- Metadata caching

### 3. Memory Management
- Efficient string handling
- Minimal object copying
- Resource cleanup

## Testing Strategy

### 1. Unit Tests
- Individual operation testing
- Handler testing
- Utility function testing

### 2. Integration Tests
- End-to-end workflow testing
- Git repository testing
- RPC integration testing

### 3. Error Testing
- Failure scenario testing
- Edge case handling
- Recovery testing

## Future Enhancements

### 1. Performance Improvements
- Parallel operation execution
- Advanced caching strategies
- Optimized git operations

### 2. Feature Additions
- Branch management
- Merge conflict resolution
- Advanced workflow support

### 3. Monitoring and Observability
- Metrics collection
- Performance monitoring
- Health checks

## Development Guidelines

### 1. Code Organization
- Follow existing module structure
- Maintain separation of concerns
- Use appropriate abstraction levels
- Leverage utility modules for common patterns

### 2. Error Handling
- Use `ErrorUtils` for standardized error messages
- Implement proper error propagation
- Log errors appropriately
- Maintain consistent error types

### 3. Argument Validation
- Use `ArgValidation` for parameter checking
- Validate argument counts and types
- Provide clear error messages for invalid arguments

### 4. Testing
- Write comprehensive tests
- Test error scenarios
- Maintain test coverage

### 5. Documentation
- Document public APIs
- Update architecture documentation
- Maintain code comments

### 6. Code Deduplication
- Identify and extract common patterns
- Use utility modules for repeated code
- Maintain consistency across modules

This architecture provides a robust, scalable foundation for version control of MOO game objects while maintaining clear separation of concerns, modular design principles, and reduced code duplication.
