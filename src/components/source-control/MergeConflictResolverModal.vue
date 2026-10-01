<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="close">
    <div class="modal-card conflict-modal-card">
      <div class="modal-header conflict-header">
        <div class="modal-title conflict-title">
          <ShieldAlertIcon viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" style="color: #f44336;" />
          <span>3-Way Merge Conflict Resolver</span>
          <span class="op-badge" :class="`op-${operationState.operation}`">
            {{ operationState.operation.toUpperCase() }}
          </span>
          <span class="conflict-counter" v-if="conflictedFiles.length > 0">
            {{ resolvedCount }} of {{ conflictedFiles.length }} file(s) resolved
          </span>
        </div>
        <div class="conflict-header-actions">
          <button 
            v-if="operationState.operation !== 'none'" 
            class="btn-abort-op" 
            @click="handleAbortOperation"
            :disabled="isProcessing"
            title="Abort this merge/rebase/cherry-pick and discard all changes"
          >
            Abort {{ operationLabel }}
          </button>
          <button 
            v-if="operationState.operation !== 'none'"
            class="btn-continue-op"
            :disabled="isProcessing || unmergedCount > 0"
            @click="handleContinueOperation"
            title="Continue and finalize this operation"
          >
            Continue {{ operationLabel }}
          </button>
          <button class="close-btn" @click="close" title="Close Resolver">
            <CloseIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
          </button>
        </div>
      </div>

      <div class="conflict-body">
        <!-- Sidebar: Conflicted files list -->
        <div class="conflict-sidebar">
          <div class="sidebar-title">
            <span>Conflicted Files ({{ conflictedFiles.length }})</span>
          </div>
          <div v-if="isLoading" class="conflict-loading">
            Loading conflicts...
          </div>
          <div v-else-if="conflictedFiles.length === 0" class="conflict-none">
            <CheckIcon viewBox="0 0 24 24" width="20" height="20" stroke="#4caf50" stroke-width="2.5" fill="none" />
            <span>No conflicts detected!</span>
          </div>
          <div v-else class="conflict-file-list">
            <div
              v-for="file in conflictedFiles"
              :key="file.path"
              class="conflict-file-item"
              :class="{ 'active': activeFilePath === file.path, 'resolved': file.is_resolved }"
              @click="selectFile(file.path)"
            >
              <span class="status-indicator" :class="file.is_resolved ? 'indicator-resolved' : 'indicator-conflicted'">
                {{ file.is_resolved ? '✓' : '!' }}
              </span>
              <span class="file-path-label" :title="file.path">{{ file.path }}</span>
            </div>
          </div>
        </div>

        <!-- Main 3-Way Editor Area -->
        <div class="conflict-main-area" v-if="activeFile">
          <!-- Top Tool Bar for Active File -->
          <div class="active-file-toolbar">
            <div class="active-file-info">
              <span class="file-name-heading">{{ activeFile.path }}</span>
              <span class="conflict-hunk-count">{{ activeFileConflicts.length }} conflict block(s)</span>
            </div>
            
            <div class="file-actions-group">
              <button class="file-action-btn" @click="acceptAllOurs" title="Accept all changes from current branch / HEAD">
                Accept All Ours
              </button>
              <button class="file-action-btn" @click="acceptAllTheirs" title="Accept all changes from incoming branch / cherry-pick">
                Accept All Theirs
              </button>
              <button 
                class="file-action-btn btn-mark-resolved" 
                :disabled="isProcessing"
                @click="saveAndMarkResolved"
                title="Save merged content and mark this file as resolved (git add)"
              >
                <CheckIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.5" fill="none" />
                Mark as Resolved
              </button>
            </div>
          </div>

          <!-- 3-Way Grid Panes -->
          <div class="three-way-grid">
            <!-- Left Pane: OURS -->
            <div class="grid-pane ours-pane">
              <div class="pane-header header-ours">
                <span class="pane-tag">OURS (Current Change)</span>
                <span class="pane-ref">{{ operationState.head_name || 'HEAD' }}</span>
              </div>
              <div class="pane-content">
                <pre class="code-pre"><code v-html="highlightedOurs"></code></pre>
              </div>
            </div>

            <!-- Center Pane: MERGED RESULT (Interactive & Editable) -->
            <div class="grid-pane result-pane">
              <div class="pane-header header-result">
                <span class="pane-tag">MERGED RESULT (Editable)</span>
                <span class="pane-subtag">Interactive Conflict Resolution</span>
              </div>
              <div class="pane-content result-content">
                <!-- Conflict Block Quick Resolvers -->
                <div v-if="activeFileConflicts.length > 0" class="conflict-resolvers-bar">
                  <div 
                    v-for="(_, cIdx) in activeFileConflicts" 
                    :key="cIdx" 
                    class="conflict-card-resolver"
                  >
                    <div class="conflict-card-header">
                      <span class="conflict-num">Conflict #{{ cIdx + 1 }}</span>
                    </div>
                    <div class="conflict-card-btns">
                      <button class="cf-btn cf-ours" @click="resolveBlock(cIdx, 'ours')">
                        Accept Ours
                      </button>
                      <button class="cf-btn cf-theirs" @click="resolveBlock(cIdx, 'theirs')">
                        Accept Theirs
                      </button>
                      <button class="cf-btn cf-both" @click="resolveBlock(cIdx, 'both')">
                        Accept Both
                      </button>
                    </div>
                  </div>
                </div>

                <!-- Live Textarea for Final Polish -->
                <div class="merged-editor-container">
                  <pre class="merged-highlight-layer" ref="highlightLayerRef"><code v-html="highlightedMergedText"></code></pre>
                  <textarea 
                    ref="textareaRef"
                    v-model="mergedText" 
                    class="merged-textarea" 
                    placeholder="Merged content with resolved conflicts..."
                    spellcheck="false"
                    @scroll="syncScroll"
                  ></textarea>
                </div>
              </div>
            </div>

            <!-- Right Pane: THEIRS -->
            <div class="grid-pane theirs-pane">
              <div class="pane-header header-theirs">
                <span class="pane-tag">THEIRS (Incoming Change)</span>
                <span class="pane-ref">{{ operationState.target_name || 'Incoming' }}</span>
              </div>
              <div class="pane-content">
                <pre class="code-pre"><code v-html="highlightedTheirs"></code></pre>
              </div>
            </div>
          </div>
        </div>

        <!-- Empty Main state -->
        <div class="conflict-main-area conflict-empty-main" v-else>
          <span>Select a conflicted file on the left to resolve conflicts.</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import CloseIcon from '../../assets/icons/close.svg?component';
