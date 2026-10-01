use crate::utils::git_command;
use crate::models::Branch;

#[tauri::command]
pub fn get_branches(path: String) -> Result<Vec<Branch>, String> {
    let output = git_command()
        .arg("for-each-ref")
        .arg("--format=%(HEAD)|%(refname:short)|%(upstream:track)")
        .arg("refs/heads")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();
    
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            let active = parts[0].trim() == "*";
            let name = parts[1].to_string();
            let mut ahead = 0;
            let mut behind = 0;
            
            if parts.len() > 2 {
                let track = parts[2];
                if track.starts_with('[') && track.ends_with(']') {
                    let content = &track[1..track.len()-1];
                    for part in content.split(", ") {
                        if part.starts_with("ahead ") {
                            ahead = part["ahead ".len()..].parse().unwrap_or(0);
                        } else if part.starts_with("behind ") {
                            behind = part["behind ".len()..].parse().unwrap_or(0);
                        }
                    }
                }
            }
            
            branches.push(Branch { name, active, ahead, behind });
        }
    }
    
    Ok(branches)
}

#[tauri::command]
pub fn get_remote_branches(path: String) -> Result<Vec<Branch>, String> {
    let output = git_command()
        .arg("branch")
        .arg("-r")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();
    
    for line in stdout.lines() {
        let name = line.trim().to_string();
        // Skip HEAD pointer lines like "origin/HEAD -> origin/main"
        if !name.is_empty() && !name.contains("->") {
            branches.push(Branch { name, active: false, ahead: 0, behind: 0 });
        }
    }
    
    Ok(branches)
}

#[tauri::command]
pub async fn checkout_branch(path: String, branch_name: String) -> Result<(), String> {
    let local_name = if branch_name.starts_with("origin/") {
        branch_name.trim_start_matches("origin/").to_string()
    } else {
        branch_name.clone()
    };
    
    let check_local = git_command()
        .arg("show-ref")
        .arg("--verify")
        .arg("--quiet")
        .arg(format!("refs/heads/{}", local_name))
        .current_dir(&path)
        .status()
        .map_err(|e| e.to_string())?;

    let mut cmd = git_command();
    cmd.current_dir(&path).arg("checkout");
    
    if check_local.success() {
        cmd.arg(&local_name);
    } else if branch_name.starts_with("origin/") {
        cmd.arg("-t").arg(&branch_name);
    } else {
        cmd.arg(&branch_name);
    }

    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git checkout error: {}", err));
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_branch(path: String, branch_name: String, force: bool) -> Result<(), String> {
    let local_name = if branch_name.starts_with("origin/") {
        branch_name["origin/".len()..].to_string()
    } else {
        branch_name.clone()
    };

    let mut cmd = git_command();
    cmd.current_dir(&path).arg("branch");
    if force {
        cmd.arg("-D");
    } else {
        cmd.arg("-d");
    }
    cmd.arg(&local_name);

    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to delete branch: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn create_branch(path: String, new_branch: String, base_branch: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("checkout")
        .arg("-b")
        .arg(&new_branch)
        .arg(&base_branch)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create branch: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_remote_branch(path: String, branch_name: String) -> Result<(), String> {
    let (remote, branch_to_delete) = if let Some((r, b)) = branch_name.split_once('/') {
        (r, b)
    } else {
        ("origin", branch_name.as_str())
    };

    let output = git_command()
        .current_dir(&path)
        .arg("push")
        .arg(remote)
        .arg("--delete")
        .arg(branch_to_delete)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to delete remote branch: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn rename_branch(path: String, old_name: String, new_name: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("branch")
        .arg("-m")
        .arg(&old_name)
        .arg(&new_name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to rename branch: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn set_upstream(path: String, branch_name: String) -> Result<(), String> {
    let remote_ref = format!("origin/{}", branch_name);
    let output = git_command()
        .current_dir(&path)
        .arg("branch")
        .arg("--set-upstream-to")
        .arg(&remote_ref)
        .arg(&branch_name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to set upstream: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn rebase_branch(path: String, branch_name: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("rebase")
        .arg(&branch_name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let out = String::from_utf8_lossy(&output.stdout);
        let git_dir = std::path::Path::new(&path).join(".git");
        let is_rebasing = git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists();
        if !is_rebasing {
            let _ = git_command()
                .current_dir(&path)
                .args(["rebase", "--abort"])
                .output();
        }
        return Err(format!("Rebase failed: {}{}", err, out));
    }
    Ok(())
}

#[tauri::command]
pub async fn fast_forward_merge(path: String, branch_name: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("merge")
        .arg("--ff-only")
        .arg(&branch_name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Fast-forward merge failed: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn merge_branch(path: String, branch_name: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("merge")
        .arg(&branch_name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Merge failed:\n{}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn push_branch(path: String, branch_name: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("push")
        .arg("--set-upstream")
        .arg("origin")
        .arg(&branch_name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to push branch: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub fn reset_branch(path: String, hash: String, mode: String) -> Result<(), String> {
    let reset_mode = match mode.as_str() {
        "soft" => "--soft",
        "mixed" => "--mixed",
        "hard" => "--hard",
        _ => return Err("Unsupported reset mode".to_string()),
    };

    let output = git_command()
        .current_dir(&path)
        .arg("reset")
        .arg(reset_mode)
        .arg(&hash)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Reset failed: {}", err));
    }

    Ok(())
}
