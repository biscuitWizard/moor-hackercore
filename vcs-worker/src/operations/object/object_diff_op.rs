use crate::operations::{Operation, OperationExample, OperationParameter, OperationRoute};
use axum::http::Method;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::database::{DatabaseRef, ObjectsTreeError};
use crate::object_diff::{compare_object_definitions_with_meta, ObjectChange};
use crate::providers::index::IndexProvider;
use crate::providers::objects::ObjectsProvider;
use crate::providers::refs::RefsProvider;
use crate::types::{User, VcsObjectType};
use moor_compiler::{program_to_tree, unparse, ObjectDefinition};
use moor_var::{v_error, v_int, v_list, v_map, v_str, E_INVARG, Var};
use moor_var::program::ProgramType;

/// Request structure for object diff operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDiffRequest {
    pub object_name: String,
    pub change_id: String,
    pub baseline_change_id: Option<String>,
}

/// Represents a single line in a diff
#[derive(Debug, Clone, PartialEq)]
enum DiffLine {
    Added(String),
    Removed(String),
    Changed(String, String), // (old, new)
    Unchanged(String),
}

/// Represents a hunk of consecutive diff lines
#[derive(Debug, Clone)]
struct DiffHunk {
    content: Vec<String>,
    start: usize,
    hunk_type: String, // "added", "removed", or "changed"
}

impl DiffHunk {
    fn to_moo_var(&self) -> Var {
        let content_list: Vec<Var> = self.content.iter().map(|s| v_str(s)).collect();
        
        v_map(&[
            (v_str("content"), v_list(&content_list)),
            (v_str("start"), v_int(self.start as i64)),
            (v_str("type"), v_str(&self.hunk_type)),
        ])
    }
}

/// Object diff operation that compares verb code between two commits
#[derive(Clone)]
pub struct ObjectDiffOperation {
    database: DatabaseRef,
}

impl ObjectDiffOperation {
    /// Create a new object diff operation
    pub fn new(database: DatabaseRef) -> Self {
        Self { database }
    }


