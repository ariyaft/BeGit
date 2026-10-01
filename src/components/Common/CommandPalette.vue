<template>
  <div v-if="visible" class="command-palette-backdrop" @click.self="close" @keydown="onKeydown">
    <div class="command-palette-modal" role="dialog" aria-modal="true" aria-label="Command Palette">
      <!-- Search Input Header -->
      <div class="palette-input-wrapper">
        <SearchIcon viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" class="palette-search-icon" />
        <input
          ref="inputRef"
          v-model="searchQuery"
          type="text"
          class="palette-input"
          :placeholder="placeholderText"
          @keydown="onInputKeydown"
          autocomplete="off"
          spellcheck="false"
        />
        <kbd class="palette-kbd">Esc</kbd>
      </div>

      <!-- Category Filter Pills -->
      <div class="palette-categories">
        <button
          v-for="cat in categories"
          :key="cat.id"
          class="category-pill"
          :class="{ active: activeCategory === cat.id }"
          @click="activeCategory = cat.id"
        >
          <component :is="cat.icon" viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" class="cat-icon" />
          <span>{{ cat.label }}</span>
          <span class="cat-count" v-if="getCategoryCount(cat.id) !== null">{{ getCategoryCount(cat.id) }}</span>
        </button>
      </div>

      <!-- Results List -->
      <div class="palette-results" ref="resultsListRef">
        <div v-if="filteredItems.length === 0" class="palette-empty">
          <p>No matching commands, branches, commits, or files found.</p>
        </div>

        <div
          v-for="(item, index) in filteredItems"
          :key="item.id"
          :id="'palette-item-' + index"
          class="palette-item"
          :class="{ selected: selectedIndex === index }"
          @click="executeItem(item)"
          @mouseenter="selectedIndex = index"
        >
          <div class="item-icon-wrapper" :class="'icon-' + item.category">
            <component :is="item.icon" viewBox="0 0 24 24" width="15" height="15" stroke="currentColor" stroke-width="2" fill="none" />
          </div>

          <div class="item-content">
            <div class="item-title-row">
              <span class="item-title">{{ item.title }}</span>
              <span v-if="item.badge" class="item-badge" :class="'badge-' + (item.badgeType || 'default')">{{ item.badge }}</span>
            </div>
            <div v-if="item.subtitle" class="item-subtitle">{{ item.subtitle }}</div>
          </div>

          <div v-if="item.shortcut" class="item-shortcut">
            <kbd>{{ item.shortcut }}</kbd>
          </div>
        </div>
      </div>

      <!-- Footer navigation hints -->
      <footer class="palette-footer">
        <div class="footer-hint">
          <kbd>↑</kbd><kbd>↓</kbd> <span>Navigate</span>
        </div>
        <div class="footer-hint">
          <kbd>↵</kbd> <span>Select</span>
        </div>
        <div class="footer-hint">
          <kbd>Tab</kbd> <span>Change Category</span>
        </div>
        <div class="footer-hint">
          <kbd>Esc</kbd> <span>Close</span>
        </div>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import type { Project, AppView } from '../../types';

// Icons
import SearchIcon from '../../assets/icons/search.svg?component';
import BranchIcon from '../../assets/icons/branch.svg?component';
import TimelineIcon from '../../assets/icons/timeline.svg?component';
import FileIcon from '../../assets/icons/file.svg?component';
import ActionIcon from '../../assets/icons/action.svg?component';
import SourceControlIcon from '../../assets/icons/source-control.svg?component';
import PullRequestIcon from '../../assets/icons/git-pull-request.svg?component';
import ContributorsIcon from '../../assets/icons/contributors.svg?component';
import ActivityLogIcon from '../../assets/icons/list.svg?component';
import FetchIcon from '../../assets/icons/fetch.svg?component';
import PullIcon from '../../assets/icons/pull.svg?component';
import PushIcon from '../../assets/icons/push.svg?component';
import StashIcon from '../../assets/icons/stash.svg?component';
import PopIcon from '../../assets/icons/pop.svg?component';
import PlusIcon from '../../assets/icons/plus.svg?component';
import MinusIcon from '../../assets/icons/minus.svg?component';
import SunIcon from '../../assets/icons/sun.svg?component';
import ProfileIcon from '../../assets/icons/profile.svg?component';
import CloneIcon from '../../assets/icons/clone.svg?component';
import UndoIcon from '../../assets/icons/undo.svg?component';

