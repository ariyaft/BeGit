<script setup lang="ts">
import BranchIcon from '../assets/icons/branch.svg?component';
import FetchIcon from '../assets/icons/fetch.svg?component';
import PullIcon from '../assets/icons/pull.svg?component';
import PushIcon from '../assets/icons/push.svg?component';
import HexagonIcon from '../assets/icons/Hexagon.svg?component';
import PopIcon from '../assets/icons/pop.svg?component';
import ActionIcon from '../assets/icons/action.svg?component';
import LeftSidebar from '../components/source-control/LeftSidebar.vue';
import RightSidebar from '../components/source-control/RightSidebar.vue';
import CommitGraph from '../components/source-control/CommitGraph.vue';
import DiffViewer from '../components/Common/DiffViewer.vue';
import ConfirmDropStashModal from '../components/source-control/ConfirmDropStashModal.vue';
import ConfirmDeleteBranchModal from '../components/source-control/ConfirmDeleteBranchModal.vue';
import CreateBranchModal from '../components/source-control/CreateBranchModal.vue';
import RenameBranchModal from '../components/source-control/RenameBranchModal.vue';
import RemoteModal from '../components/source-control/RemoteModal.vue';
import CreateTagModal from '../components/source-control/CreateTagModal.vue';
import ConfirmDeleteTagModal from '../components/source-control/ConfirmDeleteTagModal.vue';
import CreateWorktreeModal from '../components/source-control/CreateWorktreeModal.vue';
import ConfirmRemoveWorktreeModal from '../components/source-control/ConfirmRemoveWorktreeModal.vue';
import LockWorktreeModal from '../components/source-control/LockWorktreeModal.vue';
import AddSubmoduleModal from '../components/source-control/AddSubmoduleModal.vue';
import InteractiveRebaseModal from '../components/source-control/InteractiveRebaseModal.vue';
import MergeConflictResolverModal from '../components/source-control/MergeConflictResolverModal.vue';
import ConfirmDropCommitModal from '../components/Common/ConfirmDropCommitModal.vue';
import ContextMenu from '../components/menus/ContextMenu.vue';
import BranchDropMenu, { type BranchDropPayload } from '../components/menus/BranchDropMenu.vue';
import CommitContextMenu from '../components/menus/CommitContextMenu.vue';
import RemoteContextMenu from '../components/menus/RemoteContextMenu.vue';
import StashContextMenu from '../components/menus/StashContextMenu.vue';
import TagContextMenu from '../components/menus/TagContextMenu.vue';
import WorktreeContextMenu from '../components/menus/WorktreeContextMenu.vue';
import SubmoduleContextMenu from '../components/menus/SubmoduleContextMenu.vue';
import SearchIcon from '../assets/icons/search.svg?component';
import UserIcon from '../assets/icons/user.svg?component';
import TimelineIcon from '../assets/icons/timeline.svg?component';
import CloseIcon from '../assets/icons/close.svg?component';
import ChevronDownIcon from '../assets/icons/chevron-down.svg?component';
import ShieldAlertIcon from '../assets/icons/shield-alert.svg?component';
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue';
import { notify } from '../composables/useToasts';
import { invokeGit } from '../composables/useActivityLog';
import { computeGraph } from '../utils/gitHelpers';

const props = defineProps<{
  project: any;
  leftSidebarWidth: number;
  rightSidebarWidth: number;
  displayBranch: string;
  isLoading: string;
  detailsMode: 'commit' | 'changes' | null;
  hasWorkingChanges: boolean;
  workingChangeCount: number;
  commits: any[];
  selectedCommit: any;
  selectedCommits: any[];
  detailsTarget: any;
  selectedFilePath: string | null;
  selectedFileDiff: string | null;
}>();

const emit = defineEmits<{
  itemClick: [item: { type: string; id: string }];
  branchDblclick: [branch: any];
  loadAuthors: [];
  openContributorsReport: [];
  openPullRequests: [];
  openProject: [path: string];
  resizeLeft: [event: MouseEvent];
  resizeRight: [event: MouseEvent];
  fetch: [];
  pull: [];
  push: [];
  stash: [];
  pop: [];
  showWorkingChanges: [];
  selectCommit: [commit: any, event?: MouseEvent];
  closeDiff: [];
  closeDetails: [];
  changeDetailsMode: [mode: 'commit' | 'changes'];
  selectFile: [path: string];
  stageFile: [path: string];
  unstageFile: [path: string];
  stageAll: [];
  commitChanges: [message: string, amend?: boolean];
  refresh: [fetchRemote: boolean, done?: () => void];
  pending: [message: string];
  branchAction: [action: string, branch: any];
  commitAction: [action: string, commits: any[]];
  cherryPickAction: [action: 'continue' | 'skip' | 'abort'];
}>();

const dropStashModalRef = ref<{ open(stash: { id: string; message: string }): void } | null>(null);
const deleteBranchModalRef = ref<{ open(branch: { name: string }, isRemote: boolean): void } | null>(null);
const createBranchModalRef = ref<{ open(branch: { name: string }): void } | null>(null);
const renameBranchModalRef = ref<{ open(branch: { name: string }): void } | null>(null);
const remoteModalRef = ref<{
  openAdd(): void;
  openEdit(remote: { name: string; url: string }): void;
  openRemove(remote: { name: string; url: string }): void;
} | null>(null);
const createTagModalRef = ref<{ open(opts?: { targetCommit?: string; name?: string }): void } | null>(null);
const deleteTagModalRef = ref<{ open(tag: string): void } | null>(null);
const createWorktreeModalRef = ref<{ open(defaultPath?: string): void } | null>(null);
const deleteWorktreeModalRef = ref<{ open(wt: any): void } | null>(null);
const lockWorktreeModalRef = ref<{ open(wt: any): void } | null>(null);
const addSubmoduleModalRef = ref<{ open(): void } | null>(null);
const rebaseModalRef = ref<{ open(baseRef?: string, initialCommits?: any[]): void } | null>(null);
const conflictModalRef = ref<{ open(filePath?: string): void } | null>(null);
const dropCommitModalRef = ref<{ open(commits: any[] | any): void } | null>(null);

const branchMenu = ref({ visible: false, x: 0, y: 0, branch: null as any });
const branchDropMenu = ref<{
  visible: boolean;
  x: number;
  y: number;
  sourceBranch: any | null;
  targetBranch: any | null;
}>({
  visible: false,
  x: 0,
  y: 0,
  sourceBranch: null,
  targetBranch: null
});
const commitMenu = ref({ visible: false, x: 0, y: 0, commits: [] as any[] });
const remoteMenu = ref({ visible: false, x: 0, y: 0, remote: null as any });
const stashMenu = ref({ visible: false, x: 0, y: 0, stash: null as any });
const tagMenu = ref({ visible: false, x: 0, y: 0, tag: null as string | null });
const worktreeMenu = ref({ visible: false, x: 0, y: 0, worktree: null as any });
const submoduleMenu = ref({ visible: false, x: 0, y: 0, submodule: null as any });

