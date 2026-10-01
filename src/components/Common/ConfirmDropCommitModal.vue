<template>
  <div v-if="visible" class="modal-overlay" @click="close">
    <div class="modal-content drop-modal" @click.stop>
      <header class="modal-header">
        <div class="header-icon-wrapper">
          <TrashIcon viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="2" fill="none" class="danger-icon" />
        </div>
        <div class="header-text">
          <h3 class="modal-title">
            {{ targetCommits.length > 1 ? `Drop ${targetCommits.length} Commits from History` : 'Drop Commit from History' }}
          </h3>
          <p class="modal-subtitle">Excise selected commit(s) from your active branch history</p>
        </div>
        <button class="close-btn" type="button" aria-label="Close modal" @click="close">×</button>
      </header>

      <div class="modal-body">
        <!-- Target Commits List -->
        <div class="target-commits-section">
          <span class="section-label">
            {{ targetCommits.length > 1 ? `Target Commits (${targetCommits.length}):` : 'Target Commit:' }}
          </span>
          <div class="commits-scroll-list">
            <div v-for="commit in displayCommits" :key="commit.hash || commit.id" class="commit-summary-card">
              <div class="commit-card-top">
                <span class="commit-hash-pill">{{ (commit.hash || commit.id || '').substring(0, 7) }}</span>
                <span class="commit-author-tag" v-if="commit.author">{{ commit.author }}</span>
                <span class="commit-date-tag" v-if="commit.time">{{ commit.time }}</span>
              </div>
              <div class="commit-msg-text" :title="commit.message">
                {{ commit.message }}
              </div>
            </div>
            <div v-if="targetCommits.length > 4" class="more-commits-indicator">
              ...and {{ targetCommits.length - 4 }} more commit(s)
            </div>
          </div>
        </div>

        <!-- Warning Callout -->
        <div class="warning-callout">
          <div class="warning-heading">
            <AlertIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" class="alert-svg" />
            <strong>History Rewrite Operation</strong>
          </div>
          <ul class="warning-list">
            <li>Drops the selected commit(s) and replays subsequent commits on top of the parent commit.</li>
            <li>If merge conflicts occur during rebase, you can resolve them using the built-in conflict resolver.</li>
            <li>If this branch has already been pushed to a remote, you will need to force-push (<code>git push --force-with-lease</code>).</li>
            <li>Commits can still be recovered via Git reflog if needed.</li>
          </ul>
        </div>
      </div>

      <footer class="modal-footer">
        <button type="button" class="btn btn-secondary" :disabled="loading" @click="close">
          Cancel
        </button>
        <button type="button" class="btn btn-danger" :disabled="loading" @click="confirmDrop">
          <span v-if="loading" class="spinner"></span>
          <TrashIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
          <span>{{ loading ? 'Dropping Commit(s)...' : (targetCommits.length > 1 ? `Drop ${targetCommits.length} Commits` : 'Drop Commit') }}</span>
        </button>
      </footer>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invokeGit } from '../../composables/useActivityLog';
import { notify } from '../../composables/useToasts';
import TrashIcon from '../../assets/icons/context-danger-1.svg?component';
import AlertIcon from '../../assets/icons/shield-alert.svg?component';

const props = defineProps<{
  repositoryPath: string | null;
}>();

const emit = defineEmits<{
  (e: 'dropped'): void;
  (e: 'pending', message: string): void;
}>();

const visible = ref(false);
const targetCommits = ref<any[]>([]);
const loading = ref(false);

const displayCommits = computed(() => {
  return targetCommits.value.slice(0, 4);
});

function open(commits: any | any[]) {
  if (Array.isArray(commits)) {
    targetCommits.value = [...commits];
  } else if (commits) {
    targetCommits.value = [commits];
  } else {
    targetCommits.value = [];
  }
  loading.value = false;
  visible.value = true;
}

function close() {
  if (loading.value) return;
  visible.value = false;
}

