<script setup lang="ts">
import BranchIcon from '../../assets/icons/branch.svg?component';
import UserIcon from '../../assets/icons/user.svg?component';
import FolderIcon from '../../assets/icons/folder.svg?component';
import ActionIcon from '../../assets/icons/action.svg?component';
import MinusIcon from '../../assets/icons/minus.svg?component';
import PlusIcon from '../../assets/icons/plus.svg?component';
import { computed } from 'vue';
import { notify } from '../../composables/useToasts';
import { useZoom } from '../../composables/useZoom';
import packageJson from '../../../package.json';

const appVersion = packageJson.version;

const props = defineProps<{
  project: any;
  branch: string;
  profile: { name: string; email: string };
  workingChangeCount: number;
  isFetching: boolean;
}>();

const emit = defineEmits<{
  openBranch: [];
  openChanges: [];
  fetch: [];
  openProfile: [];
}>();

const { zoomLevel, zoomIn, zoomOut, resetZoom, minZoom, maxZoom } = useZoom();

const activeBranch = computed(() => props.project?.localBranches?.find((branch: any) => branch.active));
const syncStatus = computed(() => {
  const ahead = activeBranch.value?.ahead || 0;
  const behind = activeBranch.value?.behind || 0;
  if (!ahead && !behind) return 'Up to date';
  return [ahead && `↑${ahead}`, behind && `↓${behind}`].filter(Boolean).join('  ');
});

const identity = computed(() => {
  if (props.profile.name && props.profile.email) return `${props.profile.name} <${props.profile.email}>`;
  return props.profile.name || props.profile.email || 'Git identity not configured';
});

const lastFetched = computed(() => {
  const timestamp = props.project?.lastFetchTime;
  if (!timestamp) return 'Not fetched yet';
  const minutes = Math.max(0, Math.floor((Date.now() - timestamp) / 60000));
  if (minutes < 1) return 'Fetched just now';
  if (minutes < 60) return `Fetched ${minutes}m ago`;
  return `Fetched ${Math.floor(minutes / 60)}h ago`;
});

async function copyPath() {
  if (!props.project?.path) return;
  try {
    await navigator.clipboard.writeText(props.project.path);
    notify('Repository path copied.', 'success');
  } catch {
    notify('Could not copy the repository path.');
  }
}
</script>

<template>
  <footer class="repository-status-footer" aria-label="Repository status">
    <div class="status-group">
      <button class="status-item status-button" :title="`${branch || 'Detached HEAD'} — open branches`" @click="emit('openBranch')">
        <BranchIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        <span>{{ branch || 'Detached HEAD' }}</span>
      </button>
      <button class="status-item status-button status-sync" :class="{ 'status-diverged': activeBranch?.ahead || activeBranch?.behind }" :disabled="isFetching" :title="`${syncStatus} · ${lastFetched} — fetch from remote`" @click="emit('fetch')">
        {{ isFetching ? 'Fetching…' : `${syncStatus} · ${lastFetched}` }}
      </button>
      <button v-if="workingChangeCount" class="status-item status-button status-changes" :title="`${workingChangeCount} working-tree change${workingChangeCount === 1 ? '' : 's'} — open changes`" @click="emit('openChanges')">
        <ActionIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        {{ workingChangeCount }} change{{ workingChangeCount === 1 ? '' : 's' }}
      </button>
      <span v-if="project?.cherryPick?.active" class="status-item status-warning">Cherry-pick paused</span>
    </div>

    <div class="status-group status-right">
      <!-- Zoom in / out controls -->
      <div class="status-item zoom-control-widget">
        <button
          class="zoom-btn zoom-out-btn"
          :disabled="zoomLevel <= minZoom"
          title="Zoom Out (Cmd - / Ctrl -)"
          @click.stop="zoomOut"
        >
          <MinusIcon viewBox="0 0 24 24" width="10" height="10" stroke="currentColor" stroke-width="2.5" fill="none" />
        </button>
        <button
          class="zoom-btn zoom-indicator-btn"
          :title="`UI Zoom: ${zoomLevel}% — Click to reset to 100% (Cmd 0 / Ctrl 0)`"
          @click.stop="resetZoom"
        >
          <span>{{ zoomLevel }}%</span>
        </button>
        <button
          class="zoom-btn zoom-in-btn"
          :disabled="zoomLevel >= maxZoom"
          title="Zoom In (Cmd + / Ctrl +)"
          @click.stop="zoomIn"
        >
          <PlusIcon viewBox="0 0 24 24" width="10" height="10" stroke="currentColor" stroke-width="2.5" fill="none" />
        </button>
      </div>

      <button class="status-item status-button status-identity" :title="`${identity} — edit Git profile`" @click="emit('openProfile')">
        <UserIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        <span>{{ identity }}</span>
      </button>
      <button class="status-item status-button status-path" :title="`${project?.path} — copy path`" @click="copyPath">
        <FolderIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        <span>{{ project?.path }}</span>
      </button>
      <span class="status-item status-version" :title="`BeGit v${appVersion}`">
        v{{ appVersion }}
      </span>
    </div>
  </footer>