// --- Commit Graph Filter State ---
const filterKeyword = ref('');
const filterAuthor = ref('');
const filterDateRange = ref<'all' | 'today' | '7d' | '30d' | '90d' | '180d' | '365d'>('all');
const filterBranch = ref('all');
const searchInputRef = ref<HTMLInputElement | null>(null);

const isFilterActive = computed(() => {
  return Boolean(
    filterKeyword.value.trim() ||
    filterAuthor.value ||
    filterDateRange.value !== 'all' ||
    filterBranch.value !== 'all'
  );
});

function clearAllFilters() {
  filterKeyword.value = '';
  filterAuthor.value = '';
  filterDateRange.value = 'all';
  filterBranch.value = 'all';
}

const availableAuthors = computed(() => {
  if (props.project?.authors && props.project.authors.length > 0) {
    return props.project.authors.map((a: any) => a.name);
  }
  const set = new Set<string>();
  (props.project?.rawCommits || props.commits || []).forEach((c: any) => {
    if (c.author) set.add(c.author);
  });
  return Array.from(set).sort();
});

const displayCommits = computed(() => {
  if (!isFilterActive.value) {
    return props.commits;
  }

  const raw = props.project?.rawCommits || props.commits || [];
  if (raw.length === 0) return props.commits;

  const filtered = raw.filter((commit: any) => {
    // 1. Keyword search (message, hash, author)
    if (filterKeyword.value.trim()) {
      const q = filterKeyword.value.trim().toLowerCase();
      const matchMsg = commit.message?.toLowerCase().includes(q);
      const matchHash = commit.hash?.toLowerCase().includes(q) || commit.hash_short?.toLowerCase().includes(q);
      const matchAuthor = commit.author?.toLowerCase().includes(q);
      if (!matchMsg && !matchHash && !matchAuthor) return false;
    }

    // 2. Author filter
    if (filterAuthor.value) {
      if (commit.author !== filterAuthor.value) return false;
    }

    // 3. Date range
    if (filterDateRange.value !== 'all') {
      const dateMs = new Date(commit.exact_time || commit.time).getTime();
      if (!isNaN(dateMs)) {
        const now = Date.now();
        if (filterDateRange.value === 'today') {
          const startOfToday = new Date().setHours(0, 0, 0, 0);
          if (dateMs < startOfToday) return false;
        } else if (filterDateRange.value === '7d' && dateMs < now - 7 * 86400 * 1000) {
          return false;
        } else if (filterDateRange.value === '30d' && dateMs < now - 30 * 86400 * 1000) {
          return false;
        } else if (filterDateRange.value === '90d' && dateMs < now - 90 * 86400 * 1000) {
          return false;
        } else if (filterDateRange.value === '180d' && dateMs < now - 180 * 86400 * 1000) {
          return false;
        } else if (filterDateRange.value === '365d' && dateMs < now - 365 * 86400 * 1000) {
          return false;
        }
      }
    }

    // 4. Branch filter
    if (filterBranch.value !== 'all') {
      const targetBranch = filterBranch.value === 'current' ? props.displayBranch : filterBranch.value;
      if (targetBranch) {
        const matchesRef = commit.refs?.some((r: string) => r.includes(targetBranch));
        const matchesBranch = commit.branches?.includes(targetBranch);
        if (!matchesRef && !matchesBranch) return false;
      }
    }

    return true;
  });

  return computeGraph(filtered);
});

const canCherryPick = computed(() => {
  const changes = props.project?.uncommittedChanges;
  const hasChanges = (changes?.staged_files?.length || 0) + (changes?.unstaged_files?.length || 0) > 0;
  return !props.project?.cherryPick?.active && !hasChanges && !commitMenu.value.commits.some(commit => commit.hash === 'WIP');
});

function forwardBranchContext(event: MouseEvent, branch: any) {
  const isRemote = props.project?.remoteBranches?.some((candidate: any) => candidate.name === branch.name) || false;
  branchMenu.value = { visible: true, x: event.clientX, y: event.clientY, branch: { ...branch, isRemote } };
}

function forwardRemoteContext(event: MouseEvent, remote: any) {
  remoteMenu.value = { visible: true, x: event.clientX, y: event.clientY, remote };
}

function forwardStashContext(event: MouseEvent, stash: any) {
  stashMenu.value = { visible: true, x: event.clientX, y: event.clientY, stash };
}

async function openCommitContextMenu(commit: any, event: MouseEvent) {
  if (!props.selectedCommits.some(selected => selected.id === commit.id)) emit('selectCommit', commit, event);
  await nextTick();
  const commits = props.selectedCommits.some(selected => selected.id === commit.id) ? props.selectedCommits : [commit];
  commitMenu.value = { visible: true, x: event.clientX, y: event.clientY, commits };
}

function forwardSelectCommit(commit: any, event?: MouseEvent) {
  emit('selectCommit', commit, event);
}

function openAddRemote() { remoteModalRef.value?.openAdd(); }
function openEditRemote(remote: { name: string; url: string }) { remoteModalRef.value?.openEdit(remote); }
function openRemoveRemote(remote: { name: string; url: string }) { remoteModalRef.value?.openRemove(remote); }
function openRenameBranch(branch: { name: string }) { renameBranchModalRef.value?.open(branch); }
function openCreateBranch(branch?: { name: string }) { createBranchModalRef.value?.open(branch || { name: '' }); }
function openDeleteBranch(branch: { name: string }, isRemote: boolean) { deleteBranchModalRef.value?.open(branch, isRemote); }
function openDropStash(stash: { id: string; message: string }) { dropStashModalRef.value?.open(stash); }

function openCreateTag(opts?: { targetCommit?: string; name?: string }) {
  createTagModalRef.value?.open(opts);
}
function openDeleteTag(tag: string) {
  deleteTagModalRef.value?.open(tag);
}
function openCreateWorktree(defaultPath?: string) {
  createWorktreeModalRef.value?.open(defaultPath);
}
function openRemoveWorktree(wt: any) {
  deleteWorktreeModalRef.value?.open(wt);
}
function openLockWorktree(wt: any) {
  lockWorktreeModalRef.value?.open(wt);
}
function openAddSubmodule() {
  addSubmoduleModalRef.value?.open();
}
function openInteractiveRebase(baseRef?: string, initialCommits?: any[]) {
  rebaseModalRef.value?.open(baseRef, initialCommits);
}
function openConflictResolver(filePath?: string) {
  conflictModalRef.value?.open(filePath);
}
function openDropCommit(commits: any[] | any) {
  dropCommitModalRef.value?.open(commits);
}

