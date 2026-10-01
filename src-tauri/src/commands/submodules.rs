use std::collections::HashMap;
use crate::utils::git_command;
use std::fs;
use crate::models::SubmoduleInfo;

pub fn parse_submodule_status(root_path: &str, stdout: &str, gitmodules_content: Option<&str>) -> Vec<SubmoduleInfo> {
    let mut map_urls: HashMap<String, String> = HashMap::new();
    let mut map_branches: HashMap<String, String> = HashMap::new();

    if let Some(content) = gitmodules_content {
        let mut current_sub_path = String::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("[submodule") {
                current_sub_path.clear();
            } else if let Some(rest) = trimmed.strip_prefix("path =") {
                current_sub_path = rest.trim().to_string();
            } else if let Some(rest) = trimmed.strip_prefix("url =") {
                if !current_sub_path.is_empty() {
                    map_urls.insert(current_sub_path.clone(), rest.trim().to_string());
                }
            } else if let Some(rest) = trimmed.strip_prefix("branch =") {
                if !current_sub_path.is_empty() {
                    map_branches.insert(current_sub_path.clone(), rest.trim().to_string());
                }
            }
        }
    }

    let mut submodules = Vec::new();

    for line in stdout.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }

        let prefix_char = line.chars().next().unwrap_or(' ');
        let (status, is_initialized) = match prefix_char {
            '-' => ("uninitialized".to_string(), false),
            '+' => ("modified".to_string(), true),
            'U' => ("conflict".to_string(), false),
            _ => ("initialized".to_string(), true),
        };

        let rest = if prefix_char == ' ' || prefix_char == '-' || prefix_char == '+' || prefix_char == 'U' {
            &line[1..]
        } else {
            line
        };

        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let commit_hash = parts[0].to_string();
        let sub_path = if parts.len() > 1 {
            parts[1].to_string()
        } else {
            "".to_string()
        };

        let branch = if parts.len() > 2 && parts[2].starts_with('(') && parts.last().map(|s| s.ends_with(')')).unwrap_or(false) {
            let joined = parts[2..].join(" ");
            Some(joined.trim_start_matches('(').trim_end_matches(')').to_string())
        } else {
            map_branches.get(&sub_path).cloned()
        };

        let full_path = std::path::Path::new(root_path)
            .join(&sub_path)
            .to_string_lossy()
            .to_string();

        let name = sub_path.split('/').last().unwrap_or(&sub_path).to_string();
        let url = map_urls.get(&sub_path).cloned().unwrap_or_default();

        submodules.push(SubmoduleInfo {
            name,
            path: sub_path,
            full_path,
            url,
            commit_hash,
            status,
            is_initialized,
            branch,
        });
    }

    submodules
}

#[tauri::command]
pub fn get_submodules(path: String) -> Result<Vec<SubmoduleInfo>, String> {
    let output = git_command()
        .current_dir(&path)
        .arg("submodule")
        .arg("status")
        .arg("--recursive")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let gitmodules_path = std::path::Path::new(&path).join(".gitmodules");
    let gitmodules_content = fs::read_to_string(gitmodules_path).ok();

    Ok(parse_submodule_status(&path, &stdout, gitmodules_content.as_deref()))
}

#[tauri::command]
pub async fn init_submodules(path: String, sub_path: Option<String>, recursive: bool) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.current_dir(&path).arg("submodule").arg("update").arg("--init");
    if recursive {
        cmd.arg("--recursive");
    }
    if let Some(sp) = sub_path {
        let sp_trimmed = sp.trim();
        if !sp_trimmed.is_empty() {
            cmd.arg("--").arg(sp_trimmed);
        }
    }

    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to initialize submodules: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn update_submodules(
    path: String,
    sub_path: Option<String>,
    remote: bool,
    recursive: bool,
) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.current_dir(&path).arg("submodule").arg("update");
    if remote {
        cmd.arg("--remote");
    }
    if recursive {
        cmd.arg("--recursive");
    }
    if let Some(sp) = sub_path {
        let sp_trimmed = sp.trim();
        if !sp_trimmed.is_empty() {
            cmd.arg("--").arg(sp_trimmed);
        }
    }

    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to update submodules: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn sync_submodules(path: String, sub_path: Option<String>, recursive: bool) -> Result<(), String> {
    let mut cmd = git_command();
    cmd.current_dir(&path).arg("submodule").arg("sync");
    if recursive {
        cmd.arg("--recursive");
    }
    if let Some(sp) = sub_path {
        let sp_trimmed = sp.trim();
        if !sp_trimmed.is_empty() {
            cmd.arg("--").arg(sp_trimmed);
        }
    }

    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to sync submodules: {}", err.trim()));
    }
    Ok(())
}

#[tauri::command]
pub async fn add_submodule(
    path: String,
    url: String,
    sub_path: Option<String>,
    branch: Option<String>,
) -> Result<(), String> {
    let url_trimmed = url.trim();
    if url_trimmed.is_empty() {
        return Err("Submodule repository URL cannot be empty".to_string());
    }

    let mut cmd = git_command();
    cmd.current_dir(&path).arg("submodule").arg("add");

    if let Some(b) = branch {
        let b_trimmed = b.trim();
        if !b_trimmed.is_empty() {
            cmd.arg("-b").arg(b_trimmed);
        }
    }

    cmd.arg(url_trimmed);

    if let Some(sp) = sub_path {
        let sp_trimmed = sp.trim();
        if !sp_trimmed.is_empty() {
            cmd.arg(sp_trimmed);
        }
    }

    let output = cmd.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to add submodule: {}", err.trim()));
    }
    Ok(())
}

#[cfg(test)]
mod submodule_tests {
    use super::*;

    #[test]
    fn parses_submodule_status_correctly() {
        let status_output = r#" 352a945b637920703e7e6f8510f2294101e4ec33 vendor/lib (v1.0.0)
-e98b71d9f049bf6f809074a2b963174249a55734 packages/core
+a1b2c3d4e5f60123456789abcdef0123456789ab docs/theme (heads/main)
"#;

        let gitmodules = r#"[submodule "vendor/lib"]
	path = vendor/lib
	url = https://github.com/example/lib.git
	branch = v1.0.0
[submodule "packages/core"]
	path = packages/core
	url = git@github.com:example/core.git
"#;

        let submodules = parse_submodule_status("/mock/repo", status_output, Some(gitmodules));
        assert_eq!(submodules.len(), 3);

        // 1. Initialized
        assert_eq!(submodules[0].name, "lib");
        assert_eq!(submodules[0].path, "vendor/lib");
        assert_eq!(submodules[0].url, "https://github.com/example/lib.git");
        assert_eq!(submodules[0].commit_hash, "352a945b637920703e7e6f8510f2294101e4ec33");
        assert_eq!(submodules[0].status, "initialized");
        assert!(submodules[0].is_initialized);
        assert_eq!(submodules[0].branch.as_deref(), Some("v1.0.0"));

        // 2. Uninitialized
        assert_eq!(submodules[1].name, "core");
        assert_eq!(submodules[1].path, "packages/core");
        assert_eq!(submodules[1].url, "git@github.com:example/core.git");
        assert_eq!(submodules[1].status, "uninitialized");
        assert!(!submodules[1].is_initialized);

        // 3. Modified
        assert_eq!(submodules[2].name, "theme");
        assert_eq!(submodules[2].path, "docs/theme");
        assert_eq!(submodules[2].status, "modified");
        assert!(submodules[2].is_initialized);
    }
}
