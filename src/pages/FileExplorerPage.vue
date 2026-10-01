<template>
  <section class="file-explorer-page" aria-label="File Explorer">
    <!-- Resizable Left Sidebar -->
    <aside class="left-sidebar" :style="{ width: sidebarWidth + 'px' }">
      <div class="sidebar-header">
        <h3 style="margin: 0; font-size: 11px; font-weight: 700; color: var(--text-muted); letter-spacing: 0.5px; text-transform: uppercase;">Repository Files</h3>
      </div>
      <div class="file-search">
        <SearchIcon v-if="!fileSearch" class="file-search-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        <input v-model="fileSearch" type="search" placeholder="Search files or *.svg" aria-label="Search repository files" />
        <button v-if="fileSearch" class="clear-file-search" type="button" aria-label="Clear file search" @click="fileSearch = ''">×</button>
      </div>
      <div class="sidebar-scrollable tree-container">
        <div v-if="loadingFiles" style="padding: 20px; text-align: center; color: var(--text-muted); font-size: 13px;">
          Loading files...
        </div>
        <FileTreeNode
          v-else
          v-for="node in fileTree"
          :key="node.path"
          :node="node"
          :depth="0"
          :selected-path="selectedFilePath"
          :force-expanded="Boolean(fileSearch.trim())"
          @select="selectFile"
          @contextmenu="openFileContextMenu"
        />
        <div v-if="!loadingFiles && fileTree.length === 0" class="no-file-results">
          No matching files
        </div>
      </div>
    </aside>

    <div class="resizer" @mousedown.stop.prevent="startDrag($event)"></div>

    <!-- Main Content -->
    <main class="main-content">
      <div v-if="!selectedFilePath" class="empty-state">
        <FolderIcon class="empty-icon" viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" stroke-width="1.5" fill="none" />
        <p>Select a file to view its history</p>
      </div>
      
      <div v-else class="file-history-view">
        <header class="history-header">
          <div class="file-header-top">
            <div class="file-title-bar">
              <FileIcon viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" class="title-icon" />
              <h2 class="file-title">{{ selectedFilePath }}</h2>
            </div>
            <div class="file-header-actions">
              <div class="file-view-mode-tabs">
                <button class="view-mode-tab" :class="{ active: activeFileTab === 'history' }" @click="activeFileTab = 'history'">
                  <TimelineIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
                  History <span class="tab-badge" v-if="commits.length">{{ commits.length }}</span>
                </button>
                <button class="view-mode-tab" :class="{ active: activeFileTab === 'blame' }" @click="openBlameTab">
                  <ActionIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
                  Git Blame
                </button>
              </div>
              <button
                type="button"
                class="purge-file-header-btn"
                title="Purge this file from entire Git history"
                aria-label="Purge file from history"
                @click="openPurgeModal(selectedFilePath)"
              >
                <TrashIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
                <span>Purge from History</span>
              </button>
            </div>
          </div>
          <div class="file-subtitle-wrapper" v-if="activeFileTab === 'history'">
            <p class="file-subtitle">Showing all commits that modified this file</p>
            <div class="keyboard-hint" v-if="commits.length > 1">
              <kbd>Shift</kbd> + Click to compare
            </div>
          </div>
          <div class="file-subtitle-wrapper" v-else>
            <p class="file-subtitle">Line-by-line authorship &amp; commit details. Click any commit hash or line for details.</p>
          </div>
        </header>

        <GitBlameView
          v-if="activeFileTab === 'blame'"
          :lines="blameLines"
          :loading="loadingBlame"
          :error="blameError"
          :file-path="selectedFilePath"
          :repository-path="repositoryPath"
          @select-commit="$emit('select-commit', $event)"
          @refresh="loadFileBlame"
        />

        <div v-else class="history-content">
          <div v-if="loadingHistory" class="history-loading">
            Loading history...
          </div>
          <div v-else-if="historyError" class="history-error">
            {{ historyError }}
          </div>
          <div v-else class="commits-list timeline-view">
            <div 
              v-for="(commit, index) in commits" 
              :key="commit.hash" 
              class="commit-card"
              :class="{ 
                selected: selectedCommits.some(c => c.hash === commit.hash),
                'is-last': index === commits.length - 1
              }"
              @click="onCommitClick($event, commit)"
            >
              <div class="timeline-connector">
                <div class="timeline-dot"></div>
                <div class="timeline-line" v-if="index !== commits.length - 1"></div>
              </div>
              <div class="commit-card-content">
                <div class="commit-header">
                  <AuthorAvatar :author-name="commit.author" :size="20" />
                  <div class="commit-meta-top">
                    <span class="commit-author">{{ commit.author }}</span>
                    <span class="commit-time">{{ commit.time }}</span>
                  </div>
                  <span class="commit-hash" title="Commit Hash">{{ commit.hash.substring(0, 7) }}</span>
                </div>
                <div class="commit-body">
                  <p class="commit-message" :title="commit.message">{{ commit.message }}</p>
                </div>
              </div>
            </div>
          </div>

          <!-- Diff View -->
          <div class="diff-panel" v-if="selectedCommits.length > 0">
            <div class="diff-header" v-if="loadingDiff || diffError">
              <h3 v-if="!diffError">
                <span v-if="selectedCommits.length === 1">Changes in {{ selectedCommits[0].hash.substring(0, 7) }}</span>
                <span v-else>Comparing {{ selectedCommits[1].hash.substring(0, 7) }} and {{ selectedCommits[0].hash.substring(0, 7) }}</span>
              </h3>
            </div>
            <div class="diff-content" v-if="loadingDiff">Loading diff...</div>
            <div class="diff-content error" v-else-if="diffError">{{ diffError }}</div>
            <ImageDiffViewer
              v-else-if="imageVersions"
              :file-path="selectedFilePath"
              :old-image="imageVersions.old_image"
              :new-image="imageVersions.new_image"
              :is-commit-comparison="selectedCommits.length === 2"
              @close="selectedCommits = []"
            />
            <DiffViewer
              v-else-if="diffContent"
              :file-path="selectedFilePath"
              :file-diff="diffContent"
              @close="selectedCommits = []"
              style="height: 100%; border-top: 1px solid var(--border);"
            />
          </div>
        </div>
      </div>
    </main>

    <!-- Context Menu for File Items -->
    <FileContextMenu
      :visible="contextMenuVisible"
      :x="contextMenuX"
      :y="contextMenuY"
      :file-node="contextMenuNode"
      @action="handleContextMenuAction"
    />

    <!-- Purge File from History Modal -->
    <ConfirmPurgeFileModal
      ref="confirmPurgeModalRef"
      :repository-path="repositoryPath"
      @purged="onFilePurged"
      @pending="$emit('pending', $event)"
    />
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { TimelineCommit, BlameLine } from '../types';

