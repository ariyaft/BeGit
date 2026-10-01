<script setup lang="ts">
defineProps<{ message: string }>();
</script>

<template>
  <div v-if="message" class="global-pending-overlay" role="status" aria-live="polite">
    <div class="pending-content">
      <div class="spinner"></div>
      <div class="pending-message">{{ message }}</div>
    </div>
  </div>
</template>