const isCurrentFileStaged = computed(() => {
  if (!props.selectedFilePath || !props.project?.uncommittedChanges) return false;
  return (props.project.uncommittedChanges.staged_files || []).some(
    (f: any) => f.path === props.selectedFilePath
  );
});

async function handleContinueRebase() {
  if (!props.project?.path) return;
  emit('pending', 'Continuing rebase...');
  try {
    await invokeGit('continue_rebase', { path: props.project.path }, 'git rebase --continue');
    emit('refresh', false, () => emit('pending', ''));
    notify('Rebase step continued successfully', 'success');
  } catch (e: any) {
    notify(`Failed to continue rebase: ${e}`, 'error');
    emit('pending', '');
    emit('refresh', false);
  }
}

async function handleSkipRebase() {
  if (!props.project?.path) return;
  emit('pending', 'Skipping rebase commit...');
  try {
    await invokeGit('skip_rebase', { path: props.project.path }, 'git rebase --skip');
    emit('refresh', false, () => emit('pending', ''));
    notify('Rebase commit skipped', 'info');
  } catch (e: any) {
    notify(`Failed to skip rebase commit: ${e}`, 'error');
    emit('pending', '');
    emit('refresh', false);
  }
}

async function handleAbortRebase() {
  if (!props.project?.path) return;
  if (!window.confirm('Abort the interactive rebase and return repository to previous state?')) return;
  emit('pending', 'Aborting rebase...');
  try {
    await invokeGit('abort_rebase', { path: props.project.path }, 'git rebase --abort');
    emit('refresh', false, () => emit('pending', ''));
    notify('Rebase aborted', 'info');
  } catch (e: any) {
    notify(`Failed to abort rebase: ${e}`, 'error');
    emit('pending', '');
    emit('refresh', false);
  }
}

const operationTitle = computed(() => {
  const op = props.project?.operationState?.operation;
  if (!op || op === 'none') return '';
  const name = op.charAt(0).toUpperCase() + op.slice(1).replace('_', '-');
  const target = props.project?.operationState?.target_name ? ` (${props.project.operationState.target_name})` : '';
  return `${name} in progress${target}`;
});

async function handleContinueOperation() {
  if (!props.project?.path || !props.project?.operationState?.operation) return;
  const op = props.project.operationState.operation;
  emit('pending', `Continuing ${op}...`);
  try {
    await invokeGit('continue_repository_operation', { path: props.project.path, opType: op }, `git ${op} --continue`);
    emit('refresh', false, () => emit('pending', ''));
    notify(`${op} continued successfully`, 'success');
  } catch (e: any) {
    notify(`Failed to continue ${op}: ${e}`, 'error');
    emit('pending', '');
    emit('refresh', false);
  }
}

async function handleAbortOperation() {
  if (!props.project?.path || !props.project?.operationState?.operation) return;
  const op = props.project.operationState.operation;
  if (!window.confirm(`Are you sure you want to abort the current ${op} operation?`)) return;
  emit('pending', `Aborting ${op}...`);
  try {
    await invokeGit('abort_repository_operation', { path: props.project.path, opType: op }, `git ${op} --abort`);
    emit('refresh', false, () => emit('pending', ''));
    notify(`${op} aborted`, 'info');
  } catch (e: any) {
    notify(`Failed to abort ${op}: ${e}`, 'error');
    emit('pending', '');
    emit('refresh', false);
  }
}

function forwardTagContext(event: MouseEvent, tag: string) {
  closeMenus();
  tagMenu.value = { visible: true, x: event.clientX, y: event.clientY, tag };
}

function forwardWorktreeContext(event: MouseEvent, wt: any) {
  closeMenus();
  worktreeMenu.value = { visible: true, x: event.clientX, y: event.clientY, worktree: wt };
}

function forwardSubmoduleContext(event: MouseEvent, sm: any) {
  closeMenus();
  submoduleMenu.value = { visible: true, x: event.clientX, y: event.clientY, submodule: sm };
}

let dropMenuOpening = false;

function closeMenus() {
  if (dropMenuOpening) return;
  branchMenu.value.visible = false;
  branchDropMenu.value.visible = false;
  commitMenu.value.visible = false;
  remoteMenu.value.visible = false;
  stashMenu.value.visible = false;
  tagMenu.value.visible = false;
  worktreeMenu.value.visible = false;
  submoduleMenu.value.visible = false;
}

function handleBranchDrop(payload: BranchDropPayload) {
  dropMenuOpening = true;
  branchMenu.value.visible = false;
  commitMenu.value.visible = false;
  remoteMenu.value.visible = false;
  stashMenu.value.visible = false;
  tagMenu.value.visible = false;
  worktreeMenu.value.visible = false;
  submoduleMenu.value.visible = false;

  branchDropMenu.value = {
    visible: true,
    x: payload.x,
    y: payload.y,
    sourceBranch: payload.source,
    targetBranch: payload.target
  };

  setTimeout(() => {
    dropMenuOpening = false;
  }, 300);
}

async function handleBranchDropMerge(sourceRef: string, targetRef: string) {
  if (!props.project?.path) return;
  closeMenus();

  const currentActive = props.displayBranch;
  const needCheckout = targetRef !== currentActive && !targetRef.startsWith('origin/');

  emit('pending', `Merging ${sourceRef} into ${targetRef}...`);
  try {
    if (needCheckout) {
      await invokeGit('checkout_branch', { path: props.project.path, branchName: targetRef }, `git checkout ${targetRef}`);
    }
    await invokeGit('merge_branch', { path: props.project.path, branchName: sourceRef }, `git merge ${sourceRef}`);
    emit('refresh', false, () => emit('pending', ''));
    notify(`Successfully merged ${sourceRef} into ${targetRef}`, 'success');
  } catch (e: any) {
    emit('pending', '');
    emit('refresh', false);
    notify(`Merge failed: ${e}`, 'error');
  }
}

async function handleBranchDropRebase(sourceRef: string, targetRef: string) {
  if (!props.project?.path) return;
  closeMenus();

  const currentActive = props.displayBranch;
  const needCheckout = sourceRef !== currentActive && !sourceRef.startsWith('origin/');

  emit('pending', `Rebasing ${sourceRef} onto ${targetRef}...`);
  try {
    if (needCheckout) {
      await invokeGit('checkout_branch', { path: props.project.path, branchName: sourceRef }, `git checkout ${sourceRef}`);
    }
    await invokeGit('rebase_branch', { path: props.project.path, branchName: targetRef }, `git rebase ${targetRef}`);
    emit('refresh', false, () => emit('pending', ''));
    notify(`Successfully rebased ${sourceRef} onto ${targetRef}`, 'success');
  } catch (e: any) {
    emit('pending', '');
    emit('refresh', false);
    notify(`Rebase failed: ${e}`, 'error');
  }
}

