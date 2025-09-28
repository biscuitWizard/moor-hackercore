use tracing::{info, error};
use moor_var::{Var, v_str, v_map, v_list, v_obj};
use crate::config::Config;
use crate::git::GitRepository;
use crate::git::operations::stash_ops::{StashOps, StashedObject};
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
                                            let (obj_changes, _, _) = self.analyze_object_changes(None, &new_obj, Some(&change.path));
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
                                            
                                            let (obj_changes, _, _) = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(&change.path));
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
                                                
                                                let (obj_changes, _, _) = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(&change.path));
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
                                            let (obj_changes, _, _) = self.analyze_object_changes(None, &new_obj, Some(&change.path));
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
                                            
                                            let (obj_changes, _, _) = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(&change.path));
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
                                                
                                                let (obj_changes, _, _) = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(&change.path));
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
    fn analyze_object_changes(&self, old_obj: Option<&moor_compiler::ObjectDefinition>, new_obj: &moor_compiler::ObjectDefinition, filename: Option<&str>) -> (ObjectChanges, Vec<String>, Vec<String>) {
        let mut changes = ObjectChanges {
            obj_id: self.object_to_var(new_obj, filename),
            modified_verbs: Vec::new(),
            modified_props: Vec::new(),
            deleted_verbs: Vec::new(),
            deleted_props: Vec::new(),
        };
        
        let mut added_verbs = Vec::new();
        let mut added_props = Vec::new();
        
        if let Some(old) = old_obj {
            // Compare verbs by content, not just names
            let old_verbs_map: std::collections::HashMap<String, &moor_compiler::ObjVerbDef> = old.verbs.iter()
                .map(|v| {
                    let name = v.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ");
                    (name, v)
                })
                .collect();
            
            let new_verbs_map: std::collections::HashMap<String, &moor_compiler::ObjVerbDef> = new_obj.verbs.iter()
                .map(|v| {
                    let name = v.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ");
                    (name, v)
                })
                .collect();
            
            // Find modified verbs (exist in both but content changed)
            for (verb_name, new_verb) in &new_verbs_map {
                if let Some(old_verb) = old_verbs_map.get(verb_name) {
                    // Verb exists in both - check if content changed
                    let content_changed = self.verb_content_changed(old_verb, new_verb);
                    info!("Verb '{}' content changed: {}", verb_name, content_changed);
                    if content_changed {
                        changes.modified_verbs.push(verb_name.clone());
                    }
                } else {
                    info!("Verb '{}' is new (not in old object)", verb_name);
                    added_verbs.push(verb_name.clone());
                }
            }
            
            // Find deleted verbs
            for verb_name in old_verbs_map.keys() {
                if !new_verbs_map.contains_key(verb_name) {
                    changes.deleted_verbs.push(verb_name.clone());
                }
            }
            
            // Compare properties by content
            let old_props_map: std::collections::HashMap<String, &moor_compiler::ObjPropDef> = old.property_definitions.iter()
                .map(|p| (p.name.to_string(), p))
                .collect();
            let old_overrides_map: std::collections::HashMap<String, &moor_compiler::ObjPropOverride> = old.property_overrides.iter()
                .map(|p| (p.name.to_string(), p))
                .collect();
            
            let new_props_map: std::collections::HashMap<String, &moor_compiler::ObjPropDef> = new_obj.property_definitions.iter()
                .map(|p| (p.name.to_string(), p))
                .collect();
            let new_overrides_map: std::collections::HashMap<String, &moor_compiler::ObjPropOverride> = new_obj.property_overrides.iter()
                .map(|p| (p.name.to_string(), p))
                .collect();
            
            // Find modified properties (exist in both but content changed)
            for (prop_name, new_prop) in &new_props_map {
                if let Some(old_prop) = old_props_map.get(prop_name) {
                    // Property exists in both - check if content changed
                    if self.property_content_changed(old_prop, new_prop) {
                        changes.modified_props.push(prop_name.clone());
                    }
                } else {
                    // New property
                    added_props.push(prop_name.clone());
                }
            }
            
            for (prop_name, new_override) in &new_overrides_map {
                if let Some(old_override) = old_overrides_map.get(prop_name) {
                    // Override exists in both - check if content changed
                    if self.property_override_content_changed(old_override, new_override) {
                        changes.modified_props.push(prop_name.clone());
                    }
                } else if !old_props_map.contains_key(prop_name) {
                    // New override (not overriding an existing property)
                    added_props.push(prop_name.clone());
                }
            }
            
            // Find deleted properties
            for prop_name in old_props_map.keys() {
                if !new_props_map.contains_key(prop_name) && !new_overrides_map.contains_key(prop_name) {
                    changes.deleted_props.push(prop_name.clone());
                }
            }
            
            for prop_name in old_overrides_map.keys() {
                if !new_overrides_map.contains_key(prop_name) && !new_props_map.contains_key(prop_name) {
                    changes.deleted_props.push(prop_name.clone());
                }
            }
        } else {
            // New object - all verbs and properties are "added"
            for verb in &new_obj.verbs {
                let verb_name = verb.names.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(" ");
                added_verbs.push(verb_name);
            }
            
            for prop in &new_obj.property_definitions {
                added_props.push(prop.name.to_string());
            }
            
            for prop in &new_obj.property_overrides {
                added_props.push(prop.name.to_string());
            }
        }
        
        (changes, added_verbs, added_props)
    }
    
    /// Check if verb content has changed
    fn verb_content_changed(&self, old_verb: &moor_compiler::ObjVerbDef, new_verb: &moor_compiler::ObjVerbDef) -> bool {
        // Compare verb program
        if old_verb.program != new_verb.program {
            info!("  Program changed: {:?} != {:?}", old_verb.program, new_verb.program);
            return true;
        }
        
        // Compare verb flags
        if old_verb.flags != new_verb.flags {
            info!("  Flags changed: {:?} != {:?}", old_verb.flags, new_verb.flags);
            return true;
        }
        
        // Compare verb owner
        if old_verb.owner != new_verb.owner {
            info!("  Owner changed: {:?} != {:?}", old_verb.owner, new_verb.owner);
            return true;
        }
        
        // Compare verb argspec
        if old_verb.argspec != new_verb.argspec {
            info!("  Argspec changed: {:?} != {:?}", old_verb.argspec, new_verb.argspec);
            return true;
        }
        
        false
    }
    
    /// Check if property content has changed
    fn property_content_changed(&self, old_prop: &moor_compiler::ObjPropDef, new_prop: &moor_compiler::ObjPropDef) -> bool {
        // Compare property value
        if old_prop.value != new_prop.value {
            return true;
        }
        
        // Compare property permissions
        if old_prop.perms != new_prop.perms {
            return true;
        }
        
        false
    }
    
    /// Check if property override content has changed
    fn property_override_content_changed(&self, old_override: &moor_compiler::ObjPropOverride, new_override: &moor_compiler::ObjPropOverride) -> bool {
        // Compare override value
        if old_override.value != new_override.value {
            return true;
        }
        
        // Compare override permissions
        if old_override.perms_update != new_override.perms_update {
            return true;
        }
        
        false
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
        
        // Step 1: Check for uncommitted changes and stash them before pull (only for non-dry-run)
        let stashed_objects = if !dry_run {
            match self.stash_changes(repo) {
                Ok(objects) => {
                    if !objects.is_empty() {
                        info!("Stashed {} uncommitted changes before pull", objects.len());
                    }
                    objects
                }
                Err(e) => {
                    error!("Failed to stash changes before pull: {}", e);
                    return Err(e);
                }
            }
        } else {
            Vec::new()
        };

        // Step 2: Fetch the latest changes from remote
        let ssh_key_path = self.object_handler.config.ssh_key_path();
        let keys_dir = self.object_handler.config.keys_directory();
        
        match RemoteOps::fetch_remote(repo.repo(), ssh_key_path.as_ref().map(|s| s.as_str()), &keys_dir) {
            Ok(_) => {
                info!("Successfully fetched from remote");
            }
            Err(e) => {
                error!("Failed to fetch from remote: {}", e);
                // If we stashed changes, we need to replay them before returning error
                if !dry_run && !stashed_objects.is_empty() {
                    if let Err(replay_err) = self.replay_stashed_changes(repo, stashed_objects) {
                        error!("Failed to replay stashed changes after fetch error: {}", replay_err);
                    }
                }
                return Err(WorkerError::RequestError(format!("Failed to fetch from remote: {}", e)));
            }
        }
        
        // Step 3: Get the current branch and upstream
        let current_branch = match RemoteOps::get_current_branch(repo.repo()) {
            Ok(Some(branch)) => branch,
            Ok(None) => {
                error!("No current branch found");
                // If we stashed changes, we need to replay them before returning error
                if !dry_run && !stashed_objects.is_empty() {
                    if let Err(replay_err) = self.replay_stashed_changes(repo, stashed_objects) {
                        error!("Failed to replay stashed changes after branch error: {}", replay_err);
                    }
                }
                return Err(WorkerError::RequestError("No current branch found".to_string()));
            }
            Err(e) => {
                error!("Failed to get current branch: {}", e);
                // If we stashed changes, we need to replay them before returning error
                if !dry_run && !stashed_objects.is_empty() {
                    if let Err(replay_err) = self.replay_stashed_changes(repo, stashed_objects) {
                        error!("Failed to replay stashed changes after branch error: {}", replay_err);
                    }
                }
                return Err(WorkerError::RequestError(format!("Failed to get current branch: {}", e)));
            }
        };
        
        let upstream_branch = format!("origin/{}", current_branch);
        info!("Current branch: {}, upstream: {}", current_branch, upstream_branch);
        
        // Step 4: Check if there are any commits to pull
        match CommitOps::get_commits_ahead_behind(repo.repo(), &current_branch, &upstream_branch) {
            Ok((_ahead, behind)) => {
                info!("Branch is {} commits behind upstream", behind);
                
                if behind == 0 {
                    info!("No commits to pull");
                    // If we stashed changes, replay them since there's nothing to pull
                    if !dry_run && !stashed_objects.is_empty() {
                        match self.replay_stashed_changes(repo, stashed_objects) {
                            Ok(_) => {
                                info!("Successfully replayed stashed changes (no commits to pull)");
                            }
                            Err(e) => {
                                error!("Failed to replay stashed changes (no commits to pull): {}", e);
                                return Err(e);
                            }
                        }
                    }
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
                        // If we stashed changes, we need to replay them before returning error
                        if !dry_run && !stashed_objects.is_empty() {
                            if let Err(replay_err) = self.replay_stashed_changes(repo, stashed_objects) {
                                error!("Failed to replay stashed changes after commit error: {}", replay_err);
                            }
                        }
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
                    let pull_result = self.execute_pull_with_analysis(repo, &upstream_branch, &commits_to_pull);
                    
                    // Step 5: After successful pull, replay stashed changes
                    match pull_result {
                        Ok(result) => {
                            // Replay stashed changes after successful pull
                            let stashed_count = stashed_objects.len();
                            if !stashed_objects.is_empty() {
                                match self.replay_stashed_changes(repo, stashed_objects) {
                                    Ok(_) => {
                                        info!("Successfully replayed {} stashed changes after pull", stashed_count);
                                    }
                                    Err(e) => {
                                        error!("Failed to replay stashed changes after pull: {}", e);
                                        return Err(e);
                                    }
                                }
                            }
                            Ok(v_list(&self.pull_result_to_moo_vars(result)))
                        }
                        Err(e) => {
                            // If pull failed, replay stashed changes to restore working state
                            if !stashed_objects.is_empty() {
                                if let Err(replay_err) = self.replay_stashed_changes(repo, stashed_objects) {
                                    error!("Failed to replay stashed changes after pull failure: {}", replay_err);
                                }
                            }
                            Err(e)
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to check commits ahead/behind: {}", e);
                // If we stashed changes, we need to replay them before returning error
                if !dry_run && !stashed_objects.is_empty() {
                    if let Err(replay_err) = self.replay_stashed_changes(repo, stashed_objects) {
                        error!("Failed to replay stashed changes after ahead/behind error: {}", replay_err);
                    }
                }
                Err(WorkerError::RequestError(format!("Failed to check commits ahead/behind: {}", e)))
            }
        }
    }
    
    /// Stash current changes using ObjDef models
    pub fn stash_changes(&self, repo: &GitRepository) -> Result<Vec<StashedObject>, WorkerError> {
        StashOps::stash_changes(repo, &self.object_handler)
    }
    
    /// Replay stashed changes after pull
    pub fn replay_stashed_changes(&self, repo: &GitRepository, stashed_objects: Vec<StashedObject>) -> Result<(), WorkerError> {
        StashOps::replay_stashed_changes(repo, &self.object_handler, stashed_objects)
    }
    
    /// Get current changed files in detailed format
    pub fn get_current_changed_files(&self, repo: &GitRepository) -> Result<Var, WorkerError> {
        info!("Getting current changed files in detailed format");
        
        let mut changed_objects = Vec::new();
        let objects_dir = self.object_handler.config.objects_directory();
        
        // Use existing StatusOps to get status information
        let status_lines = match crate::git::operations::status_ops::StatusOps::get_status(repo.repo(), repo.work_dir()) {
            Ok(lines) => lines,
            Err(e) => {
                error!("Failed to get git status: {}", e);
                return Err(WorkerError::RequestError(format!("Failed to get git status: {}", e)));
            }
        };
        
        // Parse status lines to categorize files
        let mut added_files = Vec::new();
        let mut deleted_files = Vec::new();
        let mut modified_files = Vec::new();
        let mut renamed_files = Vec::new();
        
        for line in status_lines {
            if let Some(colon_pos) = line.find(':') {
                let change_type = &line[..colon_pos];
                let file_path = &line[colon_pos + 1..].trim();
                
                // Only process .moo files in the objects directory
                if file_path.ends_with(".moo") && file_path.starts_with(objects_dir.as_str()) {
                    if let Some(object_name) = PathUtils::extract_object_name_from_path(file_path) {
                        match change_type {
                            "Added" => added_files.push((object_name, file_path.to_string())),
                            "Deleted" => deleted_files.push((object_name, file_path.to_string())),
                            "Modified" => modified_files.push((object_name, file_path.to_string())),
                            "Renamed" => {
                                // Parse rename format: "Renamed: old_path -> new_path"
                                if let Some(arrow_pos) = file_path.find(" -> ") {
                                    let old_path = &file_path[..arrow_pos];
                                    let new_path = &file_path[arrow_pos + 4..];
                                    if let Some(old_name) = PathUtils::extract_object_name_from_path(old_path) {
                                        if let Some(new_name) = PathUtils::extract_object_name_from_path(new_path) {
                                            renamed_files.push((old_name, old_path.to_string(), new_name, new_path.to_string()));
                                        }
                                    }
                                }
                            }
                            _ => {} // Ignore other types
                        }
                    }
                }
            }
        }
        
        // Process the collected files
        self.process_changed_files(repo, added_files, deleted_files, modified_files, renamed_files, &mut changed_objects)?;
        
        info!("Found {} changed objects", changed_objects.len());
        Ok(v_list(&changed_objects))
    }
    
    /// Process changed files and handle renames
    fn process_changed_files(
        &self,
        repo: &GitRepository,
        added_files: Vec<(String, String)>,
        deleted_files: Vec<(String, String)>,
        modified_files: Vec<(String, String)>,
        renamed_files: Vec<(String, String, String, String)>,
        changed_objects: &mut Vec<Var>,
    ) -> Result<(), WorkerError> {
        // Handle renamed files first (already detected by StatusOps)
        for (old_name, old_path, _new_name, new_path) in &renamed_files {
            // Parse the new file content
            let new_full_path = repo.work_dir().join(new_path);
            if let Ok(content) = repo.read_file(&new_full_path) {
                if let Ok(new_obj) = self.object_handler.parse_object_dump(&content) {
                    // Get the old object from git history for comparison
                    let head_oid = match crate::git::operations::commit_ops::CommitOps::get_head_commit(repo.repo()) {
                        Ok(head_commit) => head_commit.id().to_string(),
                        Err(_) => continue,
                    };
                    let old_obj = match repo.get_file_content_at_commit(&head_oid, old_path) {
                        Ok(old_content) => self.object_handler.parse_object_dump(&old_content).ok(),
                        Err(_) => None,
                    };
                    
                    let obj_id = self.object_to_var(&new_obj, Some(new_path));
                    let old_obj_id = if let Some(ref old_obj) = old_obj {
                        self.object_to_var(old_obj, Some(old_path))
                    } else {
                        v_str(old_name)
                    };
                    
                    // Analyze changes for renamed object
                    let (changes, added_verbs, added_props) = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(new_path));
                    
                    let change_map = vec![
                        (v_str("obj_id"), obj_id),
                        (v_str("operation"), v_str("renamed")),
                        (v_str("old_obj_id"), old_obj_id),
                        (v_str("modified_verbs"), v_list(&changes.modified_verbs.iter().map(|v| v_str(v)).collect::<Vec<_>>())),
                        (v_str("deleted_verbs"), v_list(&changes.deleted_verbs.iter().map(|v| v_str(v)).collect::<Vec<_>>())),
                        (v_str("added_verbs"), v_list(&added_verbs.iter().map(|v| v_str(v)).collect::<Vec<_>>())),
                        (v_str("modified_props"), v_list(&changes.modified_props.iter().map(|p| v_str(p)).collect::<Vec<_>>())),
                        (v_str("deleted_props"), v_list(&changes.deleted_props.iter().map(|p| v_str(p)).collect::<Vec<_>>())),
                        (v_str("added_props"), v_list(&added_props.iter().map(|p| v_str(p)).collect::<Vec<_>>())),
                    ];
                    
                    changed_objects.push(v_map(&change_map));
                }
            }
        }
        
        // Handle added files (new files, not renames)
        for (_object_name, path) in &added_files {
            let full_path = repo.work_dir().join(path);
            if let Ok(content) = repo.read_file(&full_path) {
                if let Ok(new_obj) = self.object_handler.parse_object_dump(&content) {
                    let obj_id = self.object_to_var(&new_obj, Some(path));
                    
                    // For new objects, all verbs and properties are "added"
                    let (changes, added_verbs, added_props) = self.analyze_object_changes(None, &new_obj, Some(path));
                    
                    let change_map = vec![
                        (v_str("obj_id"), obj_id),
                        (v_str("operation"), v_str("added")),
                        (v_str("modified_verbs"), v_list(&changes.modified_verbs.iter().map(|v| v_str(v)).collect::<Vec<_>>())),
                        (v_str("deleted_verbs"), v_list(&[])),
                        (v_str("added_verbs"), v_list(&added_verbs.iter().map(|v| v_str(v)).collect::<Vec<_>>())),
                        (v_str("modified_props"), v_list(&changes.modified_props.iter().map(|p| v_str(p)).collect::<Vec<_>>())),
                        (v_str("deleted_props"), v_list(&[])),
                        (v_str("added_props"), v_list(&added_props.iter().map(|p| v_str(p)).collect::<Vec<_>>())),
                    ];
                    
                    changed_objects.push(v_map(&change_map));
                }
            }
        }
        
        // Handle deleted files
        for (_object_name, path) in &deleted_files {
                    // Get the old object from git history
                    let head_oid = match crate::git::operations::commit_ops::CommitOps::get_head_commit(repo.repo()) {
                        Ok(head_commit) => head_commit.id().to_string(),
                        Err(_) => continue,
                    };
                    if let Ok(old_content) = repo.get_file_content_at_commit(&head_oid, path) {
                if let Ok(old_obj) = self.object_handler.parse_object_dump(&old_content) {
                    let obj_id = self.object_to_var(&old_obj, Some(path));
                    
                    let change_map = vec![
                        (v_str("obj_id"), obj_id),
                        (v_str("operation"), v_str("deleted")),
                    ];
                    
                    changed_objects.push(v_map(&change_map));
                }
            }
        }
        
        // Handle modified files
        for (_, path) in &modified_files {
            let full_path = repo.work_dir().join(path);
            if let Ok(content) = repo.read_file(&full_path) {
                if let Ok(new_obj) = self.object_handler.parse_object_dump(&content) {
                    // Get the old object from git history for comparison
                    let old_obj = match crate::git::operations::commit_ops::CommitOps::get_head_commit(repo.repo()) {
                        Ok(head_commit) => {
                            let head_oid = head_commit.id().to_string();
                            match repo.get_file_content_at_commit(&head_oid, path) {
                                Ok(old_content) => {
                                    info!("Successfully loaded old content for {} from HEAD commit {}", path, head_oid);
                                    self.object_handler.parse_object_dump(&old_content).ok()
                                },
                                Err(e) => {
                                    error!("Failed to load old content for {} from HEAD commit {}: {}", path, head_oid, e);
                                    None
                                },
                            }
                        },
                        Err(e) => {
                            error!("Failed to get HEAD commit: {}", e);
                            None
                        },
                    };
                    
                    let obj_id = self.object_to_var(&new_obj, Some(path));
                    
                    // Analyze changes
                    info!("Comparing old_obj: {:?}, new_obj: {:?}", old_obj.is_some(), new_obj.verbs.len());
                    let (changes, added_verbs, added_props) = self.analyze_object_changes(old_obj.as_ref(), &new_obj, Some(path));
                    
                    let change_map = vec![
                        (v_str("obj_id"), obj_id),
                        (v_str("operation"), v_str("modified")),
                        (v_str("modified_verbs"), v_list(&changes.modified_verbs.iter().map(|v| v_str(v)).collect::<Vec<_>>())),
                        (v_str("deleted_verbs"), v_list(&changes.deleted_verbs.iter().map(|v| v_str(v)).collect::<Vec<_>>())),
                        (v_str("added_verbs"), v_list(&added_verbs.iter().map(|v| v_str(v)).collect::<Vec<_>>())),
                        (v_str("modified_props"), v_list(&changes.modified_props.iter().map(|p| v_str(p)).collect::<Vec<_>>())),
                        (v_str("deleted_props"), v_list(&changes.deleted_props.iter().map(|p| v_str(p)).collect::<Vec<_>>())),
                        (v_str("added_props"), v_list(&added_props.iter().map(|p| v_str(p)).collect::<Vec<_>>())),
                    ];
                    
                    changed_objects.push(v_map(&change_map));
                }
            }
        }
        
        Ok(())
    }
    
}
