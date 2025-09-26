# VMS Worker Protocol

This document describes the protocol for communicating with the vms-worker service. All operations are performed using the `worker_request` function with the worker type "vms".

## Object Management Operations

### update_object
Add or update a MOO object file in the repository.

```lisp
worker_request("vms", {"update_object", "object_name", {"line1", "line2", "line3"}})
```

**Parameters:**
- `object_name` (string): Name of the object file (e.g., "player.moo")
- `object_dump` (list of strings): The complete object dump content as a list of lines

### delete_object
Remove a tracked MOO object file from the repository.

```lisp
worker_request("vms", {"delete_object", "object_name"})
```

**Parameters:**
- `object_name` (string): Name of the object file to delete

**Example:**
```lisp
worker_request("vms", {"delete_object", "old_object"})
```

### rename_object
Rename a tracked MOO object file.

```lisp
worker_request("vms", {"rename_object", "old_name", "new_name"})
```

**Parameters:**
- `old_name` (string): Current name of the object file
- `new_name` (string): New name for the object file

**Example:**
```lisp
worker_request("vms", {"rename_object", "player.moo", "character.moo"})
```

## Repository Operations

### commit
Create a commit with current changes.

```lisp
worker_request("vms", {"commit", "commit_message", "author_name", "author_email"})
```

**Parameters:**
- `commit_message` (string): Commit message (required)
- `author_name` (string): Author name (optional, defaults to "vms-worker")
- `author_email` (string): Author email (optional, defaults to "vms-worker@system")

**Example:**
```lisp
worker_request("vms", {"commit", "Added new player object", "John Doe", "john@example.com"})
```

### status
Get repository status information.

```lisp
worker_request("vms", {"status"})
```

**Returns:** Repository status including upstream info, last commit, current changes, and branch name.

### list_objects
List all .moo objects with dependency ordering.

```lisp
worker_request("vms", {"list_objects"})
```

**Returns:** List of all tracked MOO object files in dependency order.

### get_objects
Get full dump contents for specified object names.

```lisp
worker_request("vms", {"get_objects", "object1", "object2", "object3"})
```

**Parameters:**
- Variable number of object names (strings)

**Example:**
```lisp
worker_request("vms", {"get_objects", "player.moo", "room.moo"})
```

### get_commits
Get paginated list of commits.

```lisp
worker_request("vms", {"get_commits", limit, offset})
```

**Parameters:**
- `limit` (integer, optional): Maximum number of commits to return
- `offset` (integer, optional): Number of commits to skip

**Example:**
```lisp
worker_request("vms", {"get_commits", 10, 0})
```

## Credential Management Operations

### set_ssh_key
Set SSH key for repository access.

```lisp
worker_request("vms", {"set_ssh_key", "key_content", "key_name"})
```

**Parameters:**
- `key_content` (string): SSH private key content
- `key_name` (string): Name/identifier for the key

**Example:**
```lisp
worker_request("vms", {"set_ssh_key", "-----BEGIN OPENSSH PRIVATE KEY-----\n...", "deploy_key"})
```

### clear_ssh_key
Remove the current SSH key.

```lisp
worker_request("vms", {"clear_ssh_key"})
```

### set_git_user
Set Git user information for commits.

```lisp
worker_request("vms", {"set_git_user", "name", "email"})
```

**Parameters:**
- `name` (string): Git user name
- `email` (string): Git user email

**Example:**
```lisp
worker_request("vms", {"set_git_user", "John Doe", "john@example.com"})
```

### test_ssh
Test SSH connection to the repository.

```lisp
worker_request("vms", {"test_ssh"})
```

## Meta File Operations

### update_ignored_properties
Update the list of ignored properties for an object.

```lisp
worker_request("vms", {"update_ignored_properties", "object_name", "prop1", "prop2", "prop3"})
```

**Parameters:**
- `object_name` (string): Name of the object file
- Variable number of property names (strings) to ignore

**Example:**
```lisp
worker_request("vms", {"update_ignored_properties", "player.moo", "last_login", "session_id"})
```

### update_ignored_verbs
Update the list of ignored verbs for an object.

```lisp
worker_request("vms", {"update_ignored_verbs", "object_name", "verb1", "verb2", "verb3"})
```

**Parameters:**
- `object_name` (string): Name of the object file
- Variable number of verb names (strings) to ignore

**Example:**
```lisp
worker_request("vms", {"update_ignored_verbs", "player.moo", "login", "logout", "save"})
```

## Error Handling

All operations return either:
- Success: A list of result variables
- Error: A `WorkerError` with descriptive message

Common error conditions:
- Missing required arguments
- Invalid argument types
- Unknown operation names
- Repository operation failures

## Notes

- All string arguments are case-sensitive
- Object names should include the `.moo` extension
- The worker processes operations sequentially
- SSH keys and Git user settings persist across operations
- Object dumps should be provided as lists of strings, not single concatenated strings
