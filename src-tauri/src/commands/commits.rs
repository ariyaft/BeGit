use crate::utils::git_command;
use crate::models::{Author, BlameLine, Commit, CommitDetails, FileChange, ImageVersions, SignatureInfo};
use crate::utils::{git_image_data_url, image_mime_type};

#[tauri::command]
pub fn get_authors(path: String) -> Result<Vec<Author>, String> {
    let output = git_command()
        .arg("shortlog")
        .arg("-sn")
        .arg("--all")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut authors = Vec::new();
    
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if let Some((count_str, name)) = line.split_once('\t') {
            let count = count_str.trim().parse::<u32>().unwrap_or(0);
            authors.push(Author {
                name: name.trim().to_string(),
                commits: count,
            });
        }
    }
    
    Ok(authors)
}

#[tauri::command]
pub fn get_commits(path: String, limit: Option<u32>, skip: Option<u32>, file_path: Option<String>) -> Result<Vec<Commit>, String> {
    let run_git_log = || -> Result<std::process::Output, String> {
        let mut cmd = git_command();
        // Do not include %G* placeholders here. Those make Git verify each
        // signed commit (and may launch GPG hundreds of times for a graph),
        // which is especially costly on Windows. Signature data is optional
        // display metadata, not part of the graph topology.
        let format_str = "--format=%x1e%H%x1f%h%x1f%P%x1f%an%x1f%ar%x1f%ad%x1f%s%x1f%D";
        cmd.arg("log")
            .arg("--all")
            .arg("--date-order")
            .arg(format_str);
            
        if let Some(s) = skip {
            cmd.arg(format!("--skip={}", s));
        }
        
        cmd.arg("-n").arg(limit.unwrap_or(500).to_string());
        
        if let Some(ref file) = file_path {
            cmd.arg("--follow");
            cmd.arg("--");
            cmd.arg(file);
        }
        
        cmd.current_dir(&path)
            .output()
            .map_err(|e| e.to_string())
    };

    let output = run_git_log()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git log error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    
    for record in stdout.split('\x1e') {
        let record = record.trim();
        if record.is_empty() { continue; }
        
        let parts: Vec<&str> = record.split('\x1f').collect();
        if parts.len() < 7 { continue; }
        
        let hash = parts[0].trim().to_string();
        let hash_short = parts[1].trim().to_string();
        let parents = parts[2].split_whitespace().map(|s| s.trim().to_string()).collect();
        let author = parts[3].trim().to_string();
        let time = parts[4].trim().to_string();
        let exact_time = parts[5].trim().to_string();
        let message = parts[6].trim().to_string();
        let refs_str = parts.get(7).map(|s| s.trim()).unwrap_or("");
        
        let mut refs = Vec::new();
        if !refs_str.is_empty() {
            for r in refs_str.split(',') {
                let trimmed = r.trim();
                if !trimmed.is_empty() {
                    refs.push(trimmed.to_string());
                }
            }
        }

        let sig_code = parts.get(8).map(|s| s.trim()).unwrap_or("");
        let signature = match sig_code {
            "G" => Some(SignatureInfo {
                status: "verified".to_string(),
                key: parts.get(9).map(|s| s.trim().to_string()).unwrap_or_default(),
                signer: parts.get(10).map(|s| s.trim().to_string()).unwrap_or_default(),
                fingerprint: parts.get(11).map(|s| s.trim().to_string()).unwrap_or_default(),
            }),
            "U" | "E" => Some(SignatureInfo {
                status: "unverified".to_string(),
                key: parts.get(9).map(|s| s.trim().to_string()).unwrap_or_default(),
                signer: parts.get(10).map(|s| s.trim().to_string()).unwrap_or_default(),
                fingerprint: parts.get(11).map(|s| s.trim().to_string()).unwrap_or_default(),
            }),
            "B" => Some(SignatureInfo {
                status: "bad".to_string(),
                key: parts.get(9).map(|s| s.trim().to_string()).unwrap_or_default(),
                signer: parts.get(10).map(|s| s.trim().to_string()).unwrap_or_default(),
                fingerprint: parts.get(11).map(|s| s.trim().to_string()).unwrap_or_default(),
            }),
            "X" | "Y" => Some(SignatureInfo {
                status: "expired".to_string(),
                key: parts.get(9).map(|s| s.trim().to_string()).unwrap_or_default(),
                signer: parts.get(10).map(|s| s.trim().to_string()).unwrap_or_default(),
                fingerprint: parts.get(11).map(|s| s.trim().to_string()).unwrap_or_default(),
            }),
            "R" => Some(SignatureInfo {
                status: "revoked".to_string(),
                key: parts.get(9).map(|s| s.trim().to_string()).unwrap_or_default(),
                signer: parts.get(10).map(|s| s.trim().to_string()).unwrap_or_default(),
                fingerprint: parts.get(11).map(|s| s.trim().to_string()).unwrap_or_default(),
            }),
            _ => None,
        };
        
        commits.push(Commit { hash, hash_short, parents, author, time, exact_time, message, refs, signature });
    }
    
    Ok(commits)
}

