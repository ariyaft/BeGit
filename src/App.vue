<script setup lang="ts">
import type { AppView } from './types';
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';

// Components
import ContributorsReport from './pages/ContributorsReport.vue';
import ProjectPickerPage from './pages/ProjectPickerPage.vue';
import SourceControlPage from './pages/SourceControlPage.vue';
import TabsBar from './components/Common/TabsBar.vue';
import ActivityBar from './components/Common/ActivityBar.vue';
import PendingOverlay from './components/Common/PendingOverlay.vue';
import ToastNotifications from './components/Common/ToastNotifications.vue';
import SplashScreen from './components/Common/SplashScreen.vue';
import GitProfileModal from './components/Common/GitProfileModal.vue';
import CloneRepositoryModal from './components/Common/CloneRepositoryModal.vue';
import RepositoryStatusFooter from './components/Common/RepositoryStatusFooter.vue';
import TimelinePage from './pages/TimelinePage.vue';
import FileExplorerPage from './pages/FileExplorerPage.vue';
import ActivityLogPage from './pages/ActivityLogPage.vue';
import PullRequestsPage from './pages/PullRequestsPage.vue';
import ReflogView from './components/Common/ReflogView.vue';
import CommandPalette from './components/Common/CommandPalette.vue';
import ResolveDivergentBranchModal from './components/source-control/ResolveDivergentBranchModal.vue';

// Composables
import { useTheme } from './composables/useTheme';
import { useZoom } from './composables/useZoom';
import { useResizer } from './composables/useResizer';
import { useProjects } from './composables/useProjects';
import { useGitActions } from './composables/useGitActions';
import { useCommits } from './composables/useCommits';
import { notify } from './composables/useToasts';
import { invokeGit } from './composables/useActivityLog';

// --- State ---
const activeView = ref<AppView>('source-control');
const globalPendingMessage = ref('');
const isAppLoading = ref(true);

async function withPending<T>(msg: string, fn: () => Promise<T>): Promise<T> {
  globalPendingMessage.value = msg;
  try {
    return await fn();
  } finally {
    globalPendingMessage.value = '';
  }
}

// --- Composables Initialization ---
const { 
  leftSidebarWidth, 
  rightSidebarWidth, 
  startDragLeft, 
  startDragRight,
  toggleLeftSidebar,
  toggleRightSidebar
} = useResizer();

const {
  projects,
  activeTabIndex,
  projectHistory,
  showProjectPicker,
  currentProject,
  loadHistory,
  removeFromHistory,
  clearHistory,
  loadProjectByPath,
  refreshProject,
  closeTab,
  commitLimit
} = useProjects();

const {
  detailsMode,
  commitDetailsLoading,
  selectedFilePath,
  selectedFileDiff,
  commits,
  selectedCommit,
  selectedCommits,
  workingChangeCount,
  hasWorkingChanges,
  detailsTarget,
  displayBranch,
  selectCommit,
  handleItemClick,
  closeRightSidebar,
  showWorkingChanges,
  selectFile
} = useCommits(currentProject, activeView, rightSidebarWidth, toggleRightSidebar);

const {
  isLoading,
  fetchRemoteAction,
  pullAction,
  pushAction,
  stashChangesAction,
  popStashAction,
  stageFile,
  stageAll,
  unstageFile,
  commitChanges,
  onBranchDoubleClick,
  handleBranchAction,
  handleCommitAction,
  handleCherryPickAction
} = useGitActions(currentProject, activeTabIndex, refreshProject, withPending, (branchName) => {
  divergentBranchModal.value?.open(branchName);
});

const divergentBranchModal = ref<InstanceType<typeof ResolveDivergentBranchModal> | null>(null);

// --- UI Navigation Methods ---
function handleSourceControlClick() {
  if (activeView.value !== 'source-control') {
    activeView.value = 'source-control';
    if (leftSidebarWidth.value === 0) toggleLeftSidebar();
  } else {
    toggleLeftSidebar();
  }
}

function selectActivityView(view: AppView) {
  if (view === 'source-control') {
    handleSourceControlClick();
  } else if (view === 'horizontal-graph' || view === 'file-explorer' || view === 'pull-requests' || view === 'contributors-report' || view === 'reflog') {
    activeView.value = view;
    leftSidebarWidth.value = 0;
  } else {
    activeView.value = view;
  }
}

