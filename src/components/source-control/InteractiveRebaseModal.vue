<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-card rebase-modal-card">
      <div class="modal-header">
        <div class="modal-title">
          <TimelineIcon viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" style="color: var(--accent, #00bcd4);" />
          <span>Interactive Rebase & Squash Editor</span>
        </div>
        <button class="close-btn" @click="close" title="Close">
          <CloseIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
        </button>
      </div>

      <div class="modal-body rebase-modal-body">
        <!-- Target & Base Selector -->
        <div class="rebase-top-bar">
          <div class="rebase-info-col">
            <span class="info-label">Active Branch</span>
            <span class="info-value branch-badge">{{ activeBranchName }}</span>
          </div>
          <div class="rebase-arrow">➔</div>
          <div class="rebase-info-col" style="flex: 1;">
            <span class="info-label">Rebase Onto Base Commit / Branch</span>
            <div class="base-select-row">
              <input 
                v-model="baseRef" 
                type="text" 
                placeholder="e.g. main, origin/main, or commit SHA" 
                class="base-input"
                @change="reloadCommits"
                @keydown.enter="reloadCommits"
              />
              <button class="reload-commits-btn" @click="reloadCommits" :disabled="isLoading" title="Load commits against this base">
                Load
              </button>
            </div>
          </div>
        </div>

        <!-- Quick actions toolbar -->
        <div class="rebase-quick-actions" v-if="todoItems.length > 0">
          <span class="commits-count-tag">{{ todoItems.length }} commit(s) to rebase</span>
          <div class="quick-btns-group">
            <button class="quick-btn" @click="squashAll" title="Squash all commits into the first one">
              Squash All into First
            </button>
            <button class="quick-btn" @click="resetAllPick" title="Reset all actions to Pick">
              Reset to Pick
            </button>
          </div>
        </div>

        <!-- Validation warning if any -->
        <div v-if="validationError" class="rebase-warning-banner">
          <AlertIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
          <span>{{ validationError }}</span>
        </div>

        <!-- Loading state -->
        <div v-if="isLoading" class="rebase-loading-state">
          <span>Loading commits between {{ baseRef }} and HEAD...</span>
        </div>

        <!-- Empty state -->
        <div v-else-if="todoItems.length === 0" class="rebase-empty-state">
          <span>No commits found between <code>{{ baseRef }}</code> and current HEAD. Choose a different base commit or branch above.</span>
        </div>

        <!-- Commits todo list (oldest to newest) -->
        <div v-else class="rebase-todo-list">
          <div 
            v-for="(item, index) in todoItems" 
            :key="item.hash"
            class="rebase-todo-item"
            :class="[`action-${item.action}`, { 'is-first': index === 0 }]"
          >
            <!-- Reorder controls -->
            <div class="todo-reorder-ctrls">
              <button 
                class="reorder-btn" 
                :disabled="index === 0" 
                @click="moveUp(index)"
                title="Move earlier in history"
              >
                ▲
              </button>
              <button 
                class="reorder-btn" 
                :disabled="index === todoItems.length - 1" 
                @click="moveDown(index)"
                title="Move later in history"
              >
                ▼
              </button>
            </div>

            <!-- Action selector -->
            <div class="todo-action-select-container">
              <select v-model="item.action" class="action-select" :class="`select-${item.action}`">
                <option value="pick">pick (keep)</option>
                <option value="reword">reword (edit msg)</option>
                <option value="squash" :disabled="index === 0">squash (meld into prev)</option>
                <option value="fixup" :disabled="index === 0">fixup (meld, discard msg)</option>
                <option value="drop">drop (delete)</option>
              </select>
            </div>

            <!-- Commit details & message editor -->
            <div class="todo-content">
              <div class="todo-top-line">
                <span class="todo-hash">{{ item.short_hash || item.hash.substring(0, 7) }}</span>
                <span class="todo-author" v-if="item.author">by {{ item.author }}</span>
                <span class="todo-action-pill" :class="`pill-${item.action}`">{{ item.action }}</span>
              </div>

              <!-- Message: read-only or editable textarea if reword/squash -->
              <div v-if="item.action === 'reword' || item.action === 'squash'" class="todo-edit-message-box">
                <textarea 
                  v-model="item.message" 
                  class="todo-msg-textarea" 
                  rows="2" 
                  placeholder="Enter revised commit message..."
                ></textarea>
              </div>
              <div v-else class="todo-original-message" :title="item.message">
                {{ item.message }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-secondary" @click="close" :disabled="isExecuting">
          Cancel
        </button>
        <button 
          class="btn btn-primary btn-start-rebase" 
          :disabled="isExecuting || todoItems.length === 0 || Boolean(validationError)"
          @click="startRebase"
        >
          <span v-if="isExecuting">Executing Rebase...</span>
          <span v-else>Start Interactive Rebase</span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import CloseIcon from '../../assets/icons/close.svg?component';
import TimelineIcon from '../../assets/icons/timeline.svg?component';
import AlertIcon from '../../assets/icons/shield-alert.svg?component';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { invokeGit } from '../../composables/useActivityLog';
import { notify } from '../../composables/useToasts';
import type { RebaseTodoItem } from '../../types';

const props = defineProps<{
  project: any;
}>();

const emit = defineEmits<{
  (e: 'rebase-complete'): void;
}>();

const isOpen = ref(false);
const baseRef = ref('');
const todoItems = ref<RebaseTodoItem[]>([]);
const isLoading = ref(false);
const isExecuting = ref(false);

const activeBranchName = computed(() => {
  const active = props.project?.localBranches?.find((b: any) => b.active);
  return active ? active.name : 'HEAD';
});

const validationError = computed(() => {
  if (todoItems.value.length === 0) return null;
  const first = todoItems.value[0];
  if (first.action === 'squash' || first.action === 'fixup') {
    return 'The first commit in the rebase list cannot be "squash" or "fixup" because there is no previous commit to combine it into.';
  }
  return null;
});

async function open(defaultBaseRef?: string, initialCommits?: any[]) {
  isOpen.value = true;
  let targetBase = defaultBaseRef || 'HEAD~5';
  if (defaultBaseRef && /^[0-9a-fA-F]{6,40}$/.test(defaultBaseRef.trim())) {
    targetBase = `${defaultBaseRef.trim()}~1`;
  }
  baseRef.value = targetBase;
  
  if (initialCommits && initialCommits.length > 0) {
    todoItems.value = initialCommits.map(c => ({
      action: 'pick' as const,
      hash: c.hash || c.id,
      short_hash: c.hash_short || (c.hash || c.id).substring(0, 7),
      message: c.message,
      author: c.author,
      is_edited: false
    }));
  } else {
    await reloadCommits();
  }
}

function close() {
  if (isExecuting.value) return;
  isOpen.value = false;
  todoItems.value = [];
}

async function reloadCommits() {
  if (!props.project?.path || !baseRef.value.trim()) return;
  isLoading.value = true;
  try {
    const commits = await invoke('get_rebase_commits', {
      path: props.project.path,
      baseRef: baseRef.value.trim()
    }) as any[];

    todoItems.value = (commits || []).map(c => ({
      action: 'pick' as const,
      hash: c.hash,
      short_hash: c.hash_short,
      message: c.message,
      author: c.author,
      is_edited: false
    }));
  } catch (err: any) {
    console.error('Failed to get rebase commits:', err);
    notify(`Failed to fetch commits against ${baseRef.value}: ${err}`, 'error');
    todoItems.value = [];
  } finally {
    isLoading.value = false;
  }
}

function moveUp(index: number) {
  if (index <= 0) return;
  const item = todoItems.value.splice(index, 1)[0];
  todoItems.value.splice(index - 1, 0, item);
}

function moveDown(index: number) {
  if (index >= todoItems.value.length - 1) return;
  const item = todoItems.value.splice(index, 1)[0];
  todoItems.value.splice(index + 1, 0, item);
}

function squashAll() {
  if (todoItems.value.length <= 1) return;
  todoItems.value[0].action = 'pick';
  for (let i = 1; i < todoItems.value.length; i++) {
    todoItems.value[i].action = 'squash';
  }
}

function resetAllPick() {
  for (const item of todoItems.value) {
    item.action = 'pick';
  }
}

async function startRebase() {
  if (!props.project?.path || todoItems.value.length === 0 || validationError.value) return;

  isExecuting.value = true;
  try {
    await invokeGit('start_interactive_rebase', {
      path: props.project.path,
      baseRef: baseRef.value.trim(),
      todo: todoItems.value
    }, `git rebase -i ${baseRef.value.trim()}`);

    notify('Interactive rebase completed successfully!', 'success');
    close();
    emit('rebase-complete');
  } catch (err: any) {
    notify(`Rebase stopped or failed: ${err}`, 'error');
    emit('rebase-complete');
  } finally {
    isExecuting.value = false;
  }
}

defineExpose({ open });
</script>

<style scoped>
.rebase-modal-card {
  width: 680px;
  max-width: 95vw;
  max-height: 88vh;
  display: flex;
  flex-direction: column;
}

.rebase-modal-body {
  padding: 16px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.rebase-top-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: var(--bg-card, #1c2128);
  border: 1px solid var(--border, #30363d);
  border-radius: 8px;
}

.rebase-info-col {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-label {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-muted, #8b949e);
  letter-spacing: 0.5px;
}

.branch-badge {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent, #00bcd4);
  background: rgba(0, 188, 212, 0.12);
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid rgba(0, 188, 212, 0.3);
}

.rebase-arrow {
  color: var(--text-muted, #8b949e);
  font-size: 16px;
}

.base-select-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.base-input {
  flex: 1;
  background: var(--input-bg);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
  outline: none;
}

.base-input:focus {
  border-color: var(--accent);
}

.reload-commits-btn {
  background: var(--surface-subtle);
  color: var(--text-main);
  border: 1px solid var(--border);
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
}

.reload-commits-btn:hover {
  background: var(--surface-hover);
}

.rebase-quick-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2px 4px;
}

.commits-count-tag {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
}

.quick-btns-group {
  display: flex;
  align-items: center;
  gap: 6px;
}

.quick-btn {
  background: transparent;
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.quick-btn:hover {
  color: var(--text-main);
  border-color: var(--accent);
  background: var(--surface-hover);
}

.rebase-warning-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  background: rgba(255, 152, 0, 0.12);
  border: 1px solid rgba(255, 152, 0, 0.3);
  color: var(--warning-text);
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
}

.rebase-loading-state, .rebase-empty-state {
  padding: 24px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}

.rebase-todo-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 440px;
  overflow-y: auto;
}

.rebase-todo-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px;
  transition: border-color 0.15s ease;
}

.rebase-todo-item:hover {
  border-color: var(--border-hover);
}

.rebase-todo-item.action-drop {
  opacity: 0.55;
  background: rgba(244, 67, 54, 0.05);
}

.rebase-todo-item.action-squash, .rebase-todo-item.action-fixup {
  border-left: 3px solid #9c27b0;
}

.todo-reorder-ctrls {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.reorder-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px 4px;
  font-size: 9px;
  border-radius: 2px;
}

.reorder-btn:hover:not(:disabled) {
  color: var(--accent);
  background: rgba(0, 120, 212, 0.15);
}

.reorder-btn:disabled {
  opacity: 0.25;
  cursor: default;
}

.action-select {
  background: var(--input-bg);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 3px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  outline: none;
  cursor: pointer;
}

.action-select.select-pick {
  color: #58a6ff;
}

.action-select.select-reword {
  color: var(--warning-text);
}

.action-select.select-squash, .action-select.select-fixup {
  color: #9c27b0;
}

.action-select.select-drop {
  color: var(--danger-text);
}

.todo-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.todo-top-line {
  display: flex;
  align-items: center;
  gap: 6px;
}

.todo-hash {
  font-family: monospace;
  font-size: 11px;
  color: var(--accent);
  font-weight: 600;
}

.todo-author {
  font-size: 11px;
  color: var(--text-muted);
}

.todo-action-pill {
  margin-left: auto;
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  padding: 1px 6px;
  border-radius: 10px;
}

.pill-pick {
  color: #58a6ff;
  background: rgba(88, 166, 255, 0.12);
}

.pill-reword {
  color: var(--warning-text);
  background: rgba(255, 152, 0, 0.12);
}

.pill-squash, .pill-fixup {
  color: #9c27b0;
  background: rgba(156, 39, 176, 0.12);
}

.pill-drop {
  color: var(--danger-text);
  background: rgba(244, 67, 54, 0.12);
}

.todo-original-message {
  font-size: 12px;
  color: var(--text-main);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.todo-msg-textarea {
  width: 100%;
  background: var(--input-bg);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 6px 8px;
  border-radius: 4px;
  font-size: 12px;
  resize: vertical;
  outline: none;
  font-family: inherit;
}

.todo-msg-textarea:focus {
  border-color: var(--accent);
}

.btn-start-rebase {
  background: var(--accent);
  color: #ffffff;
  font-weight: 600;
}
</style>
