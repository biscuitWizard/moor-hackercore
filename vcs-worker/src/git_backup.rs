use crate::config::Config;
use crate::database::DatabaseRef;
use crate::providers::index::IndexProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::refs::RefsProvider;
use crate::types::{ObjectInfo, VcsObjectType};
use moor_compiler::{CompileOptions, ObjFileContext, compile_object_definitions};
use moor_objdef::dump_object;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{error, info, warn};

/// Trigger a git backup in a background thread (non-blocking)
/// This is the main entry point called from change operations
pub fn trigger_git_backup(database: DatabaseRef, config: Config) {
    // Check if git backup is configured
    if config.git_backup_repo.is_none() {
        return;
    }

    info!("Triggering git backup in background thread");

    // Spawn a background thread to perform the backup
    std::thread::spawn(move || {
        if let Err(e) = perform_git_backup(database, config) {
            error!("Git backup failed: {}", e);
        }
    });
}

/// Perform the actual git backup (runs in background thread)
fn perform_git_backup(database: DatabaseRef, config: Config) -> Result<(), String> {
    let repo_path = config
        .git_backup_repo
        .as_ref()
        .ok_or_else(|| "No git backup repo configured".to_string())?;

    info!("Starting git backup to: {}", repo_path);

    // Set up the git repository (clone or use existing)
    let work_dir = setup_git_repo(repo_path, &config.git_backup_token)?;

    info!("Git repository ready at: {:?}", work_dir);

    // Get the computed object list from the working index (not all refs)
    // This gives us the current state of objects after applying all changes chronologically
    let all_objects = database
        .index()
        .compute_complete_object_list()
        .map_err(|e| format!("Failed to compute complete object list: {}", e))?;

    // Filter to only MOO objects (not meta objects)
    let moo_objects: Vec<&ObjectInfo> = all_objects
        .iter()
        .filter(|obj_info| obj_info.object_type == VcsObjectType::MooObject)
        .collect();

    info!("Found {} MOO objects to backup from working index", moo_objects.len());

    // Track which files we've written
    let mut written_files = HashSet::new();

    // Dump each object to a file
    for obj_info in &moo_objects {
        match dump_object_to_file(&database, obj_info, &work_dir) {
            Ok(filename) => {
                written_files.insert(filename);
            }
            Err(e) => {
                warn!("Failed to dump object '{}': {}", obj_info.name, e);
            }
        }
    }

    info!("Successfully dumped {} objects", written_files.len());

    // Remove any .moo files that no longer have corresponding objects
    cleanup_old_files(&work_dir, &written_files)?;

    // Commit and push changes
    git_commit_and_push(&work_dir, repo_path, &config.git_backup_token)?;

    info!("Git backup completed successfully");

    Ok(())
}

