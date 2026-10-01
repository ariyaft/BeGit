use std::collections::HashSet;
use crate::utils::git_command;
use crate::models::RemoteInfo;
use crate::utils::base64_encode;

#[tauri::command]
pub async fn get_remotes(path: String) -> Result<Vec<serde_json::Value>, String> {
    let output = git_command()
        .current_dir(&path)
        .arg("remote")
        .arg("-v")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Failed to get remotes".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut remotes = Vec::new();
    let mut seen = HashSet::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[0];
            let url = parts[1];
            if seen.insert(name) {
                remotes.push(serde_json::json!({
                    "name": name,
                    "url": url
                }));
            }
        }
    }
    Ok(remotes)
}

#[tauri::command]
pub async fn add_remote(path: String, name: String, url: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("remote")
        .arg("add")
        .arg(&name)
        .arg(&url)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to add remote: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_remote(path: String, name: String) -> Result<(), String> {
    let output = git_command()
        .current_dir(&path)
        .arg("remote")
        .arg("remove")
        .arg(&name)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to remove remote: {}", err));
    }
    Ok(())
}

#[tauri::command]
pub async fn fetch_remote(path: String) -> Result<(), String> {
    let output = git_command()
        .arg("fetch")
        .arg("--all")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git fetch error: {}", err));
    }

    Ok(())
}

#[tauri::command]
pub async fn pull_branch(path: String, branch_name: String) -> Result<(), String> {
    let current_branch_output = git_command()
        .current_dir(&path)
        .arg("branch")
        .arg("--show-current")
        .output()
        .map_err(|e| e.to_string())?;
        
    let current_branch = String::from_utf8_lossy(&current_branch_output.stdout).trim().to_string();

    let output = if current_branch == branch_name {
        git_command()
            .current_dir(&path)
            .arg("pull")
            .arg("origin")
            .arg(&branch_name)
            .output()
            .map_err(|e| e.to_string())?
    } else {
        git_command()
            .current_dir(&path)
            .arg("fetch")
            .arg("origin")
            .arg(format!("{}:{}", branch_name, branch_name))
            .output()
            .map_err(|e| e.to_string())?
    };

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to pull branch: {}", err));
    }
    Ok(())
}

pub fn parse_remote_url(name: &str, raw_url: &str) -> Option<RemoteInfo> {
    let url = raw_url.trim();
    if url.is_empty() {
        return None;
    }

    let clean = url.trim_end_matches('/').trim_end_matches(".git");

    let (host, path_part) = if clean.starts_with("git@") {
        let after_git = &clean[4..];
        if let Some((h, p)) = after_git.split_once(':') {
            (h, p.trim_start_matches('/'))
        } else {
            return None;
        }
    } else if clean.starts_with("ssh://") {
        let without_proto = &clean[6..];
        let without_user = if let Some((_, rest)) = without_proto.split_once('@') {
            rest
        } else {
            without_proto
        };
        if let Some((h, p)) = without_user.split_once('/') {
            (h, p)
        } else {
            return None;
        }
    } else if clean.starts_with("http://") || clean.starts_with("https://") {
        let proto_end = if clean.starts_with("https://") { 8 } else { 7 };
        let after_proto = &clean[proto_end..];
        let without_auth = if let Some((_, rest)) = after_proto.split_once('@') {
            rest
        } else {
            after_proto
        };
        if let Some((h, p)) = without_auth.split_once('/') {
            (h, p)
        } else {
            return None;
        }
    } else {
        return None;
    };

    let host_lower = host.to_lowercase();
    let provider = if host_lower.contains("github.com") {
        "github"
    } else if host_lower.contains("gitlab") {
        "gitlab"
    } else if host_lower.contains("bitbucket.org") {
        "bitbucket"
    } else if host_lower.contains("azure.com") || host_lower.contains("visualstudio.com") {
        "azure_devops"
    } else {
        "other"
    };

    let path_part = path_part.trim_matches('/');
    let path_segments: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();
    if path_segments.is_empty() {
        return None;
    }

    let (owner, repo) = if path_segments.len() == 1 {
        ("".to_string(), path_segments[0].to_string())
    } else {
        let repo = path_segments.last().unwrap().to_string();
        let owner = path_segments[..path_segments.len() - 1].join("/");
        (owner, repo)
    };

    let web_url = match provider {
        "github" => format!("https://github.com/{}/{}", owner, repo),
        "gitlab" => format!("https://{}/{}/{}", host, owner, repo),
        "bitbucket" => format!("https://bitbucket.org/{}/{}", owner, repo),
        _ => if clean.starts_with("http") { clean.to_string() } else { format!("https://{}/{}", host, path_part) },
    };

    let api_base_url = match provider {
        "github" => "https://api.github.com".to_string(),
        "gitlab" => if host_lower == "gitlab.com" { "https://gitlab.com/api/v4".to_string() } else { format!("https://{}/api/v4", host) },
        _ => String::new(),
    };

    Some(RemoteInfo {
        name: name.to_string(),
        url: url.to_string(),
        provider: provider.to_string(),
        owner,
        repo,
        web_url,
        api_base_url,
    })
}

