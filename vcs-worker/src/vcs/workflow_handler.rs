use tracing::{info, error};
use moor_var::{Var, v_str, v_map, v_list, v_obj};
use crate::config::Config;
use crate::git::GitRepository;
use crate::utils::PathUtils;
use super::types::{PullResult, CommitResult, ObjectChanges, ChangeStatus, CommitInfo};
use super::object_handler::ObjectHandler;
use moor_common::tasks::WorkerError;

/// Handles complex VCS workflows like pull, rebase, and merge operations
pub struct WorkflowHandler {
    pub object_handler: ObjectHandler,
}

impl WorkflowHandler {
    pub fn new(config: Config) -> Self {
        Self {
            object_handler: ObjectHandler::new(config),
        }
    }

    /// Convert object information to appropriate Var type
    /// If the object name is different from the OID, use v_str with the name, otherwise use v_obj with the OID
    fn object_to_var(&self, obj: &moor_compiler::ObjectDefinition, filename: Option<&str>) -> Var {
        // Check if we have a filename and if it differs from the object's name
        if let Some(fname) = filename {
            // Extract object name from filename (without .moo extension)
            if let Some(object_name) = PathUtils::extract_object_name_from_path(fname) {
                // If the filename differs from the object's name, use the filename as v_str
                if object_name != obj.name {
                    return v_str(&object_name);
                }
            }
        }
        
        // Use the object's name as v_str if it's not empty and not just a number
        if !obj.name.is_empty() && !obj.name.parse::<u32>().is_ok() {
            v_str(&obj.name)
        } else {
            // Use v_obj with the OID as fallback
            v_obj(obj.oid)
        }
    }

