<script setup lang="ts">
import FolderIcon from '../assets/icons/folder.svg?component';
import CloneIcon from '../assets/icons/clone.svg?component';
import CloseIcon from '../assets/icons/close.svg?component';

const props = defineProps<{
  history: Array<{ path: string, name: string }>,
  isNewProject?: boolean
}>();

const emit = defineEmits<{
  (e: 'open-repository'): void
  (e: 'open-clone'): void
  (e: 'open-history', path: string): void
  (e: 'remove-history', path: string): void
  (e: 'clear-history'): void
}>();
</script>

<template>
  <div class="empty-state">
    <div class="empty-state-content">
      <div class="hero-icon-container">
        <FolderIcon viewBox="0 0 24 24" width="56" height="56" stroke="var(--text-muted)" stroke-width="1.2" fill="none" />
      </div>
      <h2>{{ isNewProject ? 'Open another project' : 'Welcome to BeGit' }}</h2>
      <p>Browse for a local Git repository, clone a remote repository, or continue from your recent projects.</p>
      
      <div class="action-buttons">
        <button class="btn btn-primary open-repository-btn" @click="emit('open-repository')">
          <FolderIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
          Browse Repository
        </button>
        <button class="btn btn-secondary clone-repository-btn" @click="emit('open-clone')">
          <CloneIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
          Clone Repository
        </button>
      </div>
      
      <div v-if="history && history.length > 0" class="history-list">
        <div class="history-header">
          <h3>Recent Repositories</h3>
          <button class="clear-history-btn" @click.stop="emit('clear-history')" title="Clear History">Clear</button>
        </div>
        <ul>
          <li v-for="repo in history" :key="repo.path" @click="emit('open-history', repo.path)">
            <div class="repo-info">
              <span class="repo-name">{{ repo.name }}</span>
              <span class="repo-path">{{ repo.path }}</span>
            </div>
            <button class="remove-history-btn" @click.stop="emit('remove-history', repo.path)" title="Remove from history">
              <CloseIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            </button>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hero-icon-container {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
}

.action-buttons {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-top: 4px;
  flex-wrap: wrap;
}

.open-repository-btn,
.clone-repository-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 9px 18px;
  font-size: 0.88rem;
  font-weight: 600;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary {
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  color: var(--text-main);
}

.btn-secondary:hover {
  background: var(--surface-hover);
  border-color: var(--accent-blue);
  color: var(--text-main);
}

.history-list {
  margin-top: 32px;
  text-align: left;
  width: 100%;
  max-width: 440px;
}
.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}
.history-header h3 {
  font-size: 0.85rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 0;
}
.clear-history-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.75rem;
  cursor: pointer;
}
.clear-history-btn:hover {
  color: var(--text-main);
  text-decoration: underline;
}
.history-list ul {
  list-style: none;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.history-list li {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  background-color: var(--panel-bg);
  border: 1px solid var(--border);
  border-radius: 6px;
  cursor: pointer;
  transition: background-color 0.2s, border-color 0.2s;
}
.history-list li:hover {
  background-color: var(--row-hover);
  border-color: var(--accent-blue);
}
.repo-info {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding-right: 8px;
}
.repo-name {
  font-weight: 600;
  color: var(--text-main);
  font-size: 0.9rem;
}
.repo-path {
  font-size: 0.75rem;
  color: var(--text-muted);
  margin-top: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.remove-history-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.2s, background-color 0.2s;
  flex-shrink: 0;
}
.history-list li:hover .remove-history-btn {
  opacity: 1;
}
.remove-history-btn:hover {
  background-color: var(--bg-color);
  color: var(--text-main);
}
</style>
