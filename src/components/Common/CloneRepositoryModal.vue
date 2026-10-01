<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import CloneIcon from '../../assets/icons/clone.svg?component';
import FolderIcon from '../../assets/icons/folder.svg?component';
import KeyIcon from '../../assets/icons/key.svg?component';
import ChevronRightIcon from '../../assets/icons/chevron-right.svg?component';
import ChevronDownIcon from '../../assets/icons/chevron-down.svg?component';
import { invokeGit } from '../../composables/useActivityLog';
import { notify } from '../../composables/useToasts';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'cloned', targetPath: string): void;
}>();

const repoUrl = ref('');
const destDir = ref('');
const folderName = ref('');
const branch = ref('');
const isShallow = ref(false);
const depth = ref(1);
const showAdvanced = ref(false);

// Authentication State
const authType = ref<'none' | 'token' | 'ssh_key'>('none');
const token = ref('');
const username = ref('');
const sshKeyPath = ref('');
const showToken = ref(false);
const showAuthSection = ref(false);

const isCloning = ref(false);
const cloneError = ref<string | null>(null);
const userHasEditedFolderName = ref(false);

const STORAGE_KEY_LAST_DEST = 'begit_last_clone_dest';

function extractRepoName(url: string): string {
  const cleaned = url.trim().replace(/\/+$/, '');
  if (!cleaned) return '';
  const match = cleaned.match(/[/:][^/:]+?\/([^/]+?)(?:\.git)?$/) || cleaned.match(/\/([^/]+?)(?:\.git)?$/);
  if (match && match[1]) {
    return match[1].replace(/\.git$/, '');
  }
  const parts = cleaned.split(/[/:]/);
  const last = parts[parts.length - 1];
  return (last || '').replace(/\.git$/, '');
}

function handleUrlInput() {
  if (!userHasEditedFolderName.value) {
    folderName.value = extractRepoName(repoUrl.value);
  }
  const trimmed = repoUrl.value.trim();
  if ((trimmed.startsWith('git@') || trimmed.startsWith('ssh://')) && authType.value === 'none') {
    // Keep none as system SSH agent default, but open auth if requested
  }
}

function handleFolderNameInput() {
  userHasEditedFolderName.value = true;
}

const effectiveTargetDir = computed(() => {
  const dest = destDir.value.trim();
  const name = folderName.value.trim();
  if (!dest && !name) return '';
  if (!dest) return name;
  if (!name) return dest;
  const isWindows = dest.includes('\\');
  const sep = isWindows ? '\\' : '/';
  const cleanDest = dest.endsWith(sep) ? dest.slice(0, -1) : dest;
  return `${cleanDest}${sep}${name}`;
});

const isValid = computed(() => {
  if (
    repoUrl.value.trim().length === 0 ||
    destDir.value.trim().length === 0 ||
    folderName.value.trim().length === 0 ||
    isCloning.value
  ) {
    return false;
  }

  if (authType.value === 'token' && token.value.trim().length === 0) {
    return false;
  }

  if (authType.value === 'ssh_key' && sshKeyPath.value.trim().length === 0) {
    return false;
  }

  return true;
});

async function chooseDestination() {
  if (isCloning.value) return;
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Destination Folder'
    });
    if (selected && typeof selected === 'string') {
      destDir.value = selected;
      localStorage.setItem(STORAGE_KEY_LAST_DEST, selected);
    }
  } catch (err) {
    console.error('Failed to open destination folder dialog:', err);
  }
}

async function chooseSshKey() {
  if (isCloning.value) return;
  try {
    const selected = await open({
      directory: false,
      multiple: false,
      title: 'Select SSH Private Key File (e.g. ~/.ssh/id_ed25519)'
    });
    if (selected && typeof selected === 'string') {
      sshKeyPath.value = selected;
    }
  } catch (err) {
    console.error('Failed to open SSH key dialog:', err);
  }
}

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      cloneError.value = null;
      isCloning.value = false;
      const savedDest = localStorage.getItem(STORAGE_KEY_LAST_DEST);
      if (savedDest && !destDir.value) {
        destDir.value = savedDest;
      }
    }
  },
  { immediate: true }
);

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.visible && !isCloning.value) {
    emit('close');
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
  const savedDest = localStorage.getItem(STORAGE_KEY_LAST_DEST);
  if (savedDest) {
    destDir.value = savedDest;
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
});