async function confirmDrop() {
  if (!props.repositoryPath || targetCommits.value.length === 0) return;

  const hashes = targetCommits.value.map(c => c.hash || c.id).filter(Boolean);
  if (hashes.length === 0) return;

  loading.value = true;
  const countLabel = hashes.length > 1 ? `${hashes.length} commits` : `commit ${hashes[0].substring(0, 7)}`;
  emit('pending', `Dropping ${countLabel} from branch history...`);

  try {
    await invokeGit<void>(
      'drop_commits_from_history',
      {
        path: props.repositoryPath,
        hashes
      },
      `git rebase (drop ${countLabel})`
    );

    notify(`Successfully dropped ${countLabel} from history`, 'success');
    visible.value = false;
    emit('dropped');
  } catch (err: any) {
    notify(`Failed to drop commit(s): ${err}`, 'error');
  } finally {
    loading.value = false;
    emit('pending', '');
  }
}

defineExpose({
  open,
  close
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--overlay-bg, rgba(0, 0, 0, 0.6));
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(4px);
  animation: fadeIn 0.15s ease-out;
}

.modal-content.drop-modal {
  background: var(--panel-bg);
  border: 1px solid var(--border);
  border-radius: 10px;
  width: 90%;
  max-width: 540px;
  box-shadow: 0 16px 36px var(--shadow-color);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: slideUp 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-subtle);
}

.header-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: rgba(244, 67, 54, 0.12);
  color: var(--danger-text, #f44336);
  flex-shrink: 0;
}

.header-text {
  flex: 1;
  min-width: 0;
}

.modal-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-main);
  margin: 0;
}

.modal-subtitle {
  font-size: 12px;
  color: var(--text-muted);
  margin: 2px 0 0 0;
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.15s ease;
}

.close-btn:hover {
  color: var(--text-main);
  background: var(--surface-hover);
}

.modal-body {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
  max-height: 60vh;
}

.target-commits-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.section-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-muted);
}

.commits-scroll-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-height: 180px;
  overflow-y: auto;
}

.commit-summary-card {
  background: var(--bg-main);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.commit-card-top {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
}

.commit-hash-pill {
  font-family: monospace;
  font-weight: 700;
  color: var(--accent);
  background: var(--surface-subtle);
  padding: 1px 6px;
  border-radius: 4px;
  border: 1px solid var(--border);
}

.commit-author-tag {
  color: var(--text-main);
  font-weight: 500;
}

.commit-date-tag {
  color: var(--text-muted);
  margin-left: auto;
}

.commit-msg-text {
  font-size: 12px;
  color: var(--text-main);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-weight: 500;
}

.more-commits-indicator {
  font-size: 11px;
  color: var(--text-muted);
  text-align: center;
  padding: 4px;
  font-style: italic;
}

.warning-callout {
  background: rgba(255, 152, 0, 0.08);
  border: 1px solid rgba(255, 152, 0, 0.28);
  border-radius: 8px;
  padding: 12px 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.warning-heading {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--warning-text, #ff9800);
  font-size: 12px;
}

.alert-svg {
  flex-shrink: 0;
}

.warning-list {
  margin: 0;
  padding-left: 18px;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-main);
}

.warning-list li {
  margin-bottom: 4px;
}

.warning-list li:last-child {
  margin-bottom: 0;
}

.warning-list code {
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  padding: 1px 4px;
  border-radius: 3px;
  font-family: monospace;
  font-size: 11px;
}

.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
  background: var(--surface-subtle);
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 7px 16px;
  font-size: 12px;
  font-weight: 600;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.15s ease;
  border: 1px solid transparent;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--surface-subtle);
  border-color: var(--border);
  color: var(--text-main);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--surface-hover);
}

.btn-danger {
  background: #dc2626;
  border-color: #dc2626;
  color: #ffffff;
}

.btn-danger:hover:not(:disabled) {
  background: #b91c1c;
  border-color: #b91c1c;
}

.spinner {
  width: 13px;
  height: 13px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: #ffffff;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(12px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}
</style>
