<template>
  <aside class="right-sidebar" :style="{ width: width + 'px' }" v-show="width > 0">
    <div class="commit-details-header">
      <div class="header-top">
        <div class="details-mode-tabs" aria-label="Details view">
          <button
            v-if="hasWorkingChanges || mode === 'changes'"
            class="details-mode-tab"
            :class="{ active: mode === 'changes' }"
            @click="$emit('change-mode', 'changes')"
          >
            <ActionIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none"  />
            Working Changes
            <span class="mode-count">{{ workingChangeCount }}</span>
          </button>
          <button
            v-if="hasCommit"
            class="details-mode-tab"
            :class="{ active: mode === 'commit' }"
            @click="$emit('change-mode', 'commit')"
          >
            <FileIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none"  />
            Commit
          </button>
        </div>
        <button class="close-btn" @click="$emit('close')">
          <CloseIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none"  />
        </button>
      </div>
    </div>

    <div class="commit-details-body">
      <div v-if="mode === 'commit'" class="commit-meta">
        <div class="meta-row" v-if="activeBranch">
          <span class="meta-label">Branch</span>
          <span class="meta-value branch-value" style="display: flex; align-items: center; color: var(--accent-blue);">
            <GitBranchIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" style="margin-right: 4px;"  />
            {{ activeBranch }}
          </span>
        </div>
        <div class="meta-row" style="flex-direction: column; align-items: stretch;" v-if="commit.labels && commit.labels.length > 0">
          <span class="meta-label" style="width: 100%; margin-bottom: 6px;">Labels</span>
          <div class="labels-table-container">
            <table class="labels-table">
              <thead>
                <tr>
                  <th>Type</th>
                  <th>Name</th>
                  <th style="width: 32px; text-align: center;"></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(label, lIdx) in visibleLabels" :key="lIdx">
                  <td style="width: 1%; white-space: nowrap;">
                    <div class="branch-label" :style="getLabelStyle(label)">
                      <CloudIcon v-if="label.isRemote" :style="{ color: label.isLocal && label.isRemote ? 'var(--remote-label)' : '' }" class="label-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none"  />
                      <LocalIcon v-if="label.isLocal" :style="{ color: label.isLocal && label.isRemote ? 'var(--local-label)' : '' }" class="label-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none"  />
                      <TagIcon v-if="label.isTag" :style="{ color: label.isLocal && label.isRemote ? 'var(--remote-label)' : '' }" class="label-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none"  />
                      <span class="label-text" style="margin-left: 4px;">{{ getLabelTypeString(label) }}</span>
                    </div>
                  </td>
                  <td class="label-name-cell">{{ label.text }}</td>
                  <td style="text-align: center; padding: 2px;">
                    <button class="copy-action-btn" @click.stop="copyLabel(label.text)" title="Copy label">
                      <CopyIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
            <div v-if="commit.labels && commit.labels.length > 3" style="text-align: center; margin-top: 4px;">
              <button class="expand-labels-btn" @click="labelsExpanded = !labelsExpanded">
                {{ labelsExpanded ? 'Collapse' : `Show ${commit.labels.length - 3} more` }}
              </button>
            </div>
          </div>
        </div>
        <div class="meta-row">
          <span class="meta-label">Author</span>
          <span class="meta-value author-value" style="display: flex; align-items: center; gap: 8px;">
            <AuthorAvatar :author-name="commit.author" :size="24" />
            {{ commit.author }}
          </span>
        </div>
        <div class="meta-row">
          <span class="meta-label">Date</span>
          <span class="meta-value">{{ commit.time }}</span>
        </div>
        <div class="meta-row">
          <span class="meta-label">Hash</span>
          <span class="meta-value commit-hash-container">
            <span class="commit-hash" :title="commit.id">{{ commit.id }}</span>
            <button class="copy-btn" @click="copyHash(commit.id)" title="Copy full hash">
              <CopyIcon v-if="!copyHashStatus" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
              <CheckIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="#4caf50" stroke-width="2" fill="none"  />
            </button>
          </span>
        </div>
        <div class="meta-row">
          <span class="meta-label">Short</span>
          <span class="meta-value commit-hash-container">
            <span class="commit-hash" :title="commit.hash">{{ commit.hash }}</span>
            <button class="copy-btn" @click="copyShortHash(commit.hash)" title="Copy short hash">
              <CopyIcon v-if="!copyShortHashStatus" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
              <CheckIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="#4caf50" stroke-width="2" fill="none"  />
            </button>
          </span>
        </div>
        <div class="meta-row" v-if="commit.signature">
          <span class="meta-label">Signature</span>
          <span class="meta-value signature-details" style="display: flex; align-items: center; gap: 6px; flex-wrap: wrap;">
            <span class="signature-pill" :class="'sig-' + commit.signature.status">
              <ShieldCheckIcon v-if="commit.signature.status === 'verified'" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
              <ShieldAlertIcon v-else viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
              {{ commit.signature.status === 'verified' ? 'Verified' : commit.signature.status }}
            </span>
            <span v-if="commit.signature.signer" class="signature-signer" :title="commit.signature.key">
              {{ commit.signature.signer }}
            </span>
            <span v-if="commit.signature.key && !commit.signature.signer" class="signature-key">
              Key: {{ commit.signature.key }}
            </span>
          </span>
        </div>
      </div>

      <div v-if="mode === 'commit'" class="commit-message-full">
        {{ commit.message }}
      </div>

      <div class="file-changes-section" v-if="mode === 'commit' && commit.files">
        <div class="file-changes-header">
          <div class="file-changes-title" style="display: flex; align-items: center;">
            Changed Files 
            <span class="file-count" v-if="commit.files" style="margin-left: 6px;">{{ commit.files.length }}</span>
            <div class="file-stats-badges" v-if="commit.files">
              <span class="badge badge-added" v-if="addedCount > 0" :title="`${addedCount} file${addedCount > 1 ? 's' : ''} added`">+{{ addedCount }}</span>
              <span class="badge badge-modified" v-if="modifiedCount > 0" :title="`${modifiedCount} file${modifiedCount > 1 ? 's' : ''} modified`">~{{ modifiedCount }}</span>
              <span class="badge badge-deleted" v-if="deletedCount > 0" :title="`${deletedCount} file${deletedCount > 1 ? 's' : ''} deleted`">-{{ deletedCount }}</span>
            </div>
          </div>
          <div class="view-toggle">
            <button :class="{ active: fileViewMode === 'flat' }" @click="fileViewMode = 'flat'">Flat</button>
            <button :class="{ active: fileViewMode === 'tree' }" @click="fileViewMode = 'tree'">Tree</button>
          </div>
        </div>
        
        <ul v-if="fileViewMode === 'flat'" class="file-list">
          <li v-for="(file, idx) in commit.files" :key="idx" class="file-item clickable" @click="$emit('select-file', file.path)">
            <span class="file-status" :style="{ color: getFileStatusColor(file.status) }">
              {{ file.status.charAt(0) }}
            </span>
            <span class="file-path" :title="file.path">{{ file.path }}</span>
          </li>
        </ul>

        <ul v-else class="file-list tree-list">
          <li v-for="node in buildTree(commit.files)" :key="node.path" class="file-item" 
              :class="{ 'is-dir': node.isDir, 'clickable': !node.isDir }"
              :style="{ paddingLeft: (node.depth * 14) + 'px' }"
              @click="node.isDir ? toggleDir(node.path) : $emit('select-file', node.path)">
            <div class="tree-toggle" v-if="node.isDir">
              <ChevronDownIcon v-if="collapsedDirs.has(node.path)" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
              <ChevronRightIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
            </div>
            <div class="tree-toggle placeholder" v-else></div>
            <FolderIcon v-if="node.isDir" class="file-icon folder-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
            <span v-else class="file-status tree-status" :style="{ color: getFileStatusColor(node.status) }">{{ node.status.charAt(0) }}</span>
            <span class="file-path">{{ node.name }}</span>
          </li>
        </ul>
      </div>

      <div class="uncommitted-changes-container" v-if="mode === 'changes'">
        <div class="working-tree-summary">
          <div>
            <div class="working-tree-title">Working tree</div>
            <div class="working-tree-subtitle">Changes not yet recorded in repository history</div>
          </div>
          <span class="working-tree-count">{{ workingChangeCount }}</span>
        </div>
        <div class="file-changes-header" style="margin-bottom: 8px;">
          <div class="view-toggle">
            <button :class="{ active: fileViewMode === 'flat' }" @click="fileViewMode = 'flat'">Flat</button>
            <button :class="{ active: fileViewMode === 'tree' }" @click="fileViewMode = 'tree'">Tree</button>
          </div>
        </div>

        <!-- Commit Box -->
        <div class="commit-box-container">
          <div class="commit-box">
            <div style="display: flex; gap: 8px;">
              <select v-model="commitPrefix" class="commit-input" style="flex: 0 0 100px;">
                <option value="feat">feat</option>
                <option value="fix">fix</option>
                <option value="chore">chore</option>
                <option value="docs">docs</option>
                <option value="refactor">refactor</option>
                <option value="style">style</option>
                <option value="test">test</option>
                <option value="perf">perf</option>
                <option value="build">build</option>
                <option value="ci">ci</option>
              </select>
              <input v-model="commitScope" type="text" placeholder="Service / Scope" class="commit-input" style="flex: 1;" />
            </div>
            <input v-model="commitTitle" type="text" placeholder="Commit title..." class="commit-input" />
            <textarea v-model="commitDescription" placeholder="Description (optional)..." class="commit-input" rows="3"></textarea>

            <div class="commit-box-footer">
              <label class="amend-checkbox-label" :class="{ disabled: !headCommit }" :title="headCommit ? `Amend previous commit (${headCommit.hash || headCommit.id?.substring(0, 7)})` : 'No previous commit to amend'">
                <input
                  type="checkbox"
                  class="amend-checkbox"
                  :checked="isAmend"
                  :disabled="!headCommit"
                  @change="toggleAmend(($event.target as HTMLInputElement).checked)"
                />
                <span class="amend-label-text">Amend previous commit</span>
              </label>

              <span v-if="isAmend && headCommit" class="amend-head-badge" :title="headCommit.message">
                HEAD: {{ headCommit.hash || headCommit.id?.substring(0, 7) }}
              </span>
            </div>

            <button
              class="commit-btn"
              :class="{ 'commit-btn-amend': isAmend }"
              :disabled="!canSubmitCommit"
              @click="onCommit"
              :title="!canSubmitCommit && !isAmend && !hasStagedFiles ? 'Stage changes before committing, or check Amend' : ''"
            >
              {{ isAmend ? 'Amend Commit (HEAD)' : 'Commit' }}
            </button>
          </div>
        </div>

        <!-- Staged Changes -->
        <div class="file-changes-section" v-if="commit.staged_files && commit.staged_files.length > 0">
          <div class="file-changes-title" style="margin-bottom: 8px;">Staged Changes <span class="file-count">{{ commit.staged_files.length }}</span></div>

          <ul v-if="fileViewMode === 'flat'" class="file-list">
            <li v-for="(file, idx) in commit.staged_files" :key="idx" class="file-item clickable" @click="$emit('select-file', file.path)">
              <span class="file-status" :style="{ color: getFileStatusColor(file.status) }">{{ file.status.charAt(0) }}</span>
              <span class="file-path" :title="file.path">{{ file.path }}</span>
              <button class="stage-action-btn" @click.stop="$emit('unstage-file', file.path)" title="Unstage file">
                <MinusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"  />
              </button>
            </li>
          </ul>
          <ul v-else class="file-list tree-list">
            <li v-for="node in buildTree(commit.staged_files)" :key="node.path" class="file-item" 
                :class="{ 'is-dir': node.isDir, 'clickable': !node.isDir }"
                :style="{ paddingLeft: (node.depth * 14) + 'px' }"
                @click="node.isDir ? toggleDir(node.path) : $emit('select-file', node.path)">
              <div class="tree-toggle" v-if="node.isDir">
                <ChevronDownIcon v-if="collapsedDirs.has(node.path)" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
                <ChevronRightIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
              </div>
              <div class="tree-toggle placeholder" v-else></div>
              <FolderIcon v-if="node.isDir" class="file-icon folder-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
              <span v-else class="file-status tree-status" :style="{ color: getFileStatusColor(node.status) }">{{ node.status.charAt(0) }}</span>
              <span class="file-path">{{ node.name }}</span>
              <button class="stage-action-btn" @click.stop="$emit('unstage-file', node.path)" title="Unstage file">
                <MinusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"  />
              </button>
            </li>
          </ul>
        </div>
        
        <!-- Unstaged Changes -->
        <div class="file-changes-section" style="margin-top: 15px;" v-if="commit.unstaged_files && commit.unstaged_files.length > 0">
          <div class="file-changes-title" style="margin-bottom: 8px; display: flex; align-items: center; justify-content: space-between;">
            <div>Unstaged Changes <span class="file-count">{{ commit.unstaged_files.length }}</span></div>
            <button class="stage-all-btn" @click.stop="$emit('stage-all')" title="Stage all changes">Stage All</button>
          </div>
          <ul v-if="fileViewMode === 'flat'" class="file-list">
            <li v-for="(file, idx) in commit.unstaged_files" :key="idx" class="file-item clickable" @click="$emit('select-file', file.path)">
              <span class="file-status" :style="{ color: getFileStatusColor(file.status) }">{{ file.status.charAt(0) }}</span>
              <span class="file-path" :title="file.path">{{ file.path }}</span>
              <button class="stage-action-btn" @click.stop="$emit('stage-file', file.path)" title="Stage file">
                <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"  />
              </button>
            </li>
          </ul>
          <ul v-else class="file-list tree-list">
            <li v-for="node in buildTree(commit.unstaged_files)" :key="node.path" class="file-item" 
                :class="{ 'is-dir': node.isDir, 'clickable': !node.isDir }"
                :style="{ paddingLeft: (node.depth * 14) + 'px' }"
                @click="node.isDir ? toggleDir(node.path) : $emit('select-file', node.path)">
              <div class="tree-toggle" v-if="node.isDir">
                <ChevronDownIcon v-if="collapsedDirs.has(node.path)" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
                <ChevronRightIcon v-else viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
              </div>
              <div class="tree-toggle placeholder" v-else></div>
              <FolderIcon v-if="node.isDir" class="file-icon folder-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
              <span v-else class="file-status tree-status" :style="{ color: getFileStatusColor(node.status) }">{{ node.status.charAt(0) }}</span>
              <span class="file-path">{{ node.name }}</span>
              <button class="stage-action-btn" @click.stop="$emit('stage-file', node.path)" title="Stage file">
                <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none"  />
              </button>
            </li>
          </ul>
        </div>

        <div v-if="workingChangeCount === 0 && !isAmend" class="clean-working-tree">
          <CheckIcon viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="2" fill="none"  />
          <span>Working tree is clean</span>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import ActionIcon from '../../assets/icons/action.svg?component';
