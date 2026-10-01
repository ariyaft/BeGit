<template>
  <div class="pull-requests-page">
    <!-- Header -->
    <header class="pr-page-header">
      <div class="header-left">
        <div class="pr-header-title">
          <PullRequestIcon viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="2" fill="none" class="title-icon" />
          <h1>Pull Requests & Reviews</h1>
        </div>
        <div v-if="remoteInfo" class="remote-badge" :title="remoteInfo.url">
          <span class="provider-tag" :class="'provider-' + remoteInfo.provider">{{ providerDisplayName }}</span>
          <span class="remote-repo-name">{{ remoteInfo.owner }}/{{ remoteInfo.repo }}</span>
        </div>
      </div>

      <div class="header-actions">
        <button v-if="remoteInfo?.web_url" class="btn-secondary" @click="openCreatePR" title="Open new PR page in browser">
          <PlusIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
          <span>New Pull Request</span>
        </button>

        <button class="btn-icon" :class="{ 'has-token': !!apiToken }" @click="showTokenModal = true" :title="apiToken ? 'API Token Configured' : 'Configure Personal Access Token'">
          <KeyIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
        </button>

        <button class="btn-icon" :disabled="isLoading" @click="loadPullRequests" title="Refresh Pull Requests">
          <FetchIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" :class="{ 'spin-icon': isLoading }" />
        </button>
      </div>
    </header>

    <!-- Main Content Area -->
    <div class="pr-page-content">
      <!-- Left: PR List & Filters -->
      <aside class="pr-list-pane">
        <!-- Search & Filter Controls -->
        <div class="pr-filter-bar">
          <div class="search-input-wrapper">
            <SearchIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" class="search-icon" />
            <input
              v-model="searchQuery"
              type="text"
              placeholder="Filter by title, #number, author, branch..."
              class="pr-search-input"
            />
            <button v-if="searchQuery" class="clear-search-btn" @click="searchQuery = ''">
              <CloseIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
            </button>
          </div>

          <div class="state-tabs">
            <button
              class="state-tab"
              :class="{ active: selectedState === 'open' }"
              @click="setStateFilter('open')"
            >
              Open <span class="state-count" v-if="stateCounts.open !== undefined">{{ stateCounts.open }}</span>
            </button>
            <button
              class="state-tab"
              :class="{ active: selectedState === 'merged' }"
              @click="setStateFilter('merged')"
            >
              Merged <span class="state-count" v-if="stateCounts.merged !== undefined">{{ stateCounts.merged }}</span>
            </button>
            <button
              class="state-tab"
              :class="{ active: selectedState === 'closed' }"
              @click="setStateFilter('closed')"
            >
              Closed <span class="state-count" v-if="stateCounts.closed !== undefined">{{ stateCounts.closed }}</span>
            </button>
            <button
              class="state-tab"
              :class="{ active: selectedState === 'all' }"
              @click="setStateFilter('all')"
            >
              All
            </button>
          </div>
        </div>

        <!-- PR Items List -->
        <div class="pr-items-container">
          <!-- Loading state -->
          <div v-if="isLoading && pullRequests.length === 0" class="pr-loading-state">
            <div class="spinner"></div>
            <span>Fetching pull requests from {{ providerDisplayName }}...</span>
          </div>

          <!-- Error / Auth warning state -->
          <div v-else-if="errorMessage && pullRequests.length === 0" class="pr-error-state">
            <ErrorIcon viewBox="0 0 24 24" width="32" height="32" stroke="currentColor" stroke-width="1.5" fill="none" class="error-icon" />
            <h3>Unable to load pull requests</h3>
            <p>{{ errorMessage }}</p>
            <div class="error-actions">
              <button class="btn-primary" @click="showTokenModal = true">Configure Token (PAT)</button>
              <button class="btn-secondary" @click="loadPullRequests">Retry</button>
            </div>
          </div>

          <!-- No remote configured -->
          <div v-else-if="!remoteInfo && !isLoading" class="pr-empty-state">
            <CloudIcon viewBox="0 0 24 24" width="36" height="36" stroke="currentColor" stroke-width="1.5" fill="none" />
            <h3>No Git Remote Detected</h3>
            <p>Connect this repository to GitHub or GitLab to view and manage pull requests.</p>
          </div>

          <!-- Empty list -->
          <div v-else-if="filteredPRs.length === 0 && !isLoading" class="pr-empty-state">
            <PullRequestIcon viewBox="0 0 24 24" width="36" height="36" stroke="currentColor" stroke-width="1.5" fill="none" />
            <h3>No {{ selectedState !== 'all' ? selectedState : '' }} pull requests found</h3>
            <p v-if="searchQuery">No pull requests match "{{ searchQuery }}".</p>
            <p v-else>There are currently no {{ selectedState }} pull requests for this repository.</p>
          </div>

          <!-- PR list items -->
          <div v-else class="pr-items-list">
            <div
              v-for="pr in filteredPRs"
              :key="pr.id"
              class="pr-item-card"
              :class="{
                selected: selectedPR?.id === pr.id,
                'is-draft': pr.draft
              }"
              @click="selectPullRequest(pr)"
            >
              <div class="pr-card-header">
                <div class="pr-state-indicator" :class="'state-' + getNormalizedState(pr)">
                  <PullRequestIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2.5" fill="none" />
                </div>
                <div class="pr-card-title-group">
                  <span class="pr-number">#{{ pr.number }}</span>
                  <span class="pr-title">{{ pr.title }}</span>
                </div>
              </div>

              <!-- Labels -->
              <div v-if="pr.labels && pr.labels.length > 0" class="pr-card-labels">
                <span
                  v-for="lbl in pr.labels"
                  :key="lbl.name"
                  class="pr-label-pill"
                  :style="getLabelBadgeStyle(lbl.color)"
                >
                  {{ lbl.name }}
                </span>
              </div>

              <!-- Card Meta: Branch, Author, Time -->
              <div class="pr-card-meta">
                <div class="branch-flow">
                  <span class="branch-pill source" :title="pr.head.ref">{{ truncate(pr.head.ref, 20) }}</span>
                  <span class="flow-arrow">→</span>
                  <span class="branch-pill target" :title="pr.base.ref">{{ truncate(pr.base.ref, 15) }}</span>
                </div>

                <div class="author-and-time">
                  <AuthorAvatar :author-name="pr.user.login" :avatar-url="pr.user.avatar_url" :size="16" />
                  <span class="pr-author-name">{{ pr.user.login }}</span>
                  <span class="pr-dot">•</span>
                  <span class="pr-time" :title="pr.created_at">{{ formatRelativeTime(pr.created_at) }}</span>
                </div>

                <div v-if="(pr.comments || 0) + (pr.review_comments || 0) > 0" class="pr-comments-count" title="Comments">
                  <ActionIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
                  <span>{{ (pr.comments || 0) + (pr.review_comments || 0) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </aside>

      <!-- Right: Selected PR Detail Pane -->
      <main class="pr-detail-pane" v-if="selectedPR">
        <!-- Detail Header -->
        <div class="detail-header-card">
          <div class="detail-top-row">
            <div class="detail-title-wrapper">
              <span class="detail-number">#{{ selectedPR.number }}</span>
              <h2 class="detail-title">{{ selectedPR.title }}</h2>
            </div>
            <div class="detail-actions">
              <button
                class="btn-primary checkout-btn"
                :disabled="isCheckingOut"
                @click="checkoutSelectedPR"
                title="Fetch and checkout branch locally"
              >
                <CloudBranchIcon v-if="!isCheckingOut" viewBox="0 0 24 24" width="15" height="15" stroke="currentColor" stroke-width="2" fill="none" />
                <span v-else class="spin-icon" style="display: inline-block;">⏳</span>
                <span>{{ isCheckingOut ? 'Checking out...' : 'Checkout PR Locally' }}</span>
              </button>

              <button class="btn-secondary" @click="openInBrowser(selectedPR.html_url)" title="Open in browser">
                <ActionIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
                <span>Open on {{ providerDisplayName }}</span>
              </button>
            </div>
          </div>

          <!-- Status bar -->
          <div class="detail-status-bar">
            <span class="status-badge" :class="'status-' + getNormalizedState(selectedPR)">
              <PullRequestIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.5" fill="none" />
              <span>{{ getStatusText(selectedPR) }}</span>
            </span>

            <div class="detail-branches">
              <span class="branch-pill source" :title="selectedPR.head.ref">
                <GitBranchIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
                {{ selectedPR.head.ref }}
              </span>
              <span class="flow-arrow">into</span>
              <span class="branch-pill target" :title="selectedPR.base.ref">
                <GitBranchIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
                {{ selectedPR.base.ref }}
              </span>
            </div>

            <div class="author-info">
              <AuthorAvatar :author-name="selectedPR.user.login" :avatar-url="selectedPR.user.avatar_url" :size="20" />
              <span><strong>{{ selectedPR.user.login }}</strong> opened {{ formatRelativeTime(selectedPR.created_at) }}</span>
            </div>
          </div>

          <!-- PR Labels -->
          <div v-if="selectedPR.labels && selectedPR.labels.length > 0" class="detail-labels-row">
            <span
              v-for="lbl in selectedPR.labels"
              :key="lbl.name"
              class="pr-label-pill"
              :style="getLabelBadgeStyle(lbl.color)"
              :title="lbl.description || lbl.name"
            >
              {{ lbl.name }}
            </span>
          </div>
        </div>

        <!-- PR Overview Stats (if detailed data is loaded) -->
        <div class="detail-stats-bar" v-if="selectedPR.commits !== undefined || selectedPR.changed_files !== undefined">
          <div class="stat-item" v-if="selectedPR.commits !== undefined">
            <span class="stat-number">{{ selectedPR.commits }}</span>
            <span class="stat-label">Commits</span>
          </div>
          <div class="stat-item" v-if="selectedPR.changed_files !== undefined">
            <span class="stat-number">{{ selectedPR.changed_files }}</span>
            <span class="stat-label">Changed Files</span>
          </div>
          <div class="stat-item stat-diff" v-if="selectedPR.additions !== undefined">
            <span class="diff-add">+{{ selectedPR.additions }}</span>
            <span class="diff-del">-{{ selectedPR.deletions }}</span>
          </div>
        </div>

        <!-- PR Description / Body -->
        <div class="detail-body-section">
          <div class="section-title">Description</div>
          <div class="pr-body-content" v-if="selectedPR.body">
            <pre class="pr-body-text">{{ selectedPR.body }}</pre>
          </div>
          <div class="pr-no-body" v-else>
            <em>No description provided for this pull request.</em>
          </div>
        </div>

        <!-- Quick Branch Checkout Help -->
        <div class="checkout-help-card">
          <div class="help-header">
            <LaptopBranchIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
            <span>Local Branch Information</span>
          </div>
          <div class="help-content">
            <p>Target local branch: <code>pr/{{ selectedPR.number }}-{{ sanitizeBranch(selectedPR.head.ref) }}</code></p>
            <div class="code-snippet-box">
              <code>git fetch origin {{ remoteInfo?.provider === 'gitlab' ? `refs/merge-requests/${selectedPR.number}/head` : `refs/pull/${selectedPR.number}/head` }}:pr/{{ selectedPR.number }}-{{ sanitizeBranch(selectedPR.head.ref) }} &amp;&amp; git checkout pr/{{ selectedPR.number }}-{{ sanitizeBranch(selectedPR.head.ref) }}</code>
              <button class="btn-icon-small" @click="copyCheckoutCommand(selectedPR)" title="Copy CLI command">
                <CopyIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              </button>
            </div>
          </div>
        </div>
      </main>

      <!-- Empty detail pane when none selected -->
      <main class="pr-detail-pane pr-detail-placeholder" v-else>
        <div class="placeholder-content">
          <PullRequestIcon viewBox="0 0 24 24" width="48" height="48" stroke="currentColor" stroke-width="1.2" fill="none" />
          <h3>Select a Pull Request</h3>
          <p>Choose a pull request from the list to view its description, branches, changed stats, and checkout locally.</p>
        </div>
      </main>
    </div>

    <!-- Personal Access Token (PAT) Modal -->
    <div v-if="showTokenModal" class="modal-backdrop" @click.self="showTokenModal = false">
      <div class="modal-card">
        <div class="modal-header">
          <div class="modal-title-group">
            <KeyIcon viewBox="0 0 24 24" width="18" height="18" stroke="currentColor" stroke-width="2" fill="none" />
            <h3>Personal Access Token (PAT)</h3>
          </div>
          <button class="close-btn" @click="showTokenModal = false">
            <CloseIcon viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" />
          </button>
        </div>

        <div class="modal-body">
          <p class="modal-description">
            For private repositories or to avoid GitHub/GitLab API rate limits (60 req/hr unauthenticated), enter a Personal Access Token with <code>repo</code> or <code>read_api</code> scope.
          </p>

          <div class="form-group">
            <label>Provider: <strong>{{ providerDisplayName }}</strong></label>
            <input
              v-model="tokenInput"
              type="password"
              placeholder="ghp_... or glpat-..."
              class="text-input"
              autocomplete="off"
            />
          </div>

          <div class="token-help-links">
            <a v-if="remoteInfo?.provider === 'gitlab'" href="https://gitlab.com/-/user_settings/personal_access_tokens" target="_blank" @click.prevent="openInBrowser('https://gitlab.com/-/user_settings/personal_access_tokens')">
              Generate GitLab Token →
            </a>
            <a v-else href="https://github.com/settings/tokens/new?scopes=repo&description=BeGit%20App" target="_blank" @click.prevent="openInBrowser('https://github.com/settings/tokens/new?scopes=repo&description=BeGit%20App')">
              Generate GitHub Token (Classic) →
            </a>
          </div>
        </div>

        <div class="modal-footer">
          <button v-if="apiToken" class="btn-danger" @click="clearToken">Remove Token</button>
          <div style="flex: 1;"></div>
          <button class="btn-secondary" @click="showTokenModal = false">Cancel</button>
          <button class="btn-primary" @click="saveToken">Save Token</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import type { PullRequest, RemoteInfo, Project } from '../types';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { notify } from '../composables/useToasts';
import { invokeGit } from '../composables/useActivityLog';

// Icons
import PullRequestIcon from '../assets/icons/git-pull-request.svg?component';
import KeyIcon from '../assets/icons/key.svg?component';
import FetchIcon from '../assets/icons/fetch.svg?component';
import SearchIcon from '../assets/icons/search.svg?component';
import CloseIcon from '../assets/icons/close.svg?component';
import ActionIcon from '../assets/icons/action.svg?component';
import PlusIcon from '../assets/icons/plus.svg?component';
import CopyIcon from '../assets/icons/copy.svg?component';
import ErrorIcon from '../assets/icons/error.svg?component';
import CloudIcon from '../assets/icons/Cloud.svg?component';
import GitBranchIcon from '../assets/icons/git-branch.svg?component';
import CloudBranchIcon from '../assets/icons/cloud-branch.svg?component';
import LaptopBranchIcon from '../assets/icons/laptop-branch.svg?component';
import AuthorAvatar from '../components/Common/AuthorAvatar.vue';

const props = defineProps<{
  project: Project | null;
  active?: boolean;
}>();

const emit = defineEmits<{
  (e: 'checkout-pr', branchName: string): void;
  (e: 'navigate', view: string): void;
}>();

// State
const remoteInfo = ref<RemoteInfo | null>(null);
const pullRequests = ref<PullRequest[]>([]);
const selectedPR = ref<PullRequest | null>(null);
const isLoading = ref(false);
const isCheckingOut = ref(false);
const errorMessage = ref('');
const searchQuery = ref('');
const selectedState = ref<'open' | 'merged' | 'closed' | 'all'>('open');

// Token modal & storage
const showTokenModal = ref(false);
const tokenInput = ref('');
const apiToken = ref('');

const providerDisplayName = computed(() => {
  if (!remoteInfo.value) return 'Git Provider';
  switch (remoteInfo.value.provider) {
    case 'github': return 'GitHub';
    case 'gitlab': return 'GitLab';
    case 'bitbucket': return 'Bitbucket';
    case 'azure_devops': return 'Azure DevOps';
    default: return 'Git Remote';
  }
});

const stateCounts = computed(() => {
  const counts: Record<string, number> = { open: 0, merged: 0, closed: 0 };
  pullRequests.value.forEach(pr => {
    const s = getNormalizedState(pr);
    if (counts[s] !== undefined) counts[s]++;
  });
  return counts;
});

const filteredPRs = computed(() => {
  let list = pullRequests.value;

  if (selectedState.value !== 'all') {
    list = list.filter(pr => getNormalizedState(pr) === selectedState.value);
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase().trim();
    list = list.filter(pr => {
      const matchNumber = `#${pr.number}`.includes(q) || String(pr.number).includes(q);
      const matchTitle = pr.title.toLowerCase().includes(q);
      const matchAuthor = pr.user?.login?.toLowerCase().includes(q);
      const matchSource = pr.head?.ref?.toLowerCase().includes(q);
      const matchTarget = pr.base?.ref?.toLowerCase().includes(q);
      return matchNumber || matchTitle || matchAuthor || matchSource || matchTarget;
    });
  }

  return list;
});

function getNormalizedState(pr: PullRequest): 'open' | 'merged' | 'closed' {
  if (pr.merged_at || pr.state === 'merged') return 'merged';
  if (pr.state === 'closed') return 'closed';
  return 'open';
}

function getStatusText(pr: PullRequest): string {
  if (pr.draft) return 'Draft';
  const s = getNormalizedState(pr);
  if (s === 'merged') return 'Merged';
  if (s === 'closed') return 'Closed';
  return 'Open';
}

function getLabelBadgeStyle(colorHex?: string) {
  if (!colorHex) return { background: 'var(--surface-subtle)', color: 'var(--text-main)' };
  const hex = colorHex.startsWith('#') ? colorHex : `#${colorHex}`;
  return {
    backgroundColor: `${hex}26`,
    borderColor: `${hex}55`,
    color: hex
  };
}

function formatRelativeTime(dateStr?: string) {
  if (!dateStr) return '';
  const date = new Date(dateStr);
  const now = new Date();
  const diffSec = Math.floor((now.getTime() - date.getTime()) / 1000);

  if (diffSec < 60) return 'just now';
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
  if (diffSec < 2592000) return `${Math.floor(diffSec / 86400)}d ago`;
  return date.toLocaleDateString();
}

function truncate(str: string, maxLen: number) {
  if (!str) return '';
  return str.length > maxLen ? str.substring(0, maxLen - 1) + '…' : str;
}

function sanitizeBranch(name?: string) {
  if (!name) return 'patch';
  return name.replace(/[^a-zA-Z0-9_-]/g, '-').replace(/-+/g, '-').replace(/^-|-$/g, '');
}

function selectPullRequest(pr: PullRequest) {
  selectedPR.value = pr;
  fetchPRDetails(pr);
}

function setStateFilter(state: 'open' | 'merged' | 'closed' | 'all') {
  selectedState.value = state;
  if (remoteInfo.value) {
    loadPullRequests();
  }
}

// Token storage key
function getTokenStorageKey() {
  const provider = remoteInfo.value?.provider || 'git';
  const owner = remoteInfo.value?.owner || 'default';
  return `begit_token_${provider}_${owner}`;
}

function loadSavedToken() {
  const key = getTokenStorageKey();
  const saved = localStorage.getItem(key) || localStorage.getItem('begit_git_token') || '';
  apiToken.value = saved;
  tokenInput.value = saved;
}

function saveToken() {
  const key = getTokenStorageKey();
  const val = tokenInput.value.trim();
  if (val) {
    localStorage.setItem(key, val);
    localStorage.setItem('begit_git_token', val);
    apiToken.value = val;
    notify('API Token saved', 'success');
  } else {
    clearToken();
  }
  showTokenModal.value = false;
  loadPullRequests();
}

function clearToken() {
  const key = getTokenStorageKey();
  localStorage.removeItem(key);
  localStorage.removeItem('begit_git_token');
  apiToken.value = '';
  tokenInput.value = '';
  notify('API Token removed', 'info');
  showTokenModal.value = false;
  loadPullRequests();
}

// Fetch Remote Info
async function detectRemote() {
  if (!props.project?.path) {
    remoteInfo.value = null;
    return;
  }

  try {
    const info = await invoke<RemoteInfo | null>('get_remote_info', { path: props.project.path });
    remoteInfo.value = info;
    loadSavedToken();
    if (info) {
      await loadPullRequests();
    }
  } catch (e: any) {
    console.error('Failed to detect remote info:', e);
    remoteInfo.value = null;
  }
}

// Load PR List from Provider REST API
async function loadPullRequests() {
  if (!props.project?.path || !remoteInfo.value) return;

  const info = remoteInfo.value;
  isLoading.value = true;
  errorMessage.value = '';

  try {
    if (info.provider === 'github') {
      const stateParam = selectedState.value === 'all' ? 'all' : (selectedState.value === 'open' ? 'open' : 'closed');
      const url = `https://api.github.com/repos/${info.owner}/${info.repo}/pulls?state=${stateParam}&per_page=50&sort=updated&direction=desc`;
      
      const headers: Record<string, string> = {
        'Accept': 'application/vnd.github.v3+json',
      };
      if (apiToken.value) {
        headers['Authorization'] = `token ${apiToken.value}`;
      }

      const res = await fetch(url, { headers });
      if (!res.ok) {
        if (res.status === 401 || res.status === 403 || res.status === 404) {
          throw new Error(`GitHub API Error (${res.status}): ${res.statusText}. For private repositories or rate limits, configure a Personal Access Token.`);
        }
        throw new Error(`GitHub API Error: ${res.status} ${res.statusText}`);
      }

      const data = await res.json();
      pullRequests.value = data.map((item: any) => ({
        id: item.id,
        number: item.number,
        title: item.title,
        state: item.state,
        body: item.body,
        html_url: item.html_url,
        created_at: item.created_at,
        updated_at: item.updated_at,
        closed_at: item.closed_at,
        merged_at: item.merged_at,
        draft: item.draft || false,
        user: {
          login: item.user?.login || 'unknown',
          avatar_url: item.user?.avatar_url || '',
          html_url: item.user?.html_url,
        },
        head: {
          ref: item.head?.ref || 'head',
          sha: item.head?.sha || '',
          repo: item.head?.repo ? {
            full_name: item.head.repo.full_name,
            clone_url: item.head.repo.clone_url,
          } : null,
        },
        base: {
          ref: item.base?.ref || 'base',
          sha: item.base?.sha || '',
        },
        labels: (item.labels || []).map((l: any) => ({
          name: l.name,
          color: l.color,
          description: l.description,
        })),
        comments: item.comments || 0,
        review_comments: item.review_comments || 0,
        commits: item.commits,
        additions: item.additions,
        deletions: item.deletions,
        changed_files: item.changed_files,
      }));

      // Select first PR if none selected or selected was replaced
      if (pullRequests.value.length > 0) {
        if (!selectedPR.value || !pullRequests.value.some(p => p.id === selectedPR.value?.id)) {
          selectedPR.value = pullRequests.value[0];
          fetchPRDetails(pullRequests.value[0]);
        }
      } else {
        selectedPR.value = null;
      }

    } else if (info.provider === 'gitlab') {
      const stateParam = selectedState.value === 'all' ? 'all' : (selectedState.value === 'open' ? 'opened' : (selectedState.value === 'merged' ? 'merged' : 'closed'));
      const projectPathEncoded = encodeURIComponent(`${info.owner}/${info.repo}`);
      const apiBase = info.api_base_url || 'https://gitlab.com/api/v4';
      const url = `${apiBase}/projects/${projectPathEncoded}/merge_requests?state=${stateParam}&per_page=50`;

      const headers: Record<string, string> = {};
      if (apiToken.value) {
        headers['PRIVATE-TOKEN'] = apiToken.value;
      }

      const res = await fetch(url, { headers });
      if (!res.ok) {
        throw new Error(`GitLab API Error (${res.status}): ${res.statusText}`);
      }

      const data = await res.json();
      pullRequests.value = data.map((item: any) => ({
        id: item.id,
        number: item.iid,
        title: item.title,
        state: item.state === 'opened' ? 'open' : item.state,
        body: item.description,
        html_url: item.web_url,
        created_at: item.created_at,
        updated_at: item.updated_at,
        closed_at: item.closed_at,
        merged_at: item.merged_at,
        draft: item.draft || item.work_in_progress || false,
        user: {
          login: item.author?.username || 'unknown',
          avatar_url: item.author?.avatar_url || '',
          html_url: item.author?.web_url,
        },
        head: {
          ref: item.source_branch || 'source',
          sha: item.sha || '',
        },
        base: {
          ref: item.target_branch || 'target',
          sha: '',
        },
        labels: (item.labels || []).map((l: string) => ({
          name: l,
          color: '697689',
        })),
        comments: item.user_notes_count || 0,
        review_comments: 0,
      }));

      if (pullRequests.value.length > 0) {
        if (!selectedPR.value || !pullRequests.value.some(p => p.id === selectedPR.value?.id)) {
          selectedPR.value = pullRequests.value[0];
        }
      } else {
        selectedPR.value = null;
      }

    } else {
      errorMessage.value = `Automatic PR fetching for provider '${info.provider}' is not yet supported. You can still checkout PRs via branch refspec.`;
    }
  } catch (e: any) {
    console.error('PR Fetch error:', e);
    errorMessage.value = e.message || 'Failed to load pull requests.';
  } finally {
    isLoading.value = false;
  }
}

// Fetch single PR detailed stats (commits, changed_files, additions/deletions)
async function fetchPRDetails(pr: PullRequest) {
  if (!remoteInfo.value || remoteInfo.value.provider !== 'github') return;
  if (pr.commits !== undefined && pr.additions !== undefined) return;

  try {
    const url = `https://api.github.com/repos/${remoteInfo.value.owner}/${remoteInfo.value.repo}/pulls/${pr.number}`;
    const headers: Record<string, string> = { 'Accept': 'application/vnd.github.v3+json' };
    if (apiToken.value) headers['Authorization'] = `token ${apiToken.value}`;

    const res = await fetch(url, { headers });
    if (res.ok) {
      const data = await res.json();
      pr.commits = data.commits;
      pr.additions = data.additions;
      pr.deletions = data.deletions;
      pr.changed_files = data.changed_files;
      pr.comments = data.comments;
      pr.review_comments = data.review_comments;
    }
  } catch {
    // Non-critical background enrichment
  }
}

// 1-Click Checkout PR Locally
async function checkoutSelectedPR() {
  if (!props.project?.path || !selectedPR.value) return;

  const pr = selectedPR.value;
  isCheckingOut.value = true;

  try {
    const branchName = await invokeGit<string>(
      'checkout_pull_request',
      {
        path: props.project.path,
        prNumber: pr.number,
        branchName: pr.head.ref,
        provider: remoteInfo.value?.provider || 'github',
      },
      `Checkout PR #${pr.number}`
    );

    notify(`Checked out Pull Request #${pr.number} into branch '${branchName}'`, 'success');
    emit('checkout-pr', branchName);
  } catch (e: any) {
    notify(`Failed to checkout PR #${pr.number}: ${e}`, 'error');
  } finally {
    isCheckingOut.value = false;
  }
}

function openInBrowser(url?: string) {
  if (!url) return;
  openUrl(url).catch(() => window.open(url, '_blank'));
}

function openCreatePR() {
  if (!remoteInfo.value) return;
  const url = remoteInfo.value.provider === 'gitlab'
    ? `${remoteInfo.value.web_url}/-/merge_requests/new`
    : `${remoteInfo.value.web_url}/compare`;
  openInBrowser(url);
}

function copyCheckoutCommand(pr: PullRequest) {
  const isGitlab = remoteInfo.value?.provider === 'gitlab';
  const ref = isGitlab ? `refs/merge-requests/${pr.number}/head` : `refs/pull/${pr.number}/head`;
  const clean = sanitizeBranch(pr.head.ref);
  const cmd = `git fetch origin ${ref}:pr/${pr.number}-${clean} && git checkout pr/${pr.number}-${clean}`;
  navigator.clipboard.writeText(cmd);
  notify('Checkout command copied to clipboard', 'info');
}

watch(() => props.project?.path, () => {
  detectRemote();
});

onMounted(() => {
  detectRemote();
});
</script>

<style scoped>
.pull-requests-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  background: var(--bg-main);
  color: var(--text-main);
  overflow: hidden;
}

/* Header */
.pr-page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 20px;
  background: var(--bg-card);
  border-bottom: 1px solid var(--border);
  gap: 16px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 14px;
}

