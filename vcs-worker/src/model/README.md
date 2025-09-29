# ObjectDeltaModel - Standardized MOO Communication Interface

This module provides a standardized interface for communicating object changes and deltas back to MOO (MUD Object Oriented) systems. It converts Rust data structures into MOO-compatible variable formats using the `moor_var` crate.

## Overview

The `ObjectDeltaModel` provides a comprehensive way to track and communicate all types of object changes in a MOO world:

- **Object lifecycle**: Added, deleted, renamed objects
- **Object modifications**: Detailed tracking of verb and property changes
- **Batch processing**: Compile multiple commits into a single delta model
- **MOO compatibility**: Direct conversion to MOO variable types

## Core Structures

### ObjectDeltaModel

The main structure representing a complete set of object changes:

```rust
pub struct ObjectDeltaModel {
    pub objects_renamed: HashMap<String, String>,    // from_obj_id -> to_obj_id
    pub objects_deleted: HashSet<String>,            // list of deleted object IDs
    pub objects_added: HashSet<String>,              // list of added object IDs  
    pub objects_modified: HashSet<String>,           // list of modified object IDs
    pub changes: Vec<ObjectChange>,                  // detailed changes per object
}
```

### ObjectChange

Detailed change information for individual objects:

```rust
pub struct ObjectChange {
    pub obj_id: String,                              // Object ID or display name
    pub verbs_modified: HashSet<String>,             // Modified verbs
    pub verbs_added: HashSet<String>,                // Added verbs
    pub verbs_renamed: HashMap<String, String>,      // Renamed verbs (old -> new)
    pub verbs_deleted: HashSet<String>,              // Deleted verbs
    pub props_modified: HashSet<String>,             // Modified properties
    pub props_added: HashSet<String>,                // Added properties
    pub props_renamed: HashMap<String, String>,      // Renamed properties (old -> new)
    pub props_deleted: HashSet<String>,              // Deleted properties
}
```

## Key Features

### 1. MOO Variable Conversion

Both `ObjectDeltaModel` and `ObjectChange` can be converted to MOO variables:

```rust
let delta_model = ObjectDeltaModel::new();
// ... populate with changes ...
let moo_var = delta_model.to_moo_var(); // Returns moor_var::Var
```

### 2. Object Name Handling

The system intelligently handles object names vs. IDs:

```rust
// If object name differs from ID, use capitalized name
obj_id_to_object_name("#4", Some("foobar")) // Returns "Foobar"
obj_id_to_object_name("#4", Some("#4"))     // Returns "#4"
obj_id_to_object_name("#4", None)           // Returns "#4"
```

### 3. Commit Compilation

Process multiple commits into a unified delta model:

```rust
let commits = vec![commit1, commit2, commit3];
let name_mapping = Some(object_name_mapping);
let delta_model = compile_commits_to_delta_model(commits, name_mapping);
```

### 4. Model Merging

Combine multiple delta models:

```rust
let combined_model = compile_delta_models(vec![model1, model2, model3]);
```

## Usage Examples

### Basic Usage

```rust
use crate::model::{ObjectDeltaModel, ObjectChange};

// Create a new delta model
let mut delta_model = ObjectDeltaModel::new();

// Add object changes
delta_model.add_object_added("NewSword".to_string());
delta_model.add_object_deleted("OldShield".to_string());
delta_model.add_object_renamed("TempObject".to_string(), "PermanentObject".to_string());

// Create detailed change
let mut change = ObjectChange::new("ModifiedRoom".to_string());
change.verbs_added.insert("new_command".to_string());
change.props_modified.insert("description".to_string());
delta_model.add_object_change(change);

// Convert to MOO format
let moo_var = delta_model.to_moo_var();
```

### From Commit Data

```rust
use crate::model::compile_commits_to_delta_model;
use crate::types::{Change, ChangeStatus};

let commit = Change {
    id: "commit1".to_string(),
    name: "Add objects".to_string(),
    description: Some("Added new objects".to_string()),
    author: "player1".to_string(),
    timestamp: 1234567890,
    status: ChangeStatus::Merged,
    added_objects: vec!["#123".to_string()],
    modified_objects: vec!["#124".to_string()],
    deleted_objects: vec!["#125".to_string()],
    renamed_objects: vec![],
    index_change_id: None,
};

let delta_model = compile_commits_to_delta_model(vec![commit], None);
let moo_var = delta_model.to_moo_var();
```

## MOO Variable Format

The resulting MOO variable follows this structure:

```moo
[
  "objects_renamed" -> [from_id -> to_id, ...],
  "objects_deleted" -> [obj_id, ...],
  "objects_added" -> [obj_id, ...], 
  "objects_modified" -> [obj_id, ...],
  "changes" -> [
    [
      "obj_id" -> "ObjectName",
      "verbs_modified" -> [verb_name, ...],
      "verbs_added" -> [verb_name, ...],
      "verbs_renamed" -> [old_name -> new_name, ...],
      "verbs_deleted" -> [verb_name, ...],
      "props_modified" -> [prop_name, ...],
      "props_added" -> [prop_name, ...],
      "props_renamed" -> [old_name -> new_name, ...],
      "props_deleted" -> [prop_name, ...]
    ],
    ...
  ]
]
```

## Integration with VCS Worker

This model integrates seamlessly with the VCS worker's change tracking system:

1. **Commit Processing**: Convert `Change` objects to `ObjectDeltaModel`
2. **Object Name Resolution**: Use object name mappings for display
3. **MOO Communication**: Send delta models to MOO systems
4. **Batch Operations**: Process multiple changes efficiently

## Testing

The module includes comprehensive tests covering:

- Basic model creation and manipulation
- MOO variable conversion
- Object name handling
- Commit compilation
- Model merging
- Edge cases and error handling

Run tests with:

```bash
cargo test model
```

## Dependencies

- `moor_var`: MOO variable types and conversion
- `serde`: Serialization support
- `std::collections`: HashMaps and HashSets for efficient data storage