import FileIcon from '../../assets/icons/file.svg?component';
import CloseIcon from '../../assets/icons/close.svg?component';
import GitBranchIcon from '../../assets/icons/git-branch.svg?component';
import CloudIcon from '../../assets/icons/Cloud.svg?component';
import LocalIcon from '../../assets/icons/local.svg?component';
import TagIcon from '../../assets/icons/tag.svg?component';
import CopyIcon from '../../assets/icons/copy.svg?component';
import CheckIcon from '../../assets/icons/check.svg?component';
import ChevronDownIcon from '../../assets/icons/chevron-down.svg?component';
import ChevronRightIcon from '../../assets/icons/chevron-right.svg?component';
import FolderIcon from '../../assets/icons/folder.svg?component';
import MinusIcon from '../../assets/icons/minus.svg?component';
import PlusIcon from '../../assets/icons/plus.svg?component';
import ShieldCheckIcon from '../../assets/icons/shield-check.svg?component';
import ShieldAlertIcon from '../../assets/icons/shield-alert.svg?component';
import AuthorAvatar from '../Common/AuthorAvatar.vue';
import { ref, computed, watch } from 'vue';

const props = defineProps({
  commit: {
    type: Object,
    required: true
  },
  width: {
    type: Number,
    required: true
  },
  activeBranch: {
    type: String,
    default: ''
  },
  mode: {
    type: String,
    default: 'commit'
  },
  hasCommit: {
    type: Boolean,
    default: false
  },
  hasWorkingChanges: {
    type: Boolean,
    default: false
  },
  workingChangeCount: {
    type: Number,
    default: 0
  },
  headCommit: {
    type: Object,
    default: null
  },
  amendTrigger: {
    type: Number,
    default: 0
  }
});

