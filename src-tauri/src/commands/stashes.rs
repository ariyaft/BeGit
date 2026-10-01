use crate::utils::git_command;
use crate::models::Stash;

#[tauri::command]
pub fn get_stashes(path: String) -> Result<Vec<Stash>, String> {
    let output = git_command()
        .arg("stash")
        .arg("list")
        .arg("--format=%gd|%gs")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if err.contains("not a git repository") {
            return Err(format!("Git error: {}", err));
        }
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut stashes = Vec::new();
    
    for line in stdout.lines() {
        if line.trim().is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(2, '|').collect();
        if parts.len() == 2 {
            stashes.push(Stash {
                id: parts[0].to_string(),
                message: parts[1].to_string(),
            });
        }
    }
    
    Ok(stashes)
}

#[tauri::command]
pub fn drop_stash(path: String, stash_id: String) -> Result<(), String> {
    let output = git_command()
        .arg("stash")
        .arg("drop")
        .arg(&stash_id)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub fn stash_changes(path: String, message: String) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.arg("stash").arg("save");
    if !message.is_empty() {
        cmd.arg(&message);
    }
    let output = cmd.current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub fn pop_stash(path: String) -> Result<(), String> {
    let output = git_command()
        .arg("stash")
        .arg("pop")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
