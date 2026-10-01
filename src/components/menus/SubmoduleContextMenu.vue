<template>
  <div v-if="visible && submodule" 
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
    <div class="menu-item" @click="$emit('action', 'init')">
      Initialize submodule
    </div>
    <div class="menu-item" @click="$emit('action', 'update')">
      Update from remote
    </div>
    <div class="menu-item" @click="$emit('action', 'sync')">
      Sync URL
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SubmoduleInfo } from '../../types';

defineProps<{
  visible: boolean;
  x: number;
  y: number;
  submodule: SubmoduleInfo | null;
}>();

defineEmits<{
  action: [action: 'open_tab' | 'reveal' | 'init' | 'update' | 'sync'];
}>();
</script>

<style scoped>
</style>