const emit = defineEmits(['close', 'change-mode', 'select-file', 'stage-file', 'unstage-file', 'stage-all', 'commit-changes']);

const fileViewMode = ref('tree');
const collapsedDirs = ref<Set<string>>(new Set());
const copyHashStatus = ref(false);
const copyShortHashStatus = ref(false);
const commitPrefix = ref('feat');
const commitScope = ref('');
const commitTitle = ref('');
const commitDescription = ref('');
const labelsExpanded = ref(false);
const isAmend = ref(false);
let cachedFormState: { prefix: string; scope: string; title: string; description: string } | null = null;

const visibleLabels = computed(() => {
  if (!props.commit || !props.commit.labels) return [];
  if (labelsExpanded.value || props.commit.labels.length <= 3) {
    return props.commit.labels;
  }
  return props.commit.labels.slice(0, 3);
});

function parseCommitMessage(rawMessage: string) {
  if (!rawMessage) return { prefix: 'feat', scope: '', title: '', description: '' };
  
  const lines = rawMessage.split('\n');
  const firstLine = lines[0] || '';
  const description = lines.slice(1).join('\n').trim();

  const match = firstLine.match(/^([a-zA-Z0-9_-]+)(?:\(([^)]+)\))?!?: (.*)$/);
  if (match) {
    const validPrefixes = ['feat', 'fix', 'chore', 'docs', 'refactor', 'style', 'test', 'perf', 'build', 'ci'];
    const parsedPrefix = match[1].toLowerCase();
    const prefix = validPrefixes.includes(parsedPrefix) ? parsedPrefix : 'feat';
    const scope = match[2] || '';
    const title = match[3] || '';
    return { prefix, scope, title, description };
  }

  return {
    prefix: 'feat',
    scope: '',
    title: firstLine,
    description
  };
}