interface PaletteItem {
  id: string;
  category: 'command' | 'branch' | 'commit' | 'file';
  title: string;
  subtitle?: string;
  badge?: string;
  badgeType?: 'active' | 'remote' | 'feat' | 'fix' | 'default';
  shortcut?: string;
  icon: any;
  action: () => void;
}

const props = defineProps<{
  visible: boolean;
  project?: Project | null;
  trackedFiles?: string[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'navigate', view: AppView): void;
  (e: 'checkout-branch', branchName: string): void;
  (e: 'select-commit', commit: any): void;
  (e: 'select-file', filePath: string): void;
  (e: 'trigger-action', action: string): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);
const resultsListRef = ref<HTMLElement | null>(null);
const searchQuery = ref('');
const activeCategory = ref<'all' | 'command' | 'branch' | 'commit' | 'file'>('all');
const selectedIndex = ref(0);

const categories = [
  { id: 'all', label: 'All', icon: SearchIcon },
  { id: 'command', label: 'Commands', icon: ActionIcon },
  { id: 'branch', label: 'Branches', icon: BranchIcon },
  { id: 'commit', label: 'Commits', icon: TimelineIcon },
  { id: 'file', label: 'Files', icon: FileIcon },
] as const;

const placeholderText = computed(() => {
  switch (activeCategory.value) {
    case 'command': return 'Search git commands...';
    case 'branch': return 'Search and checkout branches...';
    case 'commit': return 'Search commits by message, author, hash...';
    case 'file': return 'Search repository files...';
    default: return 'Type a command, branch, commit, or file... (> for commands, # for branches, @ for commits, : for files)';
  }
});