async function startClone() {
  if (!isValid.value) return;
  
  const targetPath = effectiveTargetDir.value;
  const url = repoUrl.value.trim();
  const branchName = branch.value.trim() || undefined;
  const cloneDepth = isShallow.value && depth.value > 0 ? depth.value : undefined;
  const currentAuthType = authType.value !== 'none' ? authType.value : undefined;
  const usernameVal = authType.value === 'token' && username.value.trim() ? username.value.trim() : undefined;
  const tokenVal = authType.value === 'token' && token.value.trim() ? token.value.trim() : undefined;
  const sshKeyVal = authType.value === 'ssh_key' && sshKeyPath.value.trim() ? sshKeyPath.value.trim() : undefined;

  isCloning.value = true;
  cloneError.value = null;

  try {
    localStorage.setItem(STORAGE_KEY_LAST_DEST, destDir.value.trim());

    await invokeGit<string>(
      'clone_repository',
      {
        url,
        targetPath,
        branch: branchName,
        depth: cloneDepth,
        authType: currentAuthType,
        username: usernameVal,
        token: tokenVal,
        sshKeyPath: sshKeyVal
      },
      `git clone ${url}${authType.value === 'token' ? ' (authenticated)' : (authType.value === 'ssh_key' ? ' (with SSH key)' : '')}`
    );

    // Reset fields on success
    repoUrl.value = '';
    folderName.value = '';
    userHasEditedFolderName.value = false;
    branch.value = '';
    isShallow.value = false;
    showAdvanced.value = false;
    token.value = '';
    username.value = '';
    sshKeyPath.value = '';
    authType.value = 'none';
    showAuthSection.value = false;

    emit('cloned', targetPath);
  } catch (err: any) {
    const errorMsg = typeof err === 'string' ? err : err?.message || 'Failed to clone repository';
    cloneError.value = errorMsg;
    
    // Auto-open authentication section on auth error
    if (
      errorMsg.includes('Authentication') ||
      errorMsg.includes('Permission denied') ||
      errorMsg.includes('not found') ||
      errorMsg.includes('Username')
    ) {
      showAuthSection.value = true;
      if (authType.value === 'none') {
        authType.value = 'token';
      }
    }
    
    notify(errorMsg, 'error');
  } finally {
    isCloning.value = false;
  }
}
</script>

