use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RebaseTodoItem {
    pub action: String,
    pub hash: String,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RebaseStatus {
    pub active: bool,
    pub current_step: usize,
    pub total_steps: usize,
    pub has_conflicts: bool,
    pub head_name: Option<String>,
    pub onto: Option<String>,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct ConflictFileInfo {
    pub path: String,
    pub base_content: String,
    pub ours_content: String,
    pub theirs_content: String,
    pub working_content: String,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct RepoOperationState {
    pub operation: String,
    pub head_name: Option<String>,
    pub target_name: Option<String>,
    pub has_conflicts: bool,
    pub conflicted_files: Vec<String>,
    pub current_step: usize,
    pub total_steps: usize,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct ReflogEntry {
    pub selector: String,
    pub hash: String,
    pub short_hash: String,
    pub action: String,
    pub message: String,
    pub date_relative: String,
    pub date_iso: String,
    pub author_name: String,
    pub author_email: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Branch {
    pub name: String,
    pub active: bool,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct Author {
    pub name: String,
    pub commits: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct GitProfile {
    pub name: String,
    pub email: String,
    pub scope: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Contributor {
    pub name: String,
    pub email: String,
    pub commits: u64,
    pub files_changed: u64,
    pub additions: u64,
    pub deletions: u64,
    pub active_days: u64,
    pub first_commit_at: String,
    pub last_commit_at: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct ContributorReport {
    pub contributors: Vec<Contributor>,
    pub activity: Vec<ActivityPoint>,
    pub total_commits: u64,
    pub total_files_changed: u64,
    pub total_additions: u64,
    pub total_deletions: u64,
}

#[derive(Serialize, Clone, Default, Debug)]
pub struct ActivityPoint {
    pub date: String,
    pub commits: u64,
    pub additions: u64,
    pub deletions: u64,
}

pub struct ContributorAccumulator {
    pub contributor: Contributor,
    pub active_days: BTreeSet<String>,
    pub first_timestamp: i64,
    pub last_timestamp: i64,
}

#[derive(Serialize, Clone, Debug)]
pub struct Stash {
    pub id: String,
    pub message: String,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct SignatureInfo {
    pub status: String,
    pub key: String,
    pub signer: String,
    pub fingerprint: String,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
    pub provider: String,
    pub owner: String,
    pub repo: String,
    pub web_url: String,
    pub api_base_url: String,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct BlameLine {
    pub line_number: usize,
    pub commit_hash: String,
    pub commit_hash_short: String,
    pub author: String,
    pub author_mail: String,
    pub author_time: i64,
    pub time_str: String,
    pub summary: String,
    pub content: String,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct WorktreeInfo {
    pub path: String,
    pub branch: Option<String>,
    pub commit_hash: String,
    pub is_main: bool,
    pub is_locked: bool,
    pub lock_reason: Option<String>,
    pub is_prunable: bool,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct SubmoduleInfo {
    pub name: String,
    pub path: String,
    pub full_path: String,
    pub url: String,
    pub commit_hash: String,
    pub status: String,
    pub is_initialized: bool,
    pub branch: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct CherryPickStatus {
    pub active: bool,
    pub has_conflicts: bool,
    pub remaining_count: usize,
}

#[derive(Serialize, Clone, Debug)]
pub struct Commit {
    pub hash: String,
    pub hash_short: String,
    pub parents: Vec<String>,
    pub author: String,
    pub time: String,
    pub exact_time: String,
    pub message: String,
    pub refs: Vec<String>,
    pub signature: Option<SignatureInfo>,
}

#[derive(Serialize, Clone, Debug)]
pub struct FileChange {
    pub status: String,
    pub path: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CommitDetails {
    pub hash: String,
    pub files: Vec<FileChange>,
    pub staged_files: Option<Vec<FileChange>>,
    pub unstaged_files: Option<Vec<FileChange>>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ImageVersions {
    pub old_image: Option<String>,
    pub new_image: Option<String>,
}
