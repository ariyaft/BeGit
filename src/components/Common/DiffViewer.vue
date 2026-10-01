<script setup lang="ts">
import DiffIcon from '../../assets/icons/diff.svg?component';
import CloseIcon from '../../assets/icons/close.svg?component';
import PlusIcon from '../../assets/icons/plus.svg?component';
import MinusIcon from '../../assets/icons/minus.svg?component';
import { computed, ref, reactive } from 'vue';
import { parseDiffHunks, buildHunkPatch, type DiffHunkItem } from '../../utils/gitHelpers';
import { detectLanguage, highlightCodeLine, escapeHtml } from '../../utils/codeHighlighter';
import { invokeGit } from '../../composables/useActivityLog';
import { notify } from '../../composables/useToasts';

const props = withDefaults(defineProps<{
  filePath: string;
  fileDiff: string;
  projectPath?: string;
  isStaged?: boolean;
  canStageHunks?: boolean;
}>(), {
  canStageHunks: true,
  isStaged: false
});

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'patch-applied'): void;
}>();

const isSplitView = ref(false);
const languageInfo = computed(() => detectLanguage(props.filePath));

const parsedData = computed(() => {
  return parseDiffHunks(props.fileDiff || '');
});

const hunks = computed(() => parsedData.value.hunks);

const hunksWithSplitRows = computed(() => {
  return hunks.value.map(hunk => {
    const splitRows: { leftIdx: number, rightIdx: number }[] = [];
    let leftPending: number[] = [];
    let rightPending: number[] = [];

    const flush = () => {
      const max = Math.max(leftPending.length, rightPending.length);
      for (let i = 0; i < max; i++) {
        splitRows.push({
          leftIdx: i < leftPending.length ? leftPending[i] : -1,
          rightIdx: i < rightPending.length ? rightPending[i] : -1
        });
      }
      leftPending = [];
      rightPending = [];
    };

    for (let i = 0; i < hunk.lines.length; i++) {
      const line = hunk.lines[i];
      if (line.type === 'context') {
        flush();
        splitRows.push({ leftIdx: i, rightIdx: i });
      } else if (line.type === 'sub') {
        leftPending.push(i);
      } else if (line.type === 'add') {
        rightPending.push(i);
      }
    }
    flush();
    return { ...hunk, splitRows };
  });
});

// Selected line indices per hunk ID: Map<hunkId, Set<lineIndex>>
const selectedLinesByHunk = reactive<Record<string, Set<number>>>({});

function toggleLineSelection(hunkId: string, lineIdx: number) {
  if (!selectedLinesByHunk[hunkId]) {
    selectedLinesByHunk[hunkId] = new Set<number>();
  }
  const set = selectedLinesByHunk[hunkId];
  if (set.has(lineIdx)) {
    set.delete(lineIdx);
  } else {
    set.add(lineIdx);
  }
}

function clearHunkSelection(hunkId: string) {
  if (selectedLinesByHunk[hunkId]) {
    selectedLinesByHunk[hunkId].clear();
  }
}

function isLineSelected(hunkId: string, lineIdx: number) {
  return selectedLinesByHunk[hunkId]?.has(lineIdx) || false;
}

function getSelectedCount(hunkId: string) {
  return selectedLinesByHunk[hunkId]?.size || 0;
}

const isApplying = ref(false);

async function handleStageHunk(hunk: DiffHunkItem) {
  if (!props.projectPath) return;
  isApplying.value = true;
  try {
    const patch = buildHunkPatch(props.filePath, hunk);
    await invokeGit('apply_patch', {
      path: props.projectPath,
      patchContent: patch,
      cached: true,
      reverse: false
    }, `git apply --cached hunk (${props.filePath})`);
    notify('Staged hunk successfully', 'success');
    emit('patch-applied');
  } catch (err: any) {
    notify(`Failed to stage hunk: ${err}`, 'error');
  } finally {
    isApplying.value = false;
  }
}

async function handleUnstageHunk(hunk: DiffHunkItem) {
  if (!props.projectPath) return;
  isApplying.value = true;
  try {
    const patch = buildHunkPatch(props.filePath, hunk);
    await invokeGit('apply_patch', {
      path: props.projectPath,
      patchContent: patch,
      cached: true,
      reverse: true
    }, `git apply --cached --reverse hunk (${props.filePath})`);
    notify('Unstaged hunk successfully', 'success');
    emit('patch-applied');
  } catch (err: any) {
    notify(`Failed to unstage hunk: ${err}`, 'error');
  } finally {
    isApplying.value = false;
  }
}

