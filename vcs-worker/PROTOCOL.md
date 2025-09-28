# VCS Worker Protocol

This document describes the protocol for communicating with the vcs-worker service. All operations are performed using the `worker_request` function with the worker type "vcs".

## Object Management Operations

### update_object
Add or update a MOO object file in the repository.

```lisp
worker_request("vcs", {"update_object", "object_name", {"line1", "line2", "line3"}})
```

**Parameters:**
- `object_name` (string): Name of the object file (e.g., "player.moo")
- `object_dump` (list of strings): The complete object dump content as a list of lines

### delete_object
Remove a tracked MOO object file from the repository.

```lisp
worker_request("vcs", {"delete_object", "object_name"})
```

**Parameters:**
- `object_name` (string): Name of the object file to delete

**Example:**
```lisp
worker_request("vcs", {"delete_object", "old_object"})
```

### rename_object
Rename a tracked MOO object file.

```lisp
worker_request("vcs", {"rename_object", "old_name", "new_name"})
```

**Parameters:**
- `old_name` (string): Current name of the object file
- `new_name` (string): New name for the object file

**Example:**
```lisp
worker_request("vcs", {"rename_object", "player.moo", "character.moo"})
```

## Repository Operations

### commit
Create a commit with current changes. Automatically pulls remote changes before committing to avoid conflicts.

```lisp
worker_request("vcs", {"commit", "commit_message", "author_name", "author_email"})
```

**Parameters:**
- `commit_message` (string): Commit message (required)
- `author_name` (string): Author name (optional, defaults to "vcs-worker")
- `author_email` (string): Author email (optional, defaults to "vcs-worker@system")

**Example:**
```lisp
worker_request("vcs", {"commit", "Added new player object", "John Doe", "john@example.com"})
```

**Note:** The commit operation automatically performs a pull before committing to ensure the local repository is up to date and to avoid conflicts when pushing.

### status
Get repository status information.

```lisp
worker_request("vcs", {"status"})
```

**Returns:** Repository status including upstream info, last commit, current changes, and branch name.

### list_objects
List all .moo objects with dependency ordering.

```lisp
worker_request("vcs", {"list_objects"})
```

**Returns:** List of all tracked MOO object files in dependency order.

### get_objects
Get full dump contents for specified object names.

```lisp
worker_request("vcs", {"get_objects", "object1", "object2", "object3"})
```

**Parameters:**
- Variable number of object names (strings)

**Example:**
```lisp
worker_request("vcs", {"get_objects", "player.moo", "room.moo"})
```

### get_commits
Get paginated list of commits.

```lisp
worker_request("vcs", {"get_commits", limit, offset})
```

**Parameters:**
- `limit` (integer, optional): Maximum number of commits to return
- `offset` (integer, optional): Number of commits to skip

**Example:**
```lisp
worker_request("vcs", {"get_commits", 10, 0})
```

### pull
Pull remote changes with rebase strategy and automatic conflict resolution.

```lisp
worker_request("vcs", {"pull", dry_run})
```

**Parameters:**
- `dry_run` (boolean, optional): If true, returns analysis of what would be modified without making changes

**Returns:**
- **List of commit results** where each element represents changes from a single commit, ordered from oldest to newest
- Each commit result contains:
  - `commit_author`: Author name of the commit
  - `commit_id`: Short commit hash (8 characters)
  - `commit_message`: Commit message
  - `modified_objects`: List of object IDs that were modified
  - `deleted_objects`: List of object IDs that were deleted
  - `added_objects`: List of object IDs that were added
  - `renamed_objects`: List of object IDs that were renamed
  - `changes`: Array of detailed object changes, where each change contains:
    - `obj_id`: Object ID
    - `modified_verbs`: Map of verb names to verb changes
    - `deleted_verbs`: Map of verb names to deleted verbs
    - `modified_props`: Map of property names to property changes
    - `deleted_props`: Map of property names to deleted properties

**Example:**
```lisp
worker_request("vcs", {"pull", false})
worker_request("vcs", {"pull", true})
```

**Example Return Format:**
```lisp
{
  {
    "commit_author" -> "vcs-worker",
    "commit_id" -> "e38ae7f7", 
    "commit_message" -> "Add $seq_utils:levenshtein",
    "modified_objects" -> {#42, #43},
    "deleted_objects" -> {},
    "added_objects" -> {#44},
    "renamed_objects" -> {},
    "changes" -> {
      {
        "obj_id" -> #42,
        "modified_verbs" -> {"verb_name" -> "verb_changes"},
        "deleted_verbs" -> {},
        "modified_props" -> {"prop_name" -> "prop_changes"},
        "deleted_props" -> {}
      },
      {
        "obj_id" -> #43,
        "modified_verbs" -> {},
        "deleted_verbs" -> {},
        "modified_props" -> {},
        "deleted_props" -> {}
      },
      {
        "obj_id" -> #44,
        "modified_verbs" -> {},
        "deleted_verbs" -> {},
        "modified_props" -> {},
        "deleted_props" -> {}
      }
    }
  },
  // ... additional commits in chronological order
}
```

**Pull Strategy:**
The pull operation uses a rebase strategy with automatic conflict resolution:

1. **Fetch**: Downloads latest changes from remote repository
2. **Analysis**: Determines which commits need to be pulled
3. **Replay**: For each commit to be pulled:
   - Loads the complete object dump (.moo file) from the commit
   - Parses the object using the existing object handler
   - Applies current meta configuration filtering
   - Overwrites the local object with the filtered version