function handleBranchAction(action: string) {
  const branch = branchMenu.value.branch;
  branchMenu.value.visible = false;
  if (!branch) return;
  if (action === 'rename') openRenameBranch(branch);
  else if (action === 'create_branch') openCreateBranch(branch);
  else if (action === 'delete_local' || action === 'delete_remote') openDeleteBranch(branch, action === 'delete_remote');
  else emit('branchAction', action, branch);
}

const amendTrigger = ref(0);

const headCommit = computed(() => {
  const allCommits = props.commits || [];
  return allCommits.find((c: any) => c.isHead) || allCommits[0] || null;
});

function handleAmendCommitAction() {
  emit('showWorkingChanges');
  amendTrigger.value++;
  notify('Editing HEAD commit to amend. Adjust your staged files or commit message, then click Amend Commit.', 'info');
}

function handleCommitAction(action: string) {
  const commits = commitMenu.value.commits;
  commitMenu.value.visible = false;
  if (!commits.length) return;
  if (action === 'create_tag') {
    openCreateTag({ targetCommit: commits[0].hash });
  } else if (action === 'interactive_rebase') {
    openInteractiveRebase(commits[0].hash);
  } else if (action === 'drop_commit') {
    openDropCommit(commits);
  } else if (action === 'amend_commit') {
    handleAmendCommitAction();
  } else {
    emit('commitAction', action, commits);
  }
}

function handleRemoteAction(action: string) {
  const remote = remoteMenu.value.remote;
  remoteMenu.value.visible = false;
  if (!remote) return;
  if (action === 'remove') openRemoveRemote(remote);
  else if (action === 'edit_url') openEditRemote(remote);
}

function handleStashAction(action: string) {
  const stash = stashMenu.value.stash;
  stashMenu.value.visible = false;
  if (!stash) return;
  if (action === 'drop') openDropStash(stash);
  else if (action === 'apply') notify('Apply stash is not implemented yet.', 'info');
}

async function handleTagAction(action: string) {
  const tag = tagMenu.value.tag;
  tagMenu.value.visible = false;
  if (!tag || !props.project?.path) return;

  if (action === 'delete') {
    openDeleteTag(tag);
  } else if (action === 'checkout') {
    emit('pending', `Checking out tag ${tag}...`);
    try {
      await invokeGit('checkout_branch', { path: props.project.path, branchName: tag }, `git checkout ${tag}`);
      emit('refresh', false, () => emit('pending', ''));
      notify(`Checked out tag ${tag}`, 'success');
    } catch (e: any) {
      notify(`Failed to checkout tag: ${e}`, 'error');
      emit('pending', '');
    }
  } else if (action === 'push') {
    const remotes = props.project?.remotes || [];
    const remote = remotes.length > 0 ? remotes[0].name : 'origin';
    emit('pending', `Pushing tag ${tag} to ${remote}...`);
    try {
      await invokeGit('push_tag', { path: props.project.path, name: tag, remote }, `git push ${remote} ${tag}`);
      emit('refresh', false, () => emit('pending', ''));
      notify(`Pushed tag ${tag} to ${remote}`, 'success');
    } catch (e: any) {
      notify(`Failed to push tag: ${e}`, 'error');
      emit('pending', '');
    }
  } else if (action === 'push_all') {
    const remotes = props.project?.remotes || [];
    const remote = remotes.length > 0 ? remotes[0].name : 'origin';
    emit('pending', `Pushing all tags to ${remote}...`);
    try {
      await invokeGit('push_all_tags', { path: props.project.path, remote }, `git push ${remote} --tags`);
      emit('refresh', false, () => emit('pending', ''));
      notify(`All tags pushed to ${remote}`, 'success');
    } catch (e: any) {
      notify(`Failed to push all tags: ${e}`, 'error');
      emit('pending', '');
    }
  } else if (action === 'copy_name') {
    await navigator.clipboard.writeText(tag);
    notify(`Copied "${tag}" to clipboard`, 'success');
  }
}

async function handleWorktreeAction(action: string) {
  const wt = worktreeMenu.value.worktree;
  worktreeMenu.value.visible = false;
  if (!wt || !props.project?.path) return;

  if (action === 'open_tab') {
    emit('openProject', wt.path);
  } else if (action === 'lock') {
    openLockWorktree(wt);
  } else if (action === 'unlock') {
    emit('pending', `Unlocking worktree...`);
    try {
      await invokeGit('unlock_worktree', { path: props.project.path, worktreePath: wt.path }, `git worktree unlock ${wt.path}`);
      emit('refresh', false, () => emit('pending', ''));
      notify(`Worktree unlocked`, 'success');
    } catch (e: any) {
      notify(`Failed to unlock worktree: ${e}`, 'error');
      emit('pending', '');
    }
  } else if (action === 'remove') {
    openRemoveWorktree(wt);
  } else if (action === 'reveal') {
    try {
      const { openPath } = await import('@tauri-apps/plugin-opener');
      await openPath(wt.path);
    } catch (e) {
      console.error('Failed to open worktree path:', e);
    }
  }
}

async function handlePruneWorktrees() {
  if (!props.project?.path) return;
  emit('pending', 'Pruning stale worktrees...');
  try {
    await invokeGit('prune_worktrees', { path: props.project.path }, 'git worktree prune');
    emit('refresh', false, () => emit('pending', ''));
    notify('Stale worktrees pruned', 'success');
  } catch (e: any) {
    notify(`Failed to prune worktrees: ${e}`, 'error');
    emit('pending', '');
  }
}

