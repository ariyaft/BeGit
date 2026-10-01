use std::collections::{BTreeMap, BTreeSet, HashMap};
use crate::utils::git_command;
use std::process::Command;
use crate::models::{ActivityPoint, Contributor, ContributorAccumulator, ContributorReport, GitProfile};
use crate::utils::git_config_value;

#[tauri::command]
pub async fn purge_file_from_history(path: String, file_path: String, add_to_gitignore: Option<bool>) -> Result<(), String> {
    let normalized_file = file_path.trim_start_matches('/').trim_start_matches('\\').to_string();
    if normalized_file.is_empty() {
        return Err("File path cannot be empty.".to_string());
    }

    // 1. Remove physical file from working directory if it exists
    let disk_path = std::path::Path::new(&path).join(&normalized_file);
    if disk_path.exists() {
        if disk_path.is_dir() {
            let _ = std::fs::remove_dir_all(&disk_path);
        } else {
            let _ = std::fs::remove_file(&disk_path);
        }
    }

    // 2. Append to .gitignore if requested
    if add_to_gitignore.unwrap_or(false) {
        let gitignore_path = std::path::Path::new(&path).join(".gitignore");
        let mut content = if gitignore_path.exists() {
            std::fs::read_to_string(&gitignore_path).unwrap_or_default()
        } else {
            String::new()
        };

        let pattern = format!("/{}", normalized_file);
        let pattern_alt = normalized_file.clone();
        let already_present = content.lines().any(|line| {
            let l = line.trim();
            l == pattern || l == pattern_alt
        });

        if !already_present {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&format!("{}\n", pattern));
            let _ = std::fs::write(&gitignore_path, content);
        }
    }

    // 3. Run git filter-branch to remove file across all commits in all branches and tags
    let filter_cmd = format!("git rm --cached --ignore-unmatch -r -f '{}'", normalized_file.replace('\'', "'\\''"));
    let output = git_command()
        .current_dir(&path)
        .env("FILTER_BRANCH_SQUELCH_WARNING", "1")
        .arg("filter-branch")
        .arg("-f")
        .arg("--index-filter")
        .arg(&filter_cmd)
        .arg("--prune-empty")
        .arg("--tag-name-filter")
        .arg("cat")
        .arg("--")
        .arg("--branches")
        .arg("--tags")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to purge file from history: {}", err));
    }

    // 4. Clean up backup references
    let _ = Command::new("sh")
        .current_dir(&path)
        .arg("-c")
        .arg("git for-each-ref --format='%(refname)' refs/original/ 2>/dev/null | xargs -n 1 git update-ref -d 2>/dev/null || true")
        .output();

    Ok(())
}

#[tauri::command]
pub fn get_git_profile(path: Option<String>) -> Result<GitProfile, String> {
    let local_name = if let Some(repository) = path.as_deref() {
        git_config_value(Some(repository), "--local", "user.name")?
    } else {
        None
    };
    let local_email = if let Some(repository) = path.as_deref() {
        git_config_value(Some(repository), "--local", "user.email")?
    } else {
        None
    };
    let global_name = git_config_value(None, "--global", "user.name")?;
    let global_email = git_config_value(None, "--global", "user.email")?;
    let has_local_profile = local_name.is_some() || local_email.is_some();

    Ok(GitProfile {
        name: local_name.or(global_name).unwrap_or_default(),
        email: local_email.or(global_email).unwrap_or_default(),
        scope: if has_local_profile { "repository" } else { "global" }.to_string(),
    })
}

