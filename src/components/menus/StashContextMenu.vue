<template>
  <div v-if="visible && stash" 
       class="context-menu" 
       :style="{ left: x + 'px', top: y + 'px' }"
       @click.stop>
       
    <div class="menu-item" @click="$emit('action', 'apply')">
      Apply Stash
    </div>
    <div class="menu-item" @click="$emit('action', 'drop')">
      Delete (Drop) Stash
    </div>
  </div>
</template>

<script setup lang="ts">
import { PropType } from 'vue';

const props = defineProps({
  visible: Boolean,
  x: Number,
  y: Number,
  stash: {
    type: Object as PropType<any>,
    default: null
  }
});

const emit = defineEmits(['action']);
</script>