import FileTreeNode from '../components/Common/FileTreeNode.vue';
import DiffViewer from '../components/Common/DiffViewer.vue';
import ImageDiffViewer from '../components/Common/ImageDiffViewer.vue';
import type { FileNode } from '../components/Common/FileTreeNode.vue';
import AuthorAvatar from '../components/Common/AuthorAvatar.vue';
import GitBlameView from '../components/Common/GitBlameView.vue';
import FileContextMenu from '../components/menus/FileContextMenu.vue';
import ConfirmPurgeFileModal from '../components/Common/ConfirmPurgeFileModal.vue';
import { notify } from '../composables/useToasts';

import FolderIcon from '../assets/icons/folder.svg?component';
import FileIcon from '../assets/icons/file.svg?component';
import SearchIcon from '../assets/icons/search.svg?component';
import TimelineIcon from '../assets/icons/timeline.svg?component';
import ActionIcon from '../assets/icons/action.svg?component';
import TrashIcon from '../assets/icons/context-danger-1.svg?component';

const props = defineProps<{
  repositoryName: string;
  repositoryPath: string | null;
  active: boolean;
  initialFilePath?: string | null;
}>();

const emit = defineEmits<{
  (e: 'select-commit', hash: string): void;
  (e: 'pending', message: string): void;
}>();