4. **Conflict Resolution**: Automatically resolves conflicts by:
   - Loading complete object dumps from remote commits
   - Overwriting local changes with remote changes
   - Detecting and handling deletions and renames
5. **Rebase**: Applies the replayed commits on top of the current branch

This strategy ensures that conflicts are automatically resolved by always taking the remote version of object dumps, which is appropriate for MOO object files where the complete dump represents the authoritative state.

### reset
Reset the working tree to HEAD, discarding all uncommitted changes.

```lisp
worker_request("vcs", {"reset"})
```

**Returns:** Success message confirming that all changes have been discarded.

**Example:**
```lisp
worker_request("vcs", {"reset"})
```

**Warning:** This operation permanently discards all uncommitted changes in the working tree. Use with caution.

### stash
Stash current uncommitted changes using ObjDef models in memory.

```lisp
worker_request("vcs", {"stash"})
```

**Returns:** Success message confirming that changes have been stashed in memory.

**Example:**
```lisp
worker_request("vcs", {"stash"})
```

**Note:** This operation:
- Uses git status to identify all changed `.moo` files (including deleted ones)
- Loads current changes into ObjDef models in memory with operation types
- Preserves original filenames to avoid object name vs filename mismatches
- Stores operation types: `Modified`, `Deleted`, `Renamed` for proper replay
- **Rename Detection**: Compares first lines of added files with deleted files from git history
- Does not use Git's built-in stash system

### replay_stash
Replay previously stashed changes back to the working tree.

```lisp
worker_request("vcs", {"replay_stash"})
```

**Returns:** Success message confirming that stashed changes have been replayed.

**Example:**
```lisp
worker_request("vcs", {"replay_stash"})
```

**Note:** This operation:
- Replays stashed changes based on operation type
- For `Modified` files: writes content back with original filenames
- For `Deleted` files: re-deletes files from filesystem and git index
- For `Renamed` files: restores old filename and removes new filename to undo rename
- Applies meta configuration filtering to restored files
- Handles filename preservation correctly
- Retrieves stashed ObjDef models from memory
- Applies meta configuration filtering
- Writes the filtered objects back to disk
- Adds the changes to the git index
- Works seamlessly with pull operations to avoid conflicts

### changes
Get current changed files in detailed format with object-level analysis.

```lisp
worker_request("vcs", {"changes"})
```

**Returns:** List of changed objects with detailed change information.

**Example:**
```lisp
worker_request("vcs", {"changes"})
```

**Example Return Format:**
```lisp
{
  {
    "obj_id" -> #42,
    "operation" -> "modified",
    "modified_verbs" -> {"verb_name"},
    "deleted_verbs" -> {},
    "added_verbs" -> {"new_verb"},
    "modified_props" -> {"prop_name"},
    "deleted_props" -> {},
    "added_props" -> {"new_prop"}
  },
  {
    "obj_id" -> "player.moo",
    "operation" -> "renamed",
    "old_obj_id" -> "character.moo",
    "modified_verbs" -> {},
    "deleted_verbs" -> {},
    "added_verbs" -> {},
    "modified_props" -> {},
    "deleted_props" -> {},
    "added_props" -> {}
  },
  {
    "obj_id" -> #43,
    "operation" -> "deleted"
  }
}
```

**Note:** This operation:
- Analyzes current working directory changes using git status
- Detects renames by comparing first lines of added and deleted files
- Provides detailed verb and property change analysis
- Uses `v_str` for object IDs when filename doesn't match object number, otherwise `v_obj`
- For deleted objects, only includes `obj_id` and `operation` fields
- For renamed objects, includes `old_obj_id` field with the previous object identifier
- Returns empty lists for unchanged categories (verbs/properties)

## Credential Management Operations

### set_ssh_key
Set SSH key for repository access.

```lisp
worker_request("vcs", {"set_ssh_key", "key_content", "key_name"})
```

**Parameters:**
- `key_content` (string): SSH private key content
- `key_name` (string): Name/identifier for the key

**Example:**
```lisp
worker_request("vcs", {"set_ssh_key", "-----BEGIN OPENSSH PRIVATE KEY-----\n...", "deploy_key"})
```

### clear_ssh_key
Remove the current SSH key.

```lisp
worker_request("vcs", {"clear_ssh_key"})
```

### set_git_user
Set Git user information for commits.

```lisp
worker_request("vcs", {"set_git_user", "name", "email"})
```

**Parameters:**
- `name` (string): Git user name
- `email` (string): Git user email

**Example:**
```lisp
worker_request("vcs", {"set_git_user", "John Doe", "john@example.com"})
```

### test_ssh
Test SSH connection to the repository.

```lisp
worker_request("vcs", {"test_ssh"})
```

## Meta File Operations

### update_ignored_properties
Update the list of ignored properties for an object.

```lisp
worker_request("vcs", {"update_ignored_properties", "object_name", "prop1", "prop2", "prop3"})
```

**Parameters:**
- `object_name` (string): Name of the object file
- Variable number of property names (strings) to ignore

**Example:**
```lisp
worker_request("vcs", {"update_ignored_properties", "player.moo", "last_login", "session_id"})
```

### update_ignored_verbs
Update the list of ignored verbs for an object.

```lisp
worker_request("vcs", {"update_ignored_verbs", "object_name", "verb1", "verb2", "verb3"})
```

**Parameters:**
- `object_name` (string): Name of the object file
- Variable number of verb names (strings) to ignore

**Example:**
```lisp
worker_request("vcs", {"update_ignored_verbs", "player.moo", "login", "logout", "save"})
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
