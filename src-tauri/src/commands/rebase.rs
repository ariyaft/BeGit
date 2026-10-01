use std::fs;
use crate::utils::git_command;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::models::{CherryPickStatus, Commit, RebaseStatus, RebaseTodoItem};

#[tauri::command]
pub fn cherry_pick(path: String, hashes: Vec<String>) -> Result<(), String> {
    if hashes.is_empty() {
        return Err("No commits provided".to_string());
    }
    
    let mut cmd = git_command();
    cmd.arg("cherry-pick")
       .current_dir(&path);
       
    for hash in hashes {
        cmd.arg(hash);
    }
    
    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Cherry-pick failed:\n{}\n{}", err, out));
    }

    Ok(())
}

pub fn run_cherry_pick_operation(path: &str, operation: &str) -> Result<(), String> {
    let output = git_command()
        .args(["cherry-pick", operation])
        .current_dir(path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        return Ok(());
    }

    let err = String::from_utf8_lossy(&output.stderr);
    let out = String::from_utf8_lossy(&output.stdout);
    Err(format!("Cherry-pick {} failed:\n{}{}", operation, err, out))
}

#[tauri::command]
pub fn get_cherry_pick_status(path: String) -> Result<CherryPickStatus, String> {
    // Resolve both paths in one Git invocation. Previously this passive status
    // check launched three Git processes on every refresh.
    let paths = git_command()
        .args([
            "rev-parse",
            "--git-path",
            "CHERRY_PICK_HEAD",
            "--git-path",
            "sequencer/todo",
        ])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !paths.status.success() {
        return Err(String::from_utf8_lossy(&paths.stderr).to_string());
    }

    let resolve_git_path = |git_path: &str| {
        let git_path = std::path::Path::new(git_path);
        if git_path.is_absolute() {
            git_path.to_path_buf()
        } else {
            std::path::Path::new(&path).join(git_path)
        }
    };
    let resolved_paths_output = String::from_utf8_lossy(&paths.stdout);
    let mut resolved_paths = resolved_paths_output.lines();
    let cherry_pick_head_path = resolve_git_path(resolved_paths.next().unwrap_or_default().trim());
    let todo_path = resolve_git_path(resolved_paths.next().unwrap_or_default().trim());
    let has_todo = fs::metadata(&todo_path).is_ok();
    let todo = fs::read_to_string(&todo_path).unwrap_or_default();
    let remaining_count = todo
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .count();

    let has_cherry_pick_head = fs::metadata(cherry_pick_head_path).is_ok();
    let active = has_cherry_pick_head || has_todo;

    // Conflict discovery is only useful for an active cherry-pick. This avoids
    // a second process during regular idle refreshes.
    let has_conflicts = if active {
        let conflicts = git_command()
            .args(["diff", "--name-only", "--diff-filter=U"])
            .current_dir(&path)
            .output()
            .map_err(|e| e.to_string())?;
        if !conflicts.status.success() {
            return Err(String::from_utf8_lossy(&conflicts.stderr).to_string());
        }
        !String::from_utf8_lossy(&conflicts.stdout).trim().is_empty()
    } else {
        false
    };

    Ok(CherryPickStatus {
        active,
        has_conflicts,
        remaining_count,
    })
}

#[tauri::command]
pub fn continue_cherry_pick(path: String) -> Result<(), String> {
    run_cherry_pick_operation(&path, "--continue")
}

#[tauri::command]
pub fn skip_cherry_pick(path: String) -> Result<(), String> {
    run_cherry_pick_operation(&path, "--skip")
}

#[tauri::command]
pub fn abort_cherry_pick(path: String) -> Result<(), String> {
    run_cherry_pick_operation(&path, "--abort")
}