// --- Sidebar Resizer ---
const sidebarWidth = ref(260);
const isDragging = ref(false);
let startX = 0;
let startWidth = 0;

// --- Context Menu & Purge Modal State ---
const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuNode = ref<FileNode | null>(null);
const confirmPurgeModalRef = ref<InstanceType<typeof ConfirmPurgeFileModal> | null>(null);

function openFileContextMenu(node: FileNode, event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  contextMenuNode.value = node;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
  contextMenuVisible.value = true;
}

function closeContextMenu() {
  contextMenuVisible.value = false;
}

function openPurgeModal(path: string | null) {
  if (!path) return;
  confirmPurgeModalRef.value?.open(path);
}

function handleContextMenuAction(action: 'history' | 'blame' | 'copy_path' | 'copy_name' | 'purge_history') {
  if (!contextMenuNode.value) return;
  const targetNode = contextMenuNode.value;
  closeContextMenu();

  switch (action) {
    case 'history':
      selectFile(targetNode.path);
      activeFileTab.value = 'history';
      break;
    case 'blame':
      selectFile(targetNode.path);
      openBlameTab();
      break;
    case 'copy_path':
      navigator.clipboard.writeText(targetNode.path);
      notify(`Copied path: ${targetNode.path}`, 'success');
      break;
    case 'copy_name':
      navigator.clipboard.writeText(targetNode.name);
      notify(`Copied filename: ${targetNode.name}`, 'success');
      break;
    case 'purge_history':
      openPurgeModal(targetNode.path);
      break;
  }
}

async function onFilePurged(purgedPath: string) {
  if (selectedFilePath.value === purgedPath) {
    selectedFilePath.value = null;
    selectedCommits.value = [];
    commits.value = [];
    diffContent.value = '';
    blameLines.value = [];
  }
  await loadFiles();
}

function getZoomFactor(): number {
  const rootZoom = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--app-zoom'));
  if (!isNaN(rootZoom) && rootZoom > 0) return rootZoom;
  const docZoom = parseFloat((document.documentElement.style as any).zoom);
  if (!isNaN(docZoom) && docZoom > 0) return docZoom / 100;
  return 1;
}

function startDrag(e: MouseEvent) {
  e?.preventDefault?.();
  e?.stopPropagation?.();
  window.getSelection()?.removeAllRanges();
  isDragging.value = true;
  startX = e.clientX;
  startWidth = sidebarWidth.value;
  document.addEventListener('mousemove', onDrag);
  document.addEventListener('mouseup', stopDrag);
  document.body.classList.add('is-resizing');
  document.body.style.cursor = 'col-resize';
  document.body.style.userSelect = 'none';
  document.documentElement.style.userSelect = 'none';
}

function onDrag(e: MouseEvent) {
  if (isDragging.value) {
    const zoom = getZoomFactor();
    const deltaX = (e.clientX - startX) / zoom;
    const newWidth = Math.min(Math.max(160, Math.round(startWidth + deltaX)), 600);
    sidebarWidth.value = newWidth;
  }
}

function stopDrag() {
  isDragging.value = false;
  document.removeEventListener('mousemove', onDrag);
  document.removeEventListener('mouseup', stopDrag);
  document.body.classList.remove('is-resizing');
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
  document.documentElement.style.userSelect = '';
  window.getSelection()?.removeAllRanges();
}

// --- File Tree State ---
const files = ref<string[]>([]);
const loadingFiles = ref(false);
const selectedFilePath = ref<string | null>(null);
const fileSearch = ref('');