<template>
  <div v-if="visible" class="clone-modal-backdrop" @mousedown.self="!isCloning && $emit('close')">
    <form class="clone-modal" @submit.prevent="startClone">
      <header>
        <div class="clone-icon">
          <CloneIcon viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="1.8" fill="none" />
        </div>
        <div class="header-text">
          <h2>Clone a Repository</h2>
          <p>Clone a public or private Git repository to your local machine.</p>
        </div>
        <button
          type="button"
          class="clone-close"
          :disabled="isCloning"
          @click="$emit('close')"
          aria-label="Close dialog"
        >
          &times;
        </button>
      </header>

      <div class="form-body">
        <!-- Repository URL -->
        <div class="form-group">
          <label for="clone-repo-url">Repository URL</label>
          <div class="input-with-hint">
            <input
              id="clone-repo-url"
              v-model="repoUrl"
              type="text"
              placeholder="https://github.com/owner/repo.git or git@github.com:owner/repo.git"
              :disabled="isCloning"
              required
              autofocus
              @input="handleUrlInput"
            />
          </div>
        </div>

        <!-- Destination Directory -->
        <div class="form-group">
          <label for="clone-dest-dir">Destination Folder</label>
          <div class="dest-picker-row">
            <input
              id="clone-dest-dir"
              v-model="destDir"
              type="text"
              placeholder="Select parent folder..."
              :disabled="isCloning"
              required
            />
            <button
              type="button"
              class="browse-btn"
              :disabled="isCloning"
              @click="chooseDestination"
              title="Browse for destination folder"
            >
              <FolderIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              Browse…
            </button>
          </div>
        </div>

        <!-- Repository Folder Name -->
        <div class="form-group">
          <label for="clone-folder-name">Directory Name</label>
          <input
            id="clone-folder-name"
            v-model="folderName"
            type="text"
            placeholder="Folder name"
            :disabled="isCloning"
            required
            @input="handleFolderNameInput"
          />
        </div>

        <!-- Effective Path Preview -->
        <div v-if="effectiveTargetDir" class="target-path-preview">
          <span class="preview-label">Will clone to:</span>
          <span class="preview-path" :title="effectiveTargetDir">{{ effectiveTargetDir }}</span>
        </div>

        <!-- Authentication Section (for Private Repositories) -->
        <div class="auth-section" :class="{ 'has-auth': authType !== 'none' || showAuthSection }">
          <button
            type="button"
            class="auth-toggle"
            :disabled="isCloning"
            @click="showAuthSection = !showAuthSection"
          >
            <KeyIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" class="auth-icon" />
            <span class="auth-toggle-title">Authentication (Private Repository)</span>
            <span v-if="authType !== 'none'" class="auth-badge">{{ authType === 'token' ? 'Token' : 'SSH Key' }}</span>
            <ChevronDownIcon v-if="showAuthSection" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" class="chevron-icon" />
            <ChevronRightIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" class="chevron-icon" />
          </button>

          <div v-if="showAuthSection" class="auth-fields">
            <div class="auth-types-grid">
              <label class="auth-type-card" :class="{ active: authType === 'none' }">
                <input v-model="authType" type="radio" value="none" :disabled="isCloning" />
                <div class="card-info">
                  <strong>System / Default</strong>
                  <span>Existing SSH agent or OS credential helper</span>
                </div>
              </label>

              <label class="auth-type-card" :class="{ active: authType === 'token' }">
                <input v-model="authType" type="radio" value="token" :disabled="isCloning" />
                <div class="card-info">
                  <strong>Personal Access Token</strong>
                  <span>GitHub PAT, GitLab Token, or App Password</span>
                </div>
              </label>

              <label class="auth-type-card" :class="{ active: authType === 'ssh_key' }">
                <input v-model="authType" type="radio" value="ssh_key" :disabled="isCloning" />
                <div class="card-info">
                  <strong>Custom SSH Key</strong>
                  <span>Specify private key for git@... URLs</span>
                </div>
              </label>
            </div>

            <!-- Token Authentication Inputs -->
            <div v-if="authType === 'token'" class="auth-inputs-panel">
              <div class="form-group">
                <label for="clone-auth-token">Personal Access Token / Password <span class="required-star">*</span></label>
                <div class="password-input-row">
                  <input
                    id="clone-auth-token"
                    v-model="token"
                    :type="showToken ? 'text' : 'password'"
                    placeholder="ghp_... or glpat-... or password"
                    :disabled="isCloning"
                    required
                  />
                  <button
                    type="button"
                    class="toggle-pwd-btn"
                    :disabled="isCloning"
                    @click="showToken = !showToken"
                    :title="showToken ? 'Hide token' : 'Show token'"
                  >
                    {{ showToken ? 'Hide' : 'Show' }}
                  </button>
                </div>
                <span class="field-hint">Token is used for cloning and never stored in plain text.</span>
              </div>

              <div class="form-group">
                <label for="clone-auth-username">Username <span class="optional-label">(optional)</span></label>
                <input
                  id="clone-auth-username"
                  v-model="username"
                  type="text"
                  placeholder="e.g. your username (can leave blank for GitHub)"
                  :disabled="isCloning"
                />
              </div>
            </div>

            <!-- SSH Key Authentication Inputs -->
            <div v-if="authType === 'ssh_key'" class="auth-inputs-panel">
              <div class="form-group">
                <label for="clone-ssh-key">SSH Private Key Path <span class="required-star">*</span></label>
                <div class="dest-picker-row">
                  <input
                    id="clone-ssh-key"
                    v-model="sshKeyPath"
                    type="text"
                    placeholder="~/.ssh/id_ed25519 or ~/.ssh/id_rsa"
                    :disabled="isCloning"
                    required
                  />
                  <button
                    type="button"
                    class="browse-btn"
                    :disabled="isCloning"
                    @click="chooseSshKey"
                    title="Browse for private key file"
                  >
                    <FolderIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
                    Browse…
                  </button>
                </div>
                <span class="field-hint">Select the private key file corresponding to your public key on GitHub/GitLab.</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Advanced Options Toggle -->
        <div class="advanced-section">
          <button
            type="button"
            class="advanced-toggle"
            :disabled="isCloning"
            @click="showAdvanced = !showAdvanced"
          >
            <ChevronDownIcon v-if="showAdvanced" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            <ChevronRightIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            Advanced Options
          </button>

          <div v-if="showAdvanced" class="advanced-fields">
            <div class="form-group">
              <label for="clone-branch">Specific Branch / Tag (Optional)</label>
              <input
                id="clone-branch"
                v-model="branch"
                type="text"
                placeholder="e.g. main, develop, or v1.0.0"
                :disabled="isCloning"
              />
            </div>

            <div class="shallow-row">
              <label class="checkbox-label">
                <input
                  v-model="isShallow"
                  type="checkbox"
                  :disabled="isCloning"
                />
                <span>Shallow Clone (Depth limit)</span>
              </label>
              <input
                v-if="isShallow"
                v-model.number="depth"
                type="number"
                min="1"
                class="depth-input"
                :disabled="isCloning"
              />
            </div>
          </div>
        </div>

        <!-- Error alert -->
        <div v-if="cloneError" class="clone-error-box">
          <div class="error-title">Clone Error</div>
          <div class="error-message">{{ cloneError }}</div>
        </div>

        <!-- Progress status -->
        <div v-if="isCloning" class="cloning-progress">
          <div class="spinner"></div>
          <span>Cloning repository… This may take a moment.</span>
        </div>
      </div>

      <footer>
        <button
          type="button"
          class="btn-cancel"
          :disabled="isCloning"
          @click="$emit('close')"
        >
          Cancel
        </button>
        <button
          type="submit"
          class="btn-clone"
          :disabled="!isValid"
        >
          <CloneIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
          {{ isCloning ? 'Cloning…' : 'Clone Repository' }}
        </button>
      </footer>
    </form>
  </div>