// Build all palette items
const allItems = computed<PaletteItem[]>(() => {
  const items: PaletteItem[] = [];

  // 1. Navigation & Git Commands
  items.push(
    {
      id: 'cmd-source-control',
      category: 'command',
      title: 'View: Open Source Control & Commit Graph',
      subtitle: 'Switch to main commit graph and staging view',
      shortcut: '1',
      icon: SourceControlIcon,
      action: () => emit('navigate', 'source-control'),
    },
    {
      id: 'cmd-horizontal-graph',
      category: 'command',
      title: 'View: Open Horizontal Timeline',
      subtitle: 'Switch to horizontal commit swimlane visualization',
      shortcut: '2',
      icon: TimelineIcon,
      action: () => emit('navigate', 'horizontal-graph'),
    },
    {
      id: 'cmd-pull-requests',
      category: 'command',
      title: 'View: Open Pull Requests & Reviews',
      subtitle: 'Manage GitHub and GitLab pull requests',
      shortcut: '3',
      icon: PullRequestIcon,
      action: () => emit('navigate', 'pull-requests'),
    },
    {
      id: 'cmd-file-explorer',
      category: 'command',
      title: 'View: Open File Explorer & Git Blame',
      subtitle: 'Inspect repository file history and line-by-line blame',
      shortcut: '4',
      icon: FileIcon,
      action: () => emit('navigate', 'file-explorer'),
    },
    {
      id: 'cmd-contributors',
      category: 'command',
      title: 'View: Open Contributors Report',
      subtitle: 'View contributor activity and metrics',
      shortcut: '5',
      icon: ContributorsIcon,
      action: () => emit('navigate', 'contributors-report'),
    },
    {
      id: 'cmd-reflog',
      category: 'command',
      title: 'View: Open Git Reflog & Undo Safety Net',
      subtitle: 'Recover deleted branches, undo accidental resets, and inspect HEAD history',
      shortcut: '5',
      icon: UndoIcon,
      action: () => emit('navigate', 'reflog'),
    },
    {
      id: 'cmd-activity-log',
      category: 'command',
      title: 'View: Open Activity Logs',
      subtitle: 'Inspect git execution history and output',
      shortcut: '6',
      icon: ActivityLogIcon,
      action: () => emit('navigate', 'activity-log'),
    },
    {
      id: 'cmd-fetch',
      category: 'command',
      title: 'Git: Fetch from All Remotes',
      subtitle: 'Fetch latest branches and tags from remote',
      icon: FetchIcon,
      action: () => emit('trigger-action', 'fetch'),
    },
    {
      id: 'cmd-pull',
      category: 'command',
      title: 'Git: Pull from Remote',
      subtitle: 'Pull latest changes into active branch',
      icon: PullIcon,
      action: () => emit('trigger-action', 'pull'),
    },
    {
      id: 'cmd-push',
      category: 'command',
      title: 'Git: Push Active Branch',
      subtitle: 'Push local commits to upstream remote',
      icon: PushIcon,
      action: () => emit('trigger-action', 'push'),
    },
    {
      id: 'cmd-stash',
      category: 'command',
      title: 'Git: Stash Working Changes',
      subtitle: 'Save modified files to stash stack',
      icon: StashIcon,
      action: () => emit('trigger-action', 'stash'),
    },
    {
      id: 'cmd-pop',
      category: 'command',
      title: 'Git: Pop Latest Stash',
      subtitle: 'Apply and drop the most recent stash',
      icon: PopIcon,
      action: () => emit('trigger-action', 'pop'),
    },
    {
      id: 'cmd-stage-all',
      category: 'command',
      title: 'Git: Stage All Changes',
      subtitle: 'Stage all modified and untracked files',
      icon: PlusIcon,
      action: () => emit('trigger-action', 'stage-all'),
    },
    {
      id: 'cmd-new-branch',
      category: 'command',
      title: 'Git: Create New Branch',
      subtitle: 'Create a new local branch from current HEAD',
      icon: BranchIcon,
      action: () => emit('trigger-action', 'create-branch'),
    },
    {
      id: 'cmd-toggle-theme',
      category: 'command',
      title: 'Preferences: Toggle Theme (Dark / Light)',
      subtitle: 'Switch application color theme',
      icon: SunIcon,
      action: () => emit('trigger-action', 'toggle-theme'),
    },
    {
      id: 'cmd-git-profile',
      category: 'command',
      title: 'Preferences: Configure Git Profile',
      subtitle: 'Edit user.name and user.email configuration',
      icon: ProfileIcon,
      action: () => emit('trigger-action', 'git-profile'),
    },
    {
      id: 'cmd-clone-repo',
      category: 'command',
      title: 'Repository: Clone Repository',
      subtitle: 'Clone a remote git repository from URL',
      icon: CloneIcon,
      action: () => emit('trigger-action', 'clone-repository'),
    },
    {
      id: 'cmd-zoom-in',
      category: 'command',
      title: 'View: Zoom In (+10%)',
      subtitle: 'Enlarge application display scale',
      shortcut: '⌘+',
      icon: PlusIcon,
      action: () => emit('trigger-action', 'zoom-in'),
    },
    {
      id: 'cmd-zoom-out',
      category: 'command',
      title: 'View: Zoom Out (-10%)',
      subtitle: 'Reduce application display scale',
      shortcut: '⌘-',
      icon: MinusIcon,
      action: () => emit('trigger-action', 'zoom-out'),
    },
    {
      id: 'cmd-zoom-reset',
      category: 'command',
      title: 'View: Reset Zoom (100%)',
      subtitle: 'Reset application display scale to standard 100%',
      shortcut: '⌘0',
      icon: ActionIcon,
      action: () => emit('trigger-action', 'zoom-reset'),
    }
  );

  // 2. Branches
  if (props.project?.localBranches) {
    props.project.localBranches.forEach((b: any) => {
      let badge = b.active ? 'Active' : '';
      if (b.ahead > 0 || b.behind > 0) {
        badge = `${b.ahead}↑ ${b.behind}↓`;
      }
      items.push({
        id: 'branch-local-' + b.name,
        category: 'branch',
        title: b.name,
        subtitle: `Local Branch ${b.active ? '(Currently Checked Out)' : ''}`,
        badge: badge || undefined,
        badgeType: b.active ? 'active' : 'default',
        icon: BranchIcon,
        action: () => {
          emit('checkout-branch', b.name);
          emit('navigate', 'source-control');
        },
      });
    });
  }

  if (props.project?.remoteBranches) {
    props.project.remoteBranches.forEach((b: any) => {
      items.push({
        id: 'branch-remote-' + b.name,
        category: 'branch',
        title: b.name,
        subtitle: 'Remote Tracking Branch',
        badge: 'Remote',
        badgeType: 'remote',
        icon: BranchIcon,
        action: () => {
          emit('checkout-branch', b.name);
          emit('navigate', 'source-control');
        },
      });
    });
  }

  // 3. Commits
  const rawCommits = props.project?.rawCommits || props.project?.commits || [];
  rawCommits.slice(0, 100).forEach((c: any) => {
    items.push({
      id: 'commit-' + c.hash,
      category: 'commit',
      title: c.message,
      subtitle: `${c.author} • ${c.time || ''}`,
      badge: c.hash_short || c.hash.substring(0, 7),
      badgeType: 'default',
      icon: TimelineIcon,
      action: () => {
        emit('select-commit', c);
        emit('navigate', 'source-control');
      },
    });
  });

  // 4. Tracked Files
  if (props.trackedFiles && props.trackedFiles.length > 0) {
    props.trackedFiles.forEach((file: string) => {
      const fileName = file.split('/').pop() || file;
      items.push({
        id: 'file-' + file,
        category: 'file',
        title: fileName,
        subtitle: file,
        icon: FileIcon,
        action: () => {
          emit('select-file', file);
          emit('navigate', 'file-explorer');
        },
      });
    });
  }

  return items;
});