#[tauri::command]
pub fn get_rebase_commits(path: String, base_ref: String) -> Result<Vec<Commit>, String> {
    let mut cmd = git_command();
    cmd.arg("log")
        .arg("--reverse")
        .arg("--format=%H|%h|%p|%an|%ar|%ai|%s|%d");
    if base_ref == "--root" {
        cmd.arg("HEAD");
    } else {
        cmd.arg(format!("{}..HEAD", base_ref));
    }
    let output = cmd
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to get rebase commits: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 7 {
            let hash = parts[0].to_string();
            let hash_short = parts[1].to_string();
            let parents = parts[2].split_whitespace().map(|s| s.to_string()).collect();
            let author = parts[3].to_string();
            let time = parts[4].to_string();
            let exact_time = parts[5].to_string();
            let message = parts[6].to_string();
            let refs = if parts.len() > 7 && !parts[7].trim().is_empty() {
                let r = parts[7].trim();
                let clean = r.trim_start_matches('(').trim_end_matches(')');
                clean.split(',').map(|s| s.trim().to_string()).collect()
            } else {
                Vec::new()
            };

            commits.push(Commit {
                hash,
                hash_short,
                parents,
                author,
                time,
                exact_time,
                message,
                refs,
                signature: None,
            });
        }
    }

    Ok(commits)
}

#[tauri::command]
pub async fn start_interactive_rebase(path: String, base_ref: String, todo: Vec<RebaseTodoItem>) -> Result<(), String> {
    if todo.is_empty() {
        return Err("Rebase todo list cannot be empty".to_string());
    }

    let mut todo_content = String::new();
    for item in &todo {
        let action = item.action.trim().to_lowercase();
        let short_hash = if item.hash.len() > 7 { &item.hash[..7] } else { &item.hash };
        let msg = item.message.as_deref().unwrap_or("").lines().next().unwrap_or("");

        match action.as_str() {
            "drop" => {
                todo_content.push_str(&format!("drop {} {}\n", short_hash, msg));
            }
            "fixup" => {
                todo_content.push_str(&format!("fixup {} {}\n", short_hash, msg));
            }
            "squash" => {
                todo_content.push_str(&format!("squash {} {}\n", short_hash, msg));
                if let Some(ref full_msg) = item.message {
                    if !full_msg.trim().is_empty() {
                        let escaped = full_msg.replace('"', "\\\"").replace('`', "\\`").replace('$', "\\$");
                        todo_content.push_str(&format!("exec git commit --amend -m \"{}\"\n", escaped));
                    }
                }
            }
            "reword" => {
                todo_content.push_str(&format!("pick {} {}\n", short_hash, msg));
                if let Some(ref full_msg) = item.message {
                    if !full_msg.trim().is_empty() {
                        let escaped = full_msg.replace('"', "\\\"").replace('`', "\\`").replace('$', "\\$");
                        todo_content.push_str(&format!("exec git commit --amend -m \"{}\"\n", escaped));
                    }
                }
            }
            _ => {
                // "pick" or default
                todo_content.push_str(&format!("pick {} {}\n", short_hash, msg));
            }
        }
    }

    let temp_todo_path = std::env::temp_dir().join(format!(
        "begit_rebase_todo_{}.txt",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::write(&temp_todo_path, &todo_content).map_err(|e| format!("Failed to write rebase todo file: {}", e))?;

    let seq_editor = format!("sh -c 'cat \"{}\" > \"$1\"' --", temp_todo_path.to_string_lossy());

    let output = git_command()
        .env("GIT_SEQUENCE_EDITOR", &seq_editor)
        .env("GIT_EDITOR", ":")
        .current_dir(&path)
        .args(["rebase", "-i", &base_ref])
        .output();

    let _ = fs::remove_file(&temp_todo_path);

    let output = output.map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        let conflicts_out = git_command()
            .args(["diff", "--name-only", "--diff-filter=U"])
            .current_dir(&path)
            .output();
        let has_conflicts = conflicts_out.map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty()).unwrap_or(false);
        if has_conflicts {
            return Err(format!("Rebase stopped with merge conflicts. Please resolve conflicts or continue/abort.\n{}{}", err, out));
        }
        return Err(format!("Interactive rebase failed:\n{}{}", err, out));
    }

    Ok(())
}