import CheckIcon from '../../assets/icons/check.svg?component';
import ShieldAlertIcon from '../../assets/icons/shield-alert.svg?component';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { invokeGit } from '../../composables/useActivityLog';
import { notify } from '../../composables/useToasts';
import { detectLanguage, highlightCodeLine } from '../../utils/codeHighlighter';
import type { ConflictFileInfo, RepoOperationState } from '../../types';

const props = defineProps<{
  project: any;
}>();

const emit = defineEmits<{
  (e: 'operation-updated'): void;
}>();

const isOpen = ref(false);
const isLoading = ref(false);
const isProcessing = ref(false);
const conflictedFiles = ref<ConflictFileInfo[]>([]);
const activeFilePath = ref<string | null>(null);
const mergedText = ref<string>('');
const operationState = ref<RepoOperationState>({
  operation: 'none',
  has_conflicts: false,
  conflicted_files: [],
  current_step: 0,
  total_steps: 0
});

const operationLabel = computed(() => {
  const op = operationState.value.operation;
  if (op === 'merge') return 'Merge';
  if (op === 'rebase') return 'Rebase';
  if (op === 'cherry_pick') return 'Cherry-Pick';
  if (op === 'revert') return 'Revert';
  return 'Operation';
});

const activeFile = computed(() => {
  return conflictedFiles.value.find(f => f.path === activeFilePath.value) || null;
});

const resolvedCount = computed(() => {
  return conflictedFiles.value.filter(f => f.is_resolved).length;
});

const unmergedCount = computed(() => {
  return conflictedFiles.value.filter(f => !f.is_resolved).length;
});

const textareaRef = ref<HTMLTextAreaElement | null>(null);
const highlightLayerRef = ref<HTMLElement | null>(null);

function syncScroll() {
  if (textareaRef.value && highlightLayerRef.value) {
    highlightLayerRef.value.scrollTop = textareaRef.value.scrollTop;
    highlightLayerRef.value.scrollLeft = textareaRef.value.scrollLeft;
  }
}

const activeFileLang = computed(() => {
  return activeFile.value ? detectLanguage(activeFile.value.path) : null;
});