.pr-header-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pr-header-title h1 {
  font-size: 1.15rem;
  font-weight: 700;
  margin: 0;
  color: var(--text-main);
}

.title-icon {
  color: var(--accent-blue);
}

.remote-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 8px;
  border-radius: 6px;
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  font-size: 0.8rem;
}

.provider-tag {
  font-weight: 700;
  text-transform: capitalize;
  padding: 1px 5px;
  border-radius: 4px;
  font-size: 0.72rem;
}

.provider-github {
  background: rgba(88, 166, 255, 0.2);
  color: #58a6ff;
}

.provider-gitlab {
  background: rgba(252, 109, 38, 0.2);
  color: #fc6d26;
}

.remote-repo-name {
  color: var(--text-muted);
  font-family: monospace;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* Content Master-Detail */
.pr-page-content {
  display: flex;
  flex: 1;
  overflow: hidden;
}

/* Left: List Pane */
.pr-list-pane {
  width: 400px;
  min-width: 320px;
  max-width: 500px;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  background: var(--bg-card);
  overflow: hidden;
}

.pr-filter-bar {
  padding: 12px;
  border-bottom: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.search-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 10px;
  color: var(--text-muted);
  pointer-events: none;
}

.pr-search-input {
  width: 100%;
  padding: 7px 28px 7px 30px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-main);
  color: var(--text-main);
  font-size: 0.82rem;
  outline: none;
  transition: border-color 0.15s ease;
}

