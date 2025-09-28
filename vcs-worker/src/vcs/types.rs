
/// VCS operation types
#[derive(Debug, Clone)]
pub enum VcsOperation {    
    /// Add or update a MOO object file
    AddOrUpdateObject { 
        object_dump: String, 
        object_name: String,
    },
    
    /// Delete a tracked MOO object file
    DeleteObject { 
        object_name: String,
    },
    
    /// Rename a tracked MOO object file
    RenameObject { 
        old_name: String,
        new_name: String,
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
    TestSshConnection,
    
    /// Meta file operations
    UpdateIgnoredProperties { object_name: String, properties: Vec<String> },
    UpdateIgnoredVerbs { object_name: String, verbs: Vec<String> },
    
    /// Pull operation with rebase strategy
    Pull { dry_run: bool },
    
    /// Reset working tree, discarding all changes
    Reset,
    
    /// Stash current changes using ObjDef models
    Stash,
    
    /// Replay stashed changes after pull
    ReplayStash,
    
    /// Get current changed files in detailed format
    Changes,
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

/// Information about a file change in a commit
#[derive(Debug, Clone)]
pub struct CommitChange {
    /// Path to the file
    pub path: String,
    /// Old path (for renames)
    pub old_path: Option<String>,
    /// Type of change
    pub status: ChangeStatus,
}

/// Type of change in a commit
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Detailed pull result information
#[derive(Debug, Clone)]
pub struct PullResult {
    /// List of commit results in chronological order (oldest first)
    pub commit_results: Vec<CommitResult>,
}

/// Result for a single commit in the pull
#[derive(Debug, Clone)]
pub struct CommitResult {
    /// Commit information
    pub commit_info: CommitInfo,
    /// Objects that were modified in this commit (as Var types - v_str for named objects, v_obj for others)
    pub modified_objects: Vec<moor_var::Var>,
    /// Objects that were deleted in this commit (as Var types - v_str for named objects, v_obj for others)
    pub deleted_objects: Vec<moor_var::Var>,
    /// Objects that were added in this commit (as Var types - v_str for named objects, v_obj for others)
    pub added_objects: Vec<moor_var::Var>,
    /// Objects that were renamed in this commit (as list of [from, to] pairs where each is v_str for named objects, v_obj for others)
    pub renamed_objects: Vec<Vec<moor_var::Var>>,
    /// Detailed changes for each object in this commit
    pub changes: Vec<ObjectChanges>,
}

/// Changes to a specific object
#[derive(Debug, Clone)]
pub struct ObjectChanges {
    /// Object ID (as Var type - v_str for named objects, v_obj for others)
    pub obj_id: moor_var::Var,
    /// Modified verbs (added or modified)
    pub modified_verbs: Vec<String>,
    /// Modified properties (added or modified)
    pub modified_props: Vec<String>,
    /// Deleted verbs
    pub deleted_verbs: Vec<String>,
    /// Deleted properties
    pub deleted_props: Vec<String>,
}