</template>

<style scoped>
.clone-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: grid;
  place-items: center;
  padding: 20px;
  background: var(--overlay-bg);
  backdrop-filter: blur(2px);
}

.clone-modal {
  width: min(540px, 100%);
  max-height: 90vh;
  overflow-y: auto;
  padding: 22px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--panel-bg);
  box-shadow: 0 18px 52px var(--shadow-color);
  color: var(--text-main);
  display: flex;
  flex-direction: column;
  gap: 16px;
}

header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.clone-icon {
  width: 38px;
  height: 38px;
  display: grid;
  place-items: center;
  border-radius: 8px;
  background: var(--row-selected);
  color: var(--accent-blue);
  flex-shrink: 0;
}

.header-text h2 {
  font-size: 1.05rem;
  font-weight: 600;
  margin: 0;
}

.header-text p {
  margin-top: 2px;
  color: var(--text-muted);
  font-size: 0.8rem;
}

.clone-close {
  margin-left: auto;
  border: 0;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 1.5rem;
  line-height: 1;
  padding: 0 4px;
}

.clone-close:hover {
  color: var(--text-main);
}

.form-body {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  color: var(--text-muted);
  font-size: 0.78rem;
  font-weight: 600;
}

.required-star {
  color: var(--text-danger);
}

.optional-label {
  font-weight: normal;
  color: var(--text-muted);
  font-size: 0.75rem;
}

.field-hint {
  font-size: 0.72rem;
  color: var(--text-muted);
  margin-top: 2px;
}

input[type='text'],
input[type='password'],
input[type='number'] {
  width: 100%;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 8px 10px;
  background: var(--input-bg);
  color: var(--text-main);
  font: inherit;
  font-size: 0.88rem;
  transition: border-color 0.2s, box-shadow 0.2s;
}

input:focus {
  outline: none;
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-blue) 35%, transparent);
}

.dest-picker-row {
  display: flex;
  gap: 8px;
}