.pr-search-input:focus {
  border-color: var(--accent-blue);
}

.clear-search-btn {
  position: absolute;
  right: 6px;
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
}

.state-tabs {
  display: flex;
  gap: 4px;
  background: var(--surface-subtle);
  padding: 3px;
  border-radius: 6px;
}

.state-tab {
  flex: 1;
  padding: 4px 6px;
  border: none;
  background: none;
  color: var(--text-muted);
  font-size: 0.78rem;
  font-weight: 600;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  transition: all 0.15s ease;
}

.state-tab.active {
  background: var(--bg-card);
  color: var(--text-main);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.state-count {
  font-size: 0.7rem;
  padding: 1px 5px;
  border-radius: 10px;
  background: var(--surface-subtle);
  color: var(--text-muted);
}

.state-tab.active .state-count {
  background: var(--accent-blue);
  color: #fff;
}

/* PR Items List */
.pr-items-container {
  flex: 1;
  overflow-y: auto;
  position: relative;
}

.pr-items-list {
  display: flex;
  flex-direction: column;
}

.pr-item-card {
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
  cursor: pointer;
  transition: background 0.15s ease;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.pr-item-card:hover {
  background: var(--surface-hover);
}

.pr-item-card.selected {
  background: var(--surface-selected);
  border-left: 3px solid var(--accent-blue);
}

.pr-card-header {
  display: flex;
  align-items: flex-start;
  gap: 8px;
}

.pr-state-indicator {
  padding: 3px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  margin-top: 2px;
}

.state-open {
  color: #3fb950;
  background: rgba(63, 185, 80, 0.15);
}

.state-merged {
  color: #a371f7;
  background: rgba(163, 113, 247, 0.15);
}

.state-closed {
  color: #f85149;
  background: rgba(248, 81, 73, 0.15);
}

.pr-card-title-group {
  display: flex;
  gap: 6px;
  line-height: 1.35;
  flex: 1;
}

.pr-number {
  font-weight: 700;
  color: var(--text-muted);
  font-size: 0.85rem;
}

.pr-title {
  font-weight: 600;
  font-size: 0.86rem;
  color: var(--text-main);
  word-break: break-word;
}

.pr-card-labels {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  margin-left: 24px;
}

.pr-label-pill {
  font-size: 0.7rem;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 12px;
  border: 1px solid transparent;
  line-height: 1.2;
}

.pr-card-meta {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-left: 24px;
  font-size: 0.75rem;
  color: var(--text-muted);
  gap: 8px;
}

.branch-flow {
  display: flex;
  align-items: center;
  gap: 4px;
  overflow: hidden;
}

.branch-pill {
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--surface-subtle);
  font-family: monospace;
  font-size: 0.72rem;
  max-width: 110px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.branch-pill.source {
  color: var(--accent-blue);
  border: 1px solid rgba(88, 166, 255, 0.2);
}

.branch-pill.target {
  color: var(--text-muted);
}

.flow-arrow {
  color: var(--text-muted);
  font-size: 0.7rem;
}

.author-and-time {
  display: flex;
  align-items: center;
  gap: 4px;
}

.pr-dot {
  opacity: 0.5;
}

.pr-comments-count {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 0.72rem;
  font-weight: 600;
}

/* Right Detail Pane */
.pr-detail-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 24px 32px;
  overflow-y: auto;
  gap: 20px;
  background: var(--bg-main);
}

.detail-header-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding-bottom: 18px;
  border-bottom: 1px solid var(--border);
}

