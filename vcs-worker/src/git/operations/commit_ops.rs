use tracing::info;
use git2::{Repository, Commit, Signature, ResetType};
use crate::vcs::types::{CommitInfo, CommitChange, ChangeStatus};

/// Commit operations for git repositories
pub struct CommitOps;

impl CommitOps {
    /// Create a commit with the given message
    pub fn create_commit<'a>(
        repo: &'a Repository,
        message: &str,
        author_name: &str,
        author_email: &str,
    ) -> Result<Commit<'a>, Box<dyn std::error::Error>> {
        let signature = Signature::now(author_name, author_email)?;
        
        // Add all changes to the index before committing
        super::file_ops::FileOps::add_all_changes(repo)?;
        
        let mut index = repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        let parent_commit = Self::get_head_commit(repo).ok();
        
        let commit_id = repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_commit.iter().collect::<Vec<_>>(),
        )?;
        
        let commit = repo.find_commit(commit_id)?;
        
        info!("Created commit: {}", commit_id);
        Ok(commit)
    }
    
    /// Get the current HEAD commit
    pub fn get_head_commit(repo: &Repository) -> Result<Commit<'_>, Box<dyn std::error::Error>> {
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit)
    }
    
    /// Get paginated list of commits
    pub fn get_commits(
        repo: &Repository,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
        let limit = limit.unwrap_or(5);
        let offset = offset.unwrap_or(0);
        
        info!("Getting {} commits starting from offset {}", limit, offset);
        
        // Get the HEAD reference
        let head = repo.head()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.push(head.target().unwrap())?;
        revwalk.set_sorting(git2::Sort::TIME)?;
        
        let mut commits = Vec::new();
        let mut count = 0;
        let mut skipped = 0;
        
        for commit_id in revwalk {
            let commit_id = commit_id?;
            
            // Skip commits until we reach the offset
            if skipped < offset {
                skipped += 1;
                continue;
            }
            
            // Stop if we've reached the limit
            if count >= limit {
                break;
            }
            
            let commit = repo.find_commit(commit_id)?;
            let id = commit.id().to_string();
            let short_id = &id[..8];
            let datetime = commit.time();
            let timestamp = datetime.seconds();
            let message = commit.message().unwrap_or("No message").to_string();
            let author = commit.author().name().unwrap_or("Unknown").to_string();
            
            commits.push(CommitInfo {
                id: short_id.to_string(),
                full_id: id,
                datetime: timestamp,
                message: message.trim().to_string(),
                author,
            });
            
            count += 1;
        }
        
        info!("Retrieved {} commits (skipped {}, total requested: {})", count, skipped, limit);
        Ok(commits)
    }
    
    /// Get information about the last commit
    pub fn get_last_commit_info(repo: &Repository) -> Result<Option<CommitInfo>, Box<dyn std::error::Error>> {
        match Self::get_head_commit(repo) {
            Ok(commit) => {
                let id = commit.id().to_string();
                let short_id = &id[..8];
                let datetime = commit.time();
                let timestamp = datetime.seconds();
                let message = commit.message().unwrap_or("No message").to_string();
                let author = commit.author().name().unwrap_or("Unknown").to_string();
                
                Ok(Some(CommitInfo {
                    id: short_id.to_string(),
                    full_id: id,
                    datetime: timestamp,
                    message: message.trim().to_string(),
                    author,
                }))
            }
            Err(_) => Ok(None), // No commits yet
        }
    }
    
    /// Get commits ahead and behind between two branches
    pub fn get_commits_ahead_behind(
        repo: &Repository,
        local_branch: &str,
        remote_branch: &str,
    ) -> Result<(usize, usize), Box<dyn std::error::Error>> {
        let local_oid = repo.refname_to_id(&format!("refs/heads/{}", local_branch))?;
        let remote_oid = repo.refname_to_id(&format!("refs/remotes/{}", remote_branch))?;
        
        let (ahead, behind) = repo.graph_ahead_behind(local_oid, remote_oid)?;
        
        Ok((ahead, behind))
    }
    
    /// Get commits between two references
    pub fn get_commits_between(
        repo: &Repository,
        from: &str,
        to: &str,
    ) -> Result<Vec<CommitInfo>, Box<dyn std::error::Error>> {
        let from_oid = repo.refname_to_id(from)?;
        let to_ref = if to.contains("/") && !to.starts_with("refs/") {
            format!("refs/remotes/{}", to)
        } else {
            to.to_string()
        };
        let to_oid = repo.refname_to_id(&to_ref)?;
        
        let mut revwalk = repo.revwalk()?;
        revwalk.push(to_oid)?;
        revwalk.hide(from_oid)?;
        revwalk.set_sorting(git2::Sort::TIME)?;
        
        let mut commits = Vec::new();
        
        for commit_id in revwalk {
            let commit_id = commit_id?;
            let commit = repo.find_commit(commit_id)?;
            
            let id = commit.id().to_string();
            let short_id = &id[..8];
            let datetime = commit.time();
            let timestamp = datetime.seconds();
            let message = commit.message().unwrap_or("No message").to_string();
            let author = commit.author().name().unwrap_or("Unknown").to_string();
            
            commits.push(CommitInfo {
                id: short_id.to_string(),
                full_id: id,
                datetime: timestamp,
                message: message.trim().to_string(),
                author,
            });
        }
        
        Ok(commits)
    }
    
    /// Get changes in a specific commit
    pub fn get_commit_changes(
        repo: &Repository,
        commit_id: &str,
    ) -> Result<Vec<CommitChange>, Box<dyn std::error::Error>> {
        let oid = commit_id.parse::<git2::Oid>()?;
        let commit = repo.find_commit(oid)?;
        
        let mut changes = Vec::new();
        
        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let parent_tree = parent.tree()?;
            let commit_tree = commit.tree()?;
            
            let diff = repo.diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)?;
            
            for delta in diff.deltas() {
                let old_path = delta.old_file().path().map(|p| p.to_string_lossy().to_string());
                let new_path = delta.new_file().path().map(|p| p.to_string_lossy().to_string());
                
                let status = match delta.status() {
                    git2::Delta::Added => ChangeStatus::Added,
                    git2::Delta::Modified => ChangeStatus::Modified,
                    git2::Delta::Deleted => ChangeStatus::Deleted,
                    git2::Delta::Renamed => ChangeStatus::Renamed,
                    _ => continue,
                };
                
                if let Some(path) = new_path.or_else(|| old_path.clone()) {
                    changes.push(CommitChange {
                        path,
                        old_path,
                        status,
                    });
                }
            }
        } else {
            // First commit - all files are added
            let tree = commit.tree()?;
            let _ = Self::collect_tree_files(repo, &tree, "", &mut changes);
        }
        
        Ok(changes)
    }
    
    /// Recursively collect files from a tree
    fn collect_tree_files(
        repo: &Repository,
        tree: &git2::Tree,
        prefix: &str,
        changes: &mut Vec<CommitChange>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in tree.iter() {
            let name = entry.name().unwrap_or("");
            let path = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", prefix, name)
            };
            
            match entry.kind() {
                Some(git2::ObjectType::Tree) => {
                    let subtree = repo.find_tree(entry.id())?;
                    Self::collect_tree_files(repo, &subtree, &path, changes)?;
                }
                Some(git2::ObjectType::Blob) => {
                    changes.push(CommitChange {
                        path,
                        old_path: None,
                        status: ChangeStatus::Added,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    /// Get file content at a specific commit
    pub fn get_file_content_at_commit(
        repo: &Repository,
        commit_id: &str,
        file_path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let oid = commit_id.parse::<git2::Oid>()?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        
        let entry = tree.get_path(std::path::Path::new(file_path))?;
        let blob = repo.find_blob(entry.id())?;
        
        let content = String::from_utf8_lossy(blob.content()).to_string();
        Ok(content)
    }
    
    /// Rebase onto a specific branch
    pub fn rebase_onto(
        repo: &Repository,
        upstream_branch: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting rebase onto {}", upstream_branch);
        
        let upstream_oid = repo.refname_to_id(&format!("refs/remotes/{}", upstream_branch))?;
        let upstream_commit = repo.find_commit(upstream_oid)?;
        
        // Get the current HEAD
        let head = repo.head()?;
        let head_commit = head.peel_to_commit()?;
        
        // Find the common ancestor
        let merge_base = repo.merge_base(head_commit.id(), upstream_commit.id())?;
        let _base_commit = repo.find_commit(merge_base)?;
        
        // Get commits to rebase (from merge base to HEAD, excluding the base)
        let mut revwalk = repo.revwalk()?;
        revwalk.push(head_commit.id())?;
        revwalk.hide(merge_base)?;
        revwalk.set_sorting(git2::Sort::TIME)?;
        
        let mut commits_to_rebase = Vec::new();
        for commit_id in revwalk {
            let commit_id = commit_id?;
            let commit = repo.find_commit(commit_id)?;
            commits_to_rebase.push(commit);
        }
        
        // Reverse the order to apply commits in chronological order
        commits_to_rebase.reverse();
        
        // Apply each commit on top of the upstream
        let mut current_commit = upstream_commit;
        for commit in commits_to_rebase {
            // Create a new commit with the same changes but new parent
            let tree = commit.tree()?;
            let signature = commit.author();
            let message = commit.message().unwrap_or("No message");
            
            let new_commit_id = repo.commit(
                None,
                &signature,
                &signature,
                message,
                &tree,
                &[&current_commit],
            )?;
            
            current_commit = repo.find_commit(new_commit_id)?;
        }
        
        // Update HEAD to point to the new commit
        let mut head_ref = repo.head()?;
        head_ref.set_target(current_commit.id(), "Rebase completed")?;
        
        // Reset the working tree to match the new HEAD
        let head_obj = repo.find_object(current_commit.id(), None)?;
        repo.reset(&head_obj, ResetType::Hard, None)?;
        
        info!("Successfully completed rebase onto {}", upstream_branch);
        Ok(())
    }
    
    /// Rollback the last commit and restore files to staged state
    pub fn rollback_last_commit(repo: &Repository) -> Result<(), Box<dyn std::error::Error>> {
        info!("Rolling back last commit");
        
        // Get the current HEAD commit
        let head_commit = Self::get_head_commit(repo)?;
        let head_oid = head_commit.id();
        
        // Get the parent commit (the one we want to reset to)
        let parent_commit = match head_commit.parent(0) {
            Ok(parent) => parent,
            Err(_) => {
                // If there's no parent, this is the first commit
                info!("No parent commit found, resetting to empty state");
                let mut index = repo.index()?;
                index.clear()?;
                index.write()?;
                
                // Reset HEAD to point to nothing (unborn branch state)
                repo.set_head_detached(git2::Oid::zero())?;
                return Ok(());
            }
        };
        
        let parent_oid = parent_commit.id();
        info!("Rolling back from commit {} to {}", head_oid, parent_oid);
        
        // Reset HEAD to the parent commit (soft reset to keep changes staged)
        let parent_obj = repo.find_object(parent_oid, None)?;
        repo.reset(&parent_obj, ResetType::Soft, None)?;
        
        info!("Successfully rolled back last commit");
        Ok(())
    }
}