function globToRegExp(pattern: string) {
  let expression = '';
  for (let index = 0; index < pattern.length; index += 1) {
    const character = pattern[index];
    if (character === '*' && pattern[index + 1] === '*') {
      if (pattern[index + 2] === '/') {
        expression += '(?:.*/)?';
        index += 2;
      } else {
        expression += '.*';
        index += 1;
      }
    } else if (character === '*') {
      expression += '[^/]*';
    } else if (character === '?') {
      expression += '[^/]';
    } else {
      expression += character.replace(/[|\\{}()[\]^$+*?.]/g, '\\$&');
    }
  }

  // A filename-only glob, such as *.svg, searches every directory.
  return new RegExp(pattern.includes('/') ? `^${expression}$` : `(?:^|.*/)${expression}$`, 'i');
}

const filteredFiles = computed(() => {
  const query = fileSearch.value.trim();
  if (!query) return files.value;

  const glob = /[*?]/.test(query) ? globToRegExp(query) : null;
  const normalizedQuery = query.toLocaleLowerCase();
  const matches = glob
    ? (path: string) => glob.test(path)
    : (path: string) => path.toLocaleLowerCase().includes(normalizedQuery);
  return files.value.filter(matches);
});

const fileTree = computed(() => {
  const root: FileNode[] = [];
  
  for (const path of filteredFiles.value) {
    const parts = path.split('/');
    let currentLevel = root;
    
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      const isFile = i === parts.length - 1;
      const currentPath = parts.slice(0, i + 1).join('/');
      
      let existing = currentLevel.find(n => n.name === part);
      if (!existing) {
        existing = {
          name: part,
          path: currentPath,
          isDir: !isFile,
          children: isFile ? undefined : []
        };
        currentLevel.push(existing);
      }
      
      if (!isFile) {
        currentLevel = existing.children!;
      }
    }
  }
  
  // Sort: directories first, then alphabetically
  const sortTree = (nodes: FileNode[]) => {
    nodes.sort((a, b) => {
      if (a.isDir && !b.isDir) return -1;
      if (!a.isDir && b.isDir) return 1;
      return a.name.localeCompare(b.name);
    });
    for (const node of nodes) {
      if (node.children) sortTree(node.children);
    }
  };
  sortTree(root);
  return root;
});

// --- History State ---
const commits = ref<TimelineCommit[]>([]);
const loadingHistory = ref(false);
const historyError = ref<string | null>(null);

const selectedCommits = ref<TimelineCommit[]>([]);
const diffContent = ref<string>('');
const loadingDiff = ref(false);
const diffError = ref<string | null>(null);
interface ImageVersions {
  old_image: string | null;
  new_image: string | null;
}
const imageVersions = ref<ImageVersions | null>(null);

const isPreviewableImage = computed(() => {
  const path = selectedFilePath.value?.toLowerCase() ?? '';
  return /\.(png|jpe?g|gif|webp|avif|bmp|ico|svg)$/.test(path);
});

// --- Blame & View Mode State ---
const activeFileTab = ref<'history' | 'blame'>('history');
const blameLines = ref<BlameLine[]>([]);
const loadingBlame = ref(false);
const blameError = ref<string | null>(null);

async function openBlameTab() {
  activeFileTab.value = 'blame';
  if (blameLines.value.length === 0 && selectedFilePath.value) {
    await loadFileBlame();
  }
}

async function loadFileBlame() {
  if (!props.repositoryPath || !selectedFilePath.value) return;
  loadingBlame.value = true;
  blameError.value = null;
  try {
    const result = await invoke<BlameLine[]>('get_file_blame', {
      path: props.repositoryPath,
      filePath: selectedFilePath.value,
      commitHash: null,
    });
    blameLines.value = result;
  } catch (e: any) {
    blameError.value = e.toString();
  } finally {
    loadingBlame.value = false;
  }
}

