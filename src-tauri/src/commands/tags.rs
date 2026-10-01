use crate::utils::git_command;

#[tauri::command]
pub fn get_tags(path: String) -> Result<Vec<String>, String> {
    let output = git_command()
        .arg("tag")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut tags = Vec::new();
    
    for line in stdout.lines() {
        let name = line.trim().to_string();
        if !name.is_empty() {
            tags.push(name);
        }
    }
    
    Ok(tags)
}

#[tauri::command]
pub async fn create_tag(
    path: String,
    name: String,
    target_commit: Option<String>,
    message: Option<String>,
    annotate: bool,
    push_remote: Option<String>,
) -> Result<(), String> {
    let name_trimmed = name.trim();
    if name_trimmed.is_empty() {
        return Err("Tag name cannot be empty".to_string());
    }

    let mut cmd = git_command();
    cmd.current_dir(&path).arg("tag");

    let is_annotated = annotate || message.as_ref().map(|m| !m.trim().is_empty()).unwrap_or(false);
    if is_annotated {
        cmd.arg("-a");
        let msg = message.unwrap_or_default();
        let msg_str = if msg.trim().is_empty() {
            name_trimmed.to_string()
        } else {
            msg
        };
        cmd.arg("-m").arg(&msg_str);
    }

    cmd.arg(name_trimmed);

    if let Some(target) = target_commit {
        let t = target.trim();
        if !t.is_empty() {
            cmd.arg(t);
        }
    }

    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create tag: {}", err.trim()));
    }

    if let Some(remote) = push_remote {
        let remote_trimmed = remote.trim();
        if !remote_trimmed.is_empty() {
            let push_output = git_command()
                .current_dir(&path)
                .arg("push")
                .arg(remote_trimmed)
                .arg(name_trimmed)
                .output()
                .map_err(|e| e.to_string())?;

            if !push_output.status.success() {
                let err = String::from_utf8_lossy(&push_output.stderr);
                return Err(format!("Tag created locally, but failed to push: {}", err.trim()));
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn push_tag(path: String, name: String, remote: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("push")
        .arg(remote.trim())
        .arg(name.trim())
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to push tag: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn push_all_tags(path: String, remote: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("push")
        .arg(remote.trim())
        .arg("--tags")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to push tags: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_tag(path: String, name: String, delete_remote: Option<String>) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("tag")
        .arg("-d")
        .arg(name.trim())
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to delete tag: {}", err.trim()));
    }

    if let Some(remote) = delete_remote {
        let remote_trimmed = remote.trim();
        if !remote_trimmed.is_empty() {
            let push_output = git_command()
                .current_dir(&path)
                .arg("push")
                .arg(remote_trimmed)
                .arg("--delete")
                .arg(name.trim())
                .output()
                .map_err(|e| e.to_string())?;

            if !push_output.status.success() {
                let err = String::from_utf8_lossy(&push_output.stderr);
                return Err(format!("Local tag deleted, but failed to delete from remote: {}", err.trim()));
            }
        }
    }

    Ok(())
}