function toggleAmend(enable: boolean) {
  isAmend.value = enable;
  if (enable) {
    cachedFormState = {
      prefix: commitPrefix.value,
      scope: commitScope.value,
      title: commitTitle.value,
      description: commitDescription.value
    };
    if (props.headCommit?.message) {
      const parsed = parseCommitMessage(props.headCommit.message);
      commitPrefix.value = parsed.prefix;
      commitScope.value = parsed.scope;
      commitTitle.value = parsed.title;
      commitDescription.value = parsed.description;
    }
  } else {
    if (cachedFormState) {
      commitPrefix.value = cachedFormState.prefix;
      commitScope.value = cachedFormState.scope;
      commitTitle.value = cachedFormState.title;
      commitDescription.value = cachedFormState.description;
      cachedFormState = null;
    }
  }
}

watch(() => props.amendTrigger, (val) => {
  if (val > 0) {
    toggleAmend(true);
  }
});

watch(() => [props.commit?.id, props.mode], () => {
  collapsedDirs.value.clear();
  copyHashStatus.value = false;
  copyShortHashStatus.value = false;
  labelsExpanded.value = false;
  if (props.mode !== 'changes') {
    isAmend.value = false;
    commitTitle.value = '';
    commitDescription.value = '';
    commitScope.value = '';
    cachedFormState = null;
  }
});

