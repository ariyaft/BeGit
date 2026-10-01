<script setup lang="ts">
interface ChangedFile {
  status: string;
  path: string;
}

interface Commit {
  hash: string;
  id: string;
  message: string;
  author: string;
  time: string;
  files?: ChangedFile[];
}

defineProps<{
  commit: Commit;
  loading: boolean;
}>();

defineEmits<{ selectFile: [path: string] }>();

function statusLabel(status: string) {
  return status?.charAt(0) || 'M';
}

function statusClass(status: string) {
  const value = statusLabel(status);
  return value === 'A' ? 'added' : value === 'D' ? 'deleted' : 'modified';
}
</script>

<template>
  <section class="timeline-changes-tray" aria-label="Selected commit changes">
    <div class="tray-commit">
      <div class="tray-eyebrow">Selected commit</div>
      <div class="tray-message" :title="commit.message">{{ commit.message }}</div>
      <div class="tray-meta">
        <span>{{ commit.author }}</span>
        <span>{{ commit.time }}</span>
        <code :title="commit.id">{{ commit.hash }}</code>
      </div>
    </div>

    <div class="tray-files">
      <div class="tray-files-heading">
        <span>Changes</span>
        <span v-if="loading" class="tray-loading">Loading…</span>
        <span v-else class="tray-count">{{ commit.files?.length || 0 }} files</span>
      </div>
      <div v-if="loading" class="tray-files-empty">Reading changed files…</div>
      <div v-else-if="!commit.files?.length" class="tray-files-empty">No file changes recorded for this commit.</div>
      <div v-else class="tray-file-list">
        <button
          v-for="file in commit.files"
          :key="`${file.status}-${file.path}`"
          class="tray-file"
          type="button"
          :title="`Open diff for ${file.path}`"
          @click="$emit('selectFile', file.path)"
        >
          <span class="file-status" :class="statusClass(file.status)">{{ statusLabel(file.status) }}</span>
          <span class="file-path">{{ file.path }}</span>
        </button>
      </div>
    </div>
  </section>
</template>

<style scoped>
.timeline-changes-tray {
  min-height: 150px;
  max-height: 240px;
  flex: 0 0 auto;
  display: grid;
  grid-template-columns: minmax(280px, 35%) minmax(0, 1fr);
  border-top: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  background: color-mix(in srgb, var(--panel-bg) 75%, transparent);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  color: var(--text-main);
  box-shadow: 0 -8px 30px rgba(0, 0, 0, 0.15);
  z-index: 10;
  position: relative;
}

.tray-commit {
  min-width: 0;
  padding: 20px 24px;
  border-right: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.tray-eyebrow {
  margin-bottom: 8px;
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  background: linear-gradient(90deg, var(--accent, #00bcd4), #9c27b0);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.tray-message {
  overflow: hidden;
  color: var(--text-main);
  font-size: 15px;
  font-weight: 700;
  line-height: 1.4;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  white-space: normal;
  margin-bottom: 12px;
}

.tray-meta {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px 12px;
  color: var(--text-muted);
  font-size: 12px;
  font-weight: 500;
}

.tray-meta code {
  color: var(--text-main);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  background: color-mix(in srgb, var(--border) 40%, transparent);
  padding: 3px 8px;
  border-radius: 6px;
  font-size: 11px;
  border: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.1);
}

.tray-files {
  min-width: 0;
  padding: 16px 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  overflow: hidden;
}

.tray-files-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-main);
  font-size: 13px;
  font-weight: 700;
}

.tray-count, .tray-loading {
  color: var(--text-main);
  background: color-mix(in srgb, var(--bg-color) 60%, transparent);
  padding: 2px 10px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 600;
  border: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
}

.tray-loading {
  color: var(--accent, #00bcd4);
  background: color-mix(in srgb, var(--accent) 15%, transparent);
  border-color: color-mix(in srgb, var(--accent) 30%, transparent);
  animation: pulse-loading 1.5s infinite;
}

@keyframes pulse-loading {
  0% { opacity: 0.6; }
  50% { opacity: 1; }
  100% { opacity: 0.6; }
}

.tray-file-list {
  display: flex;
  flex-wrap: wrap;
  align-content: flex-start;
  gap: 8px;
  overflow-y: auto;
  padding-right: 4px;
  scrollbar-width: thin;
  scrollbar-color: var(--border) transparent;
}

.tray-file-list::-webkit-scrollbar {
  width: 6px;
}
.tray-file-list::-webkit-scrollbar-thumb {
  background-color: var(--border);
  border-radius: 3px;
}

.tray-file {
  min-width: 0;
  max-width: 300px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--surface-subtle, var(--bg-color)) 70%, transparent);
  color: var(--text-main);
  cursor: pointer;
  font: inherit;
  font-size: 12px;
  font-weight: 500;
  transition: all 0.2s cubic-bezier(0.2, 0.8, 0.2, 1);
  box-shadow: 0 2px 5px rgba(0,0,0,0.05);
}

.tray-file:hover {
  border-color: var(--accent, #00bcd4);
  background: color-mix(in srgb, var(--accent) 10%, transparent);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1), 0 0 0 1px color-mix(in srgb, var(--accent) 30%, transparent);
}

.file-status {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 18px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 10px;
  font-weight: 800;
  border-radius: 4px;
}

.file-status.added {
  color: var(--success-text, #3fb950);
  background: color-mix(in srgb, var(--success-text, #3fb950) 15%, transparent);
}

.file-status.deleted {
  color: var(--danger-text, #f85149);
  background: color-mix(in srgb, var(--danger-text, #f85149) 15%, transparent);
}

.file-status.modified {
  color: var(--warning-text, #d29922);
  background: color-mix(in srgb, var(--warning-text, #d29922) 15%, transparent);
}

.file-path {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tray-files-empty {
  color: var(--text-muted);
  font-size: 13px;
  font-weight: 500;
  font-style: italic;
  padding: 8px 0;
}

@media (max-width: 720px) {
  .timeline-changes-tray {
    grid-template-columns: 1fr;
    max-height: 320px;
  }
  .tray-commit {
    padding: 16px 20px;
    border-right: 0;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  }
  .tray-files {
    padding: 16px 20px;
  }
}
</style>
