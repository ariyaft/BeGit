use std::io::Write;
use crate::utils::git_command;
use crate::models::{CommitDetails, FileChange};

#[tauri::command]
pub fn get_uncommitted_changes(path: String) -> Result<CommitDetails, String> {
    let output = git_command()
        .arg("status")
        .arg("--porcelain")
        .arg("-uall")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git status error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut staged_files = Vec::new();
    let mut unstaged_files = Vec::new();
    
    for line in stdout.lines() {
        if line.len() > 3 {
            let x = line.chars().next().unwrap();
            let y = line.chars().nth(1).unwrap();
            
            let mut file_path = line[3..].trim().to_string();
            if file_path.contains(" -> ") {
                if let Some(idx) = file_path.find(" -> ") {
                    file_path = file_path[idx + 4..].to_string();
                }
            }
            if file_path.starts_with('"') && file_path.ends_with('"') {
                file_path = file_path[1..file_path.len()-1].to_string();
            }

            if x != ' ' && x != '?' {
                let mut status = x.to_string();
                if status == "R" || status == "C" { status = "M".to_string(); }
                staged_files.push(FileChange { status, path: file_path.clone() });
            }
            if y != ' ' {
                let mut status = y.to_string();
                if status == "?" { status = "A".to_string(); }
                else if status == "R" || status == "C" { status = "M".to_string(); }
                unstaged_files.push(FileChange { status, path: file_path.clone() });
            }
        }
    }
    
    Ok(CommitDetails { hash: "uncommitted".to_string(), files: Vec::new(), staged_files: Some(staged_files), unstaged_files: Some(unstaged_files) })
}

#[tauri::command]
pub async fn stage_file(path: String, file_path: String) -> Result<(), String> {
    let output = git_command()
        .arg("add")
        .arg("--")
        .arg(&file_path)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to stage file: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn unstage_file(path: String, file_path: String) -> Result<(), String> {
    let output = git_command()
        .arg("reset")
        .arg("HEAD")
        .arg("--")
        .arg(&file_path)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if err.contains("ambiguous argument 'HEAD'") {
            let output2 = git_command()
                .arg("rm")
                .arg("-r")
                .arg("--cached")
                .arg("--")
                .arg(&file_path)
                .current_dir(&path)
                .output()
                .map_err(|e| e.to_string())?;
            if !output2.status.success() {
                return Err(format!("Failed to unstage file: {}", String::from_utf8_lossy(&output2.stderr)));
            }
        } else {
            return Err(format!("Failed to unstage file: {}", err));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn commit_changes(path: String, message: String, amend: Option<bool>) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.arg("commit");
    if amend.unwrap_or(false) {
        cmd.arg("--amend");
    }
    cmd.arg("-m").arg(&message).current_dir(&path);
    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Failed to commit changes:\n{}{}", err, out));
    }
    Ok(())
}

#[tauri::command]
pub fn get_tracked_files(path: String) -> Result<Vec<String>, String> {
    let output = git_command()
        .arg("ls-tree")
        .arg("-r")
        .arg("--name-only")
        .arg("HEAD")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            files.push(trimmed.to_string());
        }
    }

    Ok(files)
}

#[tauri::command]
pub async fn apply_patch(path: String, patch_content: String, cached: bool, reverse: bool) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.current_dir(&path)
        .arg("apply")
        .arg("--whitespace=nowarn")
        .arg("--recount");

    if cached {
        cmd.arg("--cached");
    }
    if reverse {
        cmd.arg("--reverse");
    }
    cmd.arg("-");
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| format!("Failed to spawn git apply: {}", e))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(patch_content.as_bytes()).map_err(|e| format!("Failed to write patch to stdin: {}", e))?;
    }

    let output = child.wait_with_output().map_err(|e| format!("git apply execution error: {}", e))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Failed to apply patch:\n{}{}", err, out));
    }
    Ok(())
}