async function handlePullRequestCheckedOut(_branchName: string) {
  if (activeTabIndex.value >= 0 && activeTabIndex.value < projects.value.length) {
    await refreshProject(activeTabIndex.value, false);
  }
}

function openFooterBranches() {
  if (activeView.value !== 'source-control') activeView.value = 'source-control';
  if (leftSidebarWidth.value === 0) toggleLeftSidebar();
}

function openFooterChanges() {
  openFooterBranches();
  showWorkingChanges();
}

async function onChangeCommitLimit(newLimit: number) {
  console.log("Changing commit limit to:", newLimit);
  commitLimit.value = newLimit;
  if (activeTabIndex.value >= 0 && activeTabIndex.value < projects.value.length) {
    await refreshProject(activeTabIndex.value, false);
  }
}

async function openFromHistory(pathStr: string) {
  if (!projects.value.some(p => p.path === pathStr)) {
    try {
      await loadProjectByPath(pathStr);
    } catch (e) {
      notify(`Failed to open project: ${e}`, 'error');
      return;
    }
  }
  const idx = projects.value.findIndex(p => p.path === pathStr);
  if (idx !== -1) {
    activeTabIndex.value = idx;
    showProjectPicker.value = false;
  }
}

async function openRepository() {
  try {
    const selectedPath = await open({
      directory: true,
      multiple: false,
      title: 'Select a Git Repository'
    });
    
    if (selectedPath) {
      const pathStr = selectedPath as string;
      const existingIndex = projects.value.findIndex(p => p.path === pathStr);
      if (existingIndex !== -1) {
        activeTabIndex.value = existingIndex;
        showProjectPicker.value = false;
        return;
      }
      try {
        const newIndex = await loadProjectByPath(pathStr);
        activeTabIndex.value = newIndex;
        showProjectPicker.value = false;
      } catch (e) {
        notify(`Failed to open repository: ${e}`, 'error');
      }
    }
  } catch (e) {
    console.error("Failed to open dialog or call rust:", e);
    notify(`Error: ${e}`);
  }
}

const showCloneModal = ref(false);

async function handleRepositoryCloned(targetPath: string) {
  try {
    const existingIndex = projects.value.findIndex(p => p.path === targetPath);
    if (existingIndex !== -1) {
      activeTabIndex.value = existingIndex;
    } else {
      try {
        const newIndex = await loadProjectByPath(targetPath);
        activeTabIndex.value = newIndex;
      } catch (e) {
        notify(`Cloned repository to ${targetPath}, but failed to load its contents: ${e}`, 'error');
        return;
      }
    }
    showProjectPicker.value = false;
    showCloneModal.value = false;
    notify('Repository cloned successfully', 'success');
  } catch (e) {
    console.error('Failed to open cloned repository:', e);
    notify(`Error opening cloned repository: ${e}`);
  }
}

function openNewProjectPage() {
  showProjectPicker.value = true;
  detailsMode.value = null;
  selectedFileDiff.value = null;
  selectedFilePath.value = null;
}

function closeNewProjectPage() {
  if (projects.value.length > 0) showProjectPicker.value = false;
}

function selectProjectTab(index: number) {
  showProjectPicker.value = false;
  activeTabIndex.value = index;
}

async function loadAuthors(proj: any) {
  if (proj.authors && proj.authors.length > 0) return;
  try {
    const authors = await invoke('get_authors', { path: proj.path }) as any[];
    proj.authors = authors;
  } catch (e) {
    console.error("Failed to load authors:", e);
  }
}

// --- Persistence (Local Storage) ---
watch(() => projects.value.length, () => {
  const paths = projects.value.map(p => p.path);
  localStorage.setItem('gitTreeProjects', JSON.stringify(paths));
});

watch(activeTabIndex, async (newIndex, oldIndex) => {
  localStorage.setItem('gitTreeActiveTab', newIndex.toString());
  if (newIndex !== oldIndex && projects.value[newIndex]) {
    activeView.value = 'source-control';
    if (leftSidebarWidth.value === 0) toggleLeftSidebar();
    detailsMode.value = null;
    selectedFileDiff.value = null;
    selectedFilePath.value = null;
    await refreshProject(newIndex, true);
  }
});