async function handleSubmoduleAction(action: string) {
  const sm = submoduleMenu.value.submodule;
  submoduleMenu.value.visible = false;
  if (!sm || !props.project?.path) return;

  if (action === 'open_tab') {
    emit('openProject', sm.full_path);
  } else if (action === 'init') {
    emit('pending', `Initializing submodule ${sm.name}...`);
    try {
      await invokeGit('init_submodules', { path: props.project.path, subPath: sm.path, recursive: true }, `git submodule update --init --recursive -- ${sm.path}`);
      emit('refresh', false, () => emit('pending', ''));
      notify(`Submodule ${sm.name} initialized`, 'success');
    } catch (e: any) {
      notify(`Failed to initialize submodule: ${e}`, 'error');
      emit('pending', '');
    }
  } else if (action === 'update') {
    emit('pending', `Updating submodule ${sm.name}...`);
    try {
      await invokeGit('update_submodules', { path: props.project.path, subPath: sm.path, remote: true, recursive: true }, `git submodule update --remote --recursive -- ${sm.path}`);
      emit('refresh', false, () => emit('pending', ''));
      notify(`Submodule ${sm.name} updated`, 'success');
    } catch (e: any) {
      notify(`Failed to update submodule: ${e}`, 'error');
      emit('pending', '');
    }
  } else if (action === 'sync') {
    emit('pending', `Syncing submodule URL for ${sm.name}...`);
    try {
      await invokeGit('sync_submodules', { path: props.project.path, subPath: sm.path, recursive: true }, `git submodule sync --recursive -- ${sm.path}`);
      emit('refresh', false, () => emit('pending', ''));
      notify(`Submodule ${sm.name} synchronized`, 'success');
    } catch (e: any) {
      notify(`Failed to sync submodule: ${e}`, 'error');
      emit('pending', '');
    }
  } else if (action === 'reveal') {
    try {
      const { openPath } = await import('@tauri-apps/plugin-opener');
      await openPath(sm.full_path);
    } catch (e) {
      console.error('Failed to open submodule path:', e);
    }
  }
}

async function handleSyncAllSubmodules() {
  if (!props.project?.path) return;
  emit('pending', 'Syncing all submodules...');
  try {
    await invokeGit('init_submodules', { path: props.project.path, subPath: null, recursive: true }, 'git submodule update --init --recursive');
    await invokeGit('update_submodules', { path: props.project.path, subPath: null, remote: true, recursive: true }, 'git submodule update --remote --recursive');
    emit('refresh', false, () => emit('pending', ''));
    notify('All submodules updated and synchronized', 'success');
  } catch (e: any) {
    notify(`Failed to sync submodules: ${e}`, 'error');
    emit('pending', '');
  }
}

function handleGlobalSearchKey(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'f') {
    e.preventDefault();
    searchInputRef.value?.focus();
    searchInputRef.value?.select();
  } else if (e.key === '/' && document.activeElement?.tagName !== 'INPUT' && document.activeElement?.tagName !== 'TEXTAREA') {
    e.preventDefault();
    searchInputRef.value?.focus();
  }
}

onMounted(() => {
  document.addEventListener('click', closeMenus);
  window.addEventListener('keydown', handleGlobalSearchKey);
});

onUnmounted(() => {
  document.removeEventListener('click', closeMenus);
  window.removeEventListener('keydown', handleGlobalSearchKey);
});

</script>