    /// Execute pull with detailed analysis of changes
    pub fn execute_pull_with_analysis(&self, repo: &GitRepository, upstream_branch: &str, commits_to_pull: &[CommitInfo]) -> Result<PullResult, WorkerError> {
        info!("Executing pull with detailed analysis");
        
        let mut commit_results = Vec::new();
        
        // Analyze each commit to build the detailed change information
        for commit in commits_to_pull {
            info!("Analyzing commit: {} - {}", commit.id, commit.message);
            
            let mut commit_result = CommitResult {
                commit_info: commit.clone(),
                modified_objects: Vec::new(),
                deleted_objects: Vec::new(),
                added_objects: Vec::new(),
                renamed_objects: Vec::new(),
                changes: Vec::new(),
            };
            
            match repo.get_commit_changes(&commit.full_id) {
                Ok(changes) => {
                    for change in changes {
                        if let Some(_) = PathUtils::extract_object_name_from_path(&change.path) {
                            match change.status {
                                ChangeStatus::Added => {
                                    // Load the new object to get its OID
                                    if let Ok(content) = repo.get_file_content_at_commit(&commit.full_id, &change.path) {
                                        if let Ok(new_obj) = self.object_handler.parse_object_dump(&content) {
                                            commit_result.added_objects.push(self.object_to_var(&new_obj, Some(&change.path)));
                                            
                                            // Analyze changes for new object (no old version)
                                            let obj_changes = self.analyze_object_changes(None, &new_obj, Some(&change.path));
                                            commit_result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Modified => {
                                    // Load both old and new versions to compare
                                    let old_content = repo.get_file_content_at_commit("HEAD", &change.path).ok();
                                    let new_content = repo.get_file_content_at_commit(&commit.full_id, &change.path);
                                    
                                    if let Ok(new_content) = new_content {
                                        if let Ok(new_obj) = self.object_handler.parse_object_dump(&new_content) {
                                            commit_result.modified_objects.push(self.object_to_var(&new_obj, Some(&change.path)));
                                            
                                            // Analyze changes
                                            let old_obj = if let Some(old_content) = old_content {
                                                self.object_handler.parse_object_dump(&old_content).ok()
                                            } else {
                                                None
                                            };
                                            
                                            let obj_changes = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(&change.path));
                                            commit_result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Deleted => {
                                    // For deleted objects, we need to get the OID from the previous version
                                    if let Ok(old_content) = repo.get_file_content_at_commit("HEAD", &change.path) {
                                        if let Ok(old_obj) = self.object_handler.parse_object_dump(&old_content) {
                                            commit_result.deleted_objects.push(self.object_to_var(&old_obj, Some(&change.path)));
                                            
                                            // For deleted objects, all verbs and properties are "deleted"
                                            let obj_changes = ObjectChanges {
                                                obj_id: self.object_to_var(&old_obj, Some(&change.path)),
                                                modified_verbs: Vec::new(),
                                                modified_props: Vec::new(),
                                                deleted_verbs: old_obj.verbs.iter()
                                                    .map(|v| v.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "))
                                                    .collect(),
                                                deleted_props: {
                                                    let mut props = old_obj.property_definitions.iter()
                                                        .map(|p| p.name.to_string())
                                                        .collect::<Vec<_>>();
                                                    props.extend(old_obj.property_overrides.iter().map(|p| p.name.to_string()));
                                                    props
                                                },
                                            };
                                            commit_result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Renamed => {
                                    if let Some(old_path) = change.old_path {
                                        // Load both old and new versions
                                        let old_content = repo.get_file_content_at_commit("HEAD", &old_path).ok();
                                        let new_content = repo.get_file_content_at_commit(&commit.full_id, &change.path);
                                        
                                        if let Ok(new_content) = new_content {
                                            if let Ok(new_obj) = self.object_handler.parse_object_dump(&new_content) {
                                                // Parse old object for both from_var and changes analysis
                                                let old_obj = if let Some(old_content) = old_content {
                                                    self.object_handler.parse_object_dump(&old_content).ok()
                                                } else {
                                                    None
                                                };
                                                
                                                // Create [from, to] pair for renamed objects
                                                let from_var = if let Some(ref old_obj) = old_obj {
                                                    self.object_to_var(old_obj, Some(&old_path))
                                                } else {
                                                    v_str(&old_path)
                                                };
                                                let to_var = self.object_to_var(&new_obj, Some(&change.path));
                                                commit_result.renamed_objects.push(vec![from_var, to_var]);
                                                
                                                // Analyze changes (renamed objects are treated as modified)
                                                
                                                let obj_changes = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(&change.path));
                                                commit_result.changes.push(obj_changes);
                                            }
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
            
            commit_results.push(commit_result);
        }
        
        // Now perform the actual rebase
        match self.rebase_with_auto_resolution(repo, upstream_branch) {
            Ok(_) => {
                info!("Successfully completed rebase");
                Ok(PullResult { commit_results })
            }
            Err(e) => {
                error!("Failed to complete rebase: {}", e);
                Err(WorkerError::RequestError(format!("Failed to complete rebase: {}", e)))
            }
        }
    }

    /// Analyze what objects would be affected by a pull (dry run version)
    pub fn analyze_pull_impact_dry_run(&self, repo: &GitRepository, _upstream_branch: &str, commits_to_pull: &[CommitInfo]) -> Result<PullResult, WorkerError> {
        info!("Analyzing pull impact for dry run");
        
        let mut commit_results = Vec::new();
        
        // Analyze each commit to build the detailed change information (without executing)
        for commit in commits_to_pull {
            info!("Analyzing commit for dry run: {} - {}", commit.id, commit.message);
            
            let mut commit_result = CommitResult {
                commit_info: commit.clone(),
                modified_objects: Vec::new(),
                deleted_objects: Vec::new(),
                added_objects: Vec::new(),
                renamed_objects: Vec::new(),
                changes: Vec::new(),
            };
            
            match repo.get_commit_changes(&commit.full_id) {
                Ok(changes) => {
                    for change in changes {
                        if let Some(_) = PathUtils::extract_object_name_from_path(&change.path) {
                            match change.status {
                                ChangeStatus::Added => {
                                    // Load the new object to get its OID
                                    if let Ok(content) = repo.get_file_content_at_commit(&commit.full_id, &change.path) {
                                        if let Ok(new_obj) = self.object_handler.parse_object_dump(&content) {
                                            commit_result.added_objects.push(self.object_to_var(&new_obj, Some(&change.path)));
                                            
                                            // Analyze changes for new object (no old version)
                                            let obj_changes = self.analyze_object_changes(None, &new_obj, Some(&change.path));
                                            commit_result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Modified => {
                                    // Load both old and new versions to compare
                                    let old_content = repo.get_file_content_at_commit("HEAD", &change.path).ok();
                                    let new_content = repo.get_file_content_at_commit(&commit.full_id, &change.path);
                                    
                                    if let Ok(new_content) = new_content {
                                        if let Ok(new_obj) = self.object_handler.parse_object_dump(&new_content) {
                                            commit_result.modified_objects.push(self.object_to_var(&new_obj, Some(&change.path)));
                                            
                                            // Analyze changes
                                            let old_obj = if let Some(old_content) = old_content {
                                                self.object_handler.parse_object_dump(&old_content).ok()
                                            } else {
                                                None
                                            };
                                            
                                            let obj_changes = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(&change.path));
                                            commit_result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Deleted => {
                                    // For deleted objects, we need to get the OID from the previous version
                                    if let Ok(old_content) = repo.get_file_content_at_commit("HEAD", &change.path) {
                                        if let Ok(old_obj) = self.object_handler.parse_object_dump(&old_content) {
                                            commit_result.deleted_objects.push(self.object_to_var(&old_obj, Some(&change.path)));
                                            
                                            // For deleted objects, all verbs and properties are "deleted"
                                            let obj_changes = ObjectChanges {
                                                obj_id: self.object_to_var(&old_obj, Some(&change.path)),
                                                modified_verbs: Vec::new(),
                                                modified_props: Vec::new(),
                                                deleted_verbs: old_obj.verbs.iter()
                                                    .map(|v| v.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "))
                                                    .collect(),
                                                deleted_props: {
                                                    let mut props = old_obj.property_definitions.iter()
                                                        .map(|p| p.name.to_string())
                                                        .collect::<Vec<_>>();
                                                    props.extend(old_obj.property_overrides.iter().map(|p| p.name.to_string()));
                                                    props
                                                },
                                            };
                                            commit_result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Renamed => {
                                    if let Some(old_path) = change.old_path {
                                        // Load both old and new versions
                                        let old_content = repo.get_file_content_at_commit("HEAD", &old_path).ok();
                                        let new_content = repo.get_file_content_at_commit(&commit.full_id, &change.path);
                                        
                                        if let Ok(new_content) = new_content {
                                            if let Ok(new_obj) = self.object_handler.parse_object_dump(&new_content) {
                                                // Parse old object for both from_var and changes analysis
                                                let old_obj = if let Some(old_content) = old_content {
                                                    self.object_handler.parse_object_dump(&old_content).ok()
                                                } else {
                                                    None
                                                };
                                                
                                                // Create [from, to] pair for renamed objects
                                                let from_var = if let Some(ref old_obj) = old_obj {
                                                    self.object_to_var(old_obj, Some(&old_path))
                                                } else {
                                                    v_str(&old_path)
                                                };
                                                let to_var = self.object_to_var(&new_obj, Some(&change.path));
                                                commit_result.renamed_objects.push(vec![from_var, to_var]);
                                                
                                                // Analyze changes (renamed objects are treated as modified)
                                                
                                                let obj_changes = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(&change.path));
                                                commit_result.changes.push(obj_changes);
                                            }
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
            
            commit_results.push(commit_result);
        }
        
        Ok(PullResult { commit_results })
    }

    /// Analyze detailed changes between two object definitions
    fn analyze_object_changes(&self, old_obj: Option<&moor_compiler::ObjectDefinition>, new_obj: &moor_compiler::ObjectDefinition, filename: Option<&str>) -> ObjectChanges {
        let mut changes = ObjectChanges {
            obj_id: self.object_to_var(new_obj, filename),
            modified_verbs: Vec::new(),
            modified_props: Vec::new(),
            deleted_verbs: Vec::new(),
            deleted_props: Vec::new(),
        };
        
        if let Some(old) = old_obj {
            // Compare verbs
            let old_verb_names: std::collections::HashSet<String> = old.verbs.iter()
                .map(|v| v.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "))
                .collect();
            
            let new_verb_names: std::collections::HashSet<String> = new_obj.verbs.iter()
                .map(|v| v.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" "))
                .collect();
            
            // Find modified/added verbs
            for verb_name in &new_verb_names {
                if !old_verb_names.contains(verb_name) {
                    changes.modified_verbs.push(verb_name.clone());
                }
            }
            
            // Find deleted verbs
            for verb_name in &old_verb_names {
                if !new_verb_names.contains(verb_name) {
                    changes.deleted_verbs.push(verb_name.clone());
                }
            }
            
            // Compare properties
            let mut old_prop_names: std::collections::HashSet<String> = old.property_definitions.iter()
                .map(|p| p.name.to_string())
                .collect();
            old_prop_names.extend(old.property_overrides.iter().map(|p| p.name.to_string()));
            
            let mut new_prop_names: std::collections::HashSet<String> = new_obj.property_definitions.iter()
                .map(|p| p.name.to_string())
                .collect();
            new_prop_names.extend(new_obj.property_overrides.iter().map(|p| p.name.to_string()));
            
            // Find modified/added properties
            for prop_name in &new_prop_names {
                if !old_prop_names.contains(prop_name) {
                    changes.modified_props.push(prop_name.clone());
                }
            }
            
            // Find deleted properties
            for prop_name in &old_prop_names {
                if !new_prop_names.contains(prop_name) {
                    changes.deleted_props.push(prop_name.clone());
                }
            }
        } else {
            // New object - all verbs and properties are "added"
            for verb in &new_obj.verbs {
                let verb_name = verb.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ");
                changes.modified_verbs.push(verb_name);
            }
            
            for prop in &new_obj.property_definitions {
                changes.modified_props.push(prop.name.to_string());
            }
            
            for prop in &new_obj.property_overrides {
                changes.modified_props.push(prop.name.to_string());
            }
        }
        
        changes
    }

    /// Convert PullResult to MOO variables in the required format
    pub fn pull_result_to_moo_vars(&self, result: PullResult) -> Vec<Var> {
        // Convert each commit result to MOO format
        let commit_results_vars: Vec<Var> = result.commit_results.iter().map(|commit_result| {
            // Object lists are already Var types, so we can use them directly
            let modified_objects_vars: Vec<Var> = commit_result.modified_objects.clone();
            let deleted_objects_vars: Vec<Var> = commit_result.deleted_objects.clone();
            let added_objects_vars: Vec<Var> = commit_result.added_objects.clone();
            // Renamed objects are now Vec<Vec<Var>>, so we need to convert each pair to a list
            let renamed_objects_vars: Vec<Var> = commit_result.renamed_objects.iter().map(|pair| v_list(pair)).collect();
            
            // Convert changes to MOO format
            let changes_vars: Vec<Var> = commit_result.changes.iter().map(|change| {
                let modified_verbs_vars: Vec<Var> = change.modified_verbs.iter().map(|v| v_str(v)).collect();
                let modified_props_vars: Vec<Var> = change.modified_props.iter().map(|p| v_str(p)).collect();
                let deleted_verbs_vars: Vec<Var> = change.deleted_verbs.iter().map(|v| v_str(v)).collect();
                let deleted_props_vars: Vec<Var> = change.deleted_props.iter().map(|p| v_str(p)).collect();
                
                v_map(&[
                    (v_str("obj_id"), change.obj_id.clone()),
                    (v_str("modified_verbs"), v_list(&modified_verbs_vars)),
                    (v_str("modified_props"), v_list(&modified_props_vars)),
                    (v_str("deleted_verbs"), v_list(&deleted_verbs_vars)),
                    (v_str("deleted_props"), v_list(&deleted_props_vars)),
                ])
            }).collect();
            
            // Create the commit result map
            v_map(&[
                (v_str("commit_author"), v_str(&commit_result.commit_info.author)),
                (v_str("commit_id"), v_str(&commit_result.commit_info.id)),
                (v_str("commit_message"), v_str(&commit_result.commit_info.message)),
                (v_str("modified_objects"), v_list(&modified_objects_vars)),
                (v_str("deleted_objects"), v_list(&deleted_objects_vars)),
                (v_str("added_objects"), v_list(&added_objects_vars)),
                (v_str("renamed_objects"), v_list(&renamed_objects_vars)),
                (v_str("changes"), v_list(&changes_vars)),
            ])
        }).collect();
        
        commit_results_vars
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
    fn replay_commit(&self, repo: &GitRepository, commit: &CommitInfo) -> Result<Vec<String>, WorkerError> {
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
                                        let meta_full_path = PathUtils::object_meta_path(repo.work_dir(), &self.object_handler.config, &object_name);
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
                                                let object_path = PathUtils::object_path(repo.work_dir(), &self.object_handler.config, &object_name);
                                                
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
    
    /// Execute commit workflow (check for remote commits, abort if behind)
    pub fn execute_commit_workflow(
        &self,
        repo: &GitRepository,
        message: String,
        author_name: String,
        author_email: String,
    ) -> Result<Var, WorkerError> {
        use crate::git::operations::commit_ops::CommitOps;
        
        match CommitOps::create_commit_with_push(
            repo,
            &message,
            &author_name,
            &author_email,
        ) {
            Ok(_) => {
                info!("Successfully completed commit workflow");
                Ok(v_str(&format!("Created and pushed commit: {}", message)))
            }
            Err(e) => {
                error!("Failed to execute commit workflow: {}", e);
                Err(WorkerError::RequestError(format!("Failed to execute commit workflow: {}", e)))
            }
        }
    }
    
    /// Execute pull workflow with detailed analysis
    pub fn execute_pull_workflow(
        &self,
        repo: &GitRepository,
        dry_run: bool,
    ) -> Result<Var, WorkerError> {
        use crate::git::operations::commit_ops::CommitOps;
        use crate::git::operations::remote_ops::RemoteOps;
        
        // First, fetch the latest changes from remote
        let ssh_key_path = self.object_handler.config.ssh_key_path();
        let keys_dir = self.object_handler.config.keys_directory();
        
        match RemoteOps::fetch_remote(repo.repo(), ssh_key_path.as_ref().map(|s| s.as_str()), &keys_dir) {
            Ok(_) => {
                info!("Successfully fetched from remote");
            }
            Err(e) => {
                error!("Failed to fetch from remote: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to fetch from remote: {}", e)));
            }
        }
        
        // Get the current branch and upstream
        let current_branch = match RemoteOps::get_current_branch(repo.repo()) {
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
        match CommitOps::get_commits_ahead_behind(repo.repo(), &current_branch, &upstream_branch) {
            Ok((_ahead, behind)) => {
                info!("Branch is {} commits behind upstream", behind);
                
                if behind == 0 {
                    info!("No commits to pull");
                    let empty_result = super::types::PullResult {
                        commit_results: Vec::new(),
                    };
                    return Ok(v_list(&self.pull_result_to_moo_vars(empty_result)));
                }
                
                // Get commits that will be pulled (only remote commits not in local)
                let commits_to_pull = match CommitOps::get_commits_to_pull(repo.repo(), &current_branch, &upstream_branch) {
                    Ok(commits) => commits,
                    Err(e) => {
                        error!("Failed to get commits to pull from {} to {}: {}", current_branch, upstream_branch, e);
                        return Err(WorkerError::RequestError(format!("Failed to get commits to pull: {}", e)));
                    }
                };
                
                if dry_run {
                    info!("Performing pull dry run analysis");
                    // For dry run, analyze the changes without executing
                    self.analyze_pull_impact_dry_run(repo, &upstream_branch, &commits_to_pull)
                        .map(|result| v_list(&self.pull_result_to_moo_vars(result)))
                } else {
                    info!("Executing pull with detailed analysis");
                    // For live pull, execute with detailed analysis
                    self.execute_pull_with_analysis(repo, &upstream_branch, &commits_to_pull)
                        .map(|result| v_list(&self.pull_result_to_moo_vars(result)))
                }
            }
            Err(e) => {
                error!("Failed to check commits ahead/behind: {}", e);
                Err(WorkerError::RequestError(format!("Failed to check commits ahead/behind: {}", e)))
            }
        }
    }
    
    /// Stash current changes using ObjDef models
    pub fn stash_changes(&self, repo: &GitRepository) -> Result<Vec<moor_compiler::ObjectDefinition>, WorkerError> {
        info!("Stashing current changes using ObjDef models");
        
        let mut stashed_objects = Vec::new();
        let objects_dir = self.object_handler.config.objects_directory();
        let objects_path = repo.work_dir().join(objects_dir);
        
        // Find all .moo files that have changes
        let moo_files = match self.object_handler.find_moo_files(&objects_path) {
            Ok(files) => files,
            Err(e) => {
                error!("Failed to find .moo files: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to find .moo files: {}", e)));
            }
        };
        
        // Load each object that has changes
        for file_path in moo_files {
            if let Some(object_name) = PathUtils::extract_object_name_from_path(file_path.to_str().unwrap_or("")) {
                // Check if this file has changes
                if repo.file_has_changes(&file_path) {
                    match repo.read_file(&file_path) {
                        Ok(content) => {
                            match self.object_handler.parse_object_dump(&content) {
                                Ok(object_def) => {
                                    info!("Stashing object: {}", object_name);
                                    stashed_objects.push(object_def);
                                }
                                Err(e) => {
                                    error!("Failed to parse object dump for {}: {}", object_name, e);
                                    // Continue with other objects
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to read file {}: {}", file_path.display(), e);
                            // Continue with other files
                        }
                    }
                }
            }
        }
        
        info!("Stashed {} objects", stashed_objects.len());
        Ok(stashed_objects)
    }
    
    /// Replay stashed changes after pull
    pub fn replay_stashed_changes(&self, repo: &GitRepository, stashed_objects: Vec<moor_compiler::ObjectDefinition>) -> Result<(), WorkerError> {
        info!("Replaying {} stashed objects", stashed_objects.len());
        
        for mut object_def in stashed_objects {
            // Load meta configuration
            let meta_full_path = PathUtils::object_meta_path(repo.work_dir(), &self.object_handler.config, &object_def.name);
            let meta_config = match self.object_handler.load_or_create_meta_config(&meta_full_path) {
                Ok(config) => config,
                Err(e) => {
                    error!("Failed to load meta config for {}: {}", object_def.name, e);
                    continue;
                }
            };
            
            // Apply meta configuration filtering
            self.object_handler.apply_meta_config(&mut object_def, &meta_config);
            
            // Convert back to dump format
            match self.object_handler.to_dump(&object_def) {
                Ok(filtered_dump) => {
                    // Write the filtered object
                    let object_path = PathUtils::object_path(repo.work_dir(), &self.object_handler.config, &object_def.name);
                    
                    if let Err(e) = repo.write_file(&object_path, &filtered_dump) {
                        error!("Failed to write object {}: {}", object_def.name, e);
                        continue;
                    }
                    
                    // Add to git
                    if let Err(e) = repo.add_file(&object_path) {
                        error!("Failed to add object {} to git: {}", object_def.name, e);
                        continue;
                    }
                    
                    info!("Replayed object: {}", object_def.name);
                }
                Err(e) => {
                    error!("Failed to convert object {} to dump: {}", object_def.name, e);
                    continue;
                }
            }
        }
        
        info!("Successfully replayed all stashed changes");
        Ok(())
    }
}