// --- Lifecycle ---
onMounted(async () => {
  const minSplashTime = new Promise(resolve => setTimeout(resolve, 2000));
  
  const { toggleTheme } = useTheme();
  useZoom();
  await listen('toggle-theme', () => {
    toggleTheme();
  });

  window.addEventListener('focus', () => {
    if (activeTabIndex.value >= 0 && activeTabIndex.value < projects.value.length) {
      refreshProject(activeTabIndex.value, true);
    }
  });

  setInterval(() => {
    if (document.visibilityState === 'visible' && activeTabIndex.value >= 0 && activeTabIndex.value < projects.value.length) {
      refreshProject(activeTabIndex.value, true);
    }
  }, 30000);
  
  loadHistory();

  const saved = localStorage.getItem('gitTreeProjects');
  if (saved) {
    try {
      const paths = JSON.parse(saved) as string[];
      for (const p of paths) {
        await loadProjectByPath(p);
      }
      const active = localStorage.getItem('gitTreeActiveTab');
      if (active && !isNaN(Number(active))) {
        activeTabIndex.value = Math.min(Number(active), Math.max(0, projects.value.length - 1));
      }
    } catch (e) {
      console.error("Failed to restore projects from local storage");
    }
  }
  
  if (projects.value.length === 0) {
    showProjectPicker.value = true;
  }

  window.addEventListener('keydown', handleGlobalKeydown);

  await minSplashTime;
  isAppLoading.value = false;
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeydown);
});

// --- Command Palette State & Handlers ---
const showCommandPalette = ref(false);
const trackedFiles = ref<string[]>([]);
const explorerSelectedFilePath = ref<string | null>(null);

async function loadTrackedFiles() {
  if (!currentProject.value?.path) {
    trackedFiles.value = [];
    return;
  }
  try {
    const files = await invoke<string[]>('get_tracked_files', { path: currentProject.value.path });
    trackedFiles.value = files;
  } catch (e) {
    console.warn('Failed to load tracked files for command palette:', e);
  }
}

watch(() => currentProject.value?.path, (path) => {
  if (path) {
    loadTrackedFiles();
  } else {
    trackedFiles.value = [];
  }
}, { immediate: true });

function handleGlobalKeydown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && (e.key === 'k' || e.key === 'K' || e.key === 'p' || e.key === 'P')) {
    e.preventDefault();
    showCommandPalette.value = !showCommandPalette.value;
  }
}

async function handlePaletteAction(action: string) {
  switch (action) {
    case 'fetch':
      await fetchRemoteAction();
      break;
    case 'pull':
      await pullAction();
      break;
    case 'push':
      await pushAction();
      break;
    case 'stash':
      await stashChangesAction();
      break;
    case 'pop':
      await popStashAction();
      break;
    case 'stage-all':
      await stageAll();
      break;
    case 'toggle-theme':
      const { toggleTheme } = useTheme();
      toggleTheme();
      break;
    case 'git-profile':
      await openProfileModal();
      break;
    case 'clone-repository':
      showCloneModal.value = true;
      break;
    case 'create-branch':
      activeView.value = 'source-control';
      if (leftSidebarWidth.value === 0) toggleLeftSidebar();
      break;
    case 'zoom-in': {
      const { zoomIn } = useZoom();
      zoomIn();
      break;
    }
    case 'zoom-out': {
      const { zoomOut } = useZoom();
      zoomOut();
      break;
    }
    case 'zoom-reset': {
      const { resetZoom } = useZoom();
      resetZoom();
      break;
    }
  }
}

async function handleCommitSelectedFromPalette(commit: any) {
  activeView.value = 'source-control';
  if (leftSidebarWidth.value === 0) toggleLeftSidebar();
  selectCommit(commit);
}

async function handleCommitSelectedFromBlame(hash: string) {
  activeView.value = 'source-control';
  if (leftSidebarWidth.value === 0) toggleLeftSidebar();
  const commit = commits.value.find((c: any) => c.hash === hash || c.id === hash);
  if (commit) {
    selectCommit(commit);
  } else {
    handleItemClick({ type: 'commit', id: hash });
  }
}