<template>
  <section class="source-control-page" aria-label="Source control">
    <LeftSidebar
      :project="project"
      :width="leftSidebarWidth"
      @create-branch="openCreateBranch"
      @item-click="emit('itemClick', $event)"
      @branch-dblclick="emit('branchDblclick', $event)"
      @branch-context="forwardBranchContext"
      @remote-context="forwardRemoteContext"
      @add-remote="openAddRemote"
      @stash-context="forwardStashContext"
      @load-authors="emit('loadAuthors')"
      @open-contributor-report="emit('openContributorsReport')"
      @create-tag="openCreateTag"
      @tag-context="forwardTagContext"
      @create-worktree="openCreateWorktree"
      @worktree-context="forwardWorktreeContext"
      @open-worktree="emit('openProject', $event)"
      @prune-worktrees="handlePruneWorktrees"
      @add-submodule="openAddSubmodule"
      @submodule-context="forwardSubmoduleContext"
      @open-submodule="emit('openProject', $event)"
      @sync-all-submodules="handleSyncAllSubmodules"
    />

    <div class="resizer" @mousedown.stop.prevent="emit('resizeLeft', $event)"></div>

    <main class="main-content">
      <div class="graph-toolbar">
        <div class="graph-toolbar-left" :title="displayBranch">
          <BranchIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2.5" fill="none" style="color: var(--accent); flex-shrink: 0;" />
          <span class="graph-toolbar-branch">{{ displayBranch }}</span>
        </div>
        <div class="graph-toolbar-actions">
          <button class="repo-action-btn btn-fetch" :disabled="isLoading === 'fetch'" @click="emit('fetch')" title="Fetch from remote">
            <FetchIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            <span class="btn-label">{{ isLoading === 'fetch' ? 'Fetching…' : 'Fetch' }}</span>
          </button>
          <button class="repo-action-btn btn-pull" :disabled="isLoading === 'pull'" @click="emit('pull')" title="Pull from remote">
            <PullIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            <span class="btn-label">{{ isLoading === 'pull' ? 'Pulling…' : 'Pull' }}</span>
          </button>
          <button class="repo-action-btn btn-push" :disabled="isLoading === 'push'" @click="emit('push')" title="Push to remote">
            <PushIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            <span class="btn-label">{{ isLoading === 'push' ? 'Pushing…' : 'Push' }}</span>
          </button>
          <div class="toolbar-separator"></div>
          <button class="repo-action-btn btn-stash" :disabled="isLoading === 'stash'" @click="emit('stash')" title="Stash Changes">
            <HexagonIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            <span class="btn-label">{{ isLoading === 'stash' ? 'Stashing…' : 'Stash' }}</span>
          </button>
          <button class="repo-action-btn btn-pop" :disabled="isLoading === 'pop'" @click="emit('pop')" title="Pop Latest Stash">
            <PopIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            <span class="btn-label">{{ isLoading === 'pop' ? 'Popping…' : 'Pop' }}</span>
          </button>
          <div class="toolbar-separator"></div>
          <button
            class="repo-action-btn btn-changes"
            :class="{ active: detailsMode === 'changes' }"
            :disabled="!hasWorkingChanges"
            @click="emit('showWorkingChanges')"
            :title="hasWorkingChanges ? `Open working changes (${workingChangeCount})` : 'Working tree is clean'"
          >
            <ActionIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            <span class="btn-label">Changes</span>
            <span class="changes-count">{{ workingChangeCount }}</span>
          </button>
          <div class="toolbar-separator"></div>
          <button class="repo-action-btn btn-rebase" @click="openInteractiveRebase()" title="Interactive Rebase & Squash Editor">
            <TimelineIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            <span class="btn-label">Rebase</span>
          </button>
          <button 
            v-if="project?.operationState?.has_conflicts || (project?.uncommittedChanges?.unstaged_files || []).some((f: any) => f.has_conflicts)" 
            class="repo-action-btn btn-conflict-alert" 
            @click="openConflictResolver()" 
            title="Resolve Conflicts in 3-Way Editor"
          >
            <ShieldAlertIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" style="color: #ff9800;" />
            <span class="btn-label" style="color: #ff9800; font-weight: 600;">Resolve Conflicts</span>
          </button>
          <div v-if="project?.cherryPick?.active" class="cherry-pick-controls" role="status">
            <span class="cherry-pick-status">
              {{ project.cherryPick.has_conflicts ? 'Resolve conflicts, then continue' : 'Cherry-pick paused' }}
              <span v-if="project.cherryPick.remaining_count"> · {{ project.cherryPick.remaining_count }} queued</span>
            </span>
            <button
              class="cherry-pick-btn cherry-pick-continue"
              :disabled="project.cherryPick.has_conflicts"
              :title="project.cherryPick.has_conflicts ? 'Resolve and stage conflicted files in Changes before continuing' : 'Continue cherry-pick'"
              @click="emit('cherryPickAction', 'continue')"
            >Continue</button>
            <button class="cherry-pick-btn" title="Skip current commit" @click="emit('cherryPickAction', 'skip')">Skip</button>
            <button class="cherry-pick-btn cherry-pick-abort" title="Abort cherry-pick" @click="emit('cherryPickAction', 'abort')">Abort</button>
          </div>
        </div>
      </div>

      <!-- Repo Operation / Conflict Status Banner -->
      <div v-if="project?.operationState?.operation && project?.operationState?.operation !== 'none'" class="repo-operation-banner" role="status">
        <div class="banner-left">
          <ShieldAlertIcon class="op-icon" viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
          <div class="op-details">
            <strong class="op-name">{{ operationTitle }}</strong>
            <span class="op-status-text">
              {{ project.operationState.has_conflicts ? `Conflicts in ${project.operationState.conflicted_files.length} file(s) must be resolved` : 'Ready to continue' }}
            </span>
          </div>
        </div>
        <div class="banner-actions">
          <button v-if="project.operationState.has_conflicts" class="banner-btn banner-btn-resolve" @click="openConflictResolver()">
            Resolve in 3-Way Editor
          </button>
          <button class="banner-btn banner-btn-continue" :disabled="project.operationState.has_conflicts" @click="handleContinueOperation">
            Continue
          </button>
          <button class="banner-btn banner-btn-abort" @click="handleAbortOperation">
            Abort
          </button>
        </div>
      </div>

      <!-- Rebase in Progress Banner -->
      <div v-else-if="project?.rebaseStatus?.active" class="repo-operation-banner rebase-active-banner" role="status">
        <div class="banner-left">
          <TimelineIcon class="op-icon" viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
          <div class="op-details">
            <strong class="op-name">Interactive Rebase in Progress</strong>
            <span class="op-status-text">
              Step {{ project.rebaseStatus.step }} of {{ project.rebaseStatus.total_steps }} ({{ project.rebaseStatus.head_name }} onto {{ project.rebaseStatus.onto }})
            </span>
          </div>
        </div>
        <div class="banner-actions">
          <button v-if="project.rebaseStatus.conflicts" class="banner-btn banner-btn-resolve" @click="openConflictResolver()">
            Resolve Conflicts
          </button>
          <button class="banner-btn banner-btn-continue" :disabled="project.rebaseStatus.conflicts" @click="handleContinueRebase">
            Continue
          </button>
          <button class="banner-btn banner-btn-skip" @click="handleSkipRebase">
            Skip
          </button>
          <button class="banner-btn banner-btn-abort" @click="handleAbortRebase">
            Abort
          </button>
        </div>
      </div>

      <!-- Graph Filters Bar -->
      <div class="graph-filters-bar">
        <!-- Search Box -->
        <div class="filters-search-box" :class="{ 'has-query': Boolean(filterKeyword) }">
          <SearchIcon class="filter-search-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
          <input
            ref="searchInputRef"
            v-model="filterKeyword"
            type="text"
            placeholder="Search commits, authors, hashes..."
            class="filter-input"
            @keydown.esc="filterKeyword = ''; searchInputRef?.blur()"
          />
          <span v-if="filterKeyword" class="search-match-pill" title="Matching commits">
            {{ displayCommits.length }}
          </span>
          <button
            v-if="filterKeyword"
            class="filter-clear-btn"
            @click="filterKeyword = ''; searchInputRef?.focus()"
            title="Clear search (Esc)"
          >
            <CloseIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" />
          </button>
          <kbd v-else class="search-kbd-hint" @click="searchInputRef?.focus()" title="Press ⌘F or / to search">⌘F</kbd>
        </div>

        <div class="filter-dropdown-group">
          <!-- Author Dropdown Pill -->
          <div class="filter-pill" :class="{ 'is-active': Boolean(filterAuthor) }" :title="filterAuthor ? `Filtered by Author: ${filterAuthor}` : 'Filter by Author'">
            <UserIcon class="pill-icon" viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
            <select v-model="filterAuthor" class="pill-select">
              <option value="">All Authors</option>
              <option v-for="author in availableAuthors" :key="author" :value="author">{{ author }}</option>
            </select>
            <ChevronDownIcon class="pill-chevron" viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2" fill="none" />
          </div>

          <!-- Date Range Dropdown Pill -->
          <div class="filter-pill" :class="{ 'is-active': filterDateRange !== 'all' }" :title="filterDateRange !== 'all' ? `Filtered by Date: ${filterDateRange}` : 'Filter by Date'">
            <TimelineIcon class="pill-icon" viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
            <select v-model="filterDateRange" class="pill-select">
              <option value="all">All Time</option>
              <option value="today">Today</option>
              <option value="7d">Last 7 Days</option>
              <option value="30d">Last 30 Days</option>
              <option value="90d">Last 3 Months</option>
              <option value="180d">Last 6 Months</option>
              <option value="365d">Last Year</option>
            </select>
            <ChevronDownIcon class="pill-chevron" viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2" fill="none" />
          </div>

          <!-- Branch Dropdown Pill -->
          <div class="filter-pill" :class="{ 'is-active': filterBranch !== 'all' }" :title="filterBranch !== 'all' ? `Filtered by Branch: ${filterBranch}` : 'Filter by Branch'">
            <BranchIcon class="pill-icon" viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
            <select v-model="filterBranch" class="pill-select">
              <option value="all">All Branches</option>
              <option value="current">Current ({{ displayBranch }})</option>
              <optgroup label="Local Branches" v-if="project?.localBranches?.length">
                <option v-for="b in project.localBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
              </optgroup>
              <optgroup label="Remote Branches" v-if="project?.remoteBranches?.length">
                <option v-for="b in project.remoteBranches" :key="b.name" :value="b.name">{{ b.name }}</option>
              </optgroup>
            </select>
            <ChevronDownIcon class="pill-chevron" viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2" fill="none" />
          </div>
        </div>

        <!-- Filter info / reset button -->
        <div class="filter-summary-group" v-if="isFilterActive">
          <span class="filter-count-badge">
            {{ displayCommits.length }} / {{ (project?.rawCommits || commits).length }}
          </span>
          <button class="clear-all-filters-btn" @click="clearAllFilters" title="Reset all filters">
            <CloseIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" />
            <span>Reset</span>
          </button>
        </div>
      </div>

      <CommitGraph
        :commits="displayCommits"
        :selected-commit="selectedCommit"
        :selected-commits="selectedCommits"
        @select-commit="forwardSelectCommit"
        @contextmenu-commit="openCommitContextMenu"
        @branch-drop="handleBranchDrop"
      />

      <DiffViewer
        v-if="selectedFileDiff !== null"
        :file-path="selectedFilePath || ''"
        :file-diff="selectedFileDiff"
        :project-path="project?.path"
        :is-staged="isCurrentFileStaged"
        :can-stage-hunks="detailsMode === 'changes'"
        @patch-applied="emit('refresh', false)"
        @close="emit('closeDiff')"
      />
    </main>

    <div v-if="detailsTarget" class="resizer" @mousedown.stop.prevent="emit('resizeRight', $event)"></div>

    <RightSidebar
      v-if="detailsTarget"
      :commit="detailsTarget"
      :mode="detailsMode || 'commit'"
      :width="rightSidebarWidth"
      :active-branch="displayBranch"
      :has-commit="Boolean(selectedCommit)"
      :has-working-changes="hasWorkingChanges"
      :working-change-count="workingChangeCount"
      :head-commit="headCommit"
      :amend-trigger="amendTrigger"
      @close="emit('closeDetails')"
      @change-mode="emit('changeDetailsMode', $event)"
      @select-file="emit('selectFile', $event)"
      @stage-file="emit('stageFile', $event)"
      @unstage-file="emit('unstageFile', $event)"
      @stage-all="emit('stageAll')"
      @commit-changes="(msg: string, amend?: boolean) => emit('commitChanges', msg, amend)"
    />

    <RemoteModal ref="remoteModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @refresh="done => emit('refresh', false, done)" />
    <RenameBranchModal ref="renameBranchModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @refresh="(fetchRemote, done) => emit('refresh', fetchRemote, done)" />
    <CreateBranchModal ref="createBranchModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @refresh="done => emit('refresh', false, done)" />
    <ConfirmDeleteBranchModal ref="deleteBranchModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @refresh="(fetchRemote, done) => emit('refresh', fetchRemote, done)" />
    <ConfirmDropStashModal ref="dropStashModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @refresh="done => emit('refresh', true, done)" />
    <CreateTagModal ref="createTagModalRef" :repository-path="project?.path || null" :remotes="project?.remotes" @pending="emit('pending', $event)" @refresh="done => emit('refresh', false, done)" />
    <ConfirmDeleteTagModal ref="deleteTagModalRef" :repository-path="project?.path || null" :remotes="project?.remotes" @pending="emit('pending', $event)" @refresh="done => emit('refresh', false, done)" />
    <CreateWorktreeModal ref="createWorktreeModalRef" :repository-path="project?.path || null" :branches="project?.localBranches" @pending="emit('pending', $event)" @refresh="done => emit('refresh', false, done)" @open-worktree="emit('openProject', $event)" />
    <ConfirmRemoveWorktreeModal ref="deleteWorktreeModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @refresh="done => emit('refresh', false, done)" />
    <LockWorktreeModal ref="lockWorktreeModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @refresh="done => emit('refresh', false, done)" />
    <AddSubmoduleModal ref="addSubmoduleModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @refresh="done => emit('refresh', false, done)" />
    <InteractiveRebaseModal ref="rebaseModalRef" :project="project" @rebase-complete="emit('refresh', false)" />
    <MergeConflictResolverModal ref="conflictModalRef" :project="project" @operation-updated="emit('refresh', false)" />
    <ConfirmDropCommitModal ref="dropCommitModalRef" :repository-path="project?.path || null" @pending="emit('pending', $event)" @dropped="emit('refresh', false)" />

    <ContextMenu :visible="branchMenu.visible" :x="branchMenu.x" :y="branchMenu.y" :branch="branchMenu.branch" @action="handleBranchAction" />
    <BranchDropMenu
      :visible="branchDropMenu.visible"
      :x="branchDropMenu.x"
      :y="branchDropMenu.y"
      :source-branch="branchDropMenu.sourceBranch"
      :target-branch="branchDropMenu.targetBranch"
      :active-branch="displayBranch"
      @merge="handleBranchDropMerge"
      @rebase="handleBranchDropRebase"
      @close="branchDropMenu.visible = false"
    />
    <CommitContextMenu :visible="commitMenu.visible" :x="commitMenu.x" :y="commitMenu.y" :commits="commitMenu.commits" :can-cherry-pick="canCherryPick" @action="handleCommitAction" />
    <RemoteContextMenu :visible="remoteMenu.visible" :x="remoteMenu.x" :y="remoteMenu.y" :remote="remoteMenu.remote" @action="handleRemoteAction" />
    <StashContextMenu :visible="stashMenu.visible" :x="stashMenu.x" :y="stashMenu.y" :stash="stashMenu.stash" @action="handleStashAction" />
    <TagContextMenu :visible="tagMenu.visible" :x="tagMenu.x" :y="tagMenu.y" :tag="tagMenu.tag" @action="handleTagAction" />
    <WorktreeContextMenu :visible="worktreeMenu.visible" :x="worktreeMenu.x" :y="worktreeMenu.y" :worktree="worktreeMenu.worktree" @action="handleWorktreeAction" />
    <SubmoduleContextMenu :visible="submoduleMenu.visible" :x="submoduleMenu.x" :y="submoduleMenu.y" :submodule="submoduleMenu.submodule" @action="handleSubmoduleAction" />
  </section>
