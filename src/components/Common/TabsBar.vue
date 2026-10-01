<script setup lang="ts">
import FolderIcon from '../../assets/icons/folder.svg?component';
import CloseIcon from '../../assets/icons/close.svg?component';
import NewProjectIcon from '../../assets/icons/new-project.svg?component';
import PlusIcon from '../../assets/icons/plus.svg?component';
import UserIcon from '../../assets/icons/user.svg?component';
import SunIcon from '../../assets/icons/sun.svg?component';
import MoonIcon from '../../assets/icons/moon.svg?component';
import SearchIcon from '../../assets/icons/search.svg?component';
import { useTheme } from '../../composables/useTheme';

const props = defineProps<{
  projects: Array<{ path: string; name: string }>;
  activeTabIndex: number;
  showNewTab: boolean;
}>();

const emit = defineEmits<{
  (e: 'select-tab', index: number): void;
  (e: 'close-tab', index: number): void;
  (e: 'new-tab'): void;
  (e: 'close-new-tab'): void;
  (e: 'open-profile'): void;
  (e: 'open-palette'): void;
}>();

function onTabClick(index: number) {
  emit('select-tab', index);
}

function onCloseTab(index: number, event: Event) {
  event.stopPropagation();
  emit('close-tab', index);
}
const { isDarkTheme, toggleTheme } = useTheme();
</script>

<template>
  <header class="tabs-bar">
    <div class="tabs-list-wrapper">
      <div v-for="(project, index) in projects" :key="project.path" 
           class="tab" :class="{ 'active-tab': !showNewTab && activeTabIndex === index }"
           @click="onTabClick(index)">
        <FolderIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
        <span class="tab-name">{{ project.name }}</span>
        <button class="close-tab-btn" @click="onCloseTab(index, $event)" title="Close Tab">
          <CloseIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
        </button>
      </div>
      <div v-if="showNewTab" class="tab active-tab new-project-tab">
        <NewProjectIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
        <span class="tab-name">New Project</span>
        <button v-if="projects.length > 0" class="close-tab-btn" @click.stop="emit('close-new-tab')" title="Close New Project">
          <CloseIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
        </button>
      </div>
      <button class="new-tab-btn" @click="emit('new-tab')" title="New Project">
        <PlusIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2.5" fill="none"  />
      </button>
    </div>

    <div class="tabs-actions-wrapper">
      <button class="palette-quick-trigger" @click="emit('open-palette')" title="Open Command Palette (Cmd+K / Ctrl+K)">
        <SearchIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" class="palette-search-icon" />
        <span class="palette-text">Search...</span>
        <kbd>⌘K</kbd>
      </button>
      <button class="tab-action-btn" @click="emit('open-profile')" title="Git Profile">
        <UserIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"  />
      </button>
      <button class="tab-action-btn" @click="toggleTheme" :title="isDarkTheme ? 'Switch to Light Theme' : 'Switch to Dark Theme'">
        <SunIcon v-if="isDarkTheme" viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"  />
        <MoonIcon v-else viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"  />
      </button>
    </div>
  </header>
</template>

<style scoped>
.tabs-list-wrapper {
  display: flex;
  align-items: flex-end;
  height: 100%;
  overflow-x: auto;
  overflow-y: hidden;
  scrollbar-width: none;
  flex: 1;
  min-width: 0;
}

.tabs-list-wrapper::-webkit-scrollbar {
  display: none;
}

.tabs-actions-wrapper {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 10px;
  height: 100%;
  flex-shrink: 0;
  margin-left: auto;
}

.palette-quick-trigger {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 24px;
  padding: 0 8px;
  border-radius: 5px;
  border: 1px solid var(--border);
  background: var(--surface-subtle);
  color: var(--text-muted);
  font-size: 0.74rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.palette-quick-trigger:hover {
  background: var(--surface-hover);
  color: var(--text-main);
  border-color: var(--accent-blue, #58a6ff);
}

.palette-search-icon {
  flex-shrink: 0;
}

.palette-text {
  font-size: 0.74rem;
}

.palette-quick-trigger kbd {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 3px;
  padding: 1px 4px;
  font-size: 0.65rem;
  font-family: inherit;
  color: var(--text-muted);
}

.tab-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 5px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  padding: 0;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.tab-action-btn:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

@media (max-width: 600px) {
  .palette-quick-trigger .palette-text,
  .palette-quick-trigger kbd {
    display: none;
  }
  .palette-quick-trigger {
    padding: 0 6px;
  }
}
</style>