const highlightedOurs = computed(() => {
  if (!activeFile.value || !activeFileLang.value) return '(Empty)';
  return highlightCodeLine(activeFile.value.ours_content || '(Empty)', activeFileLang.value);
});

const highlightedTheirs = computed(() => {
  if (!activeFile.value || !activeFileLang.value) return '(Empty)';
  return highlightCodeLine(activeFile.value.theirs_content || '(Empty)', activeFileLang.value);
});

const highlightedMergedText = computed(() => {
  if (!mergedText.value || !activeFileLang.value) return '';
  return highlightCodeLine(mergedText.value, activeFileLang.value);
});

interface ConflictBlock {
  fullMatch: string;
  ours: string;
  theirs: string;
  startIndex: number;
}

const activeFileConflicts = computed<ConflictBlock[]>(() => {
  if (!mergedText.value) return [];
  // Supports both standard merge (ours/theirs) and diff3 (ours/base/theirs) styles, and CRLF line endings
  const regex = /<<<<<<<[^\n]*\r?\n(?:([\s\S]*?)\|\|\|\|\|\|\|[\s\S]*?=======\r?\n|([\s\S]*?)=======\r?\n)([\s\S]*?)>>>>>>>[^\n]*/g;
  const blocks: ConflictBlock[] = [];
  let match;
  while ((match = regex.exec(mergedText.value)) !== null) {
    blocks.push({
      fullMatch: match[0],
      ours: (match[1] !== undefined ? match[1] : match[2]) || '',
      theirs: match[3] || '',
      startIndex: match.index
    });
  }
  return blocks;
});

async function open(defaultFilePath?: string) {
  isOpen.value = true;
  await loadConflicts(defaultFilePath);
}

function close() {
  isOpen.value = false;
}

async function loadConflicts(selectPath?: string) {
  if (!props.project?.path) return;
  isLoading.value = true;
  try {
    const [files, opState] = await Promise.all([
      invoke('get_conflicted_files', { path: props.project.path }) as Promise<ConflictFileInfo[]>,
      invoke('get_repository_operation_state', { path: props.project.path }) as Promise<RepoOperationState>
    ]);

    conflictedFiles.value = (files || []).map(f => ({
      ...f,
      is_resolved: !f.working_content.includes('<<<<<<<')
    }));
    operationState.value = opState;

    if (selectPath && conflictedFiles.value.some(f => f.path === selectPath)) {
      selectFile(selectPath);
    } else if (conflictedFiles.value.length > 0) {
      selectFile(conflictedFiles.value[0].path);
    } else {
      activeFilePath.value = null;
      mergedText.value = '';
    }
  } catch (err: any) {
    console.error('Failed to load conflicts:', err);
    notify(`Failed to inspect merge conflicts: ${err}`, 'error');
  } finally {
    isLoading.value = false;
  }
}

function selectFile(path: string) {
  activeFilePath.value = path;
  const f = conflictedFiles.value.find(c => c.path === path);
  if (f) {
    mergedText.value = f.working_content;
  }
}

function resolveBlock(blockIndex: number, choice: 'ours' | 'theirs' | 'both') {
  const block = activeFileConflicts.value[blockIndex];
  if (!block) return;

  let replacement = '';
  if (choice === 'ours') {
    replacement = block.ours;
  } else if (choice === 'theirs') {
    replacement = block.theirs;
  } else if (choice === 'both') {
    replacement = block.ours + block.theirs;
  }

  // Use a replacer function to avoid `$1` or `$&` being parsed as regex substitution patterns
  mergedText.value = mergedText.value.replace(block.fullMatch, () => replacement);
}

function acceptAllOurs() {
  if (!activeFile.value) return;
  const regex = /<<<<<<<[^\n]*\r?\n(?:([\s\S]*?)\|\|\|\|\|\|\|[\s\S]*?=======\r?\n|([\s\S]*?)=======\r?\n)([\s\S]*?)>>>>>>>[^\n]*/g;
  mergedText.value = mergedText.value.replace(regex, (_match, d1, d2) => {
    return (d1 !== undefined ? d1 : d2) || '';
  });
}

function acceptAllTheirs() {
  if (!activeFile.value) return;
  const regex = /<<<<<<<[^\n]*\r?\n(?:([\s\S]*?)\|\|\|\|\|\|\|[\s\S]*?=======\r?\n|([\s\S]*?)=======\r?\n)([\s\S]*?)>>>>>>>[^\n]*/g;
  mergedText.value = mergedText.value.replace(regex, (_match, _d1, _d2, d3) => {
    return d3 || '';
  });
}

