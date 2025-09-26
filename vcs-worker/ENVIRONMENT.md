# VCS Worker Environment Variables

This document describes all environment variables that can be used to configure the vcs-worker service.

## VCS-Specific Configuration

### VCS_REPOSITORY_URL
**Type:** String (optional)  
**Default:** None  
**Description:** URL to clone the repository from instead of initializing an empty one. If not set, will initialize an empty repository.

**Example:**
```bash
export VCS_REPOSITORY_URL="https://github.com/example/moo-repo.git"
```

### VCS_REPOSITORY_PATH
**Type:** String (optional)  
**Default:** `/game`  
**Description:** Path where the git repository should be located.

**Example:**
```bash
export VCS_REPOSITORY_PATH="/custom/game/path"
```

### VCS_OBJECTS_DIRECTORY
**Type:** String (optional)  
**Default:** `objects`  
**Description:** Subdirectory within the repository where MOO and meta files should be stored.

**Example:**
```bash
export VCS_OBJECTS_DIRECTORY="moo_objects"
```

### VCS_DEBUG
**Type:** Boolean (optional)  
**Default:** `false`  
**Description:** Whether to enable debug logging. Accepts `true`, `1`, `false`, or `0`.

**Example:**
```bash
export VCS_DEBUG="true"
```

### VCS_GIT_USER_NAME
**Type:** String (optional)  
**Default:** `vcs-worker`  
**Description:** Git user name for commits.

**Example:**
```bash
export VCS_GIT_USER_NAME="John Doe"
```

### VCS_GIT_USER_EMAIL
**Type:** String (optional)  
**Default:** `vcs-worker@system`  
**Description:** Git user email for commits.

**Example:**
```bash
export VCS_GIT_USER_EMAIL="john@example.com"
```

### VCS_SSH_KEY_PATH
**Type:** String (optional)  
**Default:** None  
**Description:** Path to SSH private key for git authentication. If not set, will use default SSH key discovery.

**Example:**
```bash
export VCS_SSH_KEY_PATH="/path/to/private/key"
```

## RPC Configuration (Command Line Arguments)

The following are command line arguments that can also be set via environment variables using the `--` prefix convention:

### --rpc-address
**Type:** String  
**Default:** `ipc:///tmp/moor_rpc.sock`  
**Description:** RPC ZMQ req-reply socket address.

### --events-address
**Type:** String  
**Default:** `ipc:///tmp/moor_events.sock`  
**Description:** Events ZMQ pub-sub address.

### --public-key
**Type:** Path  
**Default:** `moor-verifying-key.pem`  
**Description:** File containing the PEM encoded public key (shared with the daemon), used for authenticating client & host connections.

### --private-key
**Type:** Path  
**Default:** `moor-signing-key.pem`  
**Description:** File containing an openssh generated ed25519 format private key (shared with the daemon), used for authenticating client & host connections.

### --workers-dispatch-address
**Type:** String  
**Default:** `ipc:///tmp/moor_workers_response.sock`  
**Description:** Workers server ZMQ pub-sub address for receiving dispatch requests.

### --workers-address
**Type:** String  
**Default:** `ipc:///tmp/moor_workers_request.sock`  
**Description:** Workers server ZMQ RPC for sending dispatch responses.

## Example Configuration

### Development Environment
```bash
export VCS_REPOSITORY_URL="https://github.com/your-org/moo-game.git"
export VCS_REPOSITORY_PATH="/home/dev/moo-game"
export VCS_OBJECTS_DIRECTORY="objects"
export VCS_DEBUG="true"
export VCS_GIT_USER_NAME="Developer"
export VCS_GIT_USER_EMAIL="dev@example.com"
export VCS_SSH_KEY_PATH="/home/dev/.ssh/id_ed25519"
```

### Production Environment
```bash
export VCS_REPOSITORY_URL="git@github.com:your-org/moo-game.git"
export VCS_REPOSITORY_PATH="/game"
export VCS_OBJECTS_DIRECTORY="objects"
export VCS_DEBUG="false"
export VCS_GIT_USER_NAME="VCS Worker"
export VCS_GIT_USER_EMAIL="vcs@yourdomain.com"
export VCS_SSH_KEY_PATH="/etc/vcs/keys/deploy_key"
```

### Docker Environment
```bash
export VCS_REPOSITORY_URL="https://github.com/your-org/moo-game.git"
export VCS_REPOSITORY_PATH="/app/game"
export VCS_OBJECTS_DIRECTORY="objects"
export VCS_DEBUG="false"
export VCS_GIT_USER_NAME="Docker VCS"
export VCS_GIT_USER_EMAIL="docker@yourdomain.com"
```

## Security Considerations

### SSH Key Permissions
- SSH keys should have permissions `600` or more restrictive
- The vcs-worker will validate key permissions and reject overly permissive keys
- Use dedicated deployment keys rather than personal SSH keys

### Repository Access
- Use HTTPS URLs for public repositories
- Use SSH URLs with dedicated keys for private repositories
- Consider using deploy tokens for GitHub/GitLab private repositories

### File System Permissions
- Ensure the repository path is writable by the vcs-worker process
- The objects directory will be created automatically if it doesn't exist
- SSH key files should be readable only by the vcs-worker process

## Validation and Defaults

- Empty environment variables will trigger warnings and use default values
- Invalid boolean values for `VCS_DEBUG` will default to `false`
- Non-existent SSH key paths will cause the worker to use default SSH key discovery
- Repository paths that don't exist will be created automatically

## Logging

The vcs-worker logs configuration information at startup:
- Info level: Successful configuration from environment variables
- Warning level: Empty environment variables (using defaults)
- Error level: Configuration validation failures

Enable debug logging with `VCS_DEBUG=true` to see detailed configuration information.
