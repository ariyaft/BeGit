<script setup lang="ts">
import { useToasts } from '../../composables/useToasts';

const { toasts, dismissToast } = useToasts();
</script>

<template>
  <div class="toast-region" aria-live="polite" aria-atomic="true">
    <TransitionGroup name="toast">
      <article v-for="toast in toasts" :key="toast.id" class="toast" :class="`toast-${toast.kind}`" role="status">
        <p>{{ toast.message }}</p>
        <button type="button" aria-label="Dismiss notification" @click="dismissToast(toast.id)">×</button>
      </article>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-region {
  position: fixed;
  right: 16px;
  bottom: 16px;
  z-index: 10000;
  display: grid;
  gap: 8px;
  width: min(420px, calc(100vw - 32px));
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 11px 12px 11px 14px;
  border: 1px solid var(--border);
  border-left: 3px solid var(--info-text);
  border-radius: 7px;
  background: var(--panel-bg);
  box-shadow: 0 8px 24px rgba(0, 0, 0, .24);
  color: var(--text-main);
  pointer-events: auto;
}

.toast-error { border-left-color: var(--danger-text); }
.toast-success { border-left-color: var(--success-text); }

.toast p { margin: 0; flex: 1; font-size: .82rem; line-height: 1.4; white-space: pre-wrap; overflow-wrap: anywhere; }
.toast button { border: 0; background: transparent; color: var(--text-muted); cursor: pointer; font-size: 1.25rem; line-height: 1; padding: 0; }
.toast button:hover { color: var(--text-main); }

.toast-enter-active, .toast-leave-active { transition: opacity .18s ease, transform .18s ease; }
.toast-enter-from, .toast-leave-to { opacity: 0; transform: translateX(12px); }
</style>