// --- Actions ---
async function loadFiles() {
  if (!props.repositoryPath) return;
  loadingFiles.value = true;
  try {
    const tracked = await invoke<string[]>('get_tracked_files', { path: props.repositoryPath });
    files.value = tracked;
  } catch (e) {
    console.error("Failed to load tracked files:", e);
  } finally {
    loadingFiles.value = false;
  }
}

async function selectFile(path: string) {
  selectedFilePath.value = path;
  selectedCommits.value = [];
  blameLines.value = [];
  if (activeFileTab.value === 'blame') {
    await loadFileBlame();
  } else {
    await loadFileHistory();
  }
}

async function loadFileHistory() {
  if (!props.repositoryPath || !selectedFilePath.value) return;
  
  loadingHistory.value = true;
  historyError.value = null;
  commits.value = [];
  
  try {
    const result = await invoke<TimelineCommit[]>('get_commits', {
      path: props.repositoryPath,
      limit: 1000,
      skip: 0,
      filePath: selectedFilePath.value
    });
    commits.value = result;
    if (result.length > 0) {
      selectedCommits.value = [result[0]];
      await loadDiff();
    }
  } catch (e: any) {
    historyError.value = e.toString();
  } finally {
    loadingHistory.value = false;
  }
}

async function onCommitClick(e: MouseEvent, commit: TimelineCommit) {
  if (e.shiftKey || e.metaKey || e.ctrlKey) {
    window.getSelection()?.removeAllRanges();
    const idx = selectedCommits.value.findIndex(c => c.hash === commit.hash);
    if (idx !== -1) {
      selectedCommits.value.splice(idx, 1);
    } else {
      if (selectedCommits.value.length >= 2) {
        selectedCommits.value.shift();
      }
      selectedCommits.value.push(commit);
    }
  } else {
    selectedCommits.value = [commit];
  }
  
  if (selectedCommits.value.length > 0) {
    await loadDiff();
  } else {
    diffContent.value = '';
  }
}

async function loadDiff() {
  if (!props.repositoryPath || !selectedFilePath.value || selectedCommits.value.length === 0) return;
  
  loadingDiff.value = true;
  diffError.value = null;
  diffContent.value = '';
  imageVersions.value = null;
  
  try {
    if (selectedCommits.value.length === 1) {
      if (isPreviewableImage.value) {
        imageVersions.value = await invoke<ImageVersions>('get_file_image_versions', {
          path: props.repositoryPath,
          newHash: selectedCommits.value[0].hash,
          filePath: selectedFilePath.value
        });
        return;
      }
      const diff = await invoke<string>('get_file_diff', {
        path: props.repositoryPath,
        hash: selectedCommits.value[0].hash,
        filePath: selectedFilePath.value
      });
      diffContent.value = diff;
    } else if (selectedCommits.value.length === 2) {
      // Sort to ensure older commit is hash1, newer commit is hash2
      // Commits are sorted newest first in the `commits` array
      const idx0 = commits.value.findIndex(c => c.hash === selectedCommits.value[0].hash);
      const idx1 = commits.value.findIndex(c => c.hash === selectedCommits.value[1].hash);
      
      let older = selectedCommits.value[0];
      let newer = selectedCommits.value[1];
      
      if (idx0 < idx1) { // idx0 is smaller -> closer to 0 -> newer
        newer = selectedCommits.value[0];
        older = selectedCommits.value[1];
      }

      if (isPreviewableImage.value) {
        imageVersions.value = await invoke<ImageVersions>('get_file_image_versions', {
          path: props.repositoryPath,
          oldHash: older.hash,
          newHash: newer.hash,
          filePath: selectedFilePath.value
        });
        return;
      }
      
      const diff = await invoke<string>('compare_file_commits', {
        path: props.repositoryPath,
        hash1: older.hash,
        hash2: newer.hash,
        filePath: selectedFilePath.value
      });
      diffContent.value = diff;
    }
  } catch (e: any) {
    diffError.value = e.toString();
  } finally {
    loadingDiff.value = false;
  }
}