    /// Compute verb-level diff using Myers algorithm
    fn compute_verb_diff(&self, old_lines: &[String], new_lines: &[String]) -> Vec<DiffLine> {
        // Simple Myers diff implementation
        let mut diff_lines = Vec::new();
        
        let old_len = old_lines.len();
        let new_len = new_lines.len();
        
        // Build LCS (Longest Common Subsequence) table
        let mut lcs = vec![vec![0; new_len + 1]; old_len + 1];
        
        for i in 1..=old_len {
            for j in 1..=new_len {
                if old_lines[i - 1] == new_lines[j - 1] {
                    lcs[i][j] = lcs[i - 1][j - 1] + 1;
                } else {
                    lcs[i][j] = std::cmp::max(lcs[i - 1][j], lcs[i][j - 1]);
                }
            }
        }
        
        // Backtrack to build diff
        let mut i = old_len;
        let mut j = new_len;
        
        while i > 0 || j > 0 {
            if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
                diff_lines.push(DiffLine::Unchanged(old_lines[i - 1].clone()));
                i -= 1;
                j -= 1;
            } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
                diff_lines.push(DiffLine::Added(new_lines[j - 1].clone()));
                j -= 1;
            } else if i > 0 {
                diff_lines.push(DiffLine::Removed(old_lines[i - 1].clone()));
                i -= 1;
            }
        }
        
        diff_lines.reverse();
        diff_lines
    }

    /// Generate hunks from diff lines
    fn generate_hunks(&self, diff_lines: &[DiffLine]) -> Vec<DiffHunk> {
        let mut hunks = Vec::new();
        let mut current_hunk: Option<DiffHunk> = None;
        let mut line_number = 1;

        for diff_line in diff_lines {
            match diff_line {
                DiffLine::Added(line) => {
                    if let Some(ref mut hunk) = current_hunk {
                        if hunk.hunk_type == "added" {
                            hunk.content.push(line.clone());
                        } else {
                            // Start a new hunk
                            hunks.push(current_hunk.take().unwrap());
                            current_hunk = Some(DiffHunk {
                                content: vec![line.clone()],
                                start: line_number,
                                hunk_type: "added".to_string(),
                            });
                        }
                    } else {
                        current_hunk = Some(DiffHunk {
                            content: vec![line.clone()],
                            start: line_number,
                            hunk_type: "added".to_string(),
                        });
                    }
                    line_number += 1;
                }
                DiffLine::Removed(line) => {
                    if let Some(ref mut hunk) = current_hunk {
                        if hunk.hunk_type == "removed" {
                            hunk.content.push(line.clone());
                        } else {
                            // Start a new hunk
                            hunks.push(current_hunk.take().unwrap());
                            current_hunk = Some(DiffHunk {
                                content: vec![line.clone()],
                                start: line_number,
                                hunk_type: "removed".to_string(),
                            });
                        }
                    } else {
                        current_hunk = Some(DiffHunk {
                            content: vec![line.clone()],
                            start: line_number,
                            hunk_type: "removed".to_string(),
                        });
                    }
                    // Don't increment line_number for removed lines
                }
                DiffLine::Changed(old, new) => {
                    if let Some(hunk) = current_hunk.take() {
                        hunks.push(hunk);
                    }
                    current_hunk = Some(DiffHunk {
                        content: vec![format!("- {}", old), format!("+ {}", new)],
                        start: line_number,
                        hunk_type: "changed".to_string(),
                    });
                    line_number += 1;
                }
                DiffLine::Unchanged(_) => {
                    if let Some(hunk) = current_hunk.take() {
                        hunks.push(hunk);
                    }
                    line_number += 1;
                }
            }
        }

        if let Some(hunk) = current_hunk {
            hunks.push(hunk);
        }

        hunks
    }

    /// Process the object diff request
    fn process_object_diff(
        &self,
        request: ObjectDiffRequest,
    ) -> Result<Var, ObjectsTreeError> {
        // Resolve change IDs
        let target_change_id = self.database.resolve_change_id(&request.change_id)?;
        info!(
            "Resolved target change ID '{}' to '{}'",
            request.change_id, target_change_id
        );

        let baseline_change_id = if let Some(ref baseline_id) = request.baseline_change_id {
            Some(self.database.resolve_change_id(baseline_id)?)
        } else {
            None
        };

        if let Some(ref baseline_id) = baseline_change_id {
            info!(
                "Resolved baseline change ID '{}' to '{}'",
                request.baseline_change_id.as_ref().unwrap(),
                baseline_id
            );
        }

        // Get the change to find the object version
        let change = self
            .database
            .index()
            .get_change(&target_change_id)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Change '{}' not found",
                    target_change_id
                ))
            })?;

        // Find the object in the change
        let object_info = change
            .modified_objects
            .iter()
            .chain(change.added_objects.iter())
            .find(|obj| {
                obj.object_type == VcsObjectType::MooObject && obj.name == request.object_name
            })
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Object '{}' not found in change '{}'",
                    request.object_name, target_change_id
                ))
            })?;

        // Use compare_object_versions to get the correct diff (same as change/status)
        use crate::object_diff::compare_object_versions;
        let object_change = compare_object_versions(
            &self.database,
            &request.object_name,
            object_info.version,
            Some(&change.verb_rename_hints),
            Some(&change.property_rename_hints),
        )?;

        info!(
            "Found {} modified verbs, {} added verbs, {} deleted verbs, {} renamed verbs",
            object_change.verbs_modified.len(),
            object_change.verbs_added.len(),
            object_change.verbs_deleted.len(),
            object_change.verbs_renamed.len()
        );

        // Get the target object definition for decompiling verbs
        let target_sha256 = self
            .database
            .refs()
            .get_ref(
                VcsObjectType::MooObject,
                &request.object_name,
                Some(object_info.version),
            )
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Object '{}' version {} not found in refs",
                    request.object_name, object_info.version
                ))
            })?;

        let target_content = self
            .database
            .objects()
            .get(&target_sha256)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
            .ok_or_else(|| {
                ObjectsTreeError::SerializationError(format!(
                    "Object '{}' content not found",
                    request.object_name
                ))
            })?;

        let target_obj = self
            .database
            .objects()
            .parse_object_dump(&target_content)
            .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

        // Get baseline object if there's a previous version
        let baseline_version = object_info.version.saturating_sub(1);
        let baseline_obj = if baseline_version > 0 {
            let baseline_sha256 = self
                .database
                .refs()
                .get_ref(VcsObjectType::MooObject, &request.object_name, Some(baseline_version))
                .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?;

            if let Some(sha256) = baseline_sha256 {
                let baseline_content = self
                    .database
                    .objects()
                    .get(&sha256)
                    .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?
                    .ok_or_else(|| {
                        ObjectsTreeError::SerializationError(format!(
                            "Baseline object content not found"
                        ))
                    })?;

                Some(
                    self.database
                        .objects()
                        .parse_object_dump(&baseline_content)
                        .map_err(|e| ObjectsTreeError::SerializationError(e.to_string()))?,
                )
            } else {
                None
            }
        } else {
            None
        };

        // Build the response with verb diffs
        let mut verb_changes = Vec::new();

        // Process modified verbs
        for verb_name in &object_change.verbs_modified {
            if let Some(baseline_verb) = baseline_obj.as_ref().and_then(|baseline| {
                baseline.verbs.iter().find(|v| {
                    v.names.iter().any(|n| n.as_string() == *verb_name)
                })
            }) {
                if let Some(target_verb) = target_obj.verbs.iter().find(|v| {
                    v.names.iter().any(|n| n.as_string() == *verb_name)
                }) {
                    // Decompile both versions
                    let baseline_code = self.decompile_verb(baseline_verb)?;
                    let target_code = self.decompile_verb(target_verb)?;

                    // Compute diff
                    let diff_lines = self.compute_verb_diff(&baseline_code, &target_code);
                    let hunks = self.generate_hunks(&diff_lines);

                    if !hunks.is_empty() {
                        let hunks_list: Vec<Var> = hunks.iter().map(|h| h.to_moo_var()).collect();
                        verb_changes.push(v_map(&[
                            (v_str("verb"), v_str(verb_name)),
                            (v_str("hunks"), v_list(&hunks_list)),
                        ]));
                    }
                }
            }
        }

        // Process added verbs
        for verb_name in &object_change.verbs_added {
            if let Some(target_verb) = target_obj.verbs.iter().find(|v| {
                v.names.iter().any(|n| n.as_string() == *verb_name)
            }) {
                let target_code = self.decompile_verb(target_verb)?;
                let diff_lines: Vec<DiffLine> = target_code
                    .iter()
                    .map(|line| DiffLine::Added(line.clone()))
                    .collect();
                let hunks = self.generate_hunks(&diff_lines);

                let hunks_list: Vec<Var> = hunks.iter().map(|h| h.to_moo_var()).collect();
                verb_changes.push(v_map(&[
                    (v_str("verb"), v_str(verb_name)),
                    (v_str("hunks"), v_list(&hunks_list)),
                ]));
            }
        }

        // Process deleted verbs
        for verb_name in &object_change.verbs_deleted {
            if let Some(baseline_verb) = baseline_obj.as_ref().and_then(|baseline| {
                baseline.verbs.iter().find(|v| {
                    v.names.iter().any(|n| n.as_string() == *verb_name)
                })
            }) {
                let baseline_code = self.decompile_verb(baseline_verb)?;
                let diff_lines: Vec<DiffLine> = baseline_code
                    .iter()
                    .map(|line| DiffLine::Removed(line.clone()))
                    .collect();
                let hunks = self.generate_hunks(&diff_lines);

                let hunks_list: Vec<Var> = hunks.iter().map(|h| h.to_moo_var()).collect();
                verb_changes.push(v_map(&[
                    (v_str("verb"), v_str(verb_name)),
                    (v_str("hunks"), v_list(&hunks_list)),
                ]));
            }
        }

        // Process renamed verbs
        for (old_name, new_name) in &object_change.verbs_renamed {
            if let Some(target_verb) = target_obj.verbs.iter().find(|v| {
                v.names.iter().any(|n| n.as_string() == *new_name)
            }) {
                if let Some(baseline_verb) = baseline_obj.as_ref().and_then(|baseline| {
                    baseline.verbs.iter().find(|v| {
                        v.names.iter().any(|n| n.as_string() == *old_name)
                    })
                }) {
                    // Decompile both versions
                    let baseline_code = self.decompile_verb(baseline_verb)?;
                    let target_code = self.decompile_verb(target_verb)?;

                    // Compute diff
                    let diff_lines = self.compute_verb_diff(&baseline_code, &target_code);
                    let hunks = self.generate_hunks(&diff_lines);

                    let hunks_list: Vec<Var> = hunks.iter().map(|h| h.to_moo_var()).collect();
                    verb_changes.push(v_map(&[
                        (v_str("verb"), v_str(new_name)),
                        (v_str("old_verb"), v_str(old_name)),
                        (v_str("hunks"), v_list(&hunks_list)),
                    ]));
                }
            }
        }

        // Convert object_name to appropriate Var type
        let obj_id_var = if let Some(stripped) = request.object_name.strip_prefix('#') {
            if let Ok(num) = stripped.parse::<i32>() {
                moor_var::v_objid(num)
            } else {
                v_str(&request.object_name)
            }
        } else {
            v_str(&request.object_name)
        };

        Ok(v_map(&[
            (v_str("obj_id"), obj_id_var),
            (v_str("changes"), v_list(&verb_changes)),
        ]))
    }

    /// Decompile a verb into source code lines
    fn decompile_verb(
        &self,
        verb: &moor_compiler::ObjVerbDef,
    ) -> Result<Vec<String>, ObjectsTreeError> {
        let ProgramType::MooR(program) = &verb.program;
        
        if program.main_vector().is_empty() {
            return Ok(Vec::new());
        }

        let ast = program_to_tree(program).map_err(|e| {
            ObjectsTreeError::SerializationError(format!("Failed to decompile verb: {}", e))
        })?;

        let lines = unparse(&ast, false, true).map_err(|e| {
            ObjectsTreeError::SerializationError(format!("Failed to unparse verb: {}", e))
        })?;

        Ok(lines)
    }
}

