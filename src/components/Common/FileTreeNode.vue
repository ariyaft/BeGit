<template>
  <div class="tree-node">
    <div 
      class="tree-node-label" 
      :class="{ 'is-dir': node.isDir, 'is-selected': selectedPath === node.path }"
      :style="{ paddingLeft: `${depth * 14 + 12}px` }"
      @click="onClick"
      @contextmenu.prevent.stop="onContextMenu"
    >
      <div class="tree-toggle" v-if="node.isDir">
        <ChevronDownIcon v-if="isExpanded" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        <ChevronRightIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
      </div>
      <div class="tree-toggle placeholder" v-else></div>

      <span class="node-icon" :class="{ 'folder-icon': node.isDir, 'file-icon': !node.isDir }">
        <FolderIcon v-if="node.isDir" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2.5" fill="none" />
        <FileTypeIcon v-else :file-name="node.name" />
      </span>

      <span class="node-name" :title="node.path">{{ node.name }}</span>

      <button
        type="button"
        class="node-action-btn"
        title="File Options"
        aria-label="File Options"
        @click.stop="onActionMenuClick"
      >
        <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
      </button>
    </div>

    <div v-if="node.isDir && isExpanded" class="tree-children">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        :selected-path="selectedPath"
        :force-expanded="forceExpanded"
        @select="$emit('select', $event)"
        @contextmenu="(n, e) => $emit('contextmenu', n, e)"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import ChevronDownIcon from '../../assets/icons/chevron-down.svg?component';
import ChevronRightIcon from '../../assets/icons/chevron-right.svg?component';
import FolderIcon from '../../assets/icons/folder.svg?component';
import MoreVerticalIcon from '../../assets/icons/more-vertical.svg?component';
import FileTypeIcon from './FileTypeIcon.vue';

export interface FileNode {
  name: string;
  path: string;
  isDir: boolean;
  children?: FileNode[];
}

const props = defineProps<{
  node: FileNode;
  depth: number;
  selectedPath: string | null;
  forceExpanded?: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', path: string): void;
  (e: 'contextmenu', node: FileNode, event: MouseEvent): void;
}>();

const collapsed = ref(true);
const isExpanded = computed(() => props.forceExpanded || !collapsed.value);

function onClick() {
  if (props.node.isDir) {
    collapsed.value = !collapsed.value;
  } else {
    emit('select', props.node.path);
  }
}

function onContextMenu(e: MouseEvent) {
  emit('contextmenu', props.node, e);
}

function onActionMenuClick(e: MouseEvent) {
  emit('contextmenu', props.node, e);
}
</script>

<style scoped>
.tree-node-label {
  display: grid;
  grid-template-columns: 20px 20px minmax(0, 1fr) 22px;
  column-gap: 6px;
  align-items: center;
  min-height: 30px;
  padding: 5px 6px 5px 0;
  cursor: pointer;
  user-select: none;
  border-radius: 4px;
  color: var(--text-main);
  font-size: 13px;
  margin: 1px 8px;
  position: relative;
}
.tree-node-label:hover {
  background: var(--surface-hover);
}
.tree-node-label.is-selected {
  background: var(--accent-blue, var(--accent));
  color: white;
}
.tree-toggle {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.7;
}
.tree-toggle.placeholder {
  visibility: hidden;
}
.node-icon {
  width: 20px;
  height: 20px;
  display: grid;
  place-items: center;
  opacity: 0.8;
}
.node-icon :deep(.file-type-icon) {
  width: 18px;
  height: 18px;
  flex-basis: 18px;
}
.folder-icon {
  color: var(--accent-blue, var(--accent));
}
.is-selected .folder-icon,
.is-selected .tree-toggle {
  color: white;
  opacity: 1;
}
.is-selected .file-icon {
  opacity: 1;
}
.node-name {
  min-width: 0;
  line-height: 20px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.node-action-btn {
  width: 22px;
  height: 22px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s ease, background 0.15s ease, color 0.15s ease;
  padding: 0;
}
.tree-node-label:hover .node-action-btn,
.tree-node-label.is-selected .node-action-btn {
  opacity: 0.75;
}
.node-action-btn:hover {
  opacity: 1 !important;
  background: rgba(255, 255, 255, 0.15);
  color: var(--text-main);
}
.is-selected .node-action-btn {
  color: white;
}
</style>