watch(() => props.repositoryPath, () => {
  files.value = [];
  selectedFilePath.value = null;
  if (props.active) {
    loadFiles();
  }
});

watch(() => props.active, (isActive) => {
  if (isActive && files.value.length === 0) {
    loadFiles();
  }
});

watch(() => props.initialFilePath, (newPath) => {
  if (newPath) {
    selectFile(newPath);
  }
});

onMounted(() => {
  window.addEventListener('click', closeContextMenu);
  if (props.active) {
    loadFiles();
  }
  if (props.initialFilePath) {
    selectFile(props.initialFilePath);
  }
});

onUnmounted(() => {
  window.removeEventListener('click', closeContextMenu);
});
</script>

<style scoped>
.file-explorer-page {
  display: flex;
  height: 100%;
  width: 100%;
  overflow: hidden;
  background-color: var(--bg-main);
}

.left-sidebar {
  display: flex;
  flex-direction: column;
  background-color: var(--panel-bg);
  border-right: 1px solid var(--border);
  height: 100%;
  flex-shrink: 0;
}

.sidebar-header {
  height: 40px;
  min-height: 40px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid var(--border);
}

.file-search {
  position: relative;
  display: flex;
  align-items: center;
  margin: 10px 12px 2px;
}

.file-search input {
  width: 100%;
  box-sizing: border-box;
  padding: 7px 28px 7px 28px;
  border: 1px solid var(--border);
  border-radius: 5px;
  outline: none;
  color: var(--text-main);
  background: var(--bg-main);
  font: inherit;
  font-size: 12px;
}

.file-search input:focus {
  border-color: var(--accent);
}

.file-search-icon {
  position: absolute;
  left: 8px;
  z-index: 1;
  color: var(--text-muted);
  pointer-events: none;
}

.clear-file-search {
  position: absolute;
  right: 5px;
  border: 0;
  padding: 1px 5px;
  border-radius: 3px;
  color: var(--text-muted);
  background: transparent;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
}

.clear-file-search:hover {
  color: var(--text-main);
  background: var(--surface-hover);
}

.no-file-results {
  padding: 20px 16px;
  color: var(--text-muted);
  font-size: 12px;
  text-align: center;
}

.sidebar-scrollable {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.resizer {
  width: 5px;
  cursor: col-resize;
  background-color: var(--border);
  flex-shrink: 0;
  position: relative;
  user-select: none;
  -webkit-user-select: none;
  transition: background-color 0.15s ease, box-shadow 0.15s ease;
  z-index: 10;
}

.resizer::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: -4px;
  right: -4px;
  background-color: transparent;
  cursor: col-resize;
  user-select: none;
  -webkit-user-select: none;
}

.resizer:hover, .resizer:active {
  background-color: var(--accent-blue, var(--accent, #58a6ff));
  box-shadow: 0 0 6px rgba(88, 166, 255, 0.4);
}

.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-width: 0;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-muted);
  gap: 16px;
}
.empty-icon {
  opacity: 0.5;
}

.file-history-view {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.history-header {
  padding: 24px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-inset);
}

.file-title-bar {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 4px;
}

.title-icon {
  color: var(--accent-blue, var(--accent));
}

.file-title {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--text-main);
}

.file-subtitle-wrapper {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.file-subtitle {
  margin: 0;
  font-size: 13px;
  color: var(--text-muted);
}

.keyboard-hint {
  font-size: 11px;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  gap: 4px;
  opacity: 0.8;
}

.keyboard-hint kbd {
  background: var(--bg-color);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 2px 6px;
  font-family: inherit;
  font-size: 10px;
  font-weight: 600;
  color: var(--text-main);
  box-shadow: 0 1px 1px rgba(0,0,0,0.1);
}

.history-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

.commits-list {
  width: 340px;
  border-right: 1px solid var(--border);
  overflow-y: auto;
  background: var(--bg-color);
  position: relative;
  user-select: none;
  -webkit-user-select: none;
}

.timeline-view {
  padding: 16px 12px;
}

.commit-card {
  display: flex;
  position: relative;
  cursor: pointer;
  margin-bottom: 8px;
  user-select: none;
}

.timeline-connector {
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 24px;
  flex-shrink: 0;
  position: relative;
  padding-top: 8px;
}

.timeline-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--border);
  border: 2px solid var(--bg-color);
  z-index: 2;
  transition: all 0.2s ease;
}

