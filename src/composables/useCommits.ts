import { ref, computed, nextTick, type Ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Project } from '../types';
import { computeGraph } from '../utils/gitHelpers';
import { notify } from './useToasts';

export function useCommits(
  currentProject: Ref<Project | undefined>,
  activeView: Ref<string>,
  rightSidebarWidth: Ref<number>,
  toggleRightSidebar: () => void
) {
  const detailsMode = ref<'commit' | 'changes' | null>(null);
  const commitDetailsLoading = ref(false);
  
  const selectedFilePath = ref<string | null>(null);
  const selectedFileDiff = ref<string | null>(null);

  const commits = computed(() => currentProject.value?.commits || []);
  const selectedCommit = computed(() => currentProject.value?.selectedCommit || null);
  const selectedCommits = computed(() => currentProject.value?.selectedCommits || []);
  
  const workingChangeCount = computed(() => {
    const changes = currentProject.value?.uncommittedChanges;
    return (changes?.staged_files?.length || 0) + (changes?.unstaged_files?.length || 0);
  });
  
  const hasWorkingChanges = computed(() => workingChangeCount.value > 0);
  
  const detailsTarget = computed(() => {
    if (detailsMode.value === 'commit') return selectedCommit.value;
    if (detailsMode.value === 'changes') return currentProject.value?.uncommittedChanges || null;
    return null;
  });

  const displayBranch = computed(() => {
    if (!currentProject.value || !currentProject.value.localBranches) return '';
    const active = currentProject.value.localBranches.find(b => b.active);
    const activeName = active ? active.name : '';
  
    if (!currentProject.value.selectedCommit || currentProject.value.selectedCommit.hash === 'WIP') {
      return activeName;
    }
  
    const commitBranches = currentProject.value.selectedCommit.branches || [];
    if (commitBranches.includes(activeName)) {
      return activeName;
    }
    return commitBranches.length > 0 ? commitBranches[0] : '';
  });

  async function selectCommit(commit: any, event?: MouseEvent) {
    if (!currentProject.value) return;
    detailsMode.value = 'commit';
    if (activeView.value === 'source-control' && rightSidebarWidth.value === 0) {
      toggleRightSidebar();
    }
    
    if (event && (event.metaKey || event.ctrlKey)) {
      const isSelected = currentProject.value.selectedCommits.some((c: any) => c.id === commit.id);
      if (isSelected) {
        currentProject.value.selectedCommits = currentProject.value.selectedCommits.filter((c: any) => c.id !== commit.id);
        if (currentProject.value.selectedCommit?.id === commit.id) {
          currentProject.value.selectedCommit = currentProject.value.selectedCommits.length > 0 
            ? currentProject.value.selectedCommits[currentProject.value.selectedCommits.length - 1] 
            : null;
        }
      } else {
        currentProject.value.selectedCommits.push(commit);
        currentProject.value.selectedCommit = commit;
      }
    } else if (event && event.shiftKey && currentProject.value.selectedCommit) {
      const allCommits = currentProject.value.commits;
      const startIdx = allCommits.findIndex((c: any) => c.id === currentProject.value!.selectedCommit.id);
      const endIdx = allCommits.findIndex((c: any) => c.id === commit.id);
      
      if (startIdx !== -1 && endIdx !== -1) {
        const minIdx = Math.min(startIdx, endIdx);
        const maxIdx = Math.max(startIdx, endIdx);
        currentProject.value.selectedCommits = allCommits.slice(minIdx, maxIdx + 1);
        currentProject.value.selectedCommit = commit;
      }
    } else {
      currentProject.value.selectedCommits = [commit];
      currentProject.value.selectedCommit = {
        ...commit,
        files: [],
        branches: []
      };
    }
  
    if (!currentProject.value.selectedCommit) return;
    const targetCommit = currentProject.value.selectedCommit;
    commitDetailsLoading.value = true;
  
    try {
      const [details, branches] = await Promise.all([
        invoke('get_commit_details', {
          path: currentProject.value.path,
          hash: targetCommit.id
        }) as Promise<any>,
        invoke('get_commit_branches', {
          path: currentProject.value.path,
          hash: targetCommit.id
        }) as Promise<string[]>
      ]);
      
      if (currentProject.value.selectedCommit?.id === targetCommit.id) {
        currentProject.value.selectedCommit.files = details.files;
        currentProject.value.selectedCommit.branches = branches;
      }
    } catch (e) {
      console.error("Failed to get commit details/branches:", e);
    } finally {
      if (currentProject.value.selectedCommit?.id === targetCommit.id) {
        commitDetailsLoading.value = false;
      }
    }
  }

  async function handleItemClick({ type, id }: { type: string, id: string }) {
    if (!currentProject.value || !currentProject.value.commits) return;
    
    let searchStr = type === 'tag' ? `tag: ${id}` : id;
    
    const findCommit = () => {
      return currentProject.value!.commits.find((c: any) => {
        if (!c.refs) return false;
        return c.refs.some((r: string) => {
          if (r === searchStr) return true;
          if (r === `HEAD -> ${searchStr}`) return true;
          if (type === 'stash' && r === 'refs/stash') return true;
          return false;
        });
      });
    };
    
    let targetCommit = findCommit();
    
    let attempt = 0;
    while (!targetCommit && attempt < 10) {
      attempt++;
      const skip = currentProject.value.rawCommits.length;
      try {
        const moreRaw = await invoke('get_commits', { path: currentProject.value.path, limit: 500, skip }) as any[];
        if (moreRaw.length === 0) break;
        currentProject.value.rawCommits.push(...moreRaw);
        currentProject.value.commits = computeGraph(currentProject.value.rawCommits);
        
        targetCommit = findCommit();
      } catch (e) {
        console.error("Failed to load more commits", e);
        break;
      }
    }
    
    if (targetCommit) {
      await nextTick();
      selectCommit(targetCommit);
      setTimeout(() => {
        const el = document.getElementById(`commit-row-${targetCommit!.hash}`);
        if (el) {
          const container = document.querySelector('.graph-container');
          if (container) {
            const rowTop = el.offsetTop;
            container.scrollTo({ top: Math.max(0, rowTop - container.clientHeight / 2), behavior: 'smooth' });
          } else {
            el.scrollIntoView({ behavior: 'smooth', block: 'center' });
          }
          el.classList.add('flash-highlight');
          setTimeout(() => el.classList.remove('flash-highlight'), 1500);
        }
      }, 50);
    } else {
      notify(`${type === 'branch' ? 'Branch' : type === 'tag' ? 'Tag' : 'Stash'} '${id}' is too old and not found even after loading extra history.`, 'info');
    }
  }

  function closeRightSidebar() {
    detailsMode.value = null;
    selectedFileDiff.value = null;
    selectedFilePath.value = null;
  }

  function showWorkingChanges() {
    if (!currentProject.value?.uncommittedChanges) return;
    detailsMode.value = 'changes';
    selectedFileDiff.value = null;
    selectedFilePath.value = null;
    if (rightSidebarWidth.value === 0) toggleRightSidebar();
  }

  async function selectFile(filePath: string) {
    if (!currentProject.value || !detailsTarget.value) return;
    selectedFilePath.value = filePath;
    selectedFileDiff.value = "Loading diff...";
    try {
      const diff = await invoke('get_file_diff', {
        path: currentProject.value.path,
        hash: detailsMode.value === 'changes' ? 'uncommitted' : currentProject.value.selectedCommit?.id,
        filePath: filePath
      }) as string;
      selectedFileDiff.value = diff.trim();
    } catch (e) {
      selectedFileDiff.value = `Error loading diff:\n${e}`;
    }
  }

  return {
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
  };
}