async function handleDiscardHunk(hunk: DiffHunkItem) {
  if (!props.projectPath) return;
  if (!window.confirm('Are you sure you want to discard changes in this hunk? This cannot be undone.')) {
    return;
  }
  isApplying.value = true;
  try {
    const patch = buildHunkPatch(props.filePath, hunk);
    await invokeGit('apply_patch', {
      path: props.projectPath,
      patchContent: patch,
      cached: false,
      reverse: true
    }, `git apply --reverse hunk (${props.filePath})`);
    notify('Discarded hunk changes', 'info');
    emit('patch-applied');
  } catch (err: any) {
    notify(`Failed to discard hunk: ${err}`, 'error');
  } finally {
    isApplying.value = false;
  }
}

async function handleStageSelectedLines(hunk: DiffHunkItem) {
  if (!props.projectPath) return;
  const selected = selectedLinesByHunk[hunk.id];
  if (!selected || selected.size === 0) return;

  isApplying.value = true;
  try {
    const patch = buildHunkPatch(props.filePath, hunk, selected);
    await invokeGit('apply_patch', {
      path: props.projectPath,
      patchContent: patch,
      cached: true,
      reverse: false
    }, `git apply --cached ${selected.size} lines (${props.filePath})`);
    notify(`Staged ${selected.size} selected line(s)`, 'success');
    clearHunkSelection(hunk.id);
    emit('patch-applied');
  } catch (err: any) {
    notify(`Failed to stage selected lines: ${err}`, 'error');
  } finally {
    isApplying.value = false;
  }
}

async function handleUnstageSelectedLines(hunk: DiffHunkItem) {
  if (!props.projectPath) return;
  const selected = selectedLinesByHunk[hunk.id];
  if (!selected || selected.size === 0) return;

  isApplying.value = true;
  try {
    const patch = buildHunkPatch(props.filePath, hunk, selected);
    await invokeGit('apply_patch', {
      path: props.projectPath,
      patchContent: patch,
      cached: true,
      reverse: true
    }, `git apply --cached --reverse ${selected.size} lines (${props.filePath})`);
    notify(`Unstaged ${selected.size} selected line(s)`, 'success');
    clearHunkSelection(hunk.id);
    emit('patch-applied');
  } catch (err: any) {
    notify(`Failed to unstage selected lines: ${err}`, 'error');
  } finally {
    isApplying.value = false;
  }
}

async function handleDiscardSelectedLines(hunk: DiffHunkItem) {
  if (!props.projectPath) return;
  const selected = selectedLinesByHunk[hunk.id];
  if (!selected || selected.size === 0) return;

  if (!window.confirm(`Discard ${selected.size} selected line(s)? This cannot be undone.`)) {
    return;
  }

  isApplying.value = true;
  try {
    const patch = buildHunkPatch(props.filePath, hunk, selected);
    await invokeGit('apply_patch', {
      path: props.projectPath,
      patchContent: patch,
      cached: false,
      reverse: true
    }, `git apply --reverse ${selected.size} lines (${props.filePath})`);
    notify(`Discarded ${selected.size} selected line(s)`, 'info');
    clearHunkSelection(hunk.id);
    emit('patch-applied');
  } catch (err: any) {
    notify(`Failed to discard lines: ${err}`, 'error');
  } finally {
    isApplying.value = false;
  }
}

function stripPrefix(content: string) {
  if (content.length > 0 && (content.startsWith('+') || content.startsWith('-') || content.startsWith(' '))) {
    return content.substring(1);
  }
  return content;
}

function renderDiffLine(content: string, type: string) {
  if (type === 'meta' || type === 'header') {
    return escapeHtml(content);
  }
  const prefix = content.startsWith('+') ? '+' : content.startsWith('-') ? '-' : ' ';
  const code = stripPrefix(content);
  return escapeHtml(prefix) + highlightCodeLine(code, languageInfo.value);
}
</script>