.detail-top-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.detail-title-wrapper {
  display: flex;
  align-items: baseline;
  gap: 10px;
  flex: 1;
}

.detail-number {
  font-size: 1.4rem;
  font-weight: 700;
  color: var(--text-muted);
}

.detail-title {
  font-size: 1.35rem;
  font-weight: 700;
  margin: 0;
  color: var(--text-main);
  line-height: 1.3;
}

.detail-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.checkout-btn {
  background: #238636;
  border-color: #2ea043;
}

.checkout-btn:hover {
  background: #2ea043;
}

.detail-status-bar {
  display: flex;
  align-items: center;
  gap: 14px;
  flex-wrap: wrap;
  font-size: 0.85rem;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 10px;
  border-radius: 16px;
  font-weight: 700;
  font-size: 0.8rem;
}

.status-open {
  background: #238636;
  color: #fff;
}

.status-merged {
  background: #8957e5;
  color: #fff;
}

.status-closed {
  background: #da3633;
  color: #fff;
}

.detail-branches {
  display: flex;
  align-items: center;
  gap: 6px;
}

.detail-branches .branch-pill {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  max-width: none;
  padding: 3px 8px;
  font-size: 0.8rem;
}

.author-info {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-muted);
}

.detail-labels-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

/* Stats Bar */
.detail-stats-bar {
  display: flex;
  gap: 20px;
  padding: 12px 18px;
  border-radius: 8px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  width: fit-content;
}

