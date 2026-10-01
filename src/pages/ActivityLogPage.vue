<script setup lang="ts">
import { computed } from 'vue';
import { useActivityLog } from '../composables/useActivityLog';

const props = defineProps<{ repositoryPath: string | null }>();
const { entries, clearActivityLog } = useActivityLog();
const visibleEntries = computed(() => entries.value.filter(entry => !props.repositoryPath || entry.path === props.repositoryPath));

function formatTime(timestamp: number) {
  return new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' }).format(timestamp);
}
</script>

<template>
  <section class="activity-log-page" aria-label="Activity logs">
    <header class="activity-log-header">
      <div><h1>Activity Logs</h1><p>Git commands run from BeGit for this repository.</p></div>
      <button :disabled="!visibleEntries.length" @click="clearActivityLog(repositoryPath || undefined)">Clear logs</button>
    </header>

    <div v-if="!visibleEntries.length" class="activity-log-empty">Commands you run from BeGit will appear here.</div>
    <ol v-else class="activity-log-list">
      <li v-for="entry in visibleEntries" :key="entry.id" class="activity-log-entry" :class="`is-${entry.status}`">
        <time :datetime="new Date(entry.timestamp).toISOString()">{{ formatTime(entry.timestamp) }}</time>
        <span class="activity-status" :title="entry.status">{{ entry.status === 'running' ? 'Running' : entry.status === 'success' ? 'Done' : 'Failed' }}</span>
        <div><code>{{ entry.command }}</code><p v-if="entry.error">{{ entry.error }}</p></div>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.activity-log-page { flex: 1; min-width: 0; overflow: auto; padding: 28px; background: var(--bg-color); color: var(--text-main); }
.activity-log-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; padding-bottom: 20px; border-bottom: 1px solid var(--border); }
.activity-log-header h1 { font-size: 1.15rem; }.activity-log-header p { margin-top: 5px; color: var(--text-muted); font-size: .84rem; }.activity-log-header button { border: 1px solid var(--border); border-radius: 5px; padding: 6px 10px; background: var(--surface-subtle); color: var(--text-main); cursor: pointer; }.activity-log-header button:disabled { opacity: .5; cursor: default; }
.activity-log-empty { display: grid; min-height: 220px; place-items: center; color: var(--text-muted); font-size: .9rem; }.activity-log-list { list-style: none; margin-top: 16px; }.activity-log-entry { display: grid; grid-template-columns: 74px 64px minmax(0, 1fr); gap: 12px; align-items: start; padding: 11px 8px; border-bottom: 1px solid var(--border); font-size: .82rem; }.activity-log-entry time { color: var(--text-muted); font-variant-numeric: tabular-nums; }.activity-status { font-weight: 650; color: var(--info-text); }.is-success .activity-status { color: var(--success-text); }.is-error .activity-status { color: var(--danger-text); }.activity-log-entry code { color: var(--text-main); font: inherit; }.activity-log-entry p { margin-top: 5px; color: var(--danger-text); white-space: pre-wrap; overflow-wrap: anywhere; }
</style>