function handleFileSelectedFromPalette(filePath: string) {
  explorerSelectedFilePath.value = filePath;
  activeView.value = 'file-explorer';
  leftSidebarWidth.value = 0;
}

// --- Git Profile Modal ---
const showProfileModal = ref(false);
const profileData = ref({ name: '', email: '', scope: 'global' as 'global' | 'repository' });
const profileLoading = ref(false);
const profileSaving = ref(false);
const profileError = ref<string | null>(null);

watch(() => currentProject.value?.path, async (path) => {
  if (!path) return;
  try {
    profileData.value = await invoke('get_git_profile', { path }) as any;
  } catch (e) {
    console.warn('Failed to load Git profile for status footer:', e);
  }
}, { immediate: true });

async function openProfileModal() {
  showProfileModal.value = true;
  profileLoading.value = true;
  profileError.value = null;
  try {
    const path = currentProject.value?.path;
    const profile = await invoke('get_git_profile', { path });
    profileData.value = profile as any;
  } catch (e: any) {
    profileError.value = e;
  } finally {
    profileLoading.value = false;
  }
}

async function saveProfile(profile: any) {
  profileSaving.value = true;
  profileError.value = null;
  try {
    const path = profile.scope === 'repository' ? currentProject.value?.path : null;
    await invokeGit('set_git_profile', {
      path,
      name: profile.name,
      email: profile.email,
      scope: profile.scope
    }, `git config user.name and user.email (${profile.scope})`);
    profileData.value = profile;
    showProfileModal.value = false;
  } catch (e: any) {
    profileError.value = e;
  } finally {
    profileSaving.value = false;
  }
}
</script>

