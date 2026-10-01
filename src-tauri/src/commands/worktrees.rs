use crate::utils::git_command;
use crate::models::WorktreeInfo;

pub fn parse_worktree_porcelain(stdout: &str) -> Vec<WorktreeInfo> {
    let mut worktrees = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_head = String::new();
    let mut current_branch: Option<String> = None;
    let mut current_locked = false;
    let mut current_lock_reason: Option<String> = None;
    let mut current_prunable = false;

    let flush = |worktrees: &mut Vec<WorktreeInfo>,
                 path: &mut Option<String>,
                 head: &mut String,
                 branch: &mut Option<String>,
                 locked: &mut bool,
                 lock_reason: &mut Option<String>,
                 prunable: &mut bool| {
        if let Some(p) = path.take() {
            let is_main = worktrees.is_empty();
            worktrees.push(WorktreeInfo {
                path: p,
                branch: branch.take(),
                commit_hash: std::mem::take(head),
                is_main,
                is_locked: *locked,
                lock_reason: lock_reason.take(),
                is_prunable: *prunable,
            });
            *locked = false;
            *prunable = false;
        }
    };

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            flush(
                &mut worktrees,
                &mut current_path,
                &mut current_head,
                &mut current_branch,
                &mut current_locked,
                &mut current_lock_reason,
                &mut current_prunable,
            );
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("worktree ") {
            flush(
                &mut worktrees,
                &mut current_path,
                &mut current_head,
                &mut current_branch,
                &mut current_locked,
                &mut current_lock_reason,
                &mut current_prunable,
            );
            current_path = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("HEAD ") {
            *(&mut current_head) = rest.trim().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("branch ") {
            let b = rest.trim();
            let branch_name = if let Some(stripped) = b.strip_prefix("refs/heads/") {
                stripped
            } else {
                b
            };
            *(&mut current_branch) = Some(branch_name.to_string());
        } else if trimmed == "detached" {
            *(&mut current_branch) = None;
        } else if trimmed == "locked" {
            *(&mut current_locked) = true;
            *(&mut current_lock_reason) = None;
        } else if let Some(rest) = trimmed.strip_prefix("locked ") {
            *(&mut current_locked) = true;
            *(&mut current_lock_reason) = Some(rest.trim().to_string());
        } else if trimmed == "prunable" || trimmed.starts_with("prunable ") {
            *(&mut current_prunable) = true;
        }
    }

    flush(
        &mut worktrees,
        &mut current_path,
        &mut current_head,
        &mut current_branch,
        &mut current_locked,
        &mut current_lock_reason,
        &mut current_prunable,
    );

    worktrees
}

#[tauri::command]
pub fn get_worktrees(path: String) -> Result<Vec<WorktreeInfo>, String> {
    let output = git_command()
        .current_dir(&path)
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to list worktrees: {}", err.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_worktree_porcelain(&stdout))
}

#[tauri::command]
pub async fn add_worktree(
    path: String,
    worktree_path: String,
    branch: Option<String>,
    new_branch: Option<String>,
    commit_or_branch: Option<String>,
    detach: Option<bool>,
) -> Result<(), String> {
    let wt_path = worktree_path.trim();
    if wt_path.is_empty() {
        return Err("Worktree path cannot be empty".to_string());
    }

    let mut cmd = git_command();
    cmd.current_dir(&path).arg("worktree").arg("add");

    if detach.unwrap_or(false) {
        cmd.arg("--detach");
    }

    if let Some(nb) = new_branch {
        let nb_trimmed = nb.trim();
        if !nb_trimmed.is_empty() {
            cmd.arg("-b").arg(nb_trimmed);
        }
    }

    cmd.arg(wt_path);

    if let Some(b) = branch {
        let b_trimmed = b.trim();
        if !b_trimmed.is_empty() {
            cmd.arg(b_trimmed);
        }
    } else if let Some(cb) = commit_or_branch {
        let cb_trimmed = cb.trim();
        if !cb_trimmed.is_empty() {
            cmd.arg(cb_trimmed);
        }
    }

    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to add worktree: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_worktree(path: String, worktree_path: String, force: bool) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.current_dir(&path).arg("worktree").arg("remove");
    if force {
        cmd.arg("--force");
    }
    cmd.arg(worktree_path.trim());

    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to remove worktree: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn lock_worktree(path: String, worktree_path: String, reason: Option<String>) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.current_dir(&path).arg("worktree").arg("lock");
    if let Some(r) = reason {
        let r_trimmed = r.trim();
        if !r_trimmed.is_empty() {
            cmd.arg("--reason").arg(r_trimmed);
        }
    }
    cmd.arg(worktree_path.trim());

    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to lock worktree: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn unlock_worktree(path: String, worktree_path: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("worktree")
        .arg("unlock")
        .arg(worktree_path.trim())
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to unlock worktree: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn prune_worktrees(path: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("worktree")
        .arg("prune")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to prune worktrees: {}", err.trim()));
    }
    Ok(())
}

#[cfg(test)]
mod worktree_tests {
    use super::*;

    #[test]
    fn parses_worktree_porcelain_correctly() {
        let sample = r#"worktree /path/to/main
HEAD 0123456789abcdef0123456789abcdef01234567
branch refs/heads/main

worktree /path/to/feature-1
HEAD fedcba9876543210fedcba9876543210fedcba98
branch refs/heads/feature-1
locked working on urgent fix

worktree /path/to/detached-wt
HEAD aaaaaabbbbbbccccccddddddeeeeeeffffff0000
detached

worktree /path/to/prunable-wt
HEAD bbbbbbccccccddddddeeeeeeffffff0000001111
branch refs/heads/test-branch
prunable gitdir file points to non-existent location
"#;

        let worktrees = parse_worktree_porcelain(sample);
        assert_eq!(worktrees.len(), 4);

        // Main worktree
        assert_eq!(worktrees[0].path, "/path/to/main");
        assert_eq!(worktrees[0].branch.as_deref(), Some("main"));
        assert_eq!(worktrees[0].commit_hash, "0123456789abcdef0123456789abcdef01234567");
        assert!(worktrees[0].is_main);
        assert!(!worktrees[0].is_locked);
        assert!(!worktrees[0].is_prunable);

        // Locked worktree
        assert_eq!(worktrees[1].path, "/path/to/feature-1");
        assert_eq!(worktrees[1].branch.as_deref(), Some("feature-1"));
        assert!(!worktrees[1].is_main);
        assert!(worktrees[1].is_locked);
        assert_eq!(worktrees[1].lock_reason.as_deref(), Some("working on urgent fix"));

        // Detached worktree
        assert_eq!(worktrees[2].path, "/path/to/detached-wt");
        assert_eq!(worktrees[2].branch, None);
        assert!(!worktrees[2].is_main);

        // Prunable worktree
        assert_eq!(worktrees[3].path, "/path/to/prunable-wt");
        assert_eq!(worktrees[3].branch.as_deref(), Some("test-branch"));
        assert!(worktrees[3].is_prunable);
    }
}

#[cfg(test)]
mod tag_and_worktree_live_tests {
    use super::*;
    use crate::commands::tags::{create_tag, delete_tag, get_tags};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn git(path: &std::path::Path, args: &[&str]) {
        let status = git_command().args(args).current_dir(path).status().unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    #[test]
    fn creates_and_deletes_tags_and_worktrees() {
        tauri::async_runtime::block_on(async {
            let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let base_dir = std::env::temp_dir().join(format!("begit-repo-{}", unique));
            let wt_dir = std::env::temp_dir().join(format!("begit-wt-{}", unique));
            fs::create_dir_all(&base_dir).unwrap();
            git(&base_dir, &["init"]);
            git(&base_dir, &["config", "user.name", "Test Dev"]);
            git(&base_dir, &["config", "user.email", "dev@example.com"]);
            fs::write(base_dir.join("file.txt"), "hello world\n").unwrap();
            git(&base_dir, &["add", "."]);
            git(&base_dir, &["commit", "-m", "Initial commit"]);

            let path_str = base_dir.to_string_lossy().to_string();

            // 1. Create tag
            let create_res = create_tag(
                path_str.clone(),
                "v1.0.0".to_string(),
                None,
                Some("Release v1.0.0".to_string()),
                true,
                None,
            ).await;
            assert!(create_res.is_ok(), "Failed to create tag: {:?}", create_res);

            let tags = get_tags(path_str.clone()).unwrap();
            assert!(tags.contains(&"v1.0.0".to_string()));

            // Delete tag
            let del_res = delete_tag(path_str.clone(), "v1.0.0".to_string(), None).await;
            assert!(del_res.is_ok(), "Failed to delete tag: {:?}", del_res);
            let tags_after = get_tags(path_str.clone()).unwrap();
            assert!(!tags_after.contains(&"v1.0.0".to_string()));

            // 2. Add worktree
            let wt_path_str = wt_dir.to_string_lossy().to_string();
            let add_wt_res = add_worktree(
                path_str.clone(),
                wt_path_str.clone(),
                None,
                Some("feature-wt".to_string()),
                None,
                None,
            ).await;
            assert!(add_wt_res.is_ok(), "Failed to add worktree: {:?}", add_wt_res);

            let worktrees = get_worktrees(path_str.clone()).unwrap();
            assert_eq!(worktrees.len(), 2);
            assert!(worktrees[0].is_main);
            assert_eq!(worktrees[1].branch.as_deref(), Some("feature-wt"));

            // Lock worktree
            let lock_res = lock_worktree(path_str.clone(), wt_path_str.clone(), Some("Work in progress".to_string())).await;
            assert!(lock_res.is_ok());

            let worktrees_locked = get_worktrees(path_str.clone()).unwrap();
            assert!(worktrees_locked[1].is_locked);
            assert_eq!(worktrees_locked[1].lock_reason.as_deref(), Some("Work in progress"));

            // Unlock worktree
            let unlock_res = unlock_worktree(path_str.clone(), wt_path_str.clone()).await;
            assert!(unlock_res.is_ok());

            // Remove worktree
            let rm_res = remove_worktree(path_str.clone(), wt_path_str.clone(), true).await;
            assert!(rm_res.is_ok());

            let worktrees_end = get_worktrees(path_str.clone()).unwrap();
            assert_eq!(worktrees_end.len(), 1);

            let _ = fs::remove_dir_all(base_dir);
            let _ = fs::remove_dir_all(wt_dir);
        });
    }
}
