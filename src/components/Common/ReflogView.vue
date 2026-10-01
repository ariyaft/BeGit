<template>
  <div class="reflog-view">
    <!-- Top Header Bar -->
    <div class="reflog-header">
      <div class="reflog-title-area">
        <TimelineIcon viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" style="color: var(--accent, #00bcd4);" />
        <div>
          <h2 class="reflog-title">Git Reflog & "Undo" Safety Net</h2>
          <span class="reflog-subtitle">Recover deleted branches, undo accidental resets, or inspect complete HEAD movement history</span>
        </div>
      </div>

      <div class="reflog-header-controls">
        <!-- Target Ref Selector -->
        <div class="ref-select-wrapper">
          <span class="ctrl-label">Ref:</span>
          <select v-model="selectedRef" @change="loadReflog" class="ref-select">
            <option value="HEAD">HEAD (all movements)</option>
            <option v-for="b in (project?.localBranches || [])" :key="b.name" :value="b.name">
              refs/heads/{{ b.name }}
            </option>
          </select>
        </div>

        <button class="refresh-reflog-btn" @click="loadReflog" :disabled="isLoading" title="Refresh reflog history">
          <FetchIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
          Refresh
        </button>
      </div>
    </div>

    <!-- Filter & Search Bar -->
    <div class="reflog-filter-bar">
      <div class="search-box">
        <SearchIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        <input 
          v-model="searchQuery" 
          type="text" 
          placeholder="Search reflog actions, commit SHAs, descriptions..." 
          class="reflog-search-input"
        />
        <button v-if="searchQuery" class="clear-btn" @click="searchQuery = ''">
          <CloseIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
        </button>
      </div>

      <!-- Action Type Pills -->
      <div class="action-pills">
        <button 
          v-for="act in actionFilters" 
          :key="act.id" 
          class="filter-pill"
          :class="{ active: selectedActionFilter === act.id }"
          @click="selectedActionFilter = act.id"
        >
          {{ act.label }}
        </button>
      </div>
    </div>

    <!-- Main List of Reflog Entries -->
    <div class="reflog-body">
      <div v-if="isLoading" class="reflog-loading">
        Loading reflog entries...
      </div>
      <div v-else-if="filteredEntries.length === 0" class="reflog-empty">
        <div v-if="searchQuery || selectedActionFilter !== 'all'">
          No reflog entries match the current filter.
        </div>
        <div v-else>
          No reflog history found for <code>{{ selectedRef }}</code>.
        </div>
      </div>
      <div v-else class="reflog-list">
        <div 
          v-for="entry in filteredEntries" 
          :key="entry.selector + entry.hash"
          class="reflog-entry-card"
          :class="`entry-action-${entry.action}`"
        >
          <!-- Left: Selector & Action Badge -->
          <div class="entry-meta-col">
            <span class="entry-selector">{{ entry.selector }}</span>
            <span class="action-badge" :class="`badge-${entry.action}`">
              {{ entry.action.toUpperCase() }}
            </span>
            <span class="entry-time" :title="entry.date_iso">{{ entry.date_relative }}</span>
          </div>

          <!-- Middle: Message & Author -->
          <div class="entry-content-col">
            <div class="entry-msg" :title="entry.message">
              {{ entry.message }}
            </div>
            <div class="entry-details-row">
              <span class="entry-hash" :title="entry.hash">{{ entry.short_hash }}</span>
              <span class="entry-author" v-if="entry.author_name">by {{ entry.author_name }}</span>
            </div>
          </div>

          <!-- Right: Quick "Undo & Safety" Action Buttons -->
          <div class="entry-actions-col">
            <button 
              class="entry-action-btn btn-create-branch" 
              @click="promptCreateBranch(entry)"
              title="Create a new branch from this exact point (Recover deleted branch)"
            >
              <BranchIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              Create Branch
            </button>
            <button 
              class="entry-action-btn btn-reset-here" 
              @click="promptReset(entry)"
              title="Reset current branch to this reflog state (Undo accidental changes)"
            >
              Reset Here
            </button>
            <button 
              class="entry-action-btn btn-checkout" 
              @click="checkoutReflog(entry)"
              title="Checkout this commit"
            >
              Checkout
            </button>
            <button 
              class="entry-action-btn btn-copy" 
              @click="copySha(entry.hash)"
              title="Copy commit hash"
            >
              <CopyIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Create Branch Dialog Modal -->
    <div v-if="createBranchModalOpen" class="modal-overlay" @click.self="createBranchModalOpen = false">
      <div class="modal-card reflog-dialog-card">
        <div class="modal-header">
          <div class="modal-title">
            <BranchIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
            <span>Create Branch from Reflog</span>
          </div>
          <button class="close-btn" @click="createBranchModalOpen = false">
            <CloseIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
          </button>
        </div>
        <div class="modal-body">
          <p class="dialog-desc">
            Create and recover a branch from commit <strong class="hash-tag">{{ targetEntry?.short_hash }}</strong> (<em>{{ targetEntry?.message }}</em>):
          </p>
          <input 
            v-model="newBranchName" 
            type="text" 
            placeholder="e.g. recovered-feature-branch" 
            class="dialog-input"
            autofocus
            @keydown.enter="executeCreateBranch"
          />
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="createBranchModalOpen = false">Cancel</button>
          <button class="btn btn-primary" :disabled="!newBranchName.trim()" @click="executeCreateBranch">
            Create Branch
          </button>
        </div>
      </div>
    </div>

    <!-- Reset Confirmation Dialog Modal -->
    <div v-if="resetModalOpen" class="modal-overlay" @click.self="resetModalOpen = false">
      <div class="modal-card reflog-dialog-card">
        <div class="modal-header">
          <div class="modal-title">
            <span>Reset Branch to {{ targetEntry?.selector }} ({{ targetEntry?.short_hash }})</span>
          </div>
          <button class="close-btn" @click="resetModalOpen = false">
            <CloseIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
          </button>
        </div>
        <div class="modal-body">
          <p class="dialog-desc">
            Choose how to reset your active branch to commit <strong>{{ targetEntry?.short_hash }}</strong>:
          </p>
          <div class="reset-modes-group">
            <label class="reset-mode-option" :class="{ selected: resetMode === 'soft' }">
              <input type="radio" v-model="resetMode" value="soft" />
              <div>
                <strong>Soft (--soft)</strong>
                <span>Keep all changes staged in index. Safe for undoing commits.</span>
              </div>
            </label>
            <label class="reset-mode-option" :class="{ selected: resetMode === 'mixed' }">
              <input type="radio" v-model="resetMode" value="mixed" />
              <div>
                <strong>Mixed (--mixed, default)</strong>
                <span>Keep all changes in working directory, but unstage them.</span>
              </div>
            </label>
            <label class="reset-mode-option reset-mode-hard" :class="{ selected: resetMode === 'hard' }">
              <input type="radio" v-model="resetMode" value="hard" />
              <div>
                <strong style="color: #f44336;">Hard (--hard, destructive)</strong>
                <span>Discard all changes in index and working copy. Matches commit exactly.</span>
              </div>
            </label>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="resetModalOpen = false">Cancel</button>
          <button class="btn btn-danger" @click="executeReset">
            Reset to Here
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import TimelineIcon from '../../assets/icons/timeline.svg?component';
import BranchIcon from '../../assets/icons/branch.svg?component';
import SearchIcon from '../../assets/icons/search.svg?component';
import CloseIcon from '../../assets/icons/close.svg?component';
import FetchIcon from '../../assets/icons/fetch.svg?component';
import CopyIcon from '../../assets/icons/copy.svg?component';
import { ref, computed, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { invokeGit } from '../../composables/useActivityLog';
import { notify } from '../../composables/useToasts';
import type { ReflogEntry } from '../../types';

const props = defineProps<{
  project: any;
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
}>();

const selectedRef = ref('HEAD');
const reflogEntries = ref<ReflogEntry[]>([]);
const isLoading = ref(false);
const searchQuery = ref('');
const selectedActionFilter = ref('all');

const actionFilters = [
  { id: 'all', label: 'All' },
  { id: 'commit', label: 'Commits' },
  { id: 'checkout', label: 'Checkouts' },
  { id: 'reset', label: 'Resets' },
  { id: 'rebase', label: 'Rebases' },
  { id: 'merge', label: 'Merges' },
  { id: 'cherry-pick', label: 'Cherry-picks' },
  { id: 'branch', label: 'Branches' }
];

const targetEntry = ref<ReflogEntry | null>(null);
const createBranchModalOpen = ref(false);
const newBranchName = ref('');
const resetModalOpen = ref(false);
const resetMode = ref<'soft' | 'mixed' | 'hard'>('mixed');

const filteredEntries = computed(() => {
  return reflogEntries.value.filter(entry => {
    // 1. Action filter
    if (selectedActionFilter.value !== 'all') {
      if (entry.action !== selectedActionFilter.value) return false;
    }

    // 2. Search query
    if (searchQuery.value.trim()) {
      const q = searchQuery.value.trim().toLowerCase();
      const matchMsg = entry.message.toLowerCase().includes(q);
      const matchHash = entry.hash.toLowerCase().includes(q) || entry.short_hash.toLowerCase().includes(q);
      const matchAuthor = entry.author_name.toLowerCase().includes(q);
      const matchSel = entry.selector.toLowerCase().includes(q);
      if (!matchMsg && !matchHash && !matchAuthor && !matchSel) return false;
    }

    return true;
  });
});

async function loadReflog() {
  if (!props.project?.path) return;
  isLoading.value = true;
  try {
    const entries = await invoke('get_reflog', {
      path: props.project.path,
      refName: selectedRef.value,
      maxCount: 200
    }) as ReflogEntry[];
    reflogEntries.value = entries || [];
  } catch (err: any) {
    console.error('Failed to load reflog:', err);
    notify(`Failed to fetch reflog: ${err}`, 'error');
  } finally {
    isLoading.value = false;
  }
}

function promptCreateBranch(entry: ReflogEntry) {
  targetEntry.value = entry;
  newBranchName.value = '';
  createBranchModalOpen.value = true;
}

async function executeCreateBranch() {
  if (!props.project?.path || !targetEntry.value || !newBranchName.value.trim()) return;
  const name = newBranchName.value.trim();
  createBranchModalOpen.value = false;

  try {
    await invokeGit('create_branch_from_reflog', {
      path: props.project.path,
      branchName: name,
      commitHash: targetEntry.value.hash
    }, `git branch ${name} ${targetEntry.value.short_hash}`);

    notify(`Branch "${name}" created successfully at ${targetEntry.value.short_hash}!`, 'success');
    emit('refresh');
  } catch (err: any) {
    notify(`Failed to create branch: ${err}`, 'error');
  }
}

function promptReset(entry: ReflogEntry) {
  targetEntry.value = entry;
  resetMode.value = 'mixed';
  resetModalOpen.value = true;
}

async function executeReset() {
  if (!props.project?.path || !targetEntry.value) return;
  resetModalOpen.value = false;

  try {
    await invokeGit('reset_to_reflog_entry', {
      path: props.project.path,
      hash: targetEntry.value.hash,
      mode: resetMode.value
    }, `git reset --${resetMode.value} ${targetEntry.value.short_hash}`);

    notify(`Reset active branch to ${targetEntry.value.short_hash} (${resetMode.value})!`, 'success');
    emit('refresh');
    loadReflog();
  } catch (err: any) {
    notify(`Reset failed: ${err}`, 'error');
  }
}

async function checkoutReflog(entry: ReflogEntry) {
  if (!props.project?.path) return;
  try {
    await invokeGit('checkout_branch', {
      path: props.project.path,
      branchName: entry.hash
    }, `git checkout ${entry.short_hash}`);

    notify(`Checked out ${entry.short_hash} (detached HEAD)`, 'info');
    emit('refresh');
    loadReflog();
  } catch (err: any) {
    notify(`Checkout failed: ${err}`, 'error');
  }
}

async function copySha(hash: string) {
  await navigator.clipboard.writeText(hash);
  notify(`Copied SHA "${hash.substring(0, 10)}..." to clipboard`, 'success');
}

watch(() => props.project?.path, () => {
  loadReflog();
}, { immediate: true });

onMounted(() => {
  loadReflog();
});
</script>

<style scoped>
.reflog-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-main);
  color: var(--text-main);
  overflow: hidden;
}