impl Operation for ObjectDiffOperation {
    fn name(&self) -> &'static str {
        "object/diff"
    }

    fn description(&self) -> &'static str {
        "Compares verb code between two commits and returns detailed line-by-line diffs"
    }

    fn philosophy(&self) -> &'static str {
        "This operation allows you to see exactly what changed in an object's verb code between \
        two commits. By default, it compares the specified commit against the state immediately \
        before it. You can also provide a custom baseline commit to compare against. The operation \
        returns detailed hunks showing added, removed, and changed lines of code for each verb \
        that has differences. This is useful for code review, understanding changes, and tracking \
        the evolution of your MOO objects over time."
    }

    fn parameters(&self) -> Vec<OperationParameter> {
        vec![
            OperationParameter {
                name: "object_name".to_string(),
                description: "The name of the MOO object to diff (e.g., '$player', '#123')"
                    .to_string(),
                required: true,
            },
            OperationParameter {
                name: "change_id".to_string(),
                description: "The change ID to examine (supports both short and long hash IDs)"
                    .to_string(),
                required: true,
            },
            OperationParameter {
                name: "baseline_change_id".to_string(),
                description:
                    "Optional baseline change ID to compare against (defaults to previous commit)"
                        .to_string(),
                required: false,
            },
        ]
    }

    fn examples(&self) -> Vec<OperationExample> {
        vec![
            OperationExample {
                description: "Compare an object against its previous commit".to_string(),
                moocode: r#"diff = worker_request("vcs", {"object/diff", "$player", "abc123def456"});
// Returns detailed verb-level diffs
for change in (diff["changes"])
  player:tell("Verb: ", change["verb"]);
  for hunk in (change["hunks"])
    player:tell("  ", hunk["type"], " at line ", hunk["start"]);
    for line in (hunk["content"])
      player:tell("    ", line);
    endfor
  endfor
endfor"#
                    .to_string(),
                http_curl: Some(
                    r#"curl -X POST http://localhost:8081/api/object/diff \
  -H "Content-Type: application/json" \
  -d '{"operation": "object/diff", "args": ["$player", "abc123def456"]}'"#
                        .to_string(),
                ),
            },
            OperationExample {
                description: "Compare an object against a specific baseline commit".to_string(),
                moocode: r#"diff = worker_request("vcs", {"object/diff", "$player", "def456", "abc123"});
// Compares the state at 'def456' against the state at 'abc123'"#
                    .to_string(),
                http_curl: None,
            },
        ]
    }

    fn routes(&self) -> Vec<OperationRoute> {
        vec![OperationRoute {
            path: "/api/object/diff".to_string(),
            method: Method::POST,
            is_json: true,
        }]
    }

    fn responses(&self) -> Vec<crate::operations::OperationResponse> {
        use crate::operations::OperationResponse;
        vec![
            OperationResponse::success(
                "Operation executed successfully",
                r##"{"obj_id": "#123", "changes": [{"verb": "look", "hunks": [{"content": ["new code line"], "start": 5, "type": "added"}]}]}"##,
            ),
            OperationResponse::new(
                400,
                "Bad Request - Missing required arguments",
                r#"E_INVARG("Object name and change ID are required")"#,
            ),
            OperationResponse::new(
                404,
                "Not Found - Object not found in change",
                r#"E_INVARG("Object '$player' not found in change 'abc123'")"#,
            ),
            OperationResponse::new(
                500,
                "Internal Server Error - Failed to process diff",
                r#"E_INVARG("Failed to decompile verb: error")"#,
            ),
        ]
    }

    fn execute(&self, args: Vec<String>, _user: &User) -> Var {
        if args.len() < 2 {
            error!("Object diff operation requires at least 2 arguments");
            return v_error(E_INVARG.msg("Object name and change ID are required"));
        }

        let object_name = args[0].clone();
        let change_id = args[1].clone();
        let baseline_change_id = args.get(2).cloned();

        let request = ObjectDiffRequest {
            object_name,
            change_id,
            baseline_change_id,
        };

        match self.process_object_diff(request) {
            Ok(result) => {
                info!("Object diff operation completed successfully");
                result
            }
            Err(e) => {
                error!("Object diff operation failed: {}", e);
                v_error(E_INVARG.msg(format!("{}", e)))
            }
        }
    }
}