async function saveAndMarkResolved() {
  if (!props.project?.path || !activeFile.value) return;

  if (mergedText.value.includes('<<<<<<<') || mergedText.value.includes('>>>>>>>')) {
    if (!window.confirm('There are still unresolved conflict markers (<<<<<<< or >>>>>>>) in the merged text. Mark as resolved anyway?')) {
      return;
    }
  }

  isProcessing.value = true;
  try {
    await invokeGit('resolve_conflict_file', {
      path: props.project.path,
      filePath: activeFile.value.path,
      resolvedContent: mergedText.value
    }, `git add ${activeFile.value.path} (resolved)`);

    activeFile.value.is_resolved = true;
    activeFile.value.working_content = mergedText.value;
    notify(`Marked "${activeFile.value.path}" as resolved!`, 'success');

    // Auto-advance to next unresolved file if available
    const nextUnresolved = conflictedFiles.value.find(f => !f.is_resolved);
    if (nextUnresolved) {
      selectFile(nextUnresolved.path);
    }

    emit('operation-updated');
  } catch (err: any) {
    notify(`Failed to mark resolved: ${err}`, 'error');
  } finally {
    isProcessing.value = false;
  }
}

async function handleAbortOperation() {
  if (!props.project?.path) return;
  if (!window.confirm(`Are you sure you want to abort this ${operationLabel.value.toLowerCase()}? All in-progress merge/rebase changes will be reverted.`)) {
    return;
  }

  isProcessing.value = true;
  try {
    await invokeGit('abort_repository_operation', {
      path: props.project.path,
      opType: operationState.value.operation
    }, `git ${operationState.value.operation} --abort`);

    notify(`${operationLabel.value} aborted successfully`, 'info');
    close();
    emit('operation-updated');
  } catch (err: any) {
    notify(`Failed to abort ${operationLabel.value}: ${err}`, 'error');
  } finally {
    isProcessing.value = false;
  }
}

async function handleContinueOperation() {
  if (!props.project?.path) return;

  isProcessing.value = true;
  try {
    await invokeGit('continue_repository_operation', {
      path: props.project.path,
      opType: operationState.value.operation
    }, `git ${operationState.value.operation} --continue`);

    notify(`${operationLabel.value} continued and completed!`, 'success');
    close();
    emit('operation-updated');
  } catch (err: any) {
    notify(`Continue failed: ${err}`, 'error');
    emit('operation-updated');
  } finally {
    isProcessing.value = false;
  }
}

defineExpose({ open });
</script>

<style scoped>
.conflict-modal-card {
  width: 96vw;
  max-width: 1400px;
  height: 90vh;
  display: flex;
  flex-direction: column;
}

.conflict-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 16px;
}

.conflict-title {
  display: flex;
  align-items: center;
  gap: 10px;
}

.op-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 2px 8px;
  border-radius: 4px;
  letter-spacing: 0.5px;
}

.op-badge.op-merge {
  color: #2196f3;
  background: rgba(33, 150, 243, 0.15);
  border: 1px solid rgba(33, 150, 243, 0.3);
}

.op-badge.op-rebase {
  color: #ff9800;
  background: rgba(255, 152, 0, 0.15);
  border: 1px solid rgba(255, 152, 0, 0.3);
}

.op-badge.op-cherry_pick {
  color: #9c27b0;
  background: rgba(156, 39, 176, 0.15);
  border: 1px solid rgba(156, 39, 176, 0.3);
}

.conflict-counter {
  font-size: 12px;
  color: var(--text-muted, #8b949e);
  font-weight: 600;
}

.conflict-header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.btn-abort-op {
  background: rgba(244, 67, 54, 0.12);
  border: 1px solid rgba(244, 67, 54, 0.4);
  color: #f44336;
  font-size: 11px;
  font-weight: 600;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
}

.btn-continue-op {
  background: #4caf50;
  border: 1px solid #4caf50;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  padding: 4px 12px;
  border-radius: 4px;
  cursor: pointer;
}

.btn-continue-op:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.conflict-body {
  display: flex;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.conflict-sidebar {
  width: 240px;
  border-right: 1px solid var(--border);
  background: var(--panel-bg);
  display: flex;
  flex-direction: column;
}

.sidebar-title {
  padding: 10px 12px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-muted);
  border-bottom: 1px solid var(--border);
}

.conflict-file-list {
  overflow-y: auto;
  flex: 1;
}

.conflict-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  font-size: 12px;
  color: var(--text-main);
  transition: background 0.15s ease;
}