</template>

<style scoped>
.source-control-page {
  display: flex;
  flex: 1;
  min-width: 0;
  overflow: hidden;
}

/* Graph Filters Bar */
.graph-filters-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border);
  font-size: 0.78rem;
  flex-wrap: wrap;
  flex-shrink: 0;
}

/* --- Graph Filters Bar --- */
.graph-filters-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 14px;
  background: var(--surface-subtle, rgba(255, 255, 255, 0.03));
  border-bottom: 1px solid var(--border, rgba(128, 128, 128, 0.15));
  font-size: 0.78rem;
  flex-wrap: wrap;
  flex-shrink: 0;
  user-select: none;
}

/* Enhanced Search Box */
.filters-search-box {
  position: relative;
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 210px;
  max-width: 320px;
  background: var(--bg-main, #0d1117);
  border: 1px solid var(--border, rgba(128, 128, 128, 0.22));
  border-radius: 6px;
  transition: all 0.15s ease;
}

.filters-search-box:focus-within {
  border-color: var(--accent-blue, #58a6ff);
  box-shadow: 0 0 0 2px rgba(88, 166, 255, 0.18);
  max-width: 360px;
}

.filters-search-box.has-query {
  border-color: rgba(88, 166, 255, 0.4);
}

.filter-search-icon {
  position: absolute;
  left: 9px;
  color: var(--text-muted, #8b949e);
  pointer-events: none;
  transition: color 0.15s ease;
}

.filters-search-box:focus-within .filter-search-icon {
  color: var(--accent-blue, #58a6ff);
}

.filter-input {
  width: 100%;
  padding: 5px 56px 5px 28px;
  border: none;
  background: transparent;
  color: var(--text-main, #c9d1d9);
  font-size: 0.78rem;
  outline: none;
}

.filter-input::placeholder {
  color: var(--text-muted, #8b949e);
  font-size: 0.76rem;
}

.search-match-pill {
  position: absolute;
  right: 28px;
  font-size: 0.68rem;
  font-weight: 700;
  color: var(--accent-blue, #58a6ff);
  background: rgba(88, 166, 255, 0.15);
  padding: 1px 5px;
  border-radius: 4px;
  pointer-events: none;
}

.search-kbd-hint {
  position: absolute;
  right: 8px;
  font-size: 0.65rem;
  font-family: inherit;
  font-weight: 600;
  color: var(--text-muted, #8b949e);
  background: var(--surface-hover, rgba(255, 255, 255, 0.07));
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  border-radius: 3px;
  padding: 1px 4px;
  cursor: pointer;
}

.filter-clear-btn {
  position: absolute;
  right: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  background: transparent;
  border: none;
  border-radius: 50%;
  color: var(--text-muted, #8b949e);
  cursor: pointer;
  transition: all 0.15s ease;
}

.filter-clear-btn:hover {
  background: var(--surface-hover, rgba(255, 255, 255, 0.12));
  color: var(--text-main, #ffffff);
}

/* Filter Dropdown Group & Pills */
.filter-dropdown-group {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-pill {
  position: relative;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 8px 0 9px;
  height: 28px;
  background: var(--bg-main, #0d1117);
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  border-radius: 6px;
  color: var(--text-muted, #8b949e);
  font-size: 0.76rem;
  transition: all 0.15s ease;
}

.filter-pill:hover {
  border-color: var(--accent-blue, #58a6ff);
  color: var(--text-main, #e6edf3);
}

.filter-pill.is-active {
  background: rgba(88, 166, 255, 0.12);
  border-color: var(--accent-blue, #58a6ff);
  color: var(--accent-blue, #58a6ff);
  font-weight: 600;
}

.pill-icon {
  flex-shrink: 0;
  opacity: 0.85;
}

.filter-pill.is-active .pill-icon {
  opacity: 1;
  color: var(--accent-blue, #58a6ff);
}

.pill-select {
  appearance: none;
  -webkit-appearance: none;
  background: transparent;
  border: none;
  color: inherit;
  font-size: 0.75rem;
  font-weight: inherit;
  outline: none;
  cursor: pointer;
  padding-right: 14px;
  max-width: 140px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.pill-chevron {
  position: absolute;
  right: 6px;
  pointer-events: none;
  opacity: 0.7;
}

/* Filter Summary & Reset */
.filter-summary-group {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
}

.filter-count-badge {
  font-size: 0.72rem;
  font-weight: 600;
  padding: 3px 8px;
  border-radius: 5px;
  background: rgba(88, 166, 255, 0.12);
  color: var(--accent-blue, #58a6ff);
  border: 1px solid rgba(88, 166, 255, 0.25);
}

.clear-all-filters-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: var(--bg-main, #0d1117);
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  color: var(--text-muted, #8b949e);
  padding: 3px 8px;
  border-radius: 5px;
  font-size: 0.72rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.clear-all-filters-btn:hover {
  background: rgba(248, 81, 73, 0.15);
  border-color: rgba(248, 81, 73, 0.4);
  color: #f85149;
}

/* --- Repo Operation & Rebase Banners --- */
.repo-operation-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 14px;
  background: linear-gradient(90deg, rgba(255, 152, 0, 0.15), rgba(244, 67, 54, 0.1));
  border-bottom: 1px solid rgba(255, 152, 0, 0.35);
  color: var(--text-main, #e0e0e0);
  font-size: 0.8rem;
  flex-shrink: 0;
  animation: bannerSlide 0.2s ease-out;
}

.rebase-active-banner {
  background: linear-gradient(90deg, rgba(0, 188, 212, 0.15), rgba(33, 150, 243, 0.1));
  border-bottom-color: rgba(0, 188, 212, 0.35);
}

@keyframes bannerSlide {
  from { opacity: 0; transform: translateY(-4px); }
  to { opacity: 1; transform: translateY(0); }
}

.banner-left {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.op-icon {
  flex-shrink: 0;
  color: #ff9800;
}

.rebase-active-banner .op-icon {
  color: var(--accent, #00bcd4);
}

.op-details {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex-wrap: wrap;
}

.op-name {
  font-weight: 600;
  color: var(--text-main, #fff);
  white-space: nowrap;
}

.op-status-text {
  color: var(--text-muted, #aaa);
  font-size: 0.76rem;
}

.banner-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.banner-btn {
  padding: 4px 10px;
  font-size: 0.75rem;
  font-weight: 600;
  border-radius: 4px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.25));
  background: var(--bg-card, #252830);
  color: var(--text-main, #e0e0e0);
  cursor: pointer;
  transition: all 0.15s ease;
}

.banner-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.3);
}

.banner-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.banner-btn-resolve {
  background: #ff9800;
  color: #111;
  border-color: #ff9800;
  font-weight: 700;
}

.banner-btn-resolve:hover:not(:disabled) {
  background: #ffa726;
  border-color: #ffa726;
}

.banner-btn-continue {
  background: #4caf50;
  color: #fff;
  border-color: #4caf50;
}

.banner-btn-continue:hover:not(:disabled) {
  background: #66bb6a;
  border-color: #66bb6a;
}

.banner-btn-skip {
  background: rgba(255, 255, 255, 0.08);
}

.banner-btn-abort {
  background: rgba(244, 67, 54, 0.15);
  color: #f44336;
  border-color: rgba(244, 67, 54, 0.3);
}

.banner-btn-abort:hover:not(:disabled) {
  background: rgba(244, 67, 54, 0.25);
  border-color: #f44336;
}

.btn-conflict-alert {
  background: rgba(255, 152, 0, 0.12) !important;
  border-color: rgba(255, 152, 0, 0.4) !important;
}

.btn-conflict-alert:hover {
  background: rgba(255, 152, 0, 0.22) !important;
}
</style>
