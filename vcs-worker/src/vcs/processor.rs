use tracing::{info, error, warn};
use moor_var::{Var, v_str};
use crate::config::Config;
use crate::git::GitRepository;
use crate::utils::PathUtils;
use super::types::{VcsOperation, PullImpact, ChangeStatus};
use super::object_handler::ObjectHandler;
use super::status_handler::StatusHandler;
use super::meta_handler::MetaHandler;
use moor_common::tasks::WorkerError;
use libc;

/// Process VCS operations
pub struct VcsProcessor {
    git_repo: Option<GitRepository>,
    config: Config,
    object_handler: ObjectHandler,
    status_handler: StatusHandler,
    meta_handler: MetaHandler,
}

impl VcsProcessor {
    pub fn new() -> Self {
        let config = Config::from_env();
        let mut processor = Self { 
            git_repo: None,
            config: config.clone(),
            object_handler: ObjectHandler::new(config.clone()),
            status_handler: StatusHandler,
            meta_handler: MetaHandler::new(config),
        };
        processor.initialize_repository();
        processor
    }
    
    pub fn with_config(config: Config) -> Self {
        let mut processor = Self { 
            git_repo: None,
            config: config.clone(),
            object_handler: ObjectHandler::new(config.clone()),
            status_handler: StatusHandler,
            meta_handler: MetaHandler::new(config),
        };
        processor.initialize_repository();
        processor
    }
    
    /// Initialize the git repository using configuration
    pub fn initialize_repository(&mut self) {
        info!("VcsProcessor: Initializing repository");
        let repo_path = self.config.repository_path();
        info!("VcsProcessor: Using repository path: {:?}", repo_path);
        
        // Check if the path exists and contains a git repository
        if repo_path.exists() && repo_path.join(".git").exists() {
            info!("VcsProcessor: Found existing .git directory at {:?}", repo_path);
            
            // Chown the repository to current user first to fix permission issues
            VcsProcessor::chown_repository_to_current_user_static(&repo_path);
            
            // Try to open existing repository
            match GitRepository::open(&repo_path, self.config.clone()) {
                Ok(repo) => {
                    info!("VcsProcessor: Successfully opened existing git repository at {:?}", repo_path);
                    self.git_repo = Some(repo);
                    return;
                }
                Err(e) => {
                    warn!("VcsProcessor: Found .git directory at {:?} but failed to open as repository: {}", repo_path, e);
                    // Don't try to clone/init if there's already a .git directory
                    // This prevents clearing existing repositories
                    self.git_repo = None;
                    return;
                }
            }
        }
        
        // Only attempt to clone or initialize if no existing repository was found
        // If we have a repository URL configured, try to clone it
        if let Some(repo_url) = self.config.repository_url() {
            info!("VcsProcessor: Attempting to clone repository from: {}", repo_url);
            match VcsProcessor::clone_repository_static(repo_url, &repo_path, &self.config) {
                Ok(repo) => {
                    info!("VcsProcessor: Successfully cloned repository from {} to {:?}", repo_url, repo_path);
                    self.git_repo = Some(repo);
                    return;
                }
                Err(e) => {
                    error!("VcsProcessor: Failed to clone repository from {}: {}", repo_url, e);
                    warn!("VcsProcessor: Falling back to initializing empty repository");
                }
            }
        }
        
        // If no URL or clone failed, initialize an empty repository
        match GitRepository::init(&repo_path, self.config.clone()) {
            Ok(repo) => {
                info!("VcsProcessor: Initialized new empty git repository at {:?}", repo_path);
                self.git_repo = Some(repo);
            }
            Err(e) => {
                error!("VcsProcessor: Failed to initialize git repository at {:?}: {}", repo_path, e);
                // Continue without git repo - operations will fail gracefully
                self.git_repo = None;
            }
        }
    }
    