.reflog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: var(--panel-bg);
  border-bottom: 1px solid var(--border);
}

.reflog-title-area {
  display: flex;
  align-items: center;
  gap: 12px;
}

.reflog-title {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  color: var(--text-main);
}

.reflog-subtitle {
  font-size: 11px;
  color: var(--text-muted);
}

.reflog-header-controls {
  display: flex;
  align-items: center;
  gap: 10px;
}

.ref-select-wrapper {
  display: flex;
  align-items: center;
  gap: 6px;
}

.ctrl-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.ref-select {
  background: var(--input-bg);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  outline: none;
  cursor: pointer;
}

.refresh-reflog-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.refresh-reflog-btn:hover {
  background: var(--surface-hover);
  border-color: var(--accent);
}

.reflog-filter-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 20px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border);
  gap: 16px;
  flex-wrap: wrap;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  padding: 4px 10px;
  border-radius: 6px;
  width: 320px;
  color: var(--text-muted);
}

.reflog-search-input {
  background: transparent;
  border: none;
  color: var(--text-main);
  font-size: 12px;
  outline: none;
  width: 100%;
}

.reflog-search-input::placeholder {
  color: var(--text-muted);
}

.clear-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.clear-btn:hover {
  color: var(--text-main);
}

.action-pills {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.filter-pill {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 3px 8px;
  border-radius: 12px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.filter-pill:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

.filter-pill.active {
  background: var(--accent);
  color: #fff;
  border-color: var(--accent);
  font-weight: 700;
}

.reflog-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px 20px;
}

.reflog-loading, .reflog-empty {
  padding: 40px;
  text-align: center;
  color: var(--text-muted);
  font-size: 14px;
}

.reflog-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.reflog-entry-card {
  display: flex;
  align-items: center;
  gap: 14px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px 14px;
  box-shadow: 0 1px 3px var(--shadow-color, rgba(0,0,0,0.06));
  transition: all 0.15s ease;
}

.reflog-entry-card:hover {
  border-color: var(--accent);
  box-shadow: 0 3px 10px var(--shadow-color, rgba(0,0,0,0.12));
}

.entry-meta-col {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 4px;
  width: 110px;
  flex-shrink: 0;
}

.entry-selector {
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  font-weight: 700;
  color: var(--accent-blue, var(--accent));
}

.action-badge {
  font-size: 9px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
}

.badge-commit {
  color: var(--success-text, #15803d);
  background: rgba(76, 175, 80, 0.14);
}

.badge-checkout {
  color: var(--info-text, #0284c7);
  background: rgba(33, 150, 243, 0.14);
}

.badge-reset {
  color: var(--danger-text, #dc2626);
  background: rgba(239, 68, 68, 0.14);
}

.badge-rebase {
  color: var(--warning-text, #d97706);
  background: rgba(245, 158, 11, 0.14);
}

.badge-merge {
  color: var(--accent-blue, #0078d4);
  background: rgba(0, 188, 212, 0.14);
}

.badge-cherry-pick {
  color: #db2777;
  background: rgba(236, 72, 153, 0.14);
}

.badge-pull {
  color: var(--success-text, #15803d);
  background: rgba(139, 195, 74, 0.14);
}

.badge-branch {
  color: var(--purple-text, #7c3aed);
  background: rgba(168, 85, 247, 0.14);
}

.badge-other {
  color: var(--text-muted);
  background: var(--surface-subtle);
}

.entry-time {
  font-size: 10px;
  color: var(--text-muted);
}

.entry-content-col {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.entry-msg {
  font-size: 13px;
  color: var(--text-main);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.entry-details-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.entry-hash {
  font-family: var(--font-mono, monospace);
  font-size: 11px;
  color: var(--accent-blue, var(--accent));
  font-weight: 600;
}

.entry-author {
  font-size: 11px;
  color: var(--text-muted);
}

.entry-actions-col {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.entry-action-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.entry-action-btn:hover {
  background: var(--surface-hover);
  border-color: var(--accent);
}

.btn-create-branch {
  color: var(--accent);
  background: rgba(0, 120, 212, 0.08);
  border-color: rgba(0, 120, 212, 0.25);
}

.btn-create-branch:hover {
  background: rgba(0, 120, 212, 0.16);
  border-color: var(--accent);
}

.btn-reset-here {
  color: var(--warning-text, #d97706);
  background: rgba(245, 158, 11, 0.08);
  border-color: rgba(245, 158, 11, 0.25);
}

.btn-reset-here:hover {
  background: rgba(245, 158, 11, 0.16);
  border-color: var(--warning-text, #d97706);
}

.btn-checkout:hover {
  color: var(--accent);
}

.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  z-index: 9998;
  display: grid;
  place-items: center;
  padding: 16px;
}

.modal-card.reflog-dialog-card {
  width: 480px;
  max-width: 90vw;
  background: var(--bg-surface, var(--bg-card));
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 16px 36px var(--shadow-color, rgba(0, 0, 0, 0.4));
  overflow: hidden;
  color: var(--text-main);
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border);
}

.modal-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 14px;
  color: var(--text-main);
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

.modal-body {
  padding: 16px 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-desc {
  font-size: 13px;
  color: var(--text-muted);
  margin-bottom: 4px;
  line-height: 1.4;
}

.hash-tag {
  font-family: var(--font-mono, monospace);
  color: var(--accent-blue, var(--accent));
}

.dialog-input {
  width: 100%;
  background: var(--input-bg);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
}

.dialog-input:focus {
  border-color: var(--accent);
}

.reset-modes-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 6px;
}

.reset-mode-option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 10px 12px;
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-main);
  transition: all 0.15s ease;
}

.reset-mode-option input {
  margin-top: 3px;
}

.reset-mode-option.selected {
  border-color: var(--accent);
  background: rgba(0, 120, 212, 0.08);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 18px;
  border-top: 1px solid var(--border);
  background: var(--bg-surface-footer, rgba(0, 0, 0, 0.03));
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 7px 14px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.btn-secondary {
  background: var(--surface-subtle);
  border-color: var(--border);
  color: var(--text-main);
}

.btn-secondary:hover {
  background: var(--surface-hover);
}

.btn-primary {
  background: var(--accent);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--accent-hover);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-danger {
  background: var(--danger-text, #dc2626);
  color: white;
}

.btn-danger:hover {
  background: #b91c1c;
}
</style>
