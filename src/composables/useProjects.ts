import { ref, computed, markRaw } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { computeGraph } from '../utils/gitHelpers';
import type { Project } from '../types';
import { invokeGit } from './useActivityLog';

export function useProjects() {
  const projects = ref<Project[]>([]);
  const activeTabIndex = ref(0);
  const projectHistory = ref<Array<{path: string, name: string}>>([]);
  const showProjectPicker = ref(false);
  const commitLimit = ref(100);
  const refreshesInFlight = new Map<string, Promise<void>>();

  const currentProject = computed(() => projects.value[activeTabIndex.value]);

  // Load history from localStorage
  function loadHistory() {
    const savedHistory = localStorage.getItem('gitTreeHistory');
    if (savedHistory) {
      try {
        projectHistory.value = JSON.parse(savedHistory);
      } catch(e) {}
    }
  }

  function addToHistory(path: string, name: string) {
    projectHistory.value = projectHistory.value.filter(h => h.path !== path);
    projectHistory.value.unshift({ path, name });
    if (projectHistory.value.length > 10) projectHistory.value.pop();
    localStorage.setItem('gitTreeHistory', JSON.stringify(projectHistory.value));
  }

  function removeFromHistory(path: string) {
    projectHistory.value = projectHistory.value.filter(h => h.path !== path);
    localStorage.setItem('gitTreeHistory', JSON.stringify(projectHistory.value));
  }

  function clearHistory() {
    projectHistory.value = [];
    localStorage.removeItem('gitTreeHistory');
  }

  async function loadProjectByPath(pathStr: string) {
    try {
      const name = pathStr.split('/').pop() || pathStr;
      addToHistory(pathStr, name);
      const [local, remote, tags, stashes, worktrees, submodules, rawCommits, cherryPick, opState, rebStatus] = await Promise.all([
        invoke('get_branches', { path: pathStr }) as Promise<any[]>,
        invoke('get_remote_branches', { path: pathStr }) as Promise<any[]>,
        invoke('get_tags', { path: pathStr }) as Promise<string[]>,
        invoke('get_stashes', { path: pathStr }) as Promise<any[]>,
        invoke('get_worktrees', { path: pathStr }) as Promise<any[]>,
        invoke('get_submodules', { path: pathStr }) as Promise<any[]>,
        invoke('get_commits', { path: pathStr, limit: commitLimit.value, skip: 0 }) as Promise<any[]>,
        invoke('get_cherry_pick_status', { path: pathStr }) as Promise<Project['cherryPick']>,
        invoke('get_repository_operation_state', { path: pathStr }) as Promise<any>,
        invoke('get_rebase_status', { path: pathStr }) as Promise<any>
      ]);
      const computedCommits = computeGraph(rawCommits);
      
      projects.value.push({
        path: pathStr,
        name,
        localBranches: local,
        remoteBranches: remote,
        remotes: [], // will be populated on first refresh
        tags,
        stashes,
        worktrees,
        submodules,
        authors: [],
        commits: markRaw(computedCommits),
        rawCommits: markRaw(rawCommits),
        selectedCommit: null,
        selectedCommits: [],
        uncommittedChanges: null,
        cherryPick,
        operationState: opState,
        rebaseStatus: rebStatus,
        lastFetchTime: 0
      });
      
      return projects.value.length - 1;
    } catch (e) {
      console.error("Failed to load project:", pathStr, e);
      throw e;
    }
  }

  async function refreshProject(index: number, doFetch = false) {
    const proj = projects.value[index];
    if (!proj) return;
    const existingRefresh = refreshesInFlight.get(proj.path);
    if (existingRefresh) return existingRefresh;

    const refresh = (async () => {
      const reloadData = async () => {
      try {
        const [local, remote, tags, stashes, remotes, worktrees, submodules, rawCommits, uncommitted, cherryPick, opState, rebStatus] = await Promise.all([
          invoke('get_branches', { path: proj.path }) as Promise<any[]>,
          invoke('get_remote_branches', { path: proj.path }) as Promise<any[]>,
          invoke('get_tags', { path: proj.path }) as Promise<string[]>,
          invoke('get_stashes', { path: proj.path }) as Promise<any[]>,
          invoke('get_remotes', { path: proj.path }) as Promise<any[]>,
          invoke('get_worktrees', { path: proj.path }) as Promise<any[]>,
          invoke('get_submodules', { path: proj.path }) as Promise<any[]>,
          invoke('get_commits', { path: proj.path, limit: commitLimit.value, skip: 0 }) as Promise<any[]>,
          invoke('get_uncommitted_changes', { path: proj.path }) as Promise<any>,
          invoke('get_cherry_pick_status', { path: proj.path }) as Promise<Project['cherryPick']>,
          invoke('get_repository_operation_state', { path: proj.path }) as Promise<any>,
          invoke('get_rebase_status', { path: proj.path }) as Promise<any>
        ]);
        
        if (projects.value[index] && projects.value[index].path === proj.path) {
          projects.value[index].localBranches = local;
          projects.value[index].remoteBranches = remote;
          projects.value[index].remotes = remotes;
          projects.value[index].tags = tags;
          projects.value[index].stashes = stashes;
          projects.value[index].worktrees = worktrees;
          projects.value[index].submodules = submodules;
          if (!projects.value[index].authors) {
            projects.value[index].authors = [];
          }
          projects.value[index].rawCommits = markRaw(rawCommits);
          projects.value[index].commits = markRaw(computeGraph(rawCommits));
          projects.value[index].uncommittedChanges = {
            id: 'uncommitted',
            hash: 'WIP',
            author: 'You',
            time: 'Now',
            message: 'Uncommitted Changes',
            staged_files: uncommitted.staged_files,
            unstaged_files: uncommitted.unstaged_files
          };
          projects.value[index].cherryPick = cherryPick;
          projects.value[index].operationState = opState;
          projects.value[index].rebaseStatus = rebStatus;
          
          if (!projects.value[index].selectedCommits) {
            projects.value[index].selectedCommits = [];
          }
        }
      } catch (e) {
        console.error("Failed to reload data for:", proj.path, e);
      }
      };

      await reloadData();

      if (doFetch) {
        const now = Date.now();
        if (!proj.lastFetchTime || (now - proj.lastFetchTime > 60000)) {
          try {
            await invokeGit('fetch_remote', { path: proj.path }, 'git fetch (automatic refresh)');
            projects.value[index].lastFetchTime = Date.now();
            await reloadData();
          } catch (e) {
            console.warn("Fetch failed:", e);
          }
        }
      }
    })();

    refreshesInFlight.set(proj.path, refresh);
    try {
      await refresh;
    } finally {
      if (refreshesInFlight.get(proj.path) === refresh) refreshesInFlight.delete(proj.path);
    }
  }

  function closeTab(index: number) {
    const wasActive = activeTabIndex.value === index;
    projects.value.splice(index, 1);
  
    if (projects.value.length === 0) {
      activeTabIndex.value = 0;
      showProjectPicker.value = true;
    } else if (index < activeTabIndex.value) {
      activeTabIndex.value -= 1;
    } else if (wasActive || activeTabIndex.value >= projects.value.length) {
      activeTabIndex.value = Math.max(0, projects.value.length - 1);
    }
  }

  return {
    projects,
    activeTabIndex,
    projectHistory,
    showProjectPicker,
    currentProject,
    loadHistory,
    addToHistory,
    removeFromHistory,
    clearHistory,
    loadProjectByPath,
    refreshProject,
    closeTab,
    commitLimit
  };
}