#[tauri::command]
pub fn get_commit_branches(path: String, hash: String) -> Result<Vec<String>, String> {
    let output = git_command()
        .arg("branch")
        .arg("--contains")
        .arg(&hash)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();
    
    for line in stdout.lines() {
        let mut name = line.trim();
        if name.starts_with('*') {
            name = name[1..].trim();
        }
        if !name.is_empty() {
            branches.push(name.to_string());
        }
    }
    
    Ok(branches)
}

#[tauri::command]
pub fn get_commit_details(path: String, hash: String) -> Result<CommitDetails, String> {
    let output = git_command()
        .arg("show")
        .arg("--name-status")
        .arg("-m")
        .arg("--first-parent")
        .arg("--pretty=format:")
        .arg(&hash)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git show error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut files = Vec::new();
    
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        
        let mut parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            parts = line.split_whitespace().collect();
        }
        
        if parts.len() >= 2 {
            let status = parts[0].to_string();
            let mut file_path = if status.starts_with('R') || status.starts_with('C') {
                parts.last().unwrap().to_string()
            } else {
                parts[1].to_string()
            };
            
            if file_path.starts_with('"') && file_path.ends_with('"') {
                file_path = file_path[1..file_path.len()-1].to_string();
            }
            
            files.push(FileChange { status, path: file_path });
        }
    }
    
    Ok(CommitDetails { hash, files, staged_files: None, unstaged_files: None })
}