const hasStagedFiles = computed(() => {
  return Boolean(props.commit?.staged_files && props.commit.staged_files.length > 0);
});

const canSubmitCommit = computed(() => {
  const hasTitle = Boolean(commitTitle.value.trim());
  if (isAmend.value) {
    return hasTitle && Boolean(props.headCommit);
  }
  return hasTitle && hasStagedFiles.value;
});

function onCommit() {
  const title = commitTitle.value.trim();
  if (title && canSubmitCommit.value) {
    let message = commitPrefix.value;
    const scope = commitScope.value.trim();
    if (scope) {
      message += `(${scope})`;
    }
    message += `: ${title}`;
    
    const desc = commitDescription.value.trim();
    if (desc) {
      message += `\n\n${desc}`;
    }
    
    emit('commit-changes', message, isAmend.value);
    
    commitTitle.value = '';
    commitDescription.value = '';
    commitScope.value = '';
    isAmend.value = false;
    cachedFormState = null;
  }
}

function getLabelStyle(label: any) {
  const GREEN = 'var(--remote-label)';
  const YELLOW = 'var(--local-label)';
  
  if (label.isLocal && label.isRemote) {
    return {
      borderTopColor: YELLOW,
      borderLeftColor: YELLOW,
      borderBottomColor: GREEN,
      borderRightColor: GREEN,
      color: 'var(--text-main)'
    };
  } else if (label.isRemote || label.isTag) {
    return {
      borderColor: GREEN,
      color: GREEN
    };
  } else {
    return {
      borderColor: YELLOW,
      color: YELLOW
    };
  }
}