.stat-item {
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.stat-number {
  font-size: 1.1rem;
  font-weight: 700;
  color: var(--text-main);
}

.stat-label {
  font-size: 0.78rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stat-diff {
  font-family: monospace;
  font-weight: 700;
  gap: 8px;
}

.diff-add {
  color: #3fb950;
}

.diff-del {
  color: #f85149;
}

/* Description */
.detail-body-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--text-main);
}

.pr-body-content {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 16px;
  max-height: 400px;
  overflow-y: auto;
}

.pr-body-text {
  margin: 0;
  font-family: inherit;
  font-size: 0.88rem;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--text-main);
}

.pr-no-body {
  padding: 16px;
  background: var(--bg-card);
  border-radius: 8px;
  border: 1px solid var(--border);
  color: var(--text-muted);
}

/* Checkout Help Card */
.checkout-help-card {
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 8px;
  padding: 14px 18px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.help-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  font-size: 0.86rem;
  color: var(--accent-blue);
}

.help-content {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 0.82rem;
  color: var(--text-muted);
}

.help-content p {
  margin: 0;
}

.code-snippet-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 6px;
  background: var(--bg-main);
  border: 1px solid var(--border);
  gap: 8px;
}

.code-snippet-box code {
  font-family: monospace;
  font-size: 0.75rem;
  color: var(--text-main);
  word-break: break-all;
}

