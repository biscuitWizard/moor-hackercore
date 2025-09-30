// Copyright (C) 2025 Ryan Daum <ryan.daum@gmail.com> This program is free
// software: you can redistribute it and/or modify it under the terms of the GNU
// General Public License as published by the Free Software Foundation, version
// 3.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.
//

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Status of a change in the VCS workflow
/// MERGED: The change has been committed and merged into the main branch
/// LOCAL: The change is currently being worked on (current working change)
/// REVIEW: The change is pending review/approval
/// IDLE: The change is inactive but preserved for future work
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeStatus {
    Merged,  // or "COMMITTED" 
    Local,   // or "WORKING"
    Review,  // Awaiting approval/review
    Idle,    // Inactive but preserved
}

/// Represents a file rename operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenamedObject {
    pub from: ObjectInfo,
    pub to: ObjectInfo,
}

/// Represents a change in the version control system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: String,
    pub timestamp: u64, // Linux UTC epoch
    pub status: ChangeStatus, // MERGED, LOCAL, REVIEW, or IDLE
    pub added_objects: Vec<ObjectInfo>,
    pub modified_objects: Vec<ObjectInfo>,
    pub deleted_objects: Vec<ObjectInfo>,
    pub renamed_objects: Vec<RenamedObject>,
    // Workspace-specific fields
    pub index_change_id: Option<String>, // The indexed change this workspace change is based on
}

/// Request structure for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRequest {
    pub operation: String,
    pub args: Vec<String>,
}

/// Information about an object in the complete object list
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ObjectInfo {
    pub name: String,
    pub version: u64,
}

/// Response structure for operations - converted from Var for HTTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse {
    pub result: serde_json::Value,
    pub success: bool,
    pub operation: String,
}

/// Request structure for HTTP requests
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub operation: String,
    pub args: Vec<String>,
}

/// Request structure for change create operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeCreateRequest {
    pub name: String,
    pub description: Option<String>,
    pub author: String,
}

/// Request structure for change status operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatusRequest {
    // No fields needed - lists status of current change
}

/// Response structure for change status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeStatusResponse {
    pub change_id: Option<String>,
    pub change_name: Option<String>,
    pub status: ChangeStatus,
}


/// Request structure for change abandon operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeAbandonRequest {
    // No fields needed - just abandons the current change
}

/// Request structure for change approve operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeApproveRequest {
    pub change_id: String,
}

/// Request structure for object get operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectGetRequest {
    pub object_name: String,
}

/// Request structure for object update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectUpdateRequest {
    pub object_name: String,
    pub vars: Vec<String>, // List of strings representing the MOO object dump
}

/// Request structure for object rename operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRenameRequest {
    pub from_name: String,
    pub to_name: String,
}

/// Request structure for object delete operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDeleteRequest {
    pub object_name: String,
}

/// Request structure for index list operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexListRequest {
    pub limit: Option<usize>,
    pub page: Option<usize>,
}

/// General error types for the VCS worker
#[derive(Error, Debug)]
pub enum ObjectsTreeError {
    #[error("Fjall database error: {0}")]
    FjallError(#[from] fjall::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Provider error: {0}")]
    ProviderError(#[from] crate::providers::error::ProviderError),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
}