function getLabelTypeString(label: any) {
  if (label.isTag) return 'Tag';
  if (label.isLocal && label.isRemote) return 'Local / Remote';
  if (label.isLocal) return 'Local Branch';
  if (label.isRemote) return 'Remote Branch';
  return 'Branch';
}

async function copyLabel(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch (e) {
    console.error("Failed to copy label:", e);
  }
}

const addedCount = computed(() => {
  if (!props.commit || !props.commit.files) return 0;
  return props.commit.files.filter((f: any) => f.status.startsWith('A')).length;
});

const modifiedCount = computed(() => {
  if (!props.commit || !props.commit.files) return 0;
  return props.commit.files.filter((f: any) => f.status.startsWith('M') || f.status.startsWith('R') || f.status.startsWith('C')).length;
});

const deletedCount = computed(() => {
  if (!props.commit || !props.commit.files) return 0;
  return props.commit.files.filter((f: any) => f.status.startsWith('D')).length;
});

async function copyHash(hash: string) {
  try {
    await navigator.clipboard.writeText(hash);
    copyHashStatus.value = true;
    setTimeout(() => {
      copyHashStatus.value = false;
    }, 2000);
  } catch (e) {
    console.error("Failed to copy", e);
  }
}

async function copyShortHash(hash: string) {
  try {
    await navigator.clipboard.writeText(hash);
    copyShortHashStatus.value = true;
    setTimeout(() => {
      copyShortHashStatus.value = false;
    }, 2000);
  } catch (e) {
    console.error("Failed to copy", e);
  }
}

function toggleDir(path: string) {
  if (collapsedDirs.value.has(path)) {
    collapsedDirs.value.delete(path);
  } else {
    collapsedDirs.value.add(path);
  }
}

function getFileStatusColor(status: string) {
  if (!status) return 'var(--text-muted)';
  if (status.startsWith('A')) return 'var(--success-text)';
  if (status.startsWith('M')) return 'var(--info-text)';
  if (status.startsWith('D')) return 'var(--danger-text)';
  if (status.startsWith('R')) return 'var(--warning-text)';
  return 'var(--text-muted)';
}

