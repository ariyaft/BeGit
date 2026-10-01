<template>
  <div v-if="visible" class="modal-overlay" @click="close">
    <div class="modal-content purge-modal" @click.stop>
      <header class="modal-header">
        <div class="header-icon-wrapper">
          <TrashIcon viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" class="danger-icon" />
        </div>
        <div class="header-text">
          <h3 class="modal-title">Purge File from History</h3>
          <p class="modal-subtitle">Permanently erase this file across all commits and branches</p>
        </div>
        <button class="close-btn" type="button" aria-label="Close modal" @click="close">×</button>
      </header>

      <div class="modal-body">
        <div class="target-file-box">
          <span class="file-label">Target File:</span>
          <code class="file-path">{{ filePath }}</code>
        </div>

        <div class="warning-callout">
          <div class="warning-heading">
            <AlertIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" class="alert-svg" />
            <strong>Irreversible Git Operation</strong>
          </div>
          <ul class="warning-list">
            <li>Rewrites all commits in repository history (all branches &amp; tags) to remove this file.</li>
            <li>Deletes the file from your local working directory if present.</li>
            <li>Commit SHA hashes will change. You will need to force push (<code>git push --force-with-lease</code>) to update remote repositories.</li>
          </ul>
        </div>

        <label class="gitignore-option">
          <input type="checkbox" v-model="addToGitignore" class="checkbox-input" />
          <span class="checkbox-label">
            <strong>Add to .gitignore</strong>
            <span class="option-desc">Prevent this file path from being tracked in future commits.</span>
          </span>
        </label>
      </div>

      <footer class="modal-footer">
        <button type="button" class="btn btn-secondary" :disabled="loading" @click="close">
          Cancel
        </button>
        <button type="button" class="btn btn-danger" :disabled="loading" @click="confirmPurge">
          <span v-if="loading" class="spinner"></span>
          <TrashIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
          <span>{{ loading ? 'Purging History...' : 'Purge from Entire History' }}</span>
        </button>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invokeGit } from '../../composables/useActivityLog';
import { notify } from '../../composables/useToasts';
import TrashIcon from '../../assets/icons/context-danger-1.svg?component';
import AlertIcon from '../../assets/icons/shield-alert.svg?component';

const props = defineProps<{
  repositoryPath: string | null;
}>();

const emit = defineEmits<{
  (e: 'purged', filePath: string): void;
  (e: 'pending', message: string): void;
}>();

const visible = ref(false);
const filePath = ref('');
const addToGitignore = ref(true);
const loading = ref(false);

function open(path: string) {
  filePath.value = path;
  addToGitignore.value = true;
  loading.value = false;
  visible.value = true;
}

function close() {
  if (loading.value) return;
  visible.value = false;
}

async function confirmPurge() {
  if (!props.repositoryPath || !filePath.value) return;

  const targetPath = filePath.value;
  loading.value = true;
  emit('pending', `Purging '${targetPath}' from all Git commits...`);

  try {
    await invokeGit<void>(
      'purge_file_from_history',
      {
        path: props.repositoryPath,
        filePath: targetPath,
        addToGitignore: addToGitignore.value,
      },
      `git purge file: ${targetPath}`
    );

    notify(`Successfully purged '${targetPath}' from repository history.`, 'success');
    visible.value = false;
    emit('purged', targetPath);
  } catch (error: any) {
    notify(error?.toString() || 'Failed to purge file from history.', 'error');
  } finally {
    loading.value = false;
    emit('pending', '');
  }
}

defineExpose({ open, close });
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  z-index: 9998;
  display: grid;
  place-items: center;
  padding: 16px;
  animation: fadeIn 0.15s ease-out;
}

.modal-content.purge-modal {
  background: var(--bg-surface, #1e1e24);
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.1));
  border-radius: 12px;
  box-shadow: 0 16px 36px rgba(0, 0, 0, 0.5);
  width: 100%;
  max-width: 480px;
  color: var(--text-main, #f0f0f0);
  font-family: inherit;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 20px 20px 16px;
  border-bottom: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
}

.header-icon-wrapper {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: rgba(239, 68, 68, 0.15);
  display: grid;
  place-items: center;
  flex-shrink: 0;
}

.danger-icon {
  color: var(--danger-text, #ef4444);
}

.header-text {
  flex: 1;
}

.modal-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-main, #f0f0f0);
}

.modal-subtitle {
  margin: 4px 0 0;
  font-size: 12px;
  color: var(--text-muted, #9ca3af);
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-muted, #9ca3af);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.close-btn:hover {
  color: var(--text-main, #fff);
  background: rgba(255, 255, 255, 0.05);
}

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.target-file-box {
  background: var(--bg-alt, rgba(0, 0, 0, 0.25));
  border: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  border-radius: 6px;
  padding: 10px 14px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.file-label {
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-muted, #9ca3af);
  font-weight: 600;
}

.file-path {
  font-family: var(--font-mono, monospace);
  font-size: 13px;
  color: var(--accent-cyan, #38bdf8);
  word-break: break-all;
}

.warning-callout {
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.25);
  border-radius: 8px;
  padding: 14px 16px;
}

.warning-heading {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--danger-text, #ef4444);
  font-size: 13px;
  font-weight: 600;
  margin-bottom: 8px;
}

.alert-svg {
  flex-shrink: 0;
}

.warning-list {
  margin: 0;
  padding-left: 18px;
  color: var(--text-main, #e5e7eb);
  font-size: 12px;
  line-height: 1.5;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.warning-list code {
  background: rgba(0, 0, 0, 0.3);
  padding: 2px 5px;
  border-radius: 3px;
  font-family: var(--font-mono, monospace);
  font-size: 11px;
}

.gitignore-option {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  cursor: pointer;
  padding: 8px;
  border-radius: 6px;
  transition: background 0.15s ease;
  user-select: none;
}

.gitignore-option:hover {
  background: var(--surface-hover, rgba(255, 255, 255, 0.03));
}

.checkbox-input {
  margin-top: 3px;
  accent-color: var(--accent, #3b82f6);
  cursor: pointer;
}

.checkbox-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 13px;
}

.option-desc {
  font-size: 11px;
  color: var(--text-muted, #9ca3af);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding: 16px 20px;
  border-top: 1px solid var(--border-color, rgba(255, 255, 255, 0.08));
  background: var(--bg-surface-footer, rgba(0, 0, 0, 0.15));
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.15s ease;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--surface-secondary, rgba(255, 255, 255, 0.08));
  border-color: var(--border-color, rgba(255, 255, 255, 0.1));
  color: var(--text-main, #f0f0f0);
}

.btn-secondary:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.12);
}

.btn-danger {
  background: var(--danger-button, #dc2626);
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #b91c1c;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}
</style>