// Filter items based on query & active category
const filteredItems = computed(() => {
  let query = searchQuery.value.trim().toLowerCase();
  let cat = activeCategory.value;

  // Prefix triggers
  if (query.startsWith('>')) {
    cat = 'command';
    query = query.substring(1).trim();
  } else if (query.startsWith('#')) {
    cat = 'branch';
    query = query.substring(1).trim();
  } else if (query.startsWith('@')) {
    cat = 'commit';
    query = query.substring(1).trim();
  } else if (query.startsWith(':')) {
    cat = 'file';
    query = query.substring(1).trim();
  }

  let list = allItems.value;
  if (cat !== 'all') {
    list = list.filter(item => item.category === cat);
  }

  if (!query) {
    return list.slice(0, 60);
  }

  const matches = list.filter(item => {
    const inTitle = item.title.toLowerCase().includes(query);
    const inSub = item.subtitle?.toLowerCase().includes(query);
    const inBadge = item.badge?.toLowerCase().includes(query);
    return inTitle || inSub || inBadge;
  });

  return matches.slice(0, 60);
});

function getCategoryCount(catId: string): number | null {
  if (catId === 'all') return null;
  return allItems.value.filter(i => i.category === catId).length;
}

function executeItem(item: PaletteItem) {
  close();
  item.action();
}

function close() {
  emit('close');
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    close();
  }
}

function onInputKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    if (filteredItems.value.length > 0) {
      selectedIndex.value = (selectedIndex.value + 1) % filteredItems.value.length;
      scrollToSelected();
    }
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    if (filteredItems.value.length > 0) {
      selectedIndex.value = (selectedIndex.value - 1 + filteredItems.value.length) % filteredItems.value.length;
      scrollToSelected();
    }
  } else if (e.key === 'Enter') {
    e.preventDefault();
    if (filteredItems.value[selectedIndex.value]) {
      executeItem(filteredItems.value[selectedIndex.value]);
    }
  } else if (e.key === 'Tab') {
    e.preventDefault();
    const catList: Array<'all' | 'command' | 'branch' | 'commit' | 'file'> = ['all', 'command', 'branch', 'commit', 'file'];
    const curIdx = catList.indexOf(activeCategory.value);
    activeCategory.value = catList[(curIdx + 1) % catList.length];
    selectedIndex.value = 0;
  }
}

function scrollToSelected() {
  nextTick(() => {
    const el = document.getElementById('palette-item-' + selectedIndex.value);
    if (el && resultsListRef.value) {
      el.scrollIntoView({ block: 'nearest' });
    }
  });
}

