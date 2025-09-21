use std::collections::HashMap;

/// VMS operation types
#[derive(Debug, Clone)]
pub enum VmsOperation {    
    /// Add or update a MOO object file
    AddOrUpdateObject { 
        object_dump: String, 
        object_name: String,
    },
    
    /// Delete a tracked MOO object file
    DeleteObject { 
        object_name: String,
    },
    
    /// Create a commit with current changes
    Commit { 
        message: String,
        author_name: String,
        author_email: String,
    },
    
    /// Get repository status
    Status,
}

/// Comprehensive repository status information
#[derive(Debug, Clone)]
pub struct RepositoryStatusInfo {
    /// Current upstream remote information
    pub upstream: Option<String>,
    /// Last commit information
    pub last_commit: Option<CommitInfo>,
    /// List of current changes
    pub changes: Vec<String>,
    /// Current branch name
    pub current_branch: Option<String>,
}

/// Information about a commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit hash (short)
    pub id: String,
    /// Commit hash (full)
    pub full_id: String,
    /// Commit timestamp
    pub datetime: String,
    /// Commit message
    pub message: String,
    /// Author name
    pub author: String,
}
