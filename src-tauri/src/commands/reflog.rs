use crate::utils::git_command;
use crate::models::ReflogEntry;

#[tauri::command]
pub fn get_reflog(path: String, ref_name: Option<String>, max_count: Option<usize>) -> Result<Vec<ReflogEntry>, String> {
    let target_ref = ref_name.unwrap_or_else(|| "HEAD".to_string());
    let count = max_count.unwrap_or(200);

    let output = git_command()
        .arg("reflog")
        .arg("show")
        .arg("--format=%gD|%H|%h|%gs|%gd|%cr|%ci|%an|%ae")
        .arg("--date=iso")
        .arg("-n")
        .arg(count.to_string())
        .arg(&target_ref)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git reflog error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 9 {
            let selector = parts[0].to_string();
            let hash = parts[1].to_string();
            let short_hash = parts[2].to_string();
            let raw_msg = parts[3].to_string();
            let date_relative = parts[5].to_string();
            let date_iso = parts[6].to_string();
            let author_name = parts[7].to_string();
            let author_email = parts[8].to_string();

            let lower_msg = raw_msg.to_lowercase();
            let action = if lower_msg.starts_with("commit:") || lower_msg.starts_with("commit (initial):") || lower_msg.starts_with("commit (amend):") {
                "commit"
            } else if lower_msg.starts_with("checkout:") {
                "checkout"
            } else if lower_msg.starts_with("rebase:") || lower_msg.starts_with("rebase (") {
                "rebase"
            } else if lower_msg.starts_with("reset:") {
                "reset"
            } else if lower_msg.starts_with("cherry-pick:") {
                "cherry-pick"
            } else if lower_msg.starts_with("merge") {
                "merge"
            } else if lower_msg.starts_with("pull:") {
                "pull"
            } else if lower_msg.starts_with("branch:") {
                "branch"
            } else if lower_msg.starts_with("clone:") {
                "clone"
            } else {
                "other"
            }.to_string();

            entries.push(ReflogEntry {
                selector,
                hash,
                short_hash,
                action,
                message: raw_msg,
                date_relative,
                date_iso,
                author_name,
                author_email,
            });
        }
    }

    Ok(entries)
}

#[tauri::command]
pub async fn reset_to_reflog_entry(path: String, hash: String, mode: String) -> Result<(), String> {
    let reset_mode = match mode.as_str() {
        "soft" => "--soft",
        "mixed" => "--mixed",
        "hard" => "--hard",
        _ => return Err("Invalid reset mode".to_string()),
    };

    let output = git_command()
        .args(["reset", reset_mode, &hash])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Reset to reflog entry failed: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn create_branch_from_reflog(path: String, branch_name: String, commit_hash: String) -> Result<(), String> {
    let output = git_command()
        .args(["branch", &branch_name, &commit_hash])
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create branch from reflog commit: {}", err));
    }
    Ok(())
}

#[cfg(test)]
mod reflog_tests {
    use super::*;
    use crate::commands::staging::apply_patch;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn git(path: &std::path::Path, args: &[&str]) {
        let status = git_command().args(args).current_dir(path).status().unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    #[test]
    fn tests_reflog_patch_and_conflict_operations() {
        tauri::async_runtime::block_on(async {
            let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let base_dir = std::env::temp_dir().join(format!("begit-power-{}", unique));
            fs::create_dir_all(&base_dir).unwrap();
            let path_str = base_dir.to_string_lossy().to_string();

            git(&base_dir, &["init"]);
            git(&base_dir, &["config", "user.name", "Test Dev"]);
            git(&base_dir, &["config", "user.email", "dev@example.com"]);

            // 1. Commit 1
            fs::write(base_dir.join("sample.txt"), "line 1\nline 2\nline 3\n").unwrap();
            git(&base_dir, &["add", "."]);
            git(&base_dir, &["commit", "-m", "Commit 1"]);

            // 2. Commit 2
            fs::write(base_dir.join("sample.txt"), "line 1\nline 2 modified\nline 3\n").unwrap();
            git(&base_dir, &["add", "."]);
            git(&base_dir, &["commit", "-m", "Commit 2"]);

            // Test Reflog
            let reflog = get_reflog(path_str.clone(), None, Some(10)).unwrap();
            assert!(!reflog.is_empty());
            assert_eq!(reflog[0].action, "commit");

            // Test Create Branch from Reflog
            let create_br_res = create_branch_from_reflog(path_str.clone(), "recovered-br".to_string(), reflog[0].hash.clone()).await;
            assert!(create_br_res.is_ok());

            // Test Apply Patch (Line / Hunk staging)
            fs::write(base_dir.join("sample.txt"), "line 1\nline 2 modified\nline 3\nline 4 added\n").unwrap();
            let patch = "diff --git a/sample.txt b/sample.txt\n--- a/sample.txt\n+++ b/sample.txt\n@@ -3,1 +3,2 @@\n line 3\n+line 4 added\n";
            let apply_res = apply_patch(path_str.clone(), patch.to_string(), true, false).await;
            assert!(apply_res.is_ok(), "apply_patch failed: {:?}", apply_res);

            // Test unapply / unstage patch
            let unstage_res = apply_patch(path_str.clone(), patch.to_string(), true, true).await;
            assert!(unstage_res.is_ok(), "unstage patch failed: {:?}", unstage_res);

            // Clean up
            let _ = fs::remove_dir_all(base_dir);
        });
    }
}
