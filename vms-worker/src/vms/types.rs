
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
    
    /// List all .moo objects with dependency ordering
    ListObjects,
    
    /// Get full dump contents for specified object names
    GetObjects {
        object_names: Vec<String>,
    },
    
    /// Get paginated list of commits
    GetCommits {
        limit: Option<usize>,
        offset: Option<usize>,
    },
    
    /// Credential management operations
    SetSshKey { key_content: String, key_name: String },
    ClearSshKey,
    SetGitUser { name: String, email: String },
    GetCredentialStatus,
    TestSshConnection,
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
    /// Commit timestamp (Linux epoch)
    pub datetime: i64,
    /// Commit message
    pub message: String,
    /// Author name
    pub author: String,
}