#[tauri::command]
pub fn set_git_profile(path: Option<String>, name: String, email: String, scope: String) -> Result<(), String> {
    let name = name.trim();
    let email = email.trim();
    if name.is_empty() || email.is_empty() || !email.contains('@') {
        return Err("Enter a Git name and a valid email address.".to_string());
    }
    let config_scope = match scope.as_str() {
        "repository" => "--local",
        "global" => "--global",
        _ => return Err("Unsupported Git profile scope.".to_string()),
    };
    if config_scope == "--local" && path.is_none() {
        return Err("Open a repository before setting a repository-specific profile.".to_string());
    }

    for (key, value) in [("user.name", name), ("user.email", email)] {
        let mut command = git_command();
        command.arg("config").arg(config_scope).arg(key).arg(value);
        if let Some(repository) = path.as_deref() {
            command.current_dir(repository);
        }
        let output = command.output().map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub fn get_contributor_report(path: String, range: Option<String>) -> Result<ContributorReport, String> {
    let since = match range.as_deref().unwrap_or("all") {
        "all" => None,
        "1d" => Some("1 day ago"),
        "1w" => Some("1 week ago"),
        "1m" => Some("1 month ago"),
        "3m" => Some("3 months ago"),
        "6m" => Some("6 months ago"),
        value => return Err(format!("Unsupported contributor report range: {}", value)),
    };

    let mut command = git_command();
    command
        .arg("log")
        .arg("--all");
    if let Some(since) = since {
        command.arg(format!("--since={}", since));
    }
    let output = command
        .arg("HEAD")
        .arg("--format=%x1e%ae%x1f%an%x1f%as%x1f%at%x1f%aI")
        .arg("--numstat")
        .current_dir(&path)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Git contributor report error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut contributors: HashMap<String, ContributorAccumulator> = HashMap::new();
    let mut total_commits = 0u64;
    let mut total_files_changed = 0u64;
    let mut total_additions = 0u64;
    let mut total_deletions = 0u64;
    let mut activity: BTreeMap<String, ActivityPoint> = BTreeMap::new();

    for record in stdout.split('\x1e').skip(1) {
        let Some((header, numstat)) = record.split_once('\n') else { continue; };
        let mut fields = header.split('\x1f');
        let email = fields.next().unwrap_or("").trim().to_string();
        let name = fields.next().unwrap_or("").trim().to_string();
        let active_day = fields.next().unwrap_or("").trim().to_string();
        let timestamp = fields.next().unwrap_or("0").trim().parse::<i64>().unwrap_or(0);
        let timestamp_iso = fields.next().unwrap_or("").trim().to_string();

        if email.is_empty() { continue; }

        let mut files_changed = 0u64;
        let mut additions = 0u64;
        let mut deletions = 0u64;
        for line in numstat.lines() {
            let mut parts = line.splitn(3, '\t');
            let Some(added) = parts.next() else { continue; };
            let Some(deleted) = parts.next() else { continue; };
            if parts.next().is_none() { continue; }

            files_changed += 1;
            additions += added.parse::<u64>().unwrap_or(0);
            deletions += deleted.parse::<u64>().unwrap_or(0);
        }

        total_commits += 1;
        total_files_changed += files_changed;
        total_additions += additions;
        total_deletions += deletions;

        if !active_day.is_empty() {
            let day = activity.entry(active_day.clone()).or_insert_with(|| ActivityPoint {
                date: active_day.clone(),
                ..Default::default()
            });
            day.commits += 1;
            day.additions += additions;
            day.deletions += deletions;
        }

        let entry = contributors.entry(email.clone()).or_insert_with(|| ContributorAccumulator {
            contributor: Contributor {
                name: name.clone(),
                email: email.clone(),
                commits: 0,
                files_changed: 0,
                additions: 0,
                deletions: 0,
                active_days: 0,
                first_commit_at: timestamp_iso.clone(),
                last_commit_at: timestamp_iso.clone(),
            },
            active_days: BTreeSet::new(),
            first_timestamp: timestamp,
            last_timestamp: timestamp,
        });

        entry.contributor.commits += 1;
        entry.contributor.files_changed += files_changed;
        entry.contributor.additions += additions;
        entry.contributor.deletions += deletions;
        if !active_day.is_empty() {
            entry.active_days.insert(active_day);
        }
        if timestamp < entry.first_timestamp {
            entry.first_timestamp = timestamp;
            entry.contributor.first_commit_at = timestamp_iso.clone();
        }
        if timestamp >= entry.last_timestamp {
            entry.last_timestamp = timestamp;
            entry.contributor.last_commit_at = timestamp_iso;
            entry.contributor.name = name;
        }
    }

    let mut rows: Vec<Contributor> = contributors.into_values().map(|mut entry| {
        entry.contributor.active_days = entry.active_days.len() as u64;
        entry.contributor
    }).collect();
    rows.sort_by(|a, b| b.commits.cmp(&a.commits).then_with(|| a.name.cmp(&b.name)));

    Ok(ContributorReport {
        contributors: rows,
        activity: activity.into_values().collect(),
        total_commits,
        total_files_changed,
        total_additions,
        total_deletions,
    })
}

#[cfg(test)]
mod contributor_report_tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn git(path: &std::path::Path, args: &[&str]) {
        let status = git_command().args(args).current_dir(path).status().unwrap();
        assert!(status.success(), "git {:?} failed", args);
    }

    fn commit(path: &std::path::Path, name: &str, email: &str, date: &str, message: &str) {
        let status = git_command()
            .args(["commit", "-m", message])
            .current_dir(path)
            .env("GIT_AUTHOR_NAME", name)
            .env("GIT_AUTHOR_EMAIL", email)
            .env("GIT_COMMITTER_NAME", name)
            .env("GIT_COMMITTER_EMAIL", email)
            .env("GIT_AUTHOR_DATE", date)
            .env("GIT_COMMITTER_DATE", date)
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[test]
    fn groups_by_email_and_counts_binary_file_activity() {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("begit-contributor-report-{}", unique));
        fs::create_dir_all(&path).unwrap();
        git(&path, &["init"]);

        fs::write(path.join("notes.txt"), "first line\n").unwrap();
        git(&path, &["add", "."]);
        commit(&path, "Alice", "alice@example.test", "2024-01-01T12:00:00+00:00", "first");

        fs::write(path.join("notes.txt"), "first line\nsecond line\n").unwrap();
        git(&path, &["add", "."]);
        commit(&path, "Alicia", "alice@example.test", "2024-01-02T12:00:00+00:00", "second");

        fs::write(path.join("asset.bin"), [0u8, 1, 2, 3]).unwrap();
        git(&path, &["add", "."]);
        commit(&path, "Alice", "other-alice@example.test", "2024-01-03T12:00:00+00:00", "binary");

        let report = get_contributor_report(path.to_string_lossy().to_string(), Some("all".to_string())).unwrap();
        let primary = report.contributors.iter().find(|row| row.email == "alice@example.test").unwrap();

        assert_eq!(report.total_commits, 3);
        assert_eq!(report.total_files_changed, 3);
        assert_eq!(report.total_additions, 2);
        assert_eq!(report.total_deletions, 0);
        assert_eq!(report.activity.len(), 3);
        assert_eq!(report.activity[0].date, "2024-01-01");
        assert_eq!(report.activity[0].commits, 1);
        assert_eq!(report.activity[0].additions, 1);
        assert_eq!(report.contributors.len(), 2);
        assert_eq!(primary.name, "Alicia");
        assert_eq!(primary.commits, 2);
        assert_eq!(primary.active_days, 2);
        assert_eq!(primary.first_commit_at, "2024-01-01T12:00:00Z");
        assert_eq!(primary.last_commit_at, "2024-01-02T12:00:00Z");

        let _ = fs::remove_dir_all(path);
    }
}