#[tauri::command]
pub async fn drop_commits_from_history(path: String, hashes: Vec<String>) -> Result<(), String> {
    if hashes.is_empty() {
        return Err("No commits selected to drop".to_string());
    }

    // 1. Check for uncommitted changes
    let status_output = git_command()
        .current_dir(&path)
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| e.to_string())?;

    if !status_output.status.success() {
        return Err("Failed to check repository status".to_string());
    }
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    if !status_str.trim().is_empty() {
        return Err("You have uncommitted changes. Please commit or stash them before dropping commits from history.".to_string());
    }

    // 2. Query rev-list to get topological commit ordering of the active branch
    let rev_list = git_command()
        .current_dir(&path)
        .args(["rev-list", "--reverse", "HEAD"])
        .output()
        .map_err(|e| e.to_string())?;

    if !rev_list.status.success() {
        return Err("Failed to retrieve branch commit list".to_string());
    }

    let all_branch_hashes: Vec<String> = String::from_utf8_lossy(&rev_list.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Find the earliest index in all_branch_hashes that matches any of the target hashes
    let mut earliest_idx = None;
    for (i, h) in all_branch_hashes.iter().enumerate() {
        if hashes.iter().any(|target| target == h || (target.len() >= 7 && h.starts_with(target)) || (h.len() >= 7 && target.starts_with(h))) {
            earliest_idx = Some(i);
            break;
        }
    }

    let earliest_idx = match earliest_idx {
        Some(idx) => idx,
        None => return Err("None of the selected commits belong to the current branch history".to_string()),
    };

    let base_ref = if earliest_idx == 0 {
        "--root".to_string()
    } else {
        format!("{}^", all_branch_hashes[earliest_idx])
    };

    let rebase_commits = get_rebase_commits(path.clone(), base_ref.clone())?;
    let mut todo = Vec::new();

    for c in rebase_commits {
        let should_drop = hashes.iter().any(|target| {
            target == &c.hash || (target.len() >= 7 && c.hash.starts_with(target)) || (c.hash.len() >= 7 && target.starts_with(&c.hash))
        });

        todo.push(RebaseTodoItem {
            hash: c.hash,
            action: if should_drop { "drop".to_string() } else { "pick".to_string() },
            message: Some(c.message),
        });
    }

    start_interactive_rebase(path, base_ref, todo).await
}

#[tauri::command]
pub fn get_rebase_status(path: String) -> Result<RebaseStatus, String> {
    let rebase_merge = std::path::Path::new(&path).join(".git").join("rebase-merge");
    let rebase_apply = std::path::Path::new(&path).join(".git").join("rebase-apply");
    let active = rebase_merge.exists() || rebase_apply.exists();
    if !active {
        return Ok(RebaseStatus::default());
    }

    let mut current_step = 0;
    let mut total_steps = 0;
    let mut head_name = None;
    let mut onto = None;

    if rebase_merge.exists() {
        if let Ok(msgno) = fs::read_to_string(rebase_merge.join("msgnum")) {
            current_step = msgno.trim().parse().unwrap_or(0);
        }
        if let Ok(end) = fs::read_to_string(rebase_merge.join("end")) {
            total_steps = end.trim().parse().unwrap_or(0);
        }
        if let Ok(head) = fs::read_to_string(rebase_merge.join("head-name")) {
            head_name = Some(head.trim().replace("refs/heads/", ""));
        }
        if let Ok(onto_str) = fs::read_to_string(rebase_merge.join("onto")) {
            onto = Some(onto_str.trim().chars().take(7).collect());
        }
    }

    let conflicts_out = git_command()
        .args(["diff", "--name-only", "--diff-filter=U"])
        .current_dir(&path)
        .output();
    let has_conflicts = conflicts_out.map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty()).unwrap_or(false);

    Ok(RebaseStatus {
        active: true,
        current_step,
        total_steps,
        has_conflicts,
        head_name,
        onto,
    })
}

#[tauri::command]
pub async fn continue_rebase(path: String) -> Result<(), String> {
    let output = git_command()
        .env("GIT_EDITOR", ":")
        .current_dir(&path)
        .args(["rebase", "--continue"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Rebase continue failed:\n{}{}", err, out));
    }
    Ok(())
}

#[tauri::command]
pub async fn skip_rebase(path: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .args(["rebase", "--skip"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Rebase skip failed: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn abort_rebase(path: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .args(["rebase", "--abort"])
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Rebase abort failed: {}", err));
    }
    Ok(())
}