#[tauri::command]
pub async fn get_remote_info(path: String) -> Result<Option<RemoteInfo>, String> {
    let remotes = get_remotes(path.clone()).await?;
    if remotes.is_empty() {
        return Ok(None);
    }

    let origin_remote = remotes.iter().find(|r| {
        r.get("name").and_then(|n| n.as_str()) == Some("origin")
    }).or_else(|| remotes.first());

    if let Some(r) = origin_remote {
        let name = r.get("name").and_then(|n| n.as_str()).unwrap_or("origin");
        let url = r.get("url").and_then(|u| u.as_str()).unwrap_or("");
        return Ok(parse_remote_url(name, url));
    }

    Ok(None)
}

#[tauri::command]
pub async fn checkout_pull_request(
    path: String,
    pr_number: u32,
    branch_name: Option<String>,
    provider: Option<String>,
) -> Result<String, String> {
    if pr_number == 0 {
        return Err("Invalid pull request number".to_string());
    }

    let sanitized_branch = branch_name
        .map(|b| {
            let s: String = b.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
            let trimmed = s.trim_matches('-').to_string();
            if trimmed.is_empty() { "patch".to_string() } else { trimmed }
        })
        .unwrap_or_else(|| "patch".to_string());

    let local_branch = format!("pr/{}-{}", pr_number, sanitized_branch);
    let is_gitlab = provider.as_deref().map(|p| p.eq_ignore_ascii_case("gitlab")).unwrap_or(false);

    let refspec = if is_gitlab {
        format!("refs/merge-requests/{}/head:refs/heads/{}", pr_number, local_branch)
    } else {
        format!("refs/pull/{}/head:refs/heads/{}", pr_number, local_branch)
    };

    let fetch_res = git_command()
        .current_dir(&path)
        .arg("fetch")
        .arg("origin")
        .arg(&refspec)
        .arg("--update-head-ok")
        .arg("--force")
        .output()
        .map_err(|e| e.to_string())?;

    if !fetch_res.status.success() {
        let alt_refspec = if is_gitlab {
            format!("refs/pull/{}/head:refs/heads/{}", pr_number, local_branch)
        } else {
            format!("refs/merge-requests/{}/head:refs/heads/{}", pr_number, local_branch)
        };

        let alt_fetch = git_command()
            .current_dir(&path)
            .arg("fetch")
            .arg("origin")
            .arg(&alt_refspec)
            .arg("--update-head-ok")
            .arg("--force")
            .output()
            .map_err(|e| e.to_string())?;

        if !alt_fetch.status.success() {
            let err = String::from_utf8_lossy(&fetch_res.stderr);
            return Err(format!("Failed to fetch pull request ref: {}", err.trim()));
        }
    }

    let checkout_res = git_command()
        .current_dir(&path)
        .arg("checkout")
        .arg(&local_branch)
        .output()
        .map_err(|e| e.to_string())?;

    if !checkout_res.status.success() {
        let err = String::from_utf8_lossy(&checkout_res.stderr);
        return Err(format!("Failed to checkout branch {}: {}", local_branch, err.trim()));
    }

    Ok(local_branch)
}

