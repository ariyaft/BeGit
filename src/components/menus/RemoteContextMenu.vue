<template>
  <div v-if="visible && remote" 
       class="context-menu" 
       :style="{ left: x + 'px', top: y + 'px' }"
       @click.stop>
       
    <div class="menu-item" @click="$emit('action', 'edit_url')">
      Edit URL ({{ remote.name }})
    </div>
    <div class="menu-item" @click="$emit('action', 'remove')">
      Remove remote
    </div>
  </div>
</template>

<script setup lang="ts">
import { PropType } from 'vue';

const props = defineProps({
  visible: Boolean,
  x: Number,
  y: Number,
  remote: {
    type: Object as PropType<any>,
    default: null
  }
});

const emit = defineEmits(['action']);
</script>

<style scoped>
/* Inherit styles from ContextMenu via App.css if possible, or add them here */
</style>