function buildTree(files: any[]) {
  if (!files) return [];
  
  const root = { name: '', path: '', isDir: true, children: [] as any[] };
  
  files.forEach((file: any) => {
    const parts = file.path.split('/');
    let current = root;
    
    parts.forEach((part: string, index: number) => {
      const isLast = index === parts.length - 1;
      let child = current.children.find((c: any) => c.name === part);
      
      if (!child) {
        child = {
          name: part,
          path: parts.slice(0, index + 1).join('/'),
          isDir: !isLast,
          children: isLast ? undefined : [],
          status: isLast ? file.status : undefined,
        };
        current.children.push(child);
      }
      current = child;
    });
  });
  
  const sortTree = (node: any) => {
    if (node.children) {
      node.children.sort((a: any, b: any) => {
        if (a.isDir !== b.isDir) return b.isDir ? -1 : 1;
        return a.name.localeCompare(b.name);
      });
      node.children.forEach(sortTree);
    }
  };
  sortTree(root);
  
  const result: any[] = [];
  const flatten = (node: any, depth = -1) => {
    if (depth >= 0) {
      result.push({ ...node, depth });
    }
    if (node.isDir && !collapsedDirs.value.has(node.path) && node.children) {
      node.children.forEach((child: any) => flatten(child, depth + 1));
    }
  };
  
  flatten(root);
  return result;
}
</script>

<style scoped>
.details-mode-tabs {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 3px;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: var(--bg-color);
}

.details-mode-tab {
  min-width: 0;
  height: 27px;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 0 8px;
  border: 0;
  border-radius: 5px;
  color: var(--text-muted);
  background: transparent;
  font: inherit;
  font-size: 11px;
  font-weight: 600;
  white-space: nowrap;
  cursor: pointer;
}

.details-mode-tab:hover {
  color: var(--text-main);
  background: var(--surface-subtle);
}

.details-mode-tab.active {
  color: var(--text-main);
  background: var(--row-selected);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 45%, transparent);
}

.mode-count {
  min-width: 17px;
  height: 17px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 5px;
  border-radius: 999px;
  color: var(--text-main);
  background: var(--surface-hover);
  font-size: 10px;
}

.working-tree-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 14px;
  padding: 12px;
  border: 1px solid var(--border);
  border-radius: 7px;
  background: var(--surface-subtle);
}

.working-tree-title {
  margin-bottom: 3px;
  color: var(--text-main);
  font-size: 13px;
  font-weight: 650;
}

.working-tree-subtitle {
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.35;
}

.working-tree-count {
  min-width: 28px;
  height: 28px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: 0 0 auto;
  border-radius: 50%;
  color: var(--warning-text);
  background: rgba(251, 191, 36, 0.12);
  font-size: 12px;
  font-weight: 700;
}

.clean-working-tree {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 32px 16px;
  color: var(--success-text);
  font-size: 12px;
}

.file-stats-badges {
  display: flex;
  gap: 4px;
  margin-left: 8px;
}
.badge {
  font-size: 0.65rem;
  padding: 1px 4px;
  border-radius: 4px;
  font-weight: 600;
}
.badge-added {
  color: var(--success-text);
  background-color: rgba(76, 175, 80, 0.15);
}
.badge-modified {
  color: var(--info-text);
  background-color: rgba(33, 150, 243, 0.15);
}
.badge-deleted {
  color: var(--danger-text);
  background-color: rgba(244, 67, 54, 0.15);
}
.stage-action-btn {
  background: none;
  border: 1px solid var(--border);
  color: var(--text-muted);
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-left: auto;
  opacity: 0;
  padding: 4px;
  transition: all 0.2s;
}
.stage-action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-main);
  border-color: var(--text-muted);
}
.file-item:hover .stage-action-btn {
  opacity: 1;
}
.stage-all-btn {
  background: var(--accent-blue);
  color: white;
  border: none;
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 0.75rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.2s;
}
.stage-all-btn:hover {
  opacity: 0.8;
}
.commit-box-container {
  margin-bottom: 14px;
}