.dest-picker-row input {
  flex: 1;
}

.password-input-row {
  display: flex;
  gap: 8px;
}

.password-input-row input {
  flex: 1;
}

.toggle-pwd-btn {
  padding: 0 12px;
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-muted);
  font-size: 0.78rem;
  cursor: pointer;
  transition: background 0.2s, color 0.2s;
}

.toggle-pwd-btn:hover:not(:disabled) {
  background: var(--surface-hover);
  color: var(--text-main);
}

.browse-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-main);
  font-size: 0.82rem;
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.2s, border-color 0.2s;
}

.browse-btn:hover:not(:disabled) {
  background: var(--surface-hover);
  border-color: var(--accent-blue);
}

.target-path-preview {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  background: var(--surface-inset);
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 0.78rem;
  overflow: hidden;
}

.preview-label {
  color: var(--text-muted);
  white-space: nowrap;
  flex-shrink: 0;
}

.preview-path {
  color: var(--accent-blue);
  font-family: monospace;
  font-size: 0.8rem;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Authentication Section */
.auth-section {
  border-top: 1px solid var(--border);
  padding-top: 12px;
}

.auth-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  color: var(--text-main);
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
  padding: 2px 0;
  width: 100%;
}

.auth-icon {
  color: var(--accent-blue);
}

.auth-toggle-title {
  flex: 1;
  text-align: left;
}

.auth-badge {
  font-size: 0.7rem;
  font-weight: normal;
  background: var(--row-selected);
  color: var(--accent-blue);
  padding: 2px 8px;
  border-radius: 12px;
}

.chevron-icon {
  color: var(--text-muted);
}

.auth-fields {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.auth-types-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 6px;
}

.auth-type-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 8px 12px;
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
}

.auth-type-card:hover {
  background: var(--surface-hover);
  border-color: var(--border-color);
}

.auth-type-card.active {
  background: color-mix(in srgb, var(--accent-blue) 12%, transparent);
  border-color: var(--accent-blue);
}

.auth-type-card input {
  margin-top: 3px;
  accent-color: var(--accent-blue);
}

.card-info {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.card-info strong {
  font-size: 0.82rem;
  color: var(--text-main);
}

.card-info span {
  font-size: 0.72rem;
  color: var(--text-muted);
}

.auth-inputs-panel {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 10px 12px;
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.advanced-section {
  border-top: 1px solid var(--border);
  padding-top: 10px;
}

.advanced-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 0.8rem;
  cursor: pointer;
  padding: 2px 0;
}

.advanced-toggle:hover {
  color: var(--text-main);
}

.advanced-fields {
  margin-top: 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 8px 10px;
  background: var(--surface-subtle);
  border-radius: 6px;
}

.shallow-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8rem;
  color: var(--text-main);
  cursor: pointer;
}

.depth-input {
  width: 70px !important;
  padding: 4px 8px !important;
  font-size: 0.8rem !important;
}

.clone-error-box {
  padding: 10px 12px;
  border: 1px solid var(--danger-text);
  background: color-mix(in srgb, var(--danger-text) 10%, transparent);
  border-radius: 6px;
  color: var(--danger-text);
  font-size: 0.82rem;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.error-title {
  font-weight: 600;
}

.error-message {
  font-family: monospace;
  font-size: 0.78rem;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 100px;
  overflow-y: auto;
}

.cloning-progress {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px;
  background: var(--surface-subtle);
  border-radius: 6px;
  font-size: 0.84rem;
  color: var(--text-main);
}

.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--border);
  border-top-color: var(--accent-blue);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 4px;
}

footer button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border-radius: 6px;
  padding: 8px 14px;
  cursor: pointer;
  font: inherit;
  font-size: 0.84rem;
  font-weight: 550;
  transition: all 0.2s;
}

.btn-cancel {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-main);
}

.btn-cancel:hover:not(:disabled) {
  background: var(--surface-hover);
}

.btn-clone {
  border: 1px solid var(--accent-blue);
  background: var(--accent-blue);
  color: white;
}

.btn-clone:hover:not(:disabled) {
  background: var(--accent-hover);
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}
</style>