</template>

<style scoped>
.repository-status-footer {
  min-height: 27px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 0 12px;
  border-top: 1px solid var(--border);
  background: var(--panel-bg);
  color: var(--text-muted);
  font-size: .74rem;
  flex-shrink: 0;
}

.status-group { display: flex; align-items: center; gap: 12px; min-width: 0; }
.status-right { justify-content: flex-end; }
.status-item { display: inline-flex; align-items: center; gap: 5px; min-width: 0; white-space: nowrap; }
.status-button { border: 0; padding: 2px 0; background: transparent; color: inherit; font: inherit; cursor: pointer; }
.status-button:hover:not(:disabled) { color: var(--text-main); }
.status-button:focus-visible { outline: 1px solid var(--accent); outline-offset: 3px; border-radius: 2px; }
.status-button:disabled { cursor: default; opacity: .7; }
.status-sync { color: var(--success-text); }
.status-diverged, .status-changes { color: var(--warning-text); }
.status-warning { color: var(--warning-text); }
.status-identity, .status-path { max-width: min(30vw, 330px); overflow: hidden; text-overflow: ellipsis; }
.status-version {
  font-size: 0.68rem;
  font-weight: 600;
  color: var(--text-muted);
  opacity: 0.85;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--surface-subtle, rgba(255, 255, 255, 0.04));
  border: 1px solid var(--border, rgba(128, 128, 128, 0.15));
  font-family: inherit;
  user-select: none;
}

/* Zoom Control Widget */
.zoom-control-widget {
  display: inline-flex;
  align-items: center;
  background: var(--surface-subtle, rgba(255, 255, 255, 0.04));
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  border-radius: 4px;
  padding: 1px 2px;
  gap: 1px;
  user-select: none;
  height: 20px;
}

.zoom-btn {
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 4px;
  height: 100%;
  border-radius: 3px;
  font-size: 0.68rem;
  font-family: inherit;
  font-weight: 600;
  transition: all 0.12s ease;
}

.zoom-btn:hover:not(:disabled) {
  background: var(--surface-hover, rgba(255, 255, 255, 0.1));
  color: var(--text-main);
}

.zoom-btn:disabled {
  opacity: 0.35;
  cursor: default;
}

.zoom-indicator-btn {
  min-width: 36px;
  text-align: center;
  font-variant-numeric: tabular-nums;
  padding: 0 2px;
}

.zoom-indicator-btn:hover {
  color: var(--accent-blue, #58a6ff);
}

@media (max-width: 880px) {
  .status-identity { max-width: 185px; }
  .status-path { display: none; }
}

@media (max-width: 620px) {
  .repository-status-footer { padding: 0 8px; }
  .status-sync { display: none; }
  .status-identity { max-width: 145px; }
}
</style>