.conflict-file-item:hover {
  background: var(--surface-hover);
}

.conflict-file-item.active {
  background: rgba(0, 120, 212, 0.12);
  border-left: 3px solid var(--accent);
}

.conflict-file-item.resolved {
  opacity: 0.8;
}

.status-indicator {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  font-size: 10px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.indicator-conflicted {
  background: rgba(244, 67, 54, 0.2);
  color: var(--danger-text);
}

.indicator-resolved {
  background: rgba(76, 175, 80, 0.2);
  color: var(--success-text);
}

.file-path-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.conflict-main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.active-file-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 14px;
  background: var(--panel-bg);
  border-bottom: 1px solid var(--border);
}

.file-name-heading {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-main);
  margin-right: 8px;
}

.conflict-hunk-count {
  font-size: 11px;
  color: var(--warning-text);
  font-weight: 600;
}

.file-actions-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-action-btn {
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  color: var(--text-main);
  padding: 4px 10px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  transition: all 0.15s ease;
}

.file-action-btn:hover:not(:disabled) {
  border-color: var(--accent);
  background: var(--surface-hover);
}

.btn-mark-resolved {
  background: rgba(76, 175, 80, 0.15);
  border-color: rgba(76, 175, 80, 0.4);
  color: var(--success-text);
}

.three-way-grid {
  flex: 1;
  display: grid;
  grid-template-columns: 1fr 1.3fr 1fr;
  min-height: 0;
  overflow: hidden;
}

.grid-pane {
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  min-height: 0;
  background: var(--bg-main);
}

.grid-pane:last-child {
  border-right: none;
}

.pane-header {
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 700;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border);
}

.header-ours {
  background: rgba(33, 150, 243, 0.1);
  color: #2196f3;
}

.header-result {
  background: rgba(0, 188, 212, 0.12);
  color: var(--accent);
}

.header-theirs {
  background: rgba(156, 39, 176, 0.1);
  color: #9c27b0;
}

.pane-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px;
}

.code-pre {
  margin: 0;
  font-family: monospace;
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-main);
  white-space: pre-wrap;
  word-break: break-all;
}

.result-content {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 8px;
}

.conflict-resolvers-bar {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.conflict-card-resolver {
  background: var(--panel-bg);
  border: 1px solid rgba(255, 152, 0, 0.3);
  border-radius: 6px;
  padding: 6px 10px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.conflict-num {
  font-size: 11px;
  font-weight: 700;
  color: var(--warning-text);
}

.conflict-card-btns {
  display: flex;
  gap: 6px;
}

.cf-btn {
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 700;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid transparent;
}

.cf-ours {
  background: rgba(33, 150, 243, 0.15);
  border-color: rgba(33, 150, 243, 0.4);
  color: #2196f3;
}

.cf-theirs {
  background: rgba(156, 39, 176, 0.15);
  border-color: rgba(156, 39, 176, 0.4);
  color: #9c27b0;
}

.cf-both {
  background: rgba(76, 175, 80, 0.15);
  border-color: rgba(76, 175, 80, 0.4);
  color: var(--success-text);
}

.merged-editor-container {
  position: relative;
  flex: 1;
  width: 100%;
  min-height: 300px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

.merged-editor-container:focus-within {
  border-color: var(--accent);
}

.merged-highlight-layer {
  margin: 0;
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 8px;
  pointer-events: none;
  font-family: monospace;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-main);
  white-space: pre-wrap;
  word-break: break-all;
  overflow: hidden;
  z-index: 1;
}

.merged-textarea {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  width: 100%;
  height: 100%;
  padding: 8px;
  background: transparent;
  color: transparent;
  caret-color: var(--text-main);
  border: none;
  outline: none;
  resize: none;
  font-family: monospace;
  font-size: 12px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
  z-index: 2;
}

.merged-textarea:focus {
  border-color: var(--accent);
}

.conflict-empty-main, .conflict-loading, .conflict-none {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  height: 100%;
  color: var(--text-muted);
  font-size: 13px;
  padding: 30px;
}
</style>