/// Set up the git repository (clone or init)
fn setup_git_repo(repo_path: &str, token: &Option<String>) -> Result<PathBuf, String> {
    // Determine if this is a remote URL or local path
    let is_remote = repo_path.starts_with("http://") || repo_path.starts_with("https://");

    // Use a temporary directory for the working copy
    let work_dir = if is_remote {
        PathBuf::from("/tmp/vcs-git-backup")
    } else {
        PathBuf::from(repo_path)
    };

    // If it's a remote repo and the directory doesn't exist, clone it
    if is_remote {
        if work_dir.exists() {
            // Directory exists, try to pull latest
            info!("Git backup directory exists, pulling latest changes");
            
            // Try to pull, but don't fail if it doesn't work (we'll force push anyway)
            let _ = Command::new("git")
                .current_dir(&work_dir)
                .args(&["pull", "--rebase"])
                .output();
        } else {
            // Clone the repository
            info!("Cloning git repository: {}", repo_path);

            let clone_url = if let Some(tok) = token {
                // Insert token into URL for authentication
                inject_token_into_url(repo_path, tok)
            } else {
                repo_path.to_string()
            };

            let output = Command::new("git")
                .args(&["clone", &clone_url, work_dir.to_str().unwrap()])
                .output()
                .map_err(|e| format!("Failed to execute git clone: {}", e))?;

            if !output.status.success() {
                return Err(format!(
                    "Git clone failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    } else {
        // Local path - create if doesn't exist and init
        if !work_dir.exists() {
            fs::create_dir_all(&work_dir)
                .map_err(|e| format!("Failed to create directory: {}", e))?;

            let output = Command::new("git")
                .current_dir(&work_dir)
                .args(&["init"])
                .output()
                .map_err(|e| format!("Failed to execute git init: {}", e))?;

            if !output.status.success() {
                return Err(format!(
                    "Git init failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            info!("Initialized new git repository at {:?}", work_dir);
        }
    }

    Ok(work_dir)
}

/// Inject authentication token into git URL
pub fn inject_token_into_url(url: &str, token: &str) -> String {
    if url.starts_with("https://") {
        // Replace https:// with https://token@
        url.replacen("https://", &format!("https://{}@", token), 1)
    } else if url.starts_with("http://") {
        // Replace http:// with http://token@
        url.replacen("http://", &format!("http://{}@", token), 1)
    } else {
        url.to_string()
    }
}

/// Dump a single object to a file in objdef format with meta filtering
fn dump_object_to_file(
    database: &DatabaseRef,
    obj_info: &ObjectInfo,
    work_dir: &Path,
) -> Result<String, String> {
    let object_name = &obj_info.name;

    // Get the object SHA256 from refs using the specific version from the working index
    let sha256 = database
        .refs()
        .get_ref(VcsObjectType::MooObject, object_name, Some(obj_info.version))
        .map_err(|e| format!("Failed to get ref for '{}' version {}: {}", object_name, obj_info.version, e))?
        .ok_or_else(|| format!("Object '{}' version {} not found in refs", object_name, obj_info.version))?;

    // Get the object content (it's already in objdef format)
    let obj_content = database
        .objects()
        .get(&sha256)
        .map_err(|e| format!("Failed to get object content: {}", e))?
        .ok_or_else(|| format!("Object content not found for sha256: {}", sha256))?;

    // Check if meta exists and apply filtering if needed
    let final_content = match database
        .refs()
        .get_ref(VcsObjectType::MooMetaObject, object_name, None)
        .map_err(|e| format!("Failed to check for meta: {}", e))?
    {
        Some(meta_sha256) => {
            // Meta exists, load it
            let meta_yaml = database
                .objects()
                .get(&meta_sha256)
                .map_err(|e| format!("Failed to get meta content: {}", e))?
                .ok_or_else(|| "Meta SHA256 exists but data not found".to_string())?;

            let meta = database
                .objects()
                .parse_meta_dump(&meta_yaml)
                .map_err(|e| format!("Failed to parse meta: {}", e))?;

            // Only filter if there are ignored properties or verbs
            if !meta.ignored_properties.is_empty() || !meta.ignored_verbs.is_empty() {
                info!(
                    "Filtering object '{}' - ignoring {} properties and {} verbs",
                    object_name,
                    meta.ignored_properties.len(),
                    meta.ignored_verbs.len()
                );
                apply_meta_filtering(&obj_content, &meta)?
            } else {
                obj_content
            }
        }
        None => obj_content,
    };

    // Sanitize object name for filename
    let filename = format!("{}.moo", sanitize_filename(object_name));
    let file_path = work_dir.join(&filename);

    // Write to file
    let mut file = fs::File::create(&file_path)
        .map_err(|e| format!("Failed to create file '{}': {}", filename, e))?;

    file.write_all(final_content.as_bytes())
        .map_err(|e| format!("Failed to write to file '{}': {}", filename, e))?;

    info!("Wrote object '{}' to file '{}'", object_name, filename);

    Ok(filename)
}

/// Apply meta filtering to an object definition in objdef format
fn apply_meta_filtering(obj_content: &str, meta: &crate::types::MooMetaObject) -> Result<String, String> {
    // Parse the object dump
    let mut context = ObjFileContext::new();
    let mut compiled_defs =
        compile_object_definitions(obj_content, &CompileOptions::default(), &mut context)
            .map_err(|e| format!("Failed to parse object for filtering: {}", e))?;

    if compiled_defs.len() != 1 {
        return Err(format!(
            "Expected exactly 1 object definition, got {}",
            compiled_defs.len()
        ));
    }

    let mut obj_def = compiled_defs.remove(0);

    // Filter property definitions
    obj_def
        .property_definitions
        .retain(|prop| !meta.ignored_properties.contains(&prop.name.as_string()));

    // Filter property overrides
    obj_def
        .property_overrides
        .retain(|prop| !meta.ignored_properties.contains(&prop.name.as_string()));

    // Filter verbs
    obj_def.verbs.retain(|verb| {
        !verb
            .names
            .iter()
            .any(|name| meta.ignored_verbs.contains(&name.as_string()))
    });

    // Re-dump the filtered object
    let index_names = HashMap::new(); // Empty index for simple object names
    let lines = dump_object(&index_names, &obj_def)
        .map_err(|e| format!("Failed to dump filtered object: {}", e))?;

    Ok(lines.join("\n"))
}

/// Sanitize a filename by replacing invalid characters
pub fn sanitize_filename(name: &str) -> String {
    let sanitized = name.replace('/', "_")
        .replace('\\', "_")
        .replace(':', "_")
        .replace('*', "_")
        .replace('?', "_")
        .replace('"', "_")
        .replace('<', "_")
        .replace('>', "_")
        .replace('|', "_")
        .replace('$', "");
    
    // Collapse consecutive underscores into a single underscore
    let mut result = String::new();
    let mut last_was_underscore = false;
    for c in sanitized.chars() {
        if c == '_' {
            if !last_was_underscore {
                result.push(c);
                last_was_underscore = true;
            }
        } else {
            result.push(c);
            last_was_underscore = false;
        }
    }
    result
}

/// Clean up old .moo files that no longer correspond to objects
fn cleanup_old_files(work_dir: &Path, current_files: &HashSet<String>) -> Result<(), String> {
    let entries = fs::read_dir(work_dir)
        .map_err(|e| format!("Failed to read work directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Only consider .moo files
                if filename.ends_with(".moo") && !current_files.contains(filename) {
                    info!("Removing old file: {}", filename);
                    fs::remove_file(&path)
                        .map_err(|e| format!("Failed to remove file '{}': {}", filename, e))?;
                }
            }
        }
    }

    Ok(())
}

/// Commit and push changes to git
fn git_commit_and_push(
    work_dir: &Path,
    repo_url: &str,
    token: &Option<String>,
) -> Result<(), String> {
    let is_remote = repo_url.starts_with("http://") || repo_url.starts_with("https://");

    // Configure git user if not already configured (needed for commits)
    let _ = Command::new("git")
        .current_dir(work_dir)
        .args(&["config", "user.email", "vcs-backup@localhost"])
        .output();

    let _ = Command::new("git")
        .current_dir(work_dir)
        .args(&["config", "user.name", "VCS Backup"])
        .output();

    // Add all changes
    let output = Command::new("git")
        .current_dir(work_dir)
        .args(&["add", "-A"])
        .output()
        .map_err(|e| format!("Failed to execute git add: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Check if there are changes to commit
    let status_output = Command::new("git")
        .current_dir(work_dir)
        .args(&["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to execute git status: {}", e))?;

    if status_output.stdout.is_empty() {
        info!("No changes to commit");
        return Ok(());
    }

    // Commit changes
    let timestamp = chrono::Utc::now().to_rfc3339();
    let commit_message = format!("VCS backup: {}", timestamp);

    let output = Command::new("git")
        .current_dir(work_dir)
        .args(&["commit", "-m", &commit_message])
        .output()
        .map_err(|e| format!("Failed to execute git commit: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Git commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    info!("Committed changes: {}", commit_message);

    // Push if remote repository
    if is_remote {
        let push_url = if let Some(tok) = token {
            inject_token_into_url(repo_url, tok)
        } else {
            repo_url.to_string()
        };

        // Set the remote URL (in case it changed or wasn't set)
        let _ = Command::new("git")
            .current_dir(work_dir)
            .args(&["remote", "remove", "origin"])
            .output();

        let output = Command::new("git")
            .current_dir(work_dir)
            .args(&["remote", "add", "origin", &push_url])
            .output()
            .map_err(|e| format!("Failed to set git remote: {}", e))?;

        if !output.status.success() {
            warn!(
                "Git remote add failed (might already exist): {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // Force push to main branch
        let output = Command::new("git")
            .current_dir(work_dir)
            .args(&["push", "--force", "origin", "HEAD:main"])
            .output()
            .map_err(|e| format!("Failed to execute git push: {}", e))?;

        if !output.status.success() {
            // Try master branch as fallback
            let output = Command::new("git")
                .current_dir(work_dir)
                .args(&["push", "--force", "origin", "HEAD:master"])
                .output()
                .map_err(|e| format!("Failed to execute git push: {}", e))?;

            if !output.status.success() {
                return Err(format!(
                    "Git push failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        info!("Force pushed changes to remote repository");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("simple"), "simple");
        assert_eq!(sanitize_filename("$player"), "player");
        assert_eq!(sanitize_filename("obj/with/slashes"), "obj_with_slashes");
        assert_eq!(sanitize_filename("obj\\with\\backslashes"), "obj_with_backslashes");
        assert_eq!(sanitize_filename("obj:with:colons"), "obj_with_colons");
        assert_eq!(sanitize_filename("obj*with*stars"), "obj_with_stars");
        assert_eq!(sanitize_filename("obj?with?questions"), "obj_with_questions");
        assert_eq!(sanitize_filename("obj\"with\"quotes"), "obj_with_quotes");
        assert_eq!(sanitize_filename("obj<with>brackets"), "obj_with_brackets");
        assert_eq!(sanitize_filename("obj|with|pipes"), "obj_with_pipes");
        assert_eq!(sanitize_filename("$room:utilities"), "room_utilities");
    }

    #[test]
    fn test_inject_token_into_url() {
        // HTTPS URLs
        assert_eq!(
            inject_token_into_url("https://github.com/user/repo.git", "my_token"),
            "https://my_token@github.com/user/repo.git"
        );

        // HTTP URLs
        assert_eq!(
            inject_token_into_url("http://example.com/repo.git", "token123"),
            "http://token123@example.com/repo.git"
        );

        // Non-HTTP URLs should remain unchanged
        assert_eq!(
            inject_token_into_url("git@github.com:user/repo.git", "token"),
            "git@github.com:user/repo.git"
        );

        // Already has auth should be replaced
        assert_eq!(
            inject_token_into_url("https://oldtoken@github.com/user/repo.git", "newtoken"),
            "https://newtoken@oldtoken@github.com/user/repo.git"
        );
    }

    #[test]
    fn test_sanitize_filename_preserves_alphanumeric() {
        let input = "abc123_test-object.name";
        let output = sanitize_filename(input);
        assert!(output.contains("abc123"));
        assert!(output.contains("test"));
        assert!(output.contains("object"));
        assert!(output.contains("name"));
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert_eq!(sanitize_filename(""), "");
    }

    #[test]
    fn test_sanitize_filename_only_special_chars() {
        assert_eq!(sanitize_filename("$*?:"), "_");
    }
}