#[tauri::command]
pub fn compare_file_commits(path: String, hash1: String, hash2: String, file_path: String) -> Result<String, String> {
    let output = git_command()
        .arg("diff")
        .arg(&hash1)
        .arg(&hash2)
        .arg("--")
        .arg(&file_path)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
pub fn get_file_image_versions(
    path: String,
    old_hash: Option<String>,
    new_hash: String,
    file_path: String,
) -> Result<ImageVersions, String> {
    let mime_type = image_mime_type(&file_path)
        .ok_or_else(|| format!("Unsupported image type: {}", file_path))?;
    let old_revision = old_hash.unwrap_or_else(|| format!("{}^", new_hash));

    Ok(ImageVersions {
        old_image: git_image_data_url(&path, &old_revision, &file_path, mime_type)?,
        new_image: git_image_data_url(&path, &new_hash, &file_path, mime_type)?,
    })
}

#[tauri::command]
pub fn get_file_diff(path: String, hash: String, file_path: String) -> Result<String, String> {
    if hash == "uncommitted" {
        let diff_head = git_command()
            .arg("diff")
            .arg("HEAD")
            .arg("--")
            .arg(&file_path)
            .current_dir(&path)
            .output()
            .map_err(|e| e.to_string())?;

        let mut diff_str = String::from_utf8_lossy(&diff_head.stdout).to_string();
        
        if diff_str.is_empty() {
            let full_path = std::path::Path::new(&path).join(&file_path);
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                diff_str = format!("diff --git a/{} b/{}\nnew file mode 100644\n--- /dev/null\n+++ b/{}\n@@ -0,0 +1,{} @@\n", file_path, file_path, file_path, content.lines().count());
                for line in content.lines() {
                    diff_str.push('+');
                    diff_str.push_str(line);
                    diff_str.push('\n');
                }
            }
        }
        return Ok(diff_str);
    }

    let output = git_command()
        .arg("show")
        .arg("-m")
        .arg("--first-parent")
        .arg("--pretty=format:")
        .arg(&hash)
        .arg("--")
        .arg(&file_path)
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git show diff error: {}", err));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub fn get_file_blame(path: String, file_path: String, commit_hash: Option<String>) -> Result<Vec<BlameLine>, String> {
    let mut cmd = git_command();
    cmd.arg("blame").arg("--line-porcelain");

    if let Some(hash) = commit_hash.as_deref() {
        if !hash.is_empty() && hash != "HEAD" && hash != "uncommitted" {
            cmd.arg(hash);
        }
    }

    cmd.arg("--").arg(&file_path).current_dir(&path);

    let output = cmd.output().map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git blame error: {}", err.trim()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = Vec::new();

    let mut current_hash = String::new();
    let mut current_final_lineno = 0usize;
    let mut current_author = String::new();
    let mut current_author_mail = String::new();
    let mut current_author_time = 0i64;
    let mut current_summary = String::new();

    for line in stdout.lines() {
        if line.starts_with('\t') {
            let content = line[1..].to_string();
            let short_hash = if current_hash.len() >= 7 {
                current_hash[..7].to_string()
            } else {
                current_hash.clone()
            };

            lines.push(BlameLine {
                line_number: current_final_lineno,
                commit_hash: current_hash.clone(),
                commit_hash_short: short_hash,
                author: current_author.clone(),
                author_mail: current_author_mail.clone(),
                author_time: current_author_time,
                time_str: String::new(),
                summary: current_summary.clone(),
                content,
            });
        } else if let Some(author) = line.strip_prefix("author ") {
            current_author = author.trim().to_string();
        } else if let Some(mail) = line.strip_prefix("author-mail ") {
            current_author_mail = mail.trim().trim_matches('<').trim_matches('>').to_string();
        } else if let Some(time_str) = line.strip_prefix("author-time ") {
            current_author_time = time_str.trim().parse::<i64>().unwrap_or(0);
        } else if let Some(summary) = line.strip_prefix("summary ") {
            current_summary = summary.trim().to_string();
        } else {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 && parts[0].len() == 40 && parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
                current_hash = parts[0].to_string();
                current_final_lineno = parts[2].parse::<usize>().unwrap_or(0);
            }
        }
    }

    Ok(lines)
}

#[cfg(test)]
mod blame_tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn git(path: &std::path::Path, args: &[&str]) {
        let status = git_command().args(args).current_dir(path).status().unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    #[test]
    fn blames_lines_correctly() {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let base_dir = std::env::temp_dir().join(format!("begit-blame-{}", unique));
        fs::create_dir_all(&base_dir).unwrap();
        git(&base_dir, &["init"]);
        git(&base_dir, &["config", "user.name", "Alice Dev"]);
        git(&base_dir, &["config", "user.email", "alice@example.com"]);

        fs::write(base_dir.join("code.ts"), "const a = 1;\n").unwrap();
        git(&base_dir, &["add", "."]);
        git(&base_dir, &["commit", "-m", "First line"]);

        git(&base_dir, &["config", "user.name", "Bob Dev"]);
        git(&base_dir, &["config", "user.email", "bob@example.com"]);
        fs::write(base_dir.join("code.ts"), "const a = 1;\nconst b = 2;\n").unwrap();
        git(&base_dir, &["add", "."]);
        git(&base_dir, &["commit", "-m", "Second line"]);

        let blame = get_file_blame(base_dir.to_string_lossy().to_string(), "code.ts".to_string(), None).unwrap();
        assert_eq!(blame.len(), 2);
        assert_eq!(blame[0].line_number, 1);
        assert_eq!(blame[0].author, "Alice Dev");
        assert_eq!(blame[0].summary, "First line");
        assert_eq!(blame[0].content, "const a = 1;");

        assert_eq!(blame[1].line_number, 2);
        assert_eq!(blame[1].author, "Bob Dev");
        assert_eq!(blame[1].summary, "Second line");
        assert_eq!(blame[1].content, "const b = 2;");

        let _ = fs::remove_dir_all(base_dir);
    }
}
