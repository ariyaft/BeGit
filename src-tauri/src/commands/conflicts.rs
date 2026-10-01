use std::fs;
use crate::utils::git_command;
use crate::models::{ConflictFileInfo, RepoOperationState};

#[tauri::command]
pub fn get_conflicted_files(path: String) -> Result<Vec<ConflictFileInfo>, String> {
    let output = git_command()
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to get conflicted files: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();

    for line in stdout.lines() {
        let rel_path = line.trim();
        if rel_path.is_empty() { continue; }

        let base_out = git_command().args(["show", &format!(":1:{}", rel_path)]).current_dir(&path).output();
        let base_content = base_out.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();

        let ours_out = git_command().args(["show", &format!(":2:{}", rel_path)]).current_dir(&path).output();
        let ours_content = ours_out.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();

        let theirs_out = git_command().args(["show", &format!(":3:{}", rel_path)]).current_dir(&path).output();
        let theirs_content = theirs_out.map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();

        let full_path = std::path::Path::new(&path).join(rel_path);
        let working_content = fs::read_to_string(&full_path).unwrap_or_default();

        files.push(ConflictFileInfo {
            path: rel_path.to_string(),
            base_content,
            ours_content,
            theirs_content,
            working_content,
        });
    }

    Ok(files)
}

#[tauri::command]
pub async fn resolve_conflict_file(path: String, file_path: String, resolved_content: String) -> Result<(), String> {
    let full_path = std::path::Path::new(&path).join(&file_path);
    fs::write(&full_path, resolved_content).map_err(|e| format!("Failed to write resolved file: {}", e))?;

    let output = git_command()
        .args(["add", "--", &file_path])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to stage resolved file: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub fn get_repository_operation_state(path: String) -> Result<RepoOperationState, String> {
    let git_dir = std::path::Path::new(&path).join(".git");
    let is_merge = git_dir.join("MERGE_HEAD").exists();
    let is_rebase = git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists();
    let is_cherry_pick = git_dir.join("CHERRY_PICK_HEAD").exists() || git_dir.join("sequencer").join("todo").exists();
    let is_revert = git_dir.join("REVERT_HEAD").exists();

    let mut operation = "none".to_string();
    let mut head_name = None;
    let mut target_name = None;
    let mut current_step = 0;
    let mut total_steps = 0;

    if is_rebase {
        operation = "rebase".to_string();
        let rm = git_dir.join("rebase-merge");
        if rm.exists() {
            if let Ok(msgno) = fs::read_to_string(rm.join("msgnum")) {
                current_step = msgno.trim().parse().unwrap_or(0);
            }
            if let Ok(end) = fs::read_to_string(rm.join("end")) {
                total_steps = end.trim().parse().unwrap_or(0);
            }
            if let Ok(head) = fs::read_to_string(rm.join("head-name")) {
                head_name = Some(head.trim().replace("refs/heads/", ""));
            }
            if let Ok(onto) = fs::read_to_string(rm.join("onto")) {
                target_name = Some(onto.trim().chars().take(7).collect());
            }
        }
    } else if is_merge {
        operation = "merge".to_string();
        if let Ok(mhead) = fs::read_to_string(git_dir.join("MERGE_HEAD")) {
            target_name = Some(mhead.trim().chars().take(7).collect());
        }
    } else if is_cherry_pick {
        operation = "cherry_pick".to_string();
        if let Ok(cphead) = fs::read_to_string(git_dir.join("CHERRY_PICK_HEAD")) {
            target_name = Some(cphead.trim().chars().take(7).collect());
        }
    } else if is_revert {
        operation = "revert".to_string();
    }

    // A clean repository cannot have an in-progress-operation conflict. Skip
    // this extra Git process for the overwhelmingly common idle case.
    let conflicted_files: Vec<String> = if operation == "none" {
        Vec::new()
    } else {
        git_command()
            .args(["diff", "--name-only", "--diff-filter=U"])
            .current_dir(&path)
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    };
    let has_conflicts = !conflicted_files.is_empty();

    Ok(RepoOperationState {
        operation,
        head_name,
        target_name,
        has_conflicts,
        conflicted_files,
        current_step,
        total_steps,
    })
}

#[tauri::command]
pub async fn abort_repository_operation(path: String, op_type: String) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.current_dir(&path);
    match op_type.as_str() {
        "merge" => cmd.args(["merge", "--abort"]),
        "rebase" => cmd.args(["rebase", "--abort"]),
        "cherry_pick" | "cherry-pick" => cmd.args(["cherry-pick", "--abort"]),
        "revert" => cmd.args(["revert", "--abort"]),
        _ => return Err(format!("Unknown operation to abort: {}", op_type)),
    };
    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Abort {} failed: {}", op_type, err));
    }
    Ok(())
}

#[tauri::command]
pub async fn continue_repository_operation(path: String, op_type: String) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.current_dir(&path);
    match op_type.as_str() {
        "merge" => {
            cmd.args(["commit", "--no-edit"]);
        },
        "rebase" => {
            cmd.env("GIT_EDITOR", ":").args(["rebase", "--continue"]);
        },
        "cherry_pick" | "cherry-pick" => {
            cmd.args(["cherry-pick", "--continue"]);
        },
        "revert" => {
            cmd.args(["revert", "--continue"]);
        },
        _ => return Err(format!("Unknown operation to continue: {}", op_type)),
    };
    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Continue {} failed:\n{}{}", op_type, err, out));
    }
    Ok(())
}