.timeline-line {
  position: absolute;
  top: 18px;
  bottom: -16px;
  width: 2px;
  background: var(--border);
  z-index: 1;
}

.commit-card:hover .timeline-dot {
  background: var(--accent);
  transform: scale(1.2);
}

.commit-card.selected .timeline-dot {
  background: var(--accent);
  box-shadow: 0 0 0 3px var(--selection-bg);
}

.commit-card-content {
  flex: 1;
  background: var(--panel-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 10px 12px;
  min-width: 0;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.commit-card:hover .commit-card-content {
  border-color: var(--accent);
  background: var(--surface-hover);
  transform: translateY(-1px);
  box-shadow: 0 4px 6px rgba(0, 0, 0, 0.05);
}

.commit-card.selected .commit-card-content {
  background: var(--selection-bg);
  border-color: var(--accent);
  box-shadow: 0 0 0 1px var(--accent);
}

.commit-header {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
  gap: 8px;
}

.commit-meta-top {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
  line-height: 1.2;
}

.commit-author {
  font-weight: 600;
  font-size: 11px;
  color: var(--text-main);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.commit-time {
  font-size: 10px;
  color: var(--text-muted);
}

.commit-hash {
  font-family: monospace;
  font-size: 11px;
  font-weight: 600;
  color: var(--accent);
  background: var(--selection-bg);
  padding: 2px 6px;
  border-radius: 4px;
}

.commit-body {
  padding-left: 28px;
}

.commit-message {
  margin: 0;
  font-size: 12px;
  color: var(--text-main);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}

.diff-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg-main);
  position: relative;
}

.diff-header {
  padding: 12px 16px;
  border-bottom: 1px solid var(--border);
  background: var(--panel-bg);
}

.diff-header h3 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-main);
}

.diff-content {
  flex: 1;
  overflow: auto;
  padding: 16px;
  font-family: monospace;
  font-size: 12px;
  line-height: 1.5;
}

.diff-pre {
  margin: 0;
}

.diff-content.error {
  color: #f44336;
}

.history-loading, .history-error {
  padding: 24px;
  color: var(--text-muted);
}

/* File View Mode Tabs */
.file-header-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.file-header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.purge-file-header-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px;
  border-radius: 6px;
  border: 1px solid rgba(239, 68, 68, 0.3);
  background: rgba(239, 68, 68, 0.08);
  color: var(--danger-text, #ef4444);
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.purge-file-header-btn:hover {
  background: rgba(239, 68, 68, 0.2);
  border-color: rgba(239, 68, 68, 0.5);
  color: #f87171;
}

.file-view-mode-tabs {
  display: flex;
  gap: 4px;
  background: var(--surface-subtle);
  padding: 3px;
  border-radius: 6px;
  border: 1px solid var(--border);
}

.view-mode-tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 4px;
  border: none;
  background: none;
  color: var(--text-muted);
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.view-mode-tab.active {
  background: var(--bg-card);
  color: var(--text-main);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.tab-badge {
  font-size: 0.7rem;
  padding: 1px 5px;
  border-radius: 8px;
  background: var(--surface-subtle);
  color: var(--text-muted);
}

.view-mode-tab.active .tab-badge {
  background: var(--accent-blue);
  color: #fff;
}

.blame-stats {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.75rem;
  color: var(--text-muted);
}

/* Blame View Container */
.blame-view-container {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: var(--bg-main);
}
</style>