<template>
  <SplashScreen v-if="isAppLoading" />
  <div v-else class="app-wrapper" @contextmenu.prevent>
    <TabsBar 
      :projects="projects"
      :active-tab-index="activeTabIndex"
      :show-new-tab="showProjectPicker || projects.length === 0"
      @select-tab="selectProjectTab"
      @close-tab="closeTab"
      @new-tab="openNewProjectPage"
      @close-new-tab="closeNewProjectPage"
      @open-profile="openProfileModal"
      @open-palette="showCommandPalette = true"
    />

    <ProjectPickerPage
      v-if="projects.length === 0 || showProjectPicker"
      :history="projectHistory"
      :is-new-project="projects.length > 0"
      @open-repository="openRepository" 
      @open-clone="showCloneModal = true"
      @open-history="openFromHistory"
      @remove-history="removeFromHistory"
      @clear-history="clearHistory"
    />

    <div v-else class="app-container">
      <ActivityBar :active-view="activeView" @select-view="selectActivityView" />

      <div class="repository-workspace">
        <div class="main-content-wrapper">
        <Transition name="page-fade" mode="out-in">
          <KeepAlive>
            <SourceControlPage
              v-if="activeView === 'source-control'"
              :key="'source-control'"
              :project="currentProject"
        :left-sidebar-width="leftSidebarWidth"
        :right-sidebar-width="rightSidebarWidth"
        :display-branch="displayBranch"
        :is-loading="isLoading"
        :details-mode="detailsMode"
        :has-working-changes="hasWorkingChanges"
        :working-change-count="workingChangeCount"
        :commits="commits"
        :selected-commit="selectedCommit"
        :selected-commits="selectedCommits"
        :details-target="detailsTarget"
        :selected-file-path="selectedFilePath"
        :selected-file-diff="selectedFileDiff"
        @item-click="handleItemClick"
        @branch-dblclick="onBranchDoubleClick"
        @load-authors="loadAuthors(currentProject)"
        @open-contributors-report="activeView = 'contributors-report'"
        @open-pull-requests="activeView = 'pull-requests'"
        @open-project="openFromHistory"
        @resize-left="startDragLeft"
        @resize-right="startDragRight"
        @fetch="fetchRemoteAction"
        @pull="pullAction"
        @push="pushAction"
        @stash="stashChangesAction"
        @pop="popStashAction"
        @show-working-changes="showWorkingChanges"
        @select-commit="selectCommit"
        @close-diff="selectedFileDiff = null; selectedFilePath = null"
        @close-details="closeRightSidebar"
        @change-details-mode="detailsMode = $event"
        @select-file="selectFile"
        @stage-file="stageFile"
        @unstage-file="unstageFile"
        @stage-all="stageAll"
        @commit-changes="commitChanges"
        @branch-action="handleBranchAction"
        @commit-action="handleCommitAction"
        @cherry-pick-action="handleCherryPickAction"
        @refresh="async (fetchRemote, done) => { await refreshProject(activeTabIndex, fetchRemote); done?.(); }"
        @pending="globalPendingMessage = $event"
      />

            <TimelinePage
              v-else-if="activeView === 'horizontal-graph'"
              :key="'horizontal-graph'"
              :commits="commits"
        :selected-commit="selectedCommit"
        :loading="commitDetailsLoading"
        :selected-file-path="selectedFilePath"
        :selected-file-diff="selectedFileDiff"
        :commit-limit="commitLimit"
        @select-commit="selectCommit"
        @select-file="selectFile"
        @close-diff="selectedFileDiff = null; selectedFilePath = null"
        @change-limit="onChangeCommitLimit"
      />

            <FileExplorerPage
              v-else-if="activeView === 'file-explorer'"
              :key="'file-explorer'"
              :repository-name="currentProject?.name || 'Repository'"
              :repository-path="currentProject?.path || null"
              :active="activeView === 'file-explorer'"
              :initial-file-path="explorerSelectedFilePath"
              @select-commit="handleCommitSelectedFromBlame"
              @pending="globalPendingMessage = $event"
            />

            <ContributorsReport
              v-else-if="activeView === 'contributors-report'"
              :key="'contributors-report'"
              :repository-name="currentProject?.name || 'Repository'"
              :repository-path="currentProject?.path || null"
              :active="activeView === 'contributors-report'"
            />
            <ActivityLogPage
              v-else-if="activeView === 'activity-log'"
              :key="'activity-log'"
              :repository-path="currentProject?.path || null"
            />
            <PullRequestsPage
              v-else-if="activeView === 'pull-requests'"
              :key="'pull-requests'"
              :project="currentProject"
              :active="activeView === 'pull-requests'"
              @checkout-pr="handlePullRequestCheckedOut"
              @navigate="activeView = ($event as AppView)"
            />
            <ReflogView
              v-else-if="activeView === 'reflog'"
              :key="'reflog'"
              :project="currentProject"
              @refresh="refreshProject(activeTabIndex, false)"
            />
          </KeepAlive>
        </Transition>
        </div>
        <RepositoryStatusFooter
          :project="currentProject"
          :branch="displayBranch"
          :profile="profileData"
          :working-change-count="workingChangeCount"
          :is-fetching="isLoading === 'fetch'"
          @open-branch="openFooterBranches"
          @open-changes="openFooterChanges"
          @fetch="fetchRemoteAction"
          @open-profile="openProfileModal"
        />
      </div>
    </div>

    <PendingOverlay :message="globalPendingMessage" />
    <ToastNotifications />

    <GitProfileModal
      :visible="showProfileModal"
      :profile="profileData"
      :loading="profileLoading"
      :saving="profileSaving"
      :error="profileError"
      :has-repository="!!currentProject"
      @close="showProfileModal = false"
      @save="saveProfile"
    />

    <CloneRepositoryModal
      :visible="showCloneModal"
      @close="showCloneModal = false"
      @cloned="handleRepositoryCloned"
    />
    
    <ResolveDivergentBranchModal
      ref="divergentBranchModal"
      :repository-path="currentProject?.path || null"
      @refresh="async (fetchRemote, done) => { await refreshProject(activeTabIndex, fetchRemote); done?.(); }"
      @pending="globalPendingMessage = $event"
    />

    <CommandPalette
      :visible="showCommandPalette"
      :project="currentProject"
      :tracked-files="trackedFiles"
      @close="showCommandPalette = false"
      @navigate="selectActivityView"
      @checkout-branch="onBranchDoubleClick"
      @select-commit="handleCommitSelectedFromPalette"
      @select-file="handleFileSelectedFromPalette"
      @trigger-action="handlePaletteAction"
    />
  </div>
</template>

<style src="./App.css"></style>
