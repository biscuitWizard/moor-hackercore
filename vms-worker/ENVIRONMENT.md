# VMS Worker Environment Variables

This document describes all environment variables that can be used to configure the vms-worker service.

## VMS-Specific Configuration

### VMS_REPOSITORY_URL
**Type:** String (optional)  
**Default:** None  
**Description:** URL to clone the repository from instead of initializing an empty one. If not set, will initialize an empty repository.

**Example:**
```bash
export VMS_REPOSITORY_URL="https://github.com/example/moo-repo.git"
```

### VMS_REPOSITORY_PATH
**Type:** String (optional)  
**Default:** `/game`  
**Description:** Path where the git repository should be located.

**Example:**
```bash
export VMS_REPOSITORY_PATH="/custom/game/path"
```

### VMS_OBJECTS_DIRECTORY
**Type:** String (optional)  
**Default:** `objects`  
**Description:** Subdirectory within the repository where MOO and meta files should be stored.

**Example:**
```bash
export VMS_OBJECTS_DIRECTORY="moo_objects"
```

### VMS_DEBUG
**Type:** Boolean (optional)  
**Default:** `false`  
**Description:** Whether to enable debug logging. Accepts `true`, `1`, `false`, or `0`.

**Example:**
```bash
export VMS_DEBUG="true"
```

### VMS_GIT_USER_NAME
**Type:** String (optional)  
**Default:** `vms-worker`  
**Description:** Git user name for commits.

**Example:**
```bash
export VMS_GIT_USER_NAME="John Doe"
```

### VMS_GIT_USER_EMAIL
**Type:** String (optional)  
**Default:** `vms-worker@system`  
**Description:** Git user email for commits.

**Example:**
```bash
export VMS_GIT_USER_EMAIL="john@example.com"
```

### VMS_SSH_KEY_PATH
**Type:** String (optional)  
**Default:** None  
**Description:** Path to SSH private key for git authentication. If not set, will use default SSH key discovery.

**Example:**
```bash
export VMS_SSH_KEY_PATH="/path/to/private/key"
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
export VMS_REPOSITORY_URL="https://github.com/your-org/moo-game.git"
export VMS_REPOSITORY_PATH="/home/dev/moo-game"
export VMS_OBJECTS_DIRECTORY="objects"
export VMS_DEBUG="true"
export VMS_GIT_USER_NAME="Developer"
export VMS_GIT_USER_EMAIL="dev@example.com"
export VMS_SSH_KEY_PATH="/home/dev/.ssh/id_ed25519"
```

### Production Environment
```bash
export VMS_REPOSITORY_URL="git@github.com:your-org/moo-game.git"
export VMS_REPOSITORY_PATH="/game"
export VMS_OBJECTS_DIRECTORY="objects"
export VMS_DEBUG="false"
export VMS_GIT_USER_NAME="VMS Worker"
export VMS_GIT_USER_EMAIL="vms@yourdomain.com"
export VMS_SSH_KEY_PATH="/etc/vms/keys/deploy_key"
```

### Docker Environment
```bash
export VMS_REPOSITORY_URL="https://github.com/your-org/moo-game.git"
export VMS_REPOSITORY_PATH="/app/game"
export VMS_OBJECTS_DIRECTORY="objects"
export VMS_DEBUG="false"
export VMS_GIT_USER_NAME="Docker VMS"
export VMS_GIT_USER_EMAIL="docker@yourdomain.com"
```

## Security Considerations

### SSH Key Permissions
- SSH keys should have permissions `600` or more restrictive
- The vms-worker will validate key permissions and reject overly permissive keys
- Use dedicated deployment keys rather than personal SSH keys

### Repository Access
- Use HTTPS URLs for public repositories
- Use SSH URLs with dedicated keys for private repositories
- Consider using deploy tokens for GitHub/GitLab private repositories

### File System Permissions
- Ensure the repository path is writable by the vms-worker process
- The objects directory will be created automatically if it doesn't exist
- SSH key files should be readable only by the vms-worker process

## Validation and Defaults

- Empty environment variables will trigger warnings and use default values
- Invalid boolean values for `VMS_DEBUG` will default to `false`
- Non-existent SSH key paths will cause the worker to use default SSH key discovery
- Repository paths that don't exist will be created automatically

## Logging

The vms-worker logs configuration information at startup:
- Info level: Successful configuration from environment variables
- Warning level: Empty environment variables (using defaults)
- Error level: Configuration validation failures

Enable debug logging with `VMS_DEBUG=true` to see detailed configuration information.