.commit-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--surface-subtle);
}

.commit-input {
  background: var(--bg-main);
  border: 1px solid var(--border);
  color: var(--text-main);
  border-radius: 4px;
  padding: 8px;
  font-family: inherit;
  font-size: 0.85rem;
  resize: vertical;
}
.commit-input:focus {
  outline: none;
  border-color: var(--accent-blue);
}
select.commit-input {
  appearance: none;
  background-image: url('data:image/svg+xml;utf8,<Unknown1Icon fill="none" stroke="%23888" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"  />');
  background-repeat: no-repeat;
  background-position: right 8px center;
  background-size: 16px;
  padding-right: 32px;
}

.commit-box-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 2px 2px;
}

.amend-checkbox-label {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  user-select: none;
  font-size: 0.76rem;
  color: var(--text-muted);
  transition: color 0.15s ease;
}

.amend-checkbox-label:hover:not(.disabled) {
  color: var(--text-main);
}

.amend-checkbox-label.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.amend-checkbox {
  cursor: pointer;
  accent-color: var(--accent-blue);
}

.amend-label-text {
  font-weight: 500;
}

.amend-head-badge {
  font-size: 0.7rem;
  font-family: monospace;
  font-weight: 600;
  color: var(--accent-blue);
  background: rgba(88, 166, 255, 0.12);
  border: 1px solid rgba(88, 166, 255, 0.25);
  padding: 1px 6px;
  border-radius: 4px;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.commit-btn {
  background: var(--accent-blue);
  color: white;
  border: none;
  border-radius: 4px;
  padding: 7px;
  font-weight: 600;
  font-size: 0.82rem;
  cursor: pointer;
  transition: all 0.2s ease;
}
.commit-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
.commit-btn:not(:disabled):hover {
  opacity: 0.9;
}

.commit-btn-amend {
  background: #d97706; /* amber-600 */
}

.commit-btn-amend:not(:disabled):hover {
  background: #b45309;
}

.labels-table-container {
  width: 100%;
  overflow-x: auto;
}
.labels-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.8rem;
  background: var(--bg-main);
  border: 1px solid var(--border);
  border-radius: 4px;
}
.labels-table th {
  text-align: left;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border);
  color: var(--text-muted);
  font-weight: 500;
  background: var(--surface-subtle);
}
.labels-table td {
  padding: 6px 8px;
  border-bottom: 1px solid var(--border);
  vertical-align: middle;
}
.labels-table tr:last-child td {
  border-bottom: none;
}
.label-name-cell {
  word-break: break-all;
  color: var(--text-main);
}
.copy-action-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}
.copy-action-btn:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}
.expand-labels-btn {
  background: none;
  border: none;
  color: var(--accent-blue);
  font-size: 0.75rem;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
}
.expand-labels-btn:hover {
  background: var(--surface-subtle);
}

.signature-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px;
  border-radius: 4px;
  font-size: 0.72rem;
  font-weight: 600;
  text-transform: capitalize;
}

.signature-pill.sig-verified {
  background: rgba(46, 160, 67, 0.15);
  color: #3fb950;
  border: 1px solid rgba(46, 160, 67, 0.35);
}

.signature-pill.sig-unverified {
  background: rgba(210, 153, 34, 0.15);
  color: #d29922;
  border: 1px solid rgba(210, 153, 34, 0.35);
}

.signature-pill.sig-bad,
.signature-pill.sig-expired,
.signature-pill.sig-revoked {
  background: rgba(248, 81, 73, 0.15);
  color: #f85149;
  border: 1px solid rgba(248, 81, 73, 0.35);
}

.signature-signer {
  font-size: 0.78rem;
  color: var(--text-main);
  max-width: 180px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.signature-key {
  font-size: 0.75rem;
  color: var(--text-muted);
  font-family: monospace;
}
</style>