watch(() => props.visible, (val) => {
  if (val) {
    searchQuery.value = '';
    activeCategory.value = 'all';
    selectedIndex.value = 0;
    nextTick(() => {
      inputRef.value?.focus();
    });
  }
});

watch(searchQuery, () => {
  selectedIndex.value = 0;
});
</script>

<style scoped>
.command-palette-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(5px);
  display: flex;
  justify-content: center;
  align-items: flex-start;
  padding-top: 80px;
  z-index: 9999;
}

.command-palette-modal {
  width: 640px;
  max-width: 92vw;
  max-height: 80vh;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 12px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: fadeInDown 0.15s ease-out;
}

@keyframes fadeInDown {
  from {
    opacity: 0;
    transform: translateY(-12px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

/* Input Header */
.palette-input-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-card);
}

.palette-search-icon {
  color: var(--accent-blue);
  flex-shrink: 0;
}

.palette-input {
  flex: 1;
  border: none;
  background: none;
  color: var(--text-main);
  font-size: 1.05rem;
  outline: none;
  font-weight: 500;
}

.palette-input::placeholder {
  color: var(--text-muted);
  font-size: 0.92rem;
  font-weight: 400;
}

.palette-kbd {
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 0.72rem;
  color: var(--text-muted);
  font-family: inherit;
}

/* Categories Bar */
.palette-categories {
  display: flex;
  gap: 6px;
  padding: 8px 16px;
  background: var(--surface-subtle);
  border-bottom: 1px solid var(--border);
  overflow-x: auto;
}

.category-pill {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 8px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: none;
  color: var(--text-muted);
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.category-pill:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

.category-pill.active {
  background: var(--bg-card);
  color: var(--accent-blue);
  border-color: var(--border);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
}

.cat-count {
  font-size: 0.7rem;
  padding: 1px 5px;
  border-radius: 8px;
  background: var(--surface-subtle);
  color: var(--text-muted);
}

/* Results */
.palette-results {
  flex: 1;
  overflow-y: auto;
  max-height: 380px;
  padding: 6px 0;
}

.palette-empty {
  padding: 32px 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 0.88rem;
}

.palette-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 18px;
  cursor: pointer;
  transition: background 0.1s ease;
}

.palette-item.selected {
  background: var(--surface-hover);
  border-left: 3px solid var(--accent-blue);
}

.item-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: var(--surface-subtle);
  color: var(--text-muted);
  flex-shrink: 0;
}

.icon-command {
  color: #a371f7;
  background: rgba(163, 113, 247, 0.15);
}

.icon-branch {
  color: var(--accent-blue);
  background: rgba(88, 166, 255, 0.15);
}

.icon-commit {
  color: #3fb950;
  background: rgba(63, 185, 80, 0.15);
}

.icon-file {
  color: #d29922;
  background: rgba(210, 153, 34, 0.15);
}

.item-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.item-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.item-title {
  font-size: 0.88rem;
  font-weight: 600;
  color: var(--text-main);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-badge {
  font-size: 0.68rem;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 4px;
  font-family: monospace;
}

.badge-active {
  background: rgba(63, 185, 80, 0.15);
  color: #3fb950;
  border: 1px solid rgba(63, 185, 80, 0.3);
}

.badge-remote {
  background: rgba(88, 166, 255, 0.15);
  color: var(--accent-blue);
  border: 1px solid rgba(88, 166, 255, 0.3);
}

.badge-default {
  background: var(--surface-subtle);
  color: var(--text-muted);
  border: 1px solid var(--border);
}

.item-subtitle {
  font-size: 0.76rem;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-shortcut kbd {
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  border-radius: 4px;
  padding: 2px 6px;
  font-size: 0.72rem;
  color: var(--text-muted);
}

/* Footer */
.palette-footer {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 8px 18px;
  border-top: 1px solid var(--border);
  background: var(--surface-subtle);
  font-size: 0.74rem;
  color: var(--text-muted);
}

.footer-hint {
  display: flex;
  align-items: center;
  gap: 4px;
}

.footer-hint kbd {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 1px 4px;
  font-size: 0.7rem;
  color: var(--text-main);
}
</style>