#[tauri::command]
pub async fn clone_repository(
    url: String,
    target_path: String,
    branch: Option<String>,
    depth: Option<u32>,
    auth_type: Option<String>,
    username: Option<String>,
    token: Option<String>,
    ssh_key_path: Option<String>,
) -> Result<String, String> {
    let trimmed_url = url.trim();
    let trimmed_target = target_path.trim();
    if trimmed_url.is_empty() {
        return Err("Repository URL cannot be empty.".to_string());
    }
    if trimmed_target.is_empty() {
        return Err("Destination folder path cannot be empty.".to_string());
    }

    let mut cmd = git_command();
    cmd.env("GIT_TERMINAL_PROMPT", "0");

    if let Ok(current_path) = std::env::var("PATH") {
        let mut paths: Vec<&str> = current_path.split(':').collect();
        let common_paths = ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"];
        let mut added = false;
        for p in common_paths {
            if !paths.contains(&p) {
                paths.push(p);
                added = true;
            }
        }
        if added {
            cmd.env("PATH", paths.join(":"));
        }
    }

    let auth_mode = auth_type.as_deref().unwrap_or("none");
    if auth_mode == "token" || (token.is_some() && !token.as_ref().unwrap().trim().is_empty()) {
        if let Some(ref t) = token {
            let t_clean = t.trim();
            if !t_clean.is_empty() {
                let u_clean = username.as_deref().unwrap_or("").trim();
                let credentials = if !u_clean.is_empty() {
                    format!("{}:{}", u_clean, t_clean)
                } else {
                    format!("{}:{}", "x-access-token", t_clean)
                };
                let encoded = base64_encode(credentials.as_bytes());
                cmd.arg("-c").arg(format!("http.extraHeader=Authorization: Basic {}", encoded));
            }
        }
    } else if auth_mode == "ssh_key" || (ssh_key_path.is_some() && !ssh_key_path.as_ref().unwrap().trim().is_empty()) {
        if let Some(ref key) = ssh_key_path {
            let key_clean = key.trim();
            if !key_clean.is_empty() {
                let expanded_key = if key_clean.starts_with("~/") || key_clean == "~" {
                    if let Ok(home) = std::env::var("HOME") {
                        key_clean.replacen('~', &home, 1)
                    } else {
                        key_clean.to_string()
                    }
                } else {
                    key_clean.to_string()
                };
                cmd.arg("-c").arg(format!(
                    "core.sshCommand=ssh -i \"{}\" -o StrictHostKeyChecking=accept-new -o IdentitiesOnly=yes",
                    expanded_key
                ));
            }
        }
    }

    cmd.arg("clone").arg("--progress");

    if let Some(ref b) = branch {
        let b_trimmed = b.trim();
        if !b_trimmed.is_empty() {
            cmd.arg("--branch").arg(b_trimmed);
        }
    }

    if let Some(d) = depth {
        if d > 0 {
            cmd.arg("--depth").arg(d.to_string());
        }
    }

    cmd.arg("--").arg(trimmed_url).arg(trimmed_target);

    let output = cmd.output().map_err(|e| format!("Failed to execute git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let msg = if !stderr.trim().is_empty() {
            stderr
        } else if !stdout.trim().is_empty() {
            stdout
        } else {
            "Git clone failed with unknown error.".to_string()
        };

        let trimmed = msg.trim();
        if trimmed.contains("terminal prompts disabled")
            || trimmed.contains("could not read Username")
            || trimmed.contains("could not read Password")
            || trimmed.contains("Authentication failed")
            || trimmed.contains("Permission denied (publickey)")
            || trimmed.contains("Invalid username or password")
            || (trimmed.contains("Repository not found") && trimmed_url.contains("github.com"))
        {
            return Err(format!(
                "Authentication required or failed: {}. For private repositories, please provide a Personal Access Token (PAT) or an authorized SSH key in the Authentication section.",
                trimmed
            ));
        }

        return Err(trimmed.to_string());
    }

    Ok(trimmed_target.to_string())
}

