import { ref, type Ref } from 'vue';
import type { Project } from '../types';
import { executeBranchAction } from '../utils/gitHelpers';
import { notify } from './useToasts';
import { invokeGit } from './useActivityLog';

export function useGitActions(
  currentProject: Ref<Project | undefined>,
  activeTabIndex: Ref<number>,
  refreshProject: (index: number, doFetch?: boolean) => Promise<void>,
  withPending: <T>(msg: string, fn: () => Promise<T>) => Promise<T>,
  onDivergentBranch?: (branchName: string) => void
) {
  const isLoading = ref<'' | 'fetch' | 'pull' | 'push' | 'stash' | 'pop'>('');

  async function fetchRemoteAction() {
    if (!currentProject.value || isLoading.value) return;
    isLoading.value = 'fetch';
    await withPending('Fetching from remote...', async () => {
      try {
        await invokeGit('fetch_remote', { path: currentProject.value!.path }, 'git fetch');
        currentProject.value!.lastFetchTime = Date.now();
        await refreshProject(activeTabIndex.value, false);
      } catch (e: any) {
        notify(`Fetch failed: ${e}`);
      } finally {
        isLoading.value = '';
      }
    });
  }

  async function pullAction() {
    if (!currentProject.value || isLoading.value) return;
    const activeBranch = currentProject.value.localBranches.find((b: any) => b.active);
    if (!activeBranch) return;
    isLoading.value = 'pull';
    await withPending('Pulling from remote...', async () => {
      try {
        await invokeGit('pull_branch', { path: currentProject.value!.path, branchName: activeBranch.name }, `git pull ${activeBranch.name}`);
        await refreshProject(activeTabIndex.value, true);
      } catch (e: any) {
        const errorMsg = String(e);
        if (errorMsg.includes('non-fast-forward') || errorMsg.includes('divergent branches') || errorMsg.includes('Not possible to fast-forward')) {
          if (onDivergentBranch) {
            onDivergentBranch(activeBranch.name);
          } else {
            notify(`Pull failed: branches have diverged (non-fast-forward).`);
          }
        } else {
          notify(`Pull failed: ${e}`);
        }
      } finally {
        isLoading.value = '';
      }
    });
  }

  async function pushAction() {
    if (!currentProject.value || isLoading.value) return;
    const activeBranch = currentProject.value.localBranches.find((b: any) => b.active);
    if (!activeBranch) return;
    isLoading.value = 'push';
    await withPending('Pushing to remote...', async () => {
      try {
        await invokeGit('push_branch', { path: currentProject.value!.path, branchName: activeBranch.name }, `git push ${activeBranch.name}`);
        await refreshProject(activeTabIndex.value, true);
      } catch (e: any) {
        notify(`Push failed: ${e}`);
      } finally {
        isLoading.value = '';
      }
    });
  }

  async function stashChangesAction() {
    if (!currentProject.value || isLoading.value) return;
    const msg = window.prompt("Enter stash message (optional):", "");
    if (msg === null) return;
    isLoading.value = 'stash';
    await withPending('Stashing changes...', async () => {
      try {
        await invokeGit('stash_changes', { path: currentProject.value!.path, message: msg }, 'git stash push');
        await refreshProject(activeTabIndex.value, true);
      } catch (e: any) {
        notify(`Stash failed: ${e}`);
      } finally {
        isLoading.value = '';
      }
    });
  }

  async function popStashAction() {
    if (!currentProject.value || isLoading.value) return;
    isLoading.value = 'pop';
    await withPending('Popping stash...', async () => {
      try {
        await invokeGit('pop_stash', { path: currentProject.value!.path }, 'git stash pop');
        await refreshProject(activeTabIndex.value, true);
      } catch (e: any) {
        notify(`Pop stash failed: ${e}`);
      } finally {
        isLoading.value = '';
      }
    });
  }

  async function stageFile(filePath: string) {
    if (!currentProject.value) return;
    try {
      await invokeGit('stage_file', { path: currentProject.value.path, filePath }, `git add ${filePath}`);
      await refreshProject(activeTabIndex.value, false);
    } catch (e) {
      console.error("Failed to stage file:", e);
    }
  }

  async function stageAll() {
    if (!currentProject.value) return;
    try {
      await invokeGit('stage_file', { path: currentProject.value.path, filePath: "." }, 'git add .');
      await refreshProject(activeTabIndex.value, false);
    } catch (e) {
      console.error("Failed to stage all files:", e);
    }
  }

  async function unstageFile(filePath: string) {
    if (!currentProject.value) return;
    try {
      await invokeGit('unstage_file', { path: currentProject.value.path, filePath }, `git restore --staged ${filePath}`);
      await refreshProject(activeTabIndex.value, false);
    } catch (e) {
      console.error("Failed to unstage file:", e);
    }
  }

  async function commitChanges(message: string, amend = false) {
    if (!currentProject.value || !message) return;
    const actionLabel = amend ? 'Amending commit...' : 'Committing changes...';
    await withPending(actionLabel, async () => {
      try {
        const cmdStr = amend
          ? `git commit --amend -m ${JSON.stringify(message)}`
          : `git commit -m ${JSON.stringify(message)}`;
        await invokeGit('commit_changes', { path: currentProject.value!.path, message, amend }, cmdStr);
        await refreshProject(activeTabIndex.value, true);
        notify(amend ? 'Successfully amended commit.' : 'Successfully committed changes.', 'success');
      } catch (e: any) {
        console.error("Failed to commit changes:", e);
        notify(`Commit failed: ${e}`, 'error');
      }
    });
  }

  async function onBranchDoubleClick(branch: any) {
    if (!currentProject.value || !branch) return;
    const proj = currentProject.value;
    await withPending(`Checking out ${branch.name}...`, async () => {
      try {
        await invokeGit('checkout_branch', {
          path: proj.path,
          branchName: branch.name
        }, `git checkout ${branch.name}`);
        await refreshProject(activeTabIndex.value, false);
        notify(`Checked out ${branch.name}.`, 'success');
      } catch (e) {
        console.error("Failed to checkout branch:", e);
        notify(e);
      }
    });
  }

  async function handleBranchAction(action: string, branch: any) {
    if (!currentProject.value || !branch) return;
    const proj = currentProject.value;
    const labels: Record<string, string> = {
      checkout: 'Checking out', merge: 'Merging', push: 'Pushing', pull: 'Pulling',
      set_upstream: 'Setting upstream for', rebase: 'Rebasing onto', ff_merge: 'Fast-forward merging'
    };
    const label = labels[action] || 'Updating';

    await withPending(`${label} ${branch.name}...`, async () => {
      const { success, fetchRemote, error } = await executeBranchAction(
        action,
        proj.path,
        branch.name
      );
      if (success) {
        await refreshProject(activeTabIndex.value, fetchRemote);
        notify(`${label} ${branch.name} complete.`, 'success');
      } else if (error) {
        const errorMsg = String(error);
        if (action === 'pull' && (errorMsg.includes('non-fast-forward') || errorMsg.includes('divergent branches') || errorMsg.includes('Not possible to fast-forward'))) {
          if (onDivergentBranch) {
            onDivergentBranch(branch.name);
          } else {
            notify(`Pull failed: branches have diverged (non-fast-forward).`);
          }
        } else if (action === 'pull') {
          notify(`Action failed: ${error}`);
        }
      }
    });
  }

  async function handleCommitAction(action: string, commits: any[]) {
    if (!commits || commits.length === 0 || !currentProject.value) return;
  
    if (action === 'copy_sha') {
      navigator.clipboard.writeText(commits.map(c => c.hash).join('\n'));
    } else if (action === 'cherry_pick') {
      const proj = currentProject.value;
      if (proj.cherryPick?.active) {
        notify('A cherry-pick is already in progress. Use the recovery controls in the toolbar to continue, skip, or abort it.', 'info');
        return;
      }
      const sortedCommits = [...commits].sort((a, b) => {
          const indexA = proj.rawCommits.findIndex((c:any) => c.hash === a.hash);
          const indexB = proj.rawCommits.findIndex((c:any) => c.hash === b.hash);
          return indexB - indexA;
      });
      
      const hashes = sortedCommits.map(c => c.hash);
      await withPending('Cherry-picking commits...', async () => {
        try {
          await invokeGit('cherry_pick', { path: proj.path, hashes }, `git cherry-pick ${hashes.join(' ')}`);
          await refreshProject(activeTabIndex.value, true);
        } catch (e: any) {
          notify("Cherry-pick failed:\n\n" + e);
        }
      });
    } else if (action === 'reset') {
      const target = commits[0];
      if (!target?.hash || target.hash === 'WIP' || target.hash === 'uncommitted') {
        notify('Choose a committed revision to reset the active branch.', 'info');
        return;
      }

      const activeBranch = currentProject.value.localBranches.find((branch: any) => branch.active);
      if (!activeBranch) {
        notify('Check out a local branch before resetting it.', 'info');
        return;
      }

      const confirmed = window.confirm(
        `Hard reset '${activeBranch.name}' to ${target.hash.substring(0, 7)}?\n\n` +
        `This moves the branch pointer and permanently discards all uncommitted changes. ` +
        `Commits removed from the branch can be recovered from Git reflog.\n\n` +
        `If this branch was pushed, you may need to force-push afterward.`
      );
      if (!confirmed) return;

      await withPending(`Resetting ${activeBranch.name}...`, async () => {
        try {
          await invokeGit('reset_branch', { path: currentProject.value!.path, hash: target.hash, mode: 'hard' }, `git reset --hard ${target.hash}`);
          await refreshProject(activeTabIndex.value, false);
        } catch (e: any) {
          notify(`Reset failed:\n\n${e}`);
        }
      });
    } else if (action === 'create_patch') {
      console.log("Create patch", commits);
    }
  }

  async function handleCherryPickAction(action: 'continue' | 'skip' | 'abort') {
    const proj = currentProject.value;
    if (!proj?.cherryPick?.active) return;

    if (action === 'abort' && !window.confirm('Abort this cherry-pick? Any conflict-resolution changes made for it will be discarded.')) return;
    if (action === 'skip' && !window.confirm('Skip the current commit? Any conflict-resolution changes made for it will be discarded.')) return;

    const labels = { continue: 'Continuing', skip: 'Skipping', abort: 'Aborting' };
    await withPending(`${labels[action]} cherry-pick...`, async () => {
      try {
        await invokeGit(`${action}_cherry_pick`, { path: proj.path }, `git cherry-pick --${action}`);
      } catch (e: any) {
        notify(`Could not ${action} the cherry-pick:\n\n${e}`);
      } finally {
        await refreshProject(activeTabIndex.value, true);
      }
    });
  }

  return {
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
  };
}
