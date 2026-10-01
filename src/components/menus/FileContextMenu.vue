<template>
  <div
    v-if="visible && fileNode"
    class="context-menu"
    :style="{ left: x + 'px', top: y + 'px' }"
    @click.stop
  >
    <div class="menu-item" @click="$emit('action', 'history')">
      <TimelineIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
      View History
    </div>
    <div class="menu-item" @click="$emit('action', 'blame')" v-if="!fileNode.isDir">
      <ActionIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
      View Git Blame
    </div>
    
    <div class="menu-divider"></div>
    
    <div class="menu-item" @click="$emit('action', 'copy_path')">
      <CopyIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
      Copy Path
    </div>
    <div class="menu-item" @click="$emit('action', 'copy_name')">
      <CopyIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
      Copy File Name
    </div>

    <div class="menu-divider"></div>

    <div class="menu-item text-danger" @click="$emit('action', 'purge_history')">
      <TrashIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
      Purge from Git History...
    </div>
  </div>
</template>

<script setup lang="ts">
import type { FileNode } from '../Common/FileTreeNode.vue';
import TimelineIcon from '../../assets/icons/timeline.svg?component';
import ActionIcon from '../../assets/icons/action.svg?component';
import CopyIcon from '../../assets/icons/copy.svg?component';
import TrashIcon from '../../assets/icons/context-danger-1.svg?component';

defineProps<{
  visible: boolean;
  x: number;
  y: number;
  fileNode: FileNode | null;
}>();

defineEmits<{
  (e: 'action', action: 'history' | 'blame' | 'copy_path' | 'copy_name' | 'purge_history'): void;
}>();
</script>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 10000;
  min-width: 200px;
  background: var(--bg-surface, #1e1e24);
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.12));
  border-radius: 8px;
  padding: 4px;
  box-shadow: 0 10px 25px rgba(0, 0, 0, 0.45);
  display: flex;
  flex-direction: column;
  gap: 2px;
  animation: menuAppear 0.12s ease-out;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: 5px;
  font-size: 13px;
  color: var(--text-main, #e5e7eb);
  cursor: pointer;
  user-select: none;
  transition: background 0.12s ease, color 0.12s ease;
}

.menu-item:hover {
  background: var(--surface-hover, rgba(255, 255, 255, 0.08));
  color: #fff;
}

.menu-item.text-danger {
  color: var(--danger-text, #ef4444);
}

.menu-item.text-danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}

.menu-divider {
  height: 1px;
  background: var(--border-color, rgba(255, 255, 255, 0.08));
  margin: 4px 2px;
}

@keyframes menuAppear {
  from {
    opacity: 0;
    transform: scale(0.96);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
</style>
