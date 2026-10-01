<template>
  <div v-if="visible && tag" 
       class="context-menu" 
       :style="{ left: x + 'px', top: y + 'px' }"
       @click.stop>
       
    <div class="menu-item" @click="$emit('action', 'checkout')">
      Checkout tag ({{ tag }})
    </div>
    <div class="menu-item" @click="$emit('action', 'push')">
      Push tag to remote
    </div>
    <div class="menu-item" @click="$emit('action', 'push_all')">
      Push all tags to remote
    </div>
    <div class="menu-divider"></div>
    <div class="menu-item" @click="$emit('action', 'copy_name')">
      Copy tag name
    </div>
    <div class="menu-divider"></div>
    <div class="menu-item danger" @click="$emit('action', 'delete')">
      Delete tag...
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  visible: boolean;
  x: number;
  y: number;
  tag: string | null;
}>();

defineEmits<{
  action: [action: 'checkout' | 'push' | 'push_all' | 'copy_name' | 'delete'];
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