    /// Change ownership of repository directory to current user
    fn chown_repository_to_current_user_static(repo_path: &std::path::Path) {
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command;
        
        info!("Changing ownership of repository at {:?} to current user", repo_path);
        
        // Get current user ID
        let uid = unsafe { libc::getuid() };
        let gid = unsafe { libc::getgid() };
        
        info!("Current user ID: {}, group ID: {}", uid, gid);
        
        // Use chown command to recursively change ownership
        match Command::new("chown")
            .args(&["-R", &format!("{}:{}", uid, gid), repo_path.to_str().unwrap_or("")])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    info!("Successfully changed ownership of repository directory");
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to change ownership of repository directory: {}", error_msg);
                }
            }
            Err(e) => {
                warn!("Failed to execute chown command: {}", e);
            }
        }
        
        // Also try to fix permissions on the directory
        if let Err(e) = std::fs::metadata(repo_path).and_then(|metadata| {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(repo_path, perms)
        }) {
            warn!("Failed to set permissions on repository directory: {}", e);
        }
    }
    
    /// Clone a repository from a URL
    fn clone_repository_static(url: &str, path: &std::path::Path, config: &Config) -> Result<GitRepository, Box<dyn std::error::Error>> {
        use git2::build::RepoBuilder;
        use std::fs;
        
        // Check if the directory exists and contains a valid git repository
        if path.exists() {
            // Check if there's a .git directory first
            if path.join(".git").exists() {
                // Try to open as a git repository
                match GitRepository::open(path, config.clone()) {
                    Ok(_) => {
                        info!("Directory {:?} already contains a valid git repository, skipping clone", path);
                        return GitRepository::open(path, config.clone());
                    }
                    Err(e) => {
                        warn!("Directory {:?} contains .git but is not a valid repository: {}", path, e);
                        // Don't remove directories that contain .git - this could be a corrupted repo
                        return Err(format!("Directory {:?} contains .git but is not a valid repository: {}", path, e).into());
                    }
                }
            } else {
                // Directory exists but has no .git directory, safe to remove
                info!("Removing existing non-git directory at {:?} before cloning", path);
                fs::remove_dir_all(path)?;
            }
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Clone the repository
        info!("Cloning repository from {} to {:?}", url, path);
        let _repo = RepoBuilder::new()
            .clone(url, path)?;
        
        // Create our GitRepository wrapper
        let git_repo = GitRepository::open(path, config.clone())?;
        
        Ok(git_repo)
    }
    
    /// Process a VCS operation
    pub fn process_operation(&mut self, operation: VcsOperation) -> Result<Vec<Var>, WorkerError> {
        info!("VcsProcessor: Processing operation: {:?}", operation);
        match operation {            
            VcsOperation::AddOrUpdateObject { object_dump, object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.add_object(repo, object_dump, object_name)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::DeleteObject { object_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.delete_object(repo, object_name, None)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::RenameObject { old_name, new_name } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.rename_object(repo, old_name, new_name)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::Commit { message, author_name, author_email } => {
                if let Some(ref repo) = self.git_repo {
                    self.create_commit(repo, message, author_name, author_email)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::Status => {
                info!("VcsProcessor: Processing Status operation");
                if let Some(ref repo) = self.git_repo {
                    info!("VcsProcessor: Git repository is available, calling status handler");
                    self.status_handler.get_repository_status(repo, &self.config)
                } else {
                    error!("VcsProcessor: Git repository not available at /game");
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::ListObjects => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.list_objects(repo)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::GetObjects { object_names } => {
                if let Some(ref repo) = self.git_repo {
                    self.object_handler.get_objects(repo, object_names)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::GetCommits { limit, offset } => {
                if let Some(ref repo) = self.git_repo {
                    self.get_commits(repo, limit, offset)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            // Credential management operations
            VcsOperation::SetSshKey { key_content, key_name } => {
                info!("Setting SSH key: {} ({} bytes)", key_name, key_content.len());
                
                let keys_dir = self.config.keys_directory();
                
                // Create keys directory if it doesn't exist
                if !keys_dir.exists() {
                    info!("Creating keys directory: {:?}", keys_dir);
                    if let Err(e) = std::fs::create_dir_all(&keys_dir) {
                        error!("Failed to create keys directory: {}", e);
                        return Err(WorkerError::RequestError(format!("Failed to create keys directory: {}", e)));
                    }
                }
                
                let key_path = keys_dir.join(&key_name);
                info!("Writing SSH key to: {:?}", key_path);
                
                // Write the key content
                if let Err(e) = std::fs::write(&key_path, key_content) {
                    error!("Failed to write SSH key to {:?}: {}", key_path, e);
                    return Err(WorkerError::RequestError(format!("Failed to write SSH key: {}", e)));
                }
                
                // Set restrictive permissions
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&key_path).map_err(|e| WorkerError::RequestError(format!("Failed to get metadata: {}", e)))?.permissions();
                perms.set_mode(0o600);
                std::fs::set_permissions(&key_path, perms).map_err(|e| WorkerError::RequestError(format!("Failed to set permissions: {}", e)))?;
                
                // Update config to use this key
                if let Err(e) = self.config.update_ssh_key(key_path.to_string_lossy().to_string()) {
                    error!("Failed to update SSH key configuration: {}", e);
                    return Err(WorkerError::RequestError(format!("Failed to update SSH key configuration: {}", e)));
                }
                
                info!("SSH key set successfully: {} at {:?}", key_name, key_path);
                Ok(vec![v_str(&format!("SSH key set successfully: {}", key_name))])
            }
            
            VcsOperation::ClearSshKey => {
                info!("Clearing SSH key configuration");
                self.config.clear_ssh_key();
                
                // Also clear the keys directory
                let keys_dir = self.config.keys_directory();
                if keys_dir.exists() {
                    info!("Clearing keys directory: {:?}", keys_dir);
                    if let Err(e) = std::fs::remove_dir_all(&keys_dir) {
                        error!("Failed to clear keys directory {:?}: {}", keys_dir, e);
                        // Don't fail the operation, just log the error
                    } else {
                        info!("Keys directory cleared successfully");
                    }
                }
                
                info!("SSH key configuration cleared");
                Ok(vec![v_str("SSH key configuration cleared")])
            }
            
            VcsOperation::SetGitUser { name, email } => {
                info!("Setting git user: {} <{}>", name, email);
                match self.config.set_git_user(name, email) {
                    Ok(_) => {
                        // Reconfigure git user in the repository
                        if let Some(ref repo) = self.git_repo {
                            if let Err(e) = repo.configure_git_user() {
                                error!("Failed to reconfigure git user in repository: {}", e);
                            } else {
                                info!("Git user reconfigured in repository successfully");
                            }
                        }
                        info!("Git user updated successfully");
                        Ok(vec![v_str("Git user updated successfully")])
                    }
                    Err(e) => {
                        error!("Failed to update git user: {}", e);
                        Err(WorkerError::RequestError(format!("Failed to update git user: {}", e)))
                    }
                }
            }
            
            
            VcsOperation::TestSshConnection => {
                info!("Testing SSH connection to remote repository");
                if let Some(ref repo) = self.git_repo {
                    match repo.test_ssh_connection() {
                        Ok(_) => {
                            info!("SSH connection test successful");
                            Ok(vec![v_str("SSH connection test successful")])
                        },
                        Err(e) => {
                            error!("SSH connection test failed: {}", e);
                            Err(WorkerError::RequestError(format!("SSH connection test failed: {}", e)))
                        }
                    }
                } else {
                    error!("Git repository not available for SSH test");
                    Err(WorkerError::RequestError("Git repository not available".to_string()))
                }
            }
            
            VcsOperation::UpdateIgnoredProperties { object_name, properties } => {
                if let Some(ref repo) = self.git_repo {
                    self.meta_handler.update_ignored_properties(repo, object_name, properties)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::UpdateIgnoredVerbs { object_name, verbs } => {
                if let Some(ref repo) = self.git_repo {
                    self.meta_handler.update_ignored_verbs(repo, object_name, verbs)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::Pull { dry_run } => {
                if let Some(ref repo) = self.git_repo {
                    self.pull_with_rebase(repo, dry_run)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
            VcsOperation::Reset => {
                if let Some(ref repo) = self.git_repo {
                    self.reset_working_tree(repo)
                } else {
                    Err(WorkerError::RequestError("Git repository not available at /game".to_string()))
                }
            }
            
        }
    }
    
    /// Create a commit with current changes
    fn create_commit(
        &self, 
        repo: &GitRepository, 
        message: String,
        author_name: String,
        author_email: String,
    ) -> Result<Vec<Var>, WorkerError> {
        // First, pull any remote changes to avoid conflicts
        info!("Pulling remote changes before committing and pushing");
        match self.pull_with_rebase(repo, false) {
            Ok(pull_result) => {
                info!("Pull completed: {:?}", pull_result);
            }
            Err(e) => {
                error!("Failed to pull remote changes: {}", e);
                // Continue with commit even if pull fails - let push handle it
            }
        }
        
        match repo.commit(&message, &author_name, &author_email) {
            Ok(_) => {
                info!("Created commit: {}", message);
                
                // Now push the commit to the remote
                match repo.push() {
                    Ok(_) => {
                        info!("Successfully pushed commit to remote");
                        Ok(vec![v_str(&format!("Created and pushed commit: {}", message))])
                    }
                    Err(e) => {
                        error!("Failed to push commit: {}, rolling back commit", e);
                        
                        // Rollback the commit since push failed
                        match repo.rollback_last_commit() {
                            Ok(_) => {
                                info!("Successfully rolled back commit after push failure");
                                Err(WorkerError::RequestError(format!("Commit failed: push to remote failed ({}). Changes have been restored to staged state.", e)))
                            }
                            Err(rollback_error) => {
                                error!("Failed to rollback commit after push failure: {}", rollback_error);
                                Err(WorkerError::RequestError(format!("Commit failed: push to remote failed ({}). Additionally, rollback failed ({}). Repository may be in an inconsistent state.", e, rollback_error)))
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to create commit: {}", e);
                Err(WorkerError::RequestError(format!("Failed to create commit: {}", e)))
            }
        }
    }
    
    /// Get paginated list of commits
    fn get_commits(
        &self, 
        repo: &GitRepository, 
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Var>, WorkerError> {
        match repo.get_commits(limit, offset) {
            Ok(commits) => {
                use moor_var::v_map;
                
                let mut result = Vec::new();
                for commit in commits {
                    let commit_info = v_map(&[
                        (v_str("id"), v_str(&commit.id)),
                        (v_str("full_id"), v_str(&commit.full_id)),
                        (v_str("datetime"), v_str(&commit.datetime.to_string())),
                        (v_str("message"), v_str(&commit.message)),
                        (v_str("author"), v_str(&commit.author)),
                    ]);
                    result.push(commit_info);
                }
                
                info!("Retrieved {} commits", result.len());
                Ok(result)
            }
            Err(e) => {
                error!("Failed to get commits: {}", e);
                Err(WorkerError::RequestError(format!("Failed to get commits: {}", e)))
            }
        }
    }
    
    /// Pull with rebase strategy and automatic conflict resolution
    fn pull_with_rebase(&self, repo: &GitRepository, dry_run: bool) -> Result<Vec<Var>, WorkerError> {
        info!("Starting pull operation with rebase strategy (dry_run: {})", dry_run);
        
        if dry_run {
            return self.pull_dry_run(repo);
        }
        
        // First, fetch the latest changes from remote
        match repo.fetch_remote() {
            Ok(_) => {
                info!("Successfully fetched from remote");
            }
            Err(e) => {
                error!("Failed to fetch from remote: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to fetch from remote: {}", e)));
            }
        }
        
        // Get the current branch and upstream
        let current_branch = match repo.get_current_branch() {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                error!("No current branch found");
                return Err(WorkerError::RequestError("No current branch found".to_string()));
            }
            Err(e) => {
                error!("Failed to get current branch: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to get current branch: {}", e)));
            }
        };
        
        let upstream_branch = format!("origin/{}", current_branch);
        info!("Current branch: {}, upstream: {}", current_branch, upstream_branch);
        
        // Check if there are any commits to pull
        match repo.get_commits_ahead_behind(&current_branch, &upstream_branch) {
            Ok((ahead, behind)) => {
                info!("Branch is {} commits ahead, {} commits behind upstream", ahead, behind);
                
                if behind == 0 {
                    info!("No commits to pull");
                    return Ok(vec![v_str("No commits to pull - repository is up to date")]);
                }
                
                // Perform rebase with automatic conflict resolution
                match self.rebase_with_auto_resolution(repo, &upstream_branch) {
                    Ok(modified_objects) => {
                        let message = if modified_objects.is_empty() {
                            format!("Successfully pulled {} commits with no conflicts", behind)
                        } else {
                            format!("Successfully pulled {} commits, automatically resolved conflicts in {} objects", behind, modified_objects.len())
                        };
                        info!("{}", message);
                        Ok(vec![v_str(&message)])
                    }
                    Err(e) => {
                        error!("Failed to rebase: {}", e);
                        Err(WorkerError::RequestError(format!("Failed to rebase: {}", e)))
                    }
                }
            }
            Err(e) => {
                error!("Failed to check commits ahead/behind: {}", e);
                Err(WorkerError::RequestError(format!("Failed to check commits ahead/behind: {}", e)))
            }
        }
    }
    
    /// Dry run for pull operation - returns what would be modified
    fn pull_dry_run(&self, repo: &GitRepository) -> Result<Vec<Var>, WorkerError> {
        info!("Performing pull dry run");
        
        // Fetch to get latest remote info
        match repo.fetch_remote() {
            Ok(_) => {
                info!("Successfully fetched from remote for dry run");
            }
            Err(e) => {
                error!("Failed to fetch from remote for dry run: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to fetch from remote: {}", e)));
            }
        }
        
        let current_branch = match repo.get_current_branch() {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                error!("No current branch found");
                return Err(WorkerError::RequestError("No current branch found".to_string()));
            }
            Err(e) => {
                error!("Failed to get current branch: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to get current branch: {}", e)));
            }
        };
        
        let upstream_branch = format!("origin/{}", current_branch);
        
        match repo.get_commits_ahead_behind(&current_branch, &upstream_branch) {
            Ok((ahead, behind)) => {
                if behind == 0 {
                    return Ok(vec![v_str("No commits to pull - repository is up to date")]);
                }
                
                // Analyze what objects would be affected
                match self.analyze_pull_impact(repo, &upstream_branch) {
                    Ok(impact) => {
                        use moor_var::v_map;
                        
                        let result = v_map(&[
                            (v_str("commits_to_pull"), v_str(&behind.to_string())),
                            (v_str("commits_ahead"), v_str(&ahead.to_string())),
                            (v_str("modified_objects"), v_str(&impact.modified_objects.join(", "))),
                            (v_str("deleted_objects"), v_str(&impact.deleted_objects.join(", "))),
                            (v_str("renamed_objects"), v_str(&impact.renamed_objects.join(", "))),
                        ]);
                        
                        Ok(vec![result])
                    }
                    Err(e) => {
                        error!("Failed to analyze pull impact: {}", e);
                        Err(WorkerError::RequestError(format!("Failed to analyze pull impact: {}", e)))
                    }
                }
            }
            Err(e) => {
                error!("Failed to check commits ahead/behind: {}", e);
                Err(WorkerError::RequestError(format!("Failed to check commits ahead/behind: {}", e)))
            }
        }
    }
    
    /// Analyze what objects would be affected by a pull
    fn analyze_pull_impact(&self, repo: &GitRepository, upstream_branch: &str) -> Result<PullImpact, WorkerError> {
        // Get the list of commits that would be pulled
        let commits_to_pull = match repo.get_commits_between("HEAD", upstream_branch) {
            Ok(commits) => commits,
            Err(e) => {
                error!("Failed to get commits between HEAD and {}: {}", upstream_branch, e);
                return Err(WorkerError::RequestError(format!("Failed to get commits: {}", e)));
            }
        };
        
        let mut modified_objects = std::collections::HashSet::new();
        let mut deleted_objects = std::collections::HashSet::new();
        let mut renamed_objects = std::collections::HashSet::new();
        
        // Analyze each commit to see what objects it affects
        for commit in commits_to_pull {
            match repo.get_commit_changes(&commit.full_id) {
                Ok(changes) => {
                    for change in changes {
                        if let Some(object_name) = PathUtils::extract_object_name_from_path(&change.path) {
                            match change.status {
                                ChangeStatus::Added | ChangeStatus::Modified => {
                                    modified_objects.insert(object_name);
                                }
                                ChangeStatus::Deleted => {
                                    deleted_objects.insert(object_name);
                                }
                                ChangeStatus::Renamed => {
                                    if let Some(old_name) = change.old_path.as_ref() {
                                        if let Some(old_object_name) = PathUtils::extract_object_name_from_path(old_name) {
                                            renamed_objects.insert(format!("{} -> {}", old_object_name, object_name));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to get changes for commit {}: {}", commit.full_id, e);
                    // Continue with other commits
                }
            }
        }
        
        Ok(PullImpact {
            modified_objects: modified_objects.into_iter().collect(),
            deleted_objects: deleted_objects.into_iter().collect(),
            renamed_objects: renamed_objects.into_iter().collect(),
        })
    }
    
    
    /// Rebase with automatic conflict resolution using object dump replay
    fn rebase_with_auto_resolution(&self, repo: &GitRepository, upstream_branch: &str) -> Result<Vec<String>, WorkerError> {
        info!("Starting rebase with automatic conflict resolution");
        
        // Get the list of commits to rebase
        let commits_to_rebase = match repo.get_commits_between("HEAD", upstream_branch) {
            Ok(commits) => commits,
            Err(e) => {
                error!("Failed to get commits to rebase: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to get commits: {}", e)));
            }
        };
        
        let mut modified_objects = Vec::new();
        
        // Replay each commit by loading object dumps and applying them
        for commit in commits_to_rebase {
            info!("Replaying commit: {} - {}", commit.id, commit.message);
            
            match self.replay_commit(repo, &commit) {
                Ok(commit_modified) => {
                    modified_objects.extend(commit_modified);
                }
                Err(e) => {
                    error!("Failed to replay commit {}: {}", commit.id, e);
                    return Err(WorkerError::RequestError(format!("Failed to replay commit {}: {}", commit.id, e)));
                }
            }
        }
        
        // Perform the actual rebase
        match repo.rebase_onto(upstream_branch) {
            Ok(_) => {
                info!("Successfully completed rebase");
                Ok(modified_objects)
            }
            Err(e) => {
                error!("Failed to complete rebase: {}", e);
                Err(WorkerError::RequestError(format!("Failed to complete rebase: {}", e)))
            }
        }
    }
    
    /// Replay a single commit by loading object dumps and applying them
    fn replay_commit(&self, repo: &GitRepository, commit: &crate::vcs::types::CommitInfo) -> Result<Vec<String>, WorkerError> {
        let mut modified_objects = Vec::new();
        
        // Get the changes in this commit
        let changes = match repo.get_commit_changes(&commit.full_id) {
            Ok(changes) => changes,
            Err(e) => {
                error!("Failed to get changes for commit {}: {}", commit.full_id, e);
                return Err(WorkerError::RequestError(format!("Failed to get commit changes: {}", e)));
            }
        };
        
        // Process each change
        for change in changes {
            if let Some(object_name) = PathUtils::extract_object_name_from_path(&change.path) {
                match change.status {
                    ChangeStatus::Added | ChangeStatus::Modified => {
                        // Load the object dump from the commit
                        match repo.get_file_content_at_commit(&commit.full_id, &change.path) {
                            Ok(content) => {
                                // Parse and apply the object dump
                                match self.object_handler.parse_object_dump(&content) {
                                    Ok(mut object_def) => {
                                        // Load current meta config
                                        let meta_path = self.object_handler.meta_path(&object_name);
                                        let meta_full_path = repo.work_dir().join(&meta_path);
                                        let meta_config = match self.object_handler.load_or_create_meta_config(&meta_full_path) {
                                            Ok(config) => config,
                                            Err(e) => {
                                                error!("Failed to load meta config for {}: {}", object_name, e);
                                                continue;
                                            }
                                        };
                                        
                                        // Apply meta configuration filtering
                                        self.object_handler.apply_meta_config(&mut object_def, &meta_config);
                                        
                                        // Convert back to dump format
                                        match self.object_handler.to_dump(&object_def) {
                                            Ok(filtered_dump) => {
                                                // Write the filtered object
                                                let objects_dir = self.object_handler.config.objects_directory();
                                                let object_path = repo.work_dir().join(objects_dir).join(&format!("{}.moo", object_name));
                                                
                                                if let Err(e) = repo.write_file(&object_path, &filtered_dump) {
                                                    error!("Failed to write object {}: {}", object_name, e);
                                                    continue;
                                                }
                                                
                                                // Add to git
                                                if let Err(e) = repo.add_file(&object_path) {
                                                    error!("Failed to add object {} to git: {}", object_name, e);
                                                    continue;
                                                }
                                                
                                                modified_objects.push(object_name.clone());
                                                info!("Applied object dump for: {}", object_name);
                                            }
                                            Err(e) => {
                                                error!("Failed to convert object {} to dump: {}", object_name, e);
                                                continue;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("Failed to parse object dump for {}: {}", object_name, e);
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to get file content for {}: {}", object_name, e);
                                continue;
                            }
                        }
                    }
                    ChangeStatus::Deleted => {
                        // Remove the object
                        let object_name_clone = object_name.clone();
                        if let Err(e) = self.object_handler.delete_object(repo, object_name, None) {
                            error!("Failed to delete object {}: {}", object_name_clone, e);
                            continue;
                        }
                        modified_objects.push(object_name_clone.clone());
                        info!("Deleted object: {}", object_name_clone);
                    }
                    ChangeStatus::Renamed => {
                        if let Some(old_path) = change.old_path {
                            if let Some(old_object_name) = PathUtils::extract_object_name_from_path(&old_path) {
                                let old_name_clone = old_object_name.clone();
                                let new_name_clone = object_name.clone();
                                if let Err(e) = self.object_handler.rename_object(repo, old_object_name, object_name) {
                                    error!("Failed to rename object {} to {}: {}", old_name_clone, new_name_clone, e);
                                    continue;
                                }
                                modified_objects.push(new_name_clone.clone());
                                info!("Renamed object: {} -> {}", old_name_clone, new_name_clone);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(modified_objects)
    }
    
    /// Reset working tree, discarding all changes
    fn reset_working_tree(&self, repo: &GitRepository) -> Result<Vec<Var>, WorkerError> {
        info!("VCS Processor: Starting reset working tree operation");
        
        // Check repository status before reset
        match repo.status() {
            Ok(changes) => {
                if changes.is_empty() {
                    info!("VCS Processor: No changes to discard, working tree is already clean");
                    return Ok(vec![v_str("Working tree is already clean - no changes to discard")]);
                } else {
                    info!("VCS Processor: Found {} changes to discard: {:?}", changes.len(), changes);
                }
            }
            Err(e) => {
                warn!("VCS Processor: Could not check repository status before reset: {}", e);
            }
        }
        
        match repo.reset_working_tree() {
            Ok(_) => {
                info!("VCS Processor: Successfully reset working tree");
                
                // Verify the reset worked
                match repo.status() {
                    Ok(changes) => {
                        if changes.is_empty() {
                            info!("VCS Processor: Reset verification successful - working tree is now clean");
                            Ok(vec![v_str("Working tree reset - all changes discarded")])
                        } else {
                            warn!("VCS Processor: Reset completed but {} changes remain: {:?}", changes.len(), changes);
                            Ok(vec![v_str(&format!("Working tree reset completed, but {} changes remain", changes.len()))])
                        }
                    }
                    Err(e) => {
                        warn!("VCS Processor: Could not verify reset status: {}", e);
                        Ok(vec![v_str("Working tree reset completed (verification failed)")])
                    }
                }
            }
            Err(e) => {
                error!("VCS Processor: Failed to reset working tree: {}", e);
                Err(WorkerError::RequestError(format!("Failed to reset working tree: {}", e)))
            }
        }
    }
    
}
