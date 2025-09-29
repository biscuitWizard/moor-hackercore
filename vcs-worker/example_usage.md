# Object Operations Usage

This document shows how to use the object operations in the VCS worker.

## HTTP API Usage

### Update Object: POST /object/update
### Update Object: POST /api/object/update

**Request Body:**
```json
{
  "object_name": "my_object",
  "vars": [
    "@create #123",
    "name: \"My Object\"",
    "description: \"A test object\"",
    "parent: #0"
  ]
}
```

**Response:**
```json
{
  "result": "Object 'my_object' updated successfully",
  "success": true,
  "operation": "object/update"
}
```

### Get Object: POST /object/get
### Get Object: POST /api/object/get

**Request Body:**
```json
{
  "object_name": "my_object"
}
```

**Response:**
```json
{
  "result": "@create #123\nname: \"My Object\"\ndescription: \"A test object\"\nparent: #0",
  "success": true,
  "operation": "object/get"
}
```

## RPC Usage

You can also call the operations via RPC:

### Update Operation
Operation name: `"object/update"` with arguments:
1. First argument: object name (string)
2. Second argument: either:
   - A JSON-encoded array of strings: `["@create #123", "name: \"My Object\"", ...]`
   - A single string if only one var
   - Or multiple separate string arguments (for HTTP-style calls)

**MOO RPC Example:**
```moo
worker_request("vcs", {"object/update", "vcs", dump_object($vcs)})
```
This will send the object name "vcs" and the dump_object result as a JSON array of strings.

### Get Operation
Operation name: `"object/get"` with arguments:
1. First argument: object name (string)

## Configuration

The sled database path can be configured via the `VCS_DB_PATH` environment variable. If not set, it defaults to `./game/` directory.

## Background Flushing

The VCS worker uses background flushing to ensure data persistence without blocking requests:
- **Immediate Flush**: Triggered after each object update operation
- **Periodic Flush**: Automatic flush every 5 seconds
- **Non-blocking**: All flush operations happen in the background
- **Reliable**: Data is guaranteed to be written to disk within 5 seconds

## Example MOO Object Definition

The `vars` list should contain lines of a MOO object definition, such as:

```
@create #123
name: "My Test Object"
description: "A test object for demonstration"
parent: #0
location: #0
```

### Update Operation Flow
The update operation will:
1. Join all the var strings with newlines
2. Parse the resulting MOO object definition using the moor compiler
3. Store the original MOO dump in the sled database
4. Trigger a background flush to ensure data is persisted to disk
5. Return a success message immediately (non-blocking)

### Get Operation Flow
The get operation will:
1. Look up the object by name in the sled database
2. Return the original MOO object dump that was stored
3. Return an error if the object is not found

## Error Handling

The operation will return error messages if:
- The object name is missing
- No vars are provided
- The MOO object definition fails to compile
- The database operation fails
