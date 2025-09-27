use tracing::{info, error};
use moor_var::{Var, v_str, v_map, v_list, v_obj};
use crate::config::Config;
use crate::git::GitRepository;
use crate::utils::PathUtils;
use super::types::{PullResult, ObjectChanges, ChangeStatus, CommitInfo};
use super::object_handler::ObjectHandler;
use moor_common::tasks::WorkerError;

/// Handles complex VCS workflows like pull, rebase, and merge operations
pub struct WorkflowHandler {
    object_handler: ObjectHandler,
}

impl WorkflowHandler {
    pub fn new(config: Config) -> Self {
        Self {
            object_handler: ObjectHandler::new(config),
        }
    }

    /// Execute pull with detailed analysis of changes
    pub fn execute_pull_with_analysis(&self, repo: &GitRepository, upstream_branch: &str, commits_to_pull: &[CommitInfo]) -> Result<PullResult, WorkerError> {
        info!("Executing pull with detailed analysis");
        
        let mut result = PullResult {
            modified_objects: Vec::new(),
            deleted_objects: Vec::new(),
            added_objects: Vec::new(),
            renamed_objects: Vec::new(),
            changes: Vec::new(),
            commits_behind: commits_to_pull.to_vec(),
        };
        
        // Analyze each commit to build the detailed change information
        for commit in commits_to_pull {
            info!("Analyzing commit: {} - {}", commit.id, commit.message);
            
            match repo.get_commit_changes(&commit.full_id) {
                Ok(changes) => {
                    for change in changes {
                        if let Some(_) = PathUtils::extract_object_name_from_path(&change.path) {
                            match change.status {
                                ChangeStatus::Added => {
                                    // Load the new object to get its OID
                                    if let Ok(content) = repo.get_file_content_at_commit(&commit.full_id, &change.path) {
                                        if let Ok(new_obj) = self.object_handler.parse_object_dump(&content) {
                                            result.added_objects.push(new_obj.oid);
                                            
                                            // Analyze changes for new object (no old version)
                                            let obj_changes = self.analyze_object_changes(None, &new_obj);
                                            result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Modified => {
                                    // Load both old and new versions to compare
                                    let old_content = repo.get_file_content_at_commit("HEAD", &change.path).ok();
                                    let new_content = repo.get_file_content_at_commit(&commit.full_id, &change.path);
                                    
                                    if let Ok(new_content) = new_content {
                                        if let Ok(new_obj) = self.object_handler.parse_object_dump(&new_content) {
                                            result.modified_objects.push(new_obj.oid);
                                            
                                            // Analyze changes
                                            let old_obj = if let Some(old_content) = old_content {
                                                self.object_handler.parse_object_dump(&old_content).ok()
                                            } else {
                                                None
                                            };
                                            
                                            let obj_changes = self.analyze_object_changes(old_obj.as_ref(), &new_obj);
                                            result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Deleted => {
                                    // For deleted objects, we need to get the OID from the previous version
                                    if let Ok(old_content) = repo.get_file_content_at_commit("HEAD", &change.path) {
                                        if let Ok(old_obj) = self.object_handler.parse_object_dump(&old_content) {
                                            result.deleted_objects.push(old_obj.oid);
                                            
                                            // For deleted objects, all verbs and properties are "deleted"
                                            let obj_changes = ObjectChanges {
                                                obj_id: old_obj.oid,
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
                                            result.changes.push(obj_changes);
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
                                                result.renamed_objects.push(new_obj.oid);
                                                
                                                // Analyze changes (renamed objects are treated as modified)
                                                let old_obj = if let Some(old_content) = old_content {
                                                    self.object_handler.parse_object_dump(&old_content).ok()
                                                } else {
                                                    None
                                                };
                                                
                                                let obj_changes = self.analyze_object_changes(old_obj.as_ref(), &new_obj);
                                                result.changes.push(obj_changes);
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
        }
        
        // Now perform the actual rebase
        match self.rebase_with_auto_resolution(repo, upstream_branch) {
            Ok(_) => {
                info!("Successfully completed rebase");
                Ok(result)
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
        
        let mut result = PullResult {
            modified_objects: Vec::new(),
            deleted_objects: Vec::new(),
            added_objects: Vec::new(),
            renamed_objects: Vec::new(),
            changes: Vec::new(),
            commits_behind: commits_to_pull.to_vec(),
        };
        
        // Analyze each commit to build the detailed change information (without executing)
        for commit in commits_to_pull {
            info!("Analyzing commit for dry run: {} - {}", commit.id, commit.message);
            
            match repo.get_commit_changes(&commit.full_id) {
                Ok(changes) => {
                    for change in changes {
                        if let Some(_) = PathUtils::extract_object_name_from_path(&change.path) {
                            match change.status {
                                ChangeStatus::Added => {
                                    // Load the new object to get its OID
                                    if let Ok(content) = repo.get_file_content_at_commit(&commit.full_id, &change.path) {
                                        if let Ok(new_obj) = self.object_handler.parse_object_dump(&content) {
                                            result.added_objects.push(new_obj.oid);
                                            
                                            // Analyze changes for new object (no old version)
                                            let obj_changes = self.analyze_object_changes(None, &new_obj);
                                            result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Modified => {
                                    // Load both old and new versions to compare
                                    let old_content = repo.get_file_content_at_commit("HEAD", &change.path).ok();
                                    let new_content = repo.get_file_content_at_commit(&commit.full_id, &change.path);
                                    
                                    if let Ok(new_content) = new_content {
                                        if let Ok(new_obj) = self.object_handler.parse_object_dump(&new_content) {
                                            result.modified_objects.push(new_obj.oid);
                                            
                                            // Analyze changes
                                            let old_obj = if let Some(old_content) = old_content {
                                                self.object_handler.parse_object_dump(&old_content).ok()
                                            } else {
                                                None
                                            };
                                            
                                            let obj_changes = self.analyze_object_changes(old_obj.as_ref(), &new_obj);
                                            result.changes.push(obj_changes);
                                        }
                                    }
                                }
                                ChangeStatus::Deleted => {
                                    // For deleted objects, we need to get the OID from the previous version
                                    if let Ok(old_content) = repo.get_file_content_at_commit("HEAD", &change.path) {
                                        if let Ok(old_obj) = self.object_handler.parse_object_dump(&old_content) {
                                            result.deleted_objects.push(old_obj.oid);
                                            
                                            // For deleted objects, all verbs and properties are "deleted"
                                            let obj_changes = ObjectChanges {
                                                obj_id: old_obj.oid,
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
                                            result.changes.push(obj_changes);
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
                                                result.renamed_objects.push(new_obj.oid);
                                                
                                                // Analyze changes (renamed objects are treated as modified)
                                                let old_obj = if let Some(old_content) = old_content {
                                                    self.object_handler.parse_object_dump(&old_content).ok()
                                                } else {
                                                    None
                                                };
                                                
                                                let obj_changes = self.analyze_object_changes(old_obj.as_ref(), &new_obj);
                                                result.changes.push(obj_changes);
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
        }
        
        Ok(result)
    }

    /// Analyze detailed changes between two object definitions
    fn analyze_object_changes(&self, old_obj: Option<&moor_compiler::ObjectDefinition>, new_obj: &moor_compiler::ObjectDefinition) -> ObjectChanges {
        let mut changes = ObjectChanges {
            obj_id: new_obj.oid,
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
        // Convert object lists to v_obj variables
        let modified_objects_vars: Vec<Var> = result.modified_objects.iter().map(|&oid| v_obj(oid)).collect();
        let deleted_objects_vars: Vec<Var> = result.deleted_objects.iter().map(|&oid| v_obj(oid)).collect();
        let added_objects_vars: Vec<Var> = result.added_objects.iter().map(|&oid| v_obj(oid)).collect();
        let renamed_objects_vars: Vec<Var> = result.renamed_objects.iter().map(|&oid| v_obj(oid)).collect();
        
        // Convert changes to MOO format
        let changes_vars: Vec<Var> = result.changes.iter().map(|change| {
            let modified_verbs_vars: Vec<Var> = change.modified_verbs.iter().map(|v| v_str(v)).collect();
            let modified_props_vars: Vec<Var> = change.modified_props.iter().map(|p| v_str(p)).collect();
            let deleted_verbs_vars: Vec<Var> = change.deleted_verbs.iter().map(|v| v_str(v)).collect();
            let deleted_props_vars: Vec<Var> = change.deleted_props.iter().map(|p| v_str(p)).collect();
            
            v_map(&[
                (v_str("obj_id"), v_obj(change.obj_id)),
                (v_str("modified_verbs"), v_list(&modified_verbs_vars)),
                (v_str("modified_props"), v_list(&modified_props_vars)),
                (v_str("deleted_verbs"), v_list(&deleted_verbs_vars)),
                (v_str("deleted_props"), v_list(&deleted_props_vars)),
            ])
        }).collect();
        
        // Convert commits to MOO format
        let commits_vars: Vec<Var> = result.commits_behind.iter().map(|commit| {
            v_map(&[
                (v_str("commit_id"), v_str(&commit.id)),
                (v_str("author"), v_str(&commit.author)),
                (v_str("message"), v_str(&commit.message)),
            ])
        }).collect();
        
        // Create the main result map
        let result_map = v_map(&[
            (v_str("modified_objects"), v_list(&modified_objects_vars)),
            (v_str("deleted_objects"), v_list(&deleted_objects_vars)),
            (v_str("added_objects"), v_list(&added_objects_vars)),
            (v_str("renamed_objects"), v_list(&renamed_objects_vars)),
            (v_str("changes"), v_list(&changes_vars)),
            (v_str("commits_behind"), v_list(&commits_vars)),
        ]);
        
        vec![result_map]
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
        use crate::git::operations::pull_ops::PullOps;
        
        match PullOps::pull_with_rebase(repo.repo(), &self.object_handler.config, dry_run) {
            Ok(_) => {
                if dry_run {
                    info!("Successfully analyzed pull impact for dry run");
                    // For dry run, we need to do the analysis without executing
                    self.analyze_pull_impact_dry_run(repo, "origin/main", &[])
                        .map(|result| v_list(&self.pull_result_to_moo_vars(result)))
                } else {
                    info!("Successfully completed pull workflow");
                    // Return empty result for now - could be enhanced to return actual changes
                    let empty_result = super::types::PullResult {
                        modified_objects: Vec::new(),
                        deleted_objects: Vec::new(),
                        added_objects: Vec::new(),
                        renamed_objects: Vec::new(),
                        changes: Vec::new(),
                        commits_behind: Vec::new(),
                    };
                    Ok(v_list(&self.pull_result_to_moo_vars(empty_result)))
                }
            }
            Err(e) => {
                error!("Failed to execute pull workflow: {}", e);
                Err(WorkerError::RequestError(format!("Failed to execute pull workflow: {}", e)))
            }
        }
    }
}