/* Placeholder state */
.pr-detail-placeholder {
  align-items: center;
  justify-content: center;
}

.placeholder-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  max-width: 360px;
  color: var(--text-muted);
  gap: 10px;
}

.placeholder-content h3 {
  margin: 0;
  color: var(--text-main);
  font-size: 1.1rem;
}

.placeholder-content p {
  margin: 0;
  font-size: 0.85rem;
  line-height: 1.5;
}

/* States */
.pr-loading-state,
.pr-empty-state,
.pr-error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 40px 20px;
  gap: 12px;
  color: var(--text-muted);
}

.pr-error-state .error-icon {
  color: var(--danger-text);
}

.error-actions {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.spinner {
  width: 24px;
  height: 24px;
  border: 3px solid rgba(88, 166, 255, 0.2);
  border-top-color: var(--accent-blue);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.spin-icon {
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Buttons */
.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  border-radius: 6px;
  font-size: 0.82rem;
  font-weight: 600;
  background: var(--accent-blue);
  color: #fff;
  border: 1px solid var(--accent-blue);
  cursor: pointer;
  transition: opacity 0.15s ease;
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.82rem;
  font-weight: 600;
  background: var(--surface-subtle);
  color: var(--text-main);
  border: 1px solid var(--border);
  cursor: pointer;
  transition: background 0.15s ease;
}

.btn-secondary:hover {
  background: var(--surface-hover);
}

.btn-icon {
  background: var(--surface-subtle);
  border: 1px solid var(--border);
  color: var(--text-muted);
  padding: 6px;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.btn-icon:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

.btn-icon.has-token {
  color: #3fb950;
  border-color: rgba(63, 185, 80, 0.35);
}

.btn-icon-small {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.btn-icon-small:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

.btn-danger {
  background: rgba(248, 81, 73, 0.15);
  border: 1px solid rgba(248, 81, 73, 0.35);
  color: #f85149;
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 0.82rem;
  font-weight: 600;
  cursor: pointer;
}

/* Modal */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-card {
  width: 480px;
  max-width: 90vw;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: 10px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.modal-title-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-title-group h3 {
  margin: 0;
  font-size: 1rem;
  font-weight: 700;
  color: var(--text-main);
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 4px;
}

.modal-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.modal-description {
  margin: 0;
  font-size: 0.85rem;
  line-height: 1.5;
  color: var(--text-muted);
}

.modal-description code {
  background: var(--surface-subtle);
  padding: 2px 5px;
  border-radius: 4px;
  font-size: 0.8rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 0.82rem;
  color: var(--text-muted);
}

.text-input {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-main);
  color: var(--text-main);
  font-size: 0.85rem;
  outline: none;
}

.text-input:focus {
  border-color: var(--accent-blue);
}

.token-help-links a {
  color: var(--accent-blue);
  font-size: 0.8rem;
  text-decoration: none;
}

.token-help-links a:hover {
  text-decoration: underline;
}

.modal-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 20px;
  border-top: 1px solid var(--border);
  background: var(--surface-subtle);
}
</style>