#[cfg(test)]
mod clone_repository_tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn git(path: &std::path::Path, args: &[&str]) {
        let status = git_command().args(args).current_dir(path).status().unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    #[test]
    fn clones_local_repository_successfully() {
        tauri::async_runtime::block_on(async {
            let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let base_dir = std::env::temp_dir().join(format!("begit-clone-src-{}", unique));
            let clone_dest = std::env::temp_dir().join(format!("begit-clone-dest-{}", unique));
            fs::create_dir_all(&base_dir).unwrap();
            git(&base_dir, &["init"]);
            git(&base_dir, &["config", "user.name", "Test User"]);
            git(&base_dir, &["config", "user.email", "test@example.com"]);
            fs::write(base_dir.join("README.md"), "# Hello Clone\n").unwrap();
            git(&base_dir, &["add", "."]);
            git(&base_dir, &["commit", "-m", "Initial clone test commit"]);

            let result = clone_repository(
                base_dir.to_string_lossy().to_string(),
                clone_dest.to_string_lossy().to_string(),
                None,
                None,
                None,
                None,
                None,
                None,
            ).await;

            assert!(result.is_ok(), "Expected clone to succeed, got {:?}", result);
            assert!(clone_dest.join("README.md").exists(), "Expected cloned README to exist");

            let _ = fs::remove_dir_all(base_dir);
            let _ = fs::remove_dir_all(clone_dest);
        });
    }

    #[test]
    fn fails_on_empty_url() {
        tauri::async_runtime::block_on(async {
            let result = clone_repository("".to_string(), "/tmp/dest".to_string(), None, None, None, None, None, None).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Repository URL cannot be empty.");
        });
    }

    #[test]
    fn fails_on_empty_target() {
        tauri::async_runtime::block_on(async {
            let result = clone_repository("https://github.com/example/repo.git".to_string(), "  ".to_string(), None, None, None, None, None, None).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), "Destination folder path cannot be empty.");
        });
    }
}

#[cfg(test)]
mod remote_url_tests {
    use super::*;

    #[test]
    fn parses_github_ssh_url() {
        let info = parse_remote_url("origin", "git@github.com:ariyaft/BeGit.git").unwrap();
        assert_eq!(info.provider, "github");
        assert_eq!(info.owner, "ariyaft");
        assert_eq!(info.repo, "BeGit");
        assert_eq!(info.web_url, "https://github.com/ariyaft/BeGit");
        assert_eq!(info.api_base_url, "https://api.github.com");
    }

    #[test]
    fn parses_github_https_url() {
        let info = parse_remote_url("origin", "https://github.com/ariyaft/BeGit.git").unwrap();
        assert_eq!(info.provider, "github");
        assert_eq!(info.owner, "ariyaft");
        assert_eq!(info.repo, "BeGit");
        assert_eq!(info.web_url, "https://github.com/ariyaft/BeGit");
    }

    #[test]
    fn parses_gitlab_url() {
        let info = parse_remote_url("origin", "git@gitlab.com:group/subgroup/repo.git").unwrap();
        assert_eq!(info.provider, "gitlab");
        assert_eq!(info.owner, "group/subgroup");
        assert_eq!(info.repo, "repo");
        assert_eq!(info.web_url, "https://gitlab.com/group/subgroup/repo");
    }
}