<template>
  <div class="diff-viewer">
    <div class="diff-header">
      <div class="diff-title">
        <DiffIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
        <span class="file-path-text">{{ filePath }}</span>
        <span class="diff-lang-badge" v-if="languageInfo && languageInfo.id !== 'plaintext'">
          {{ languageInfo.name }}
        </span>
        <span v-if="projectPath" class="stage-state-pill" :class="isStaged ? 'pill-staged' : 'pill-unstaged'">
          {{ isStaged ? 'Staged Diff' : 'Unstaged Diff' }}
        </span>
      </div>
      <div class="diff-actions">
        <button class="view-toggle" @click="isSplitView = !isSplitView">
          {{ isSplitView ? 'Unified View' : 'Split View' }}
        </button>
        <button class="close-btn" @click="emit('close')">
          <CloseIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
        </button>
      </div>
    </div>
    <div class="diff-body">
      <div v-if="!fileDiff?.includes('@@')" class="diff-loading">
        {{ fileDiff || 'Loading diff...' }}
      </div>
      <div v-else class="diff-scroll-container">
        
        <!-- File headers if any -->
        <div v-if="parsedData.fileHeaders.length > 0" class="file-diff-headers-box">
          <div v-for="(fh, fIdx) in parsedData.fileHeaders" :key="fIdx" class="file-header-line">
            {{ fh }}
          </div>
        </div>

        <!-- Hunks Container -->
        <div v-for="hunk in hunksWithSplitRows" :key="hunk.id" class="diff-hunk-card">
          <!-- Hunk Control Header -->
          <div class="hunk-header-bar">
            <div class="hunk-meta-info">
              <span class="hunk-badge">{{ hunk.header }}</span>
            </div>

            <!-- Hunk Action Buttons -->
            <div class="hunk-actions" v-if="projectPath && canStageHunks">
              <!-- If lines are selected in this hunk -->
              <template v-if="getSelectedCount(hunk.id) > 0">
                <span class="selected-lines-count">{{ getSelectedCount(hunk.id) }} line(s) selected</span>
                <button
                  v-if="!isStaged"
                  class="hunk-action-btn btn-stage-selected"
                  :disabled="isApplying"
                  @click="handleStageSelectedLines(hunk)"
                  title="Stage only selected lines"
                >
                  <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
                  Stage Lines
                </button>
                <button
                  v-if="isStaged"
                  class="hunk-action-btn btn-unstage-selected"
                  :disabled="isApplying"
                  @click="handleUnstageSelectedLines(hunk)"
                  title="Unstage only selected lines"
                >
                  <MinusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
                  Unstage Lines
                </button>
                <button
                  v-if="!isStaged"
                  class="hunk-action-btn btn-discard-selected"
                  :disabled="isApplying"
                  @click="handleDiscardSelectedLines(hunk)"
                  title="Discard selected lines"
                >
                  Discard Lines
                </button>
                <button class="hunk-action-btn btn-clear-sel" @click="clearHunkSelection(hunk.id)" title="Clear line selection">
                  Clear
                </button>
              </template>

              <!-- Whole Hunk Actions -->
              <template v-else>
                <button
                  v-if="!isStaged"
                  class="hunk-action-btn btn-stage-hunk"
                  :disabled="isApplying"
                  @click="handleStageHunk(hunk)"
                  title="Stage this entire hunk"
                >
                  <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
                  Stage Hunk
                </button>
                <button
                  v-if="isStaged"
                  class="hunk-action-btn btn-unstage-hunk"
                  :disabled="isApplying"
                  @click="handleUnstageHunk(hunk)"
                  title="Unstage this entire hunk"
                >
                  <MinusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
                  Unstage Hunk
                </button>
                <button
                  v-if="!isStaged"
                  class="hunk-action-btn btn-discard-hunk"
                  :disabled="isApplying"
                  @click="handleDiscardHunk(hunk)"
                  title="Discard changes in this hunk"
                >
                  Discard Hunk
                </button>
              </template>
            </div>
          </div>

          <!-- Lines list -->
          <div class="hunk-lines-container" :class="{ 'is-split-view': isSplitView }">
            <template v-if="!isSplitView">
              <div
                v-for="(line, lineIdx) in hunk.lines"
                :key="lineIdx"
                class="diff-line-row"
                :class="[
                  `diff-row-${line.type}`,
                  {
                    'line-selected': isLineSelected(hunk.id, lineIdx),
                    'line-selectable': (line.type === 'add' || line.type === 'sub') && projectPath
                  }
                ]"
                @click="(line.type === 'add' || line.type === 'sub') && projectPath ? toggleLineSelection(hunk.id, lineIdx) : null"
              >
                <!-- Line select checkbox / indicator -->
                <div v-if="projectPath && (line.type === 'add' || line.type === 'sub')" class="diff-line-select">
                  <input
                    type="checkbox"
                    :checked="isLineSelected(hunk.id, lineIdx)"
                    @click.stop="toggleLineSelection(hunk.id, lineIdx)"
                    class="line-checkbox"
                  />
                </div>
                <div v-else-if="projectPath" class="diff-line-select empty"></div>

                <div class="diff-num old-num">{{ line.oldNum }}</div>
                <div class="diff-num new-num">{{ line.newNum }}</div>
                <div class="diff-line-content" v-html="renderDiffLine(line.content, line.type)"></div>
              </div>
            </template>
            <template v-else>
              <div
                v-for="(row, rowIdx) in hunk.splitRows"
                :key="'split' + rowIdx"
                class="diff-split-row"
              >
                <!-- LEFT PANE -->
                <div
                  class="split-pane left-pane"
                  :class="[ 
                    row.leftIdx !== -1 ? `diff-row-${hunk.lines[row.leftIdx].type}` : 'diff-row-empty',
                    {
                      'line-selected': row.leftIdx !== -1 && isLineSelected(hunk.id, row.leftIdx),
                      'line-selectable': row.leftIdx !== -1 && (hunk.lines[row.leftIdx].type === 'sub') && projectPath
                    }
                  ]"
                  @click="row.leftIdx !== -1 && (hunk.lines[row.leftIdx].type === 'sub') && projectPath ? toggleLineSelection(hunk.id, row.leftIdx) : null"
                >
                  <template v-if="row.leftIdx !== -1">
                    <div v-if="projectPath && (hunk.lines[row.leftIdx].type === 'sub')" class="diff-line-select">
                      <input
                        type="checkbox"
                        :checked="isLineSelected(hunk.id, row.leftIdx)"
                        @click.stop="toggleLineSelection(hunk.id, row.leftIdx)"
                        class="line-checkbox"
                      />
                    </div>
                    <div v-else-if="projectPath" class="diff-line-select empty"></div>
                    <div class="diff-num old-num">{{ hunk.lines[row.leftIdx].oldNum }}</div>
                    <div class="diff-num new-num"></div>
                    <div class="diff-line-content" v-html="renderDiffLine(hunk.lines[row.leftIdx].content, hunk.lines[row.leftIdx].type)"></div>
                  </template>
                </div>

                <!-- RIGHT PANE -->
                <div
                  class="split-pane right-pane"
                  :class="[ 
                    row.rightIdx !== -1 ? `diff-row-${hunk.lines[row.rightIdx].type}` : 'diff-row-empty',
                    {
                      'line-selected': row.rightIdx !== -1 && isLineSelected(hunk.id, row.rightIdx),
                      'line-selectable': row.rightIdx !== -1 && (hunk.lines[row.rightIdx].type === 'add') && projectPath
                    }
                  ]"
                  @click="row.rightIdx !== -1 && (hunk.lines[row.rightIdx].type === 'add') && projectPath ? toggleLineSelection(hunk.id, row.rightIdx) : null"
                >
                  <template v-if="row.rightIdx !== -1">
                    <div v-if="projectPath && (hunk.lines[row.rightIdx].type === 'add')" class="diff-line-select">
                      <input
                        type="checkbox"
                        :checked="isLineSelected(hunk.id, row.rightIdx)"
                        @click.stop="toggleLineSelection(hunk.id, row.rightIdx)"
                        class="line-checkbox"
                      />
                    </div>
                    <div v-else-if="projectPath" class="diff-line-select empty"></div>
                    <div class="diff-num old-num"></div>
                    <div class="diff-num new-num">{{ hunk.lines[row.rightIdx].newNum }}</div>
                    <div class="diff-line-content" v-html="renderDiffLine(hunk.lines[row.rightIdx].content, hunk.lines[row.rightIdx].type)"></div>
                  </template>
                </div>
              </div>
            </template>
          </div>
        </div>
        
      </div>
    </div>
  </div>
