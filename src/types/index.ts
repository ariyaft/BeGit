export interface WorktreeInfo {
  path: string;
  branch: string | null;
  commit_hash: string;
  is_main: boolean;
  is_locked: boolean;
  lock_reason?: string | null;
  is_prunable: boolean;
}

export interface SubmoduleInfo {
  name: string;
  path: string;
  full_path: string;
  url: string;
  commit_hash: string;
  status: string; // 'initialized' | 'uninitialized' | 'modified' | 'conflict'
  is_initialized: boolean;
  branch?: string | null;
}

export interface Project {
  path: string;
  name: string;
  localBranches: Array<{name: string; active: boolean; ahead: number; behind: number}>;
  remoteBranches: Array<{name: string; active: boolean}>;
  remotes: Array<{name: string; url: string}>;
  tags: Array<string>;
  stashes: Array<{id: string; message: string}>;
  worktrees?: Array<WorktreeInfo>;
  submodules?: Array<SubmoduleInfo>;
  authors: Array<any>;
  commits: Array<any>;
  rawCommits: Array<any>;
  selectedCommit: any | null;
  selectedCommits: Array<any>;
  uncommittedChanges: any | null;
  cherryPick: { active: boolean; has_conflicts: boolean; remaining_count: number };
  rebaseStatus?: RebaseStatus;
  operationState?: RepoOperationState;
  lastFetchTime?: number;
}

export type AppView = 'source-control' | 'horizontal-graph' | 'contributors-report' | 'file-explorer' | 'activity-log' | 'pull-requests' | 'reflog';

export interface RebaseTodoItem {
  action: 'pick' | 'reword' | 'squash' | 'fixup' | 'drop';
  hash: string;
  short_hash?: string;
  message: string;
  author?: string;
  is_edited?: boolean;
}

export interface RebaseStatus {
  active: boolean;
  current_step: number;
  total_steps: number;
  has_conflicts: boolean;
  head_name?: string;
  onto?: string;
}

export interface ConflictFileInfo {
  path: string;
  base_content: string;
  ours_content: string;
  theirs_content: string;
  working_content: string;
  is_resolved?: boolean;
}

export interface RepoOperationState {
  operation: 'none' | 'merge' | 'rebase' | 'cherry_pick' | 'revert';
  head_name?: string;
  target_name?: string;
  has_conflicts: boolean;
  conflicted_files: string[];
  current_step: number;
  total_steps: number;
}

export interface ReflogEntry {
  selector: string;
  hash: string;
  short_hash: string;
  action: string;
  message: string;
  date_relative: string;
  date_iso: string;
  author_name: string;
  author_email: string;
}

export interface DiffHunk {
  id: string;
  header: string;
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
  lines: DiffLine[];
}

export interface SignatureInfo {
  status: 'verified' | 'unverified' | 'bad' | 'expired' | 'revoked';
  key?: string;
  signer?: string;
  fingerprint?: string;
}

export interface RemoteInfo {
  name: string;
  url: string;
  provider: 'github' | 'gitlab' | 'bitbucket' | 'azure_devops' | 'other';
  owner: string;
  repo: string;
  web_url: string;
  api_base_url: string;
}

export interface PullRequest {
  id: number | string;
  number: number;
  title: string;
  state: 'open' | 'closed' | 'merged';
  body: string | null;
  html_url: string;
  created_at: string;
  updated_at: string;
  closed_at?: string | null;
  merged_at?: string | null;
  user: {
    login: string;
    avatar_url: string;
    html_url?: string;
  };
  head: {
    ref: string;
    sha: string;
    repo?: {
      full_name: string;
      clone_url: string;
    } | null;
  };
  base: {
    ref: string;
    sha: string;
  };
  draft?: boolean;
  labels?: Array<{
    name: string;
    color: string;
    description?: string;
  }>;
  comments?: number;
  review_comments?: number;
  commits?: number;
  additions?: number;
  deletions?: number;
  changed_files?: number;
}

export interface TimelineCommit {
  id: string;
  hash: string;
  author: string;
  time: string;
  exact_time?: string;
  message: string;
  parents: string[];
  branches: string[];
  refs: string[];
  files: any[];
  signature?: SignatureInfo | null;
}

export interface Lane {
  id: string;
  commitHash: string;
  isHead: boolean;
  isLocal: boolean;
  isRemote: boolean;
  isTag: boolean;
}

export interface LayoutCommit extends TimelineCommit {
  laneIndex: number;
  connections: { fromLane: number; toLane: number; isMerge: boolean }[];
  isLaneStart: boolean;
  isLaneEnd: boolean;
}

export interface Profile {
  name: string;
  email: string;
  scope: 'global' | 'repository';
}

export interface Contributor {
  name: string;
  email: string;
  commits: number;
  files_changed: number;
  additions: number;
  deletions: number;
  active_days: number;
  first_commit_at: string;
  last_commit_at: string;
}

export type ContributorRange = 'all' | '1d' | '1w' | '1m' | '3m' | '6m';

export interface ContributorReport {
  contributors: Contributor[];
  activity: Array<{ date: string; commits: number; additions: number; deletions: number }>;
  total_commits: number;
  total_files_changed: number;
  total_additions: number;
  total_deletions: number;
}

export type RemoteModalMode = 'add' | 'edit' | 'remove';

export interface ChangedFile {
  path: string;
  status: string;
  additions: number;
  deletions: number;
}

export interface DiffLine {
  type: 'context' | 'addition' | 'deletion' | 'hunk';
  content: string;
  oldLineno?: number;
  newLineno?: number;
}

export interface BlameLine {
  line_number: number;
  commit_hash: string;
  commit_hash_short: string;
  author: string;
  author_mail: string;
  author_time: number;
  time_str?: string;
  summary: string;
  content: string;
}
