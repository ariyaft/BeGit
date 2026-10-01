<template>
  <div v-if="visible && worktree" 
       class="context-menu" 
       :style="{ left: x + 'px', top: y + 'px' }"
       @click.stop>
       
    <div class="menu-item" @click="$emit('action', 'open_tab')">
      Open in new tab
    </div>
    <div class="menu-item" @click="$emit('action', 'reveal')">
      Reveal in Finder / Explorer
    </div>
    <div class="menu-divider"></div>
    <div v-if="!worktree.is_locked" class="menu-item" @click="$emit('action', 'lock')">
      Lock worktree...
    </div>
    <div v-else class="menu-item" @click="$emit('action', 'unlock')">
      Unlock worktree
    </div>
    <template v-if="!worktree.is_main">
      <div class="menu-divider"></div>
      <div class="menu-item danger" @click="$emit('action', 'remove')">
        Remove worktree...
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import type { WorktreeInfo } from '../../types';

defineProps<{
  visible: boolean;
  x: number;
  y: number;
  worktree: WorktreeInfo | null;
}>();

defineEmits<{
  action: [action: 'open_tab' | 'reveal' | 'lock' | 'unlock' | 'remove'];
}>();
</script>

<style scoped>
.menu-item.danger {
  color: var(--accent-red, #ff5252);
}
.menu-item.danger:hover {
  background: rgba(255, 82, 82, 0.15);
}
</style>