</template>

<style scoped>
.diff-scroll-container {
  padding: 12px;
}

.diff-lang-badge {
  font-size: 0.68rem;
  padding: 2px 6px;
  border-radius: 4px;
  background: var(--surface-subtle);
  color: var(--accent);
  border: 1px solid var(--border);
  font-weight: 600;
  margin-left: 6px;
}

.stage-state-pill {
  font-size: 0.65rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 2px 8px;
  border-radius: 10px;
  margin-left: 8px;
}

.stage-state-pill.pill-staged {
  color: var(--success-text);
  background: rgba(76, 175, 80, 0.12);
  border: 1px solid rgba(76, 175, 80, 0.3);
}

.stage-state-pill.pill-unstaged {
  color: var(--warning-text);
  background: rgba(255, 152, 0, 0.12);
  border: 1px solid rgba(255, 152, 0, 0.3);
}

.file-diff-headers-box {
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  border-radius: 6px;
  margin-bottom: 12px;
  padding: 6px 12px;
  font-family: monospace;
  font-size: 11px;
  color: var(--text-muted);
}

.file-header-line {
  line-height: 1.5;
}

.diff-hunk-card {
  margin-bottom: 14px;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
  background: var(--bg-main);
  box-shadow: 0 1px 3px var(--shadow-color);
}

.hunk-header-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: var(--panel-bg);
  padding: 6px 12px;
  border-bottom: 1px solid var(--border);
  gap: 8px;
}

.hunk-badge {
  font-family: monospace;
  font-size: 11px;
  color: var(--accent);
  font-weight: 600;
  background: var(--surface-subtle);
  padding: 2px 6px;
  border-radius: 4px;
  border: 1px solid var(--border);
}

.hunk-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.selected-lines-count {
  font-size: 11px;
  color: var(--accent);
  font-weight: 600;
  margin-right: 4px;
}

.hunk-action-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 8px;
  font-size: 11px;
  font-weight: 600;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid var(--border);
  transition: all 0.15s ease;
  background: var(--surface-subtle);
  color: var(--text-main);
}

.hunk-action-btn:hover:not(:disabled) {
  background: var(--surface-hover);
  transform: translateY(-1px);
}

.hunk-action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-stage-hunk, .btn-stage-selected {
  background: rgba(76, 175, 80, 0.12);
  border-color: rgba(76, 175, 80, 0.35);
  color: var(--success-text);
}

.btn-stage-hunk:hover:not(:disabled), .btn-stage-selected:hover:not(:disabled) {
  background: rgba(76, 175, 80, 0.22);
  border-color: rgba(76, 175, 80, 0.5);
}

.btn-unstage-hunk, .btn-unstage-selected {
  background: rgba(255, 152, 0, 0.12);
  border-color: rgba(255, 152, 0, 0.35);
  color: var(--warning-text);
}

.btn-unstage-hunk:hover:not(:disabled), .btn-unstage-selected:hover:not(:disabled) {
  background: rgba(255, 152, 0, 0.22);
  border-color: rgba(255, 152, 0, 0.5);
}

.btn-discard-hunk, .btn-discard-selected {
  background: rgba(244, 67, 54, 0.1);
  border-color: rgba(244, 67, 54, 0.3);
  color: var(--danger-text);
}

.btn-discard-hunk:hover:not(:disabled), .btn-discard-selected:hover:not(:disabled) {
  background: rgba(244, 67, 54, 0.2);
  border-color: rgba(244, 67, 54, 0.45);
}

.btn-clear-sel {
  background: var(--surface-subtle);
  border-color: var(--border);
  color: var(--text-muted);
}

.btn-clear-sel:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

.diff-line-select {
  width: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.diff-line-select.empty {
  visibility: hidden;
}

.line-checkbox {
  cursor: pointer;
  accent-color: var(--accent);
}

.line-selectable {
  cursor: pointer;
}

.line-selectable:hover {
  background-color: var(--surface-hover);
}

.line-selected {
  background-color: rgba(0, 120, 212, 0.15) !important;
  outline: 1px solid rgba(0, 120, 212, 0.35);
}

.is-split-view {
  display: flex;
  flex-direction: column;
}

.diff-split-row {
  display: flex;
  width: 100%;
}

.split-pane {
  flex: 1 1 50%;
  min-width: 0;
  display: flex;
  align-items: stretch;
  border-right: 1px solid var(--border);
}

.split-pane.right-pane {
  border-right: none;
}

.diff-row-empty {
  background: var(--surface-subtle);
}
</style>
