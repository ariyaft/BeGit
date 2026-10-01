import { notify } from '../composables/useToasts';
import { invokeGit } from '../composables/useActivityLog';

// Gitkraken-like branch colors
const branchColors = ['#00bcd4', '#e91e63', '#4caf50', '#9c27b0', '#ffeb3b', '#ff9800', '#03a9f4'];

export function computeGraph(commitsData: any[]) {
  const columns: Array<{ hash: string, color: string, isMain: boolean } | null> = []; 
  const processed: Array<any> = [];
  
  commitsData.forEach((commit) => {
    let waitingCols: number[] = [];
    for (let i = 0; i < columns.length; i++) {
      if (columns[i] && columns[i]!.hash === commit.hash) {
        waitingCols.push(i);
      }
    }
    
    let col = -1;
    let nodeColor = '';
    let isMain = false;
    let hasMainLabel = commit.refs && commit.refs.some((r: string) => r.includes('main') || r.includes('master'));
    
    if (waitingCols.length > 0) {
      col = waitingCols[0];
      nodeColor = columns[col]!.color;
      isMain = columns[col]!.isMain || hasMainLabel;
      columns[col]!.isMain = isMain;
      
      for (let i = 1; i < waitingCols.length; i++) {
        columns[waitingCols[i]] = null;
      }
    } else {
      col = columns.findIndex(c => c === null);
      if (col === -1) col = columns.length;
      nodeColor = branchColors[col % branchColors.length];
      isMain = hasMainLabel;
      columns[col] = { hash: commit.hash, color: nodeColor, isMain: isMain };
    }
    
    let isCommitHead = false;
    
    const labelMap = new Map<string, { isLocal: boolean; isRemote: boolean; isTag: boolean; isHeadBranch: boolean }>();
    commit.refs.forEach((r: string) => {
      let isTag = r.includes('tag: ');
      let isRemote = r.startsWith('origin/');
      let isLocal = !isTag && !isRemote;
      let isHeadBranch = false;
      
      let text = r.replace('tag: ', '').trim();
      
      if (text.startsWith('origin/HEAD -> ')) {
        return; // ignore the remote HEAD pointer
      }
      
      if (isRemote && text.startsWith('origin/')) {
        text = text.substring(7); // strip 'origin/' to merge with local
      }
      
      if (text === 'HEAD' || text === 'origin/HEAD') {
        if (text === 'HEAD') {
          isCommitHead = true;
          if (!labelMap.has('HEAD')) {
            labelMap.set('HEAD', { isLocal: true, isRemote: false, isTag: false, isHeadBranch: true });
          }
        }
        return; // Don't process as normal text label yet
      }
      
      if (text.startsWith('HEAD -> ')) {
        isCommitHead = true;
        isHeadBranch = true;
        text = text.replace('HEAD -> ', '').trim();
      }
      
      if (!labelMap.has(text)) {
        labelMap.set(text, { isLocal: false, isRemote: false, isTag: false, isHeadBranch: false });
      }
      const entry = labelMap.get(text)!;
      if (isLocal) entry.isLocal = true;
      if (isRemote) entry.isRemote = true;
      if (isTag) entry.isTag = true;
      if (isHeadBranch) entry.isHeadBranch = true;
    });
    
    // If detached HEAD and no local branches exist, show HEAD icon label
    if (isCommitHead && !Array.from(labelMap.values()).some(l => l.isHeadBranch)) {
       if (labelMap.has('HEAD')) {
         const headEntry = labelMap.get('HEAD')!;
         headEntry.isHeadBranch = true;
       }
    } else {
       labelMap.delete('HEAD'); // Remove detached HEAD entry if attached
    }
    
    const labels = Array.from(labelMap.entries()).map(([text, types]) => {
      return { text, ...types, active: false };
    });
    
    processed.push({
      id: commit.hash,
      hash: commit.hash_short,
      message: commit.message,
      author: commit.author,
      time: commit.time,
      exact_time: commit.exact_time,
      parents: commit.parents,
      col: col,
      color: nodeColor,
      labels: labels,
      isHead: isCommitHead,
      isMain: isMain,
      refs: commit.refs,
      signature: commit.signature,
    });
    
    // Update columns for parents
    if (commit.parents.length === 0) {
      columns[col] = null;
    } else {
      let p0 = commit.parents[0];
      let existing = columns.findIndex(c => c && c!.hash === p0);
      
      if (existing !== -1 && existing < col) {
        // Parent already tracked in a lower column, free this column
        columns[col] = null;
        if (isMain) columns[existing]!.isMain = true; // Propagate main flag
      } else {
        columns[col] = { hash: p0, color: nodeColor, isMain: isMain };
        if (existing !== -1 && existing > col) {
          if (columns[existing]!.isMain) columns[col]!.isMain = true;
          columns[existing] = null;
        }
      }
      
      for (let i = 1; i < commit.parents.length; i++) {
        let pHash = commit.parents[i];
        if (!columns.some(c => c && c!.hash === pHash)) {
          let emptyCol = columns.findIndex(c => c === null);
          if (emptyCol === -1) emptyCol = columns.length;
          columns[emptyCol] = { hash: pHash, color: branchColors[emptyCol % branchColors.length], isMain: false };
        }
      }
    }
  });
  
  return processed;
}

export interface DiffLine {
  type: 'add' | 'sub' | 'context' | 'meta';
  oldNum: number | string;
  newNum: number | string;
  content: string;
}

export interface DiffHunkItem {
  id: string;
  header: string;
  oldStart: number;
  oldCount: number;
  newStart: number;
  newCount: number;
  lines: DiffLine[];
}

export function parseDiffHunks(diffText: string): { fileHeaders: string[]; hunks: DiffHunkItem[] } {
  if (!diffText) return { fileHeaders: [], hunks: [] };
  const lines = diffText.split('\n');
  const fileHeaders: string[] = [];
  const hunks: DiffHunkItem[] = [];

  let currentHunk: DiffHunkItem | null = null;
  let oldNum = 0;
  let newNum = 0;
  let hunkIndex = 0;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    if (line.startsWith('diff ') || line.startsWith('index ') || line.startsWith('--- ') || line.startsWith('+++ ') || line.startsWith('new file ') || line.startsWith('deleted file ')) {
      fileHeaders.push(line);
      continue;
    }

    if (line.startsWith('@@ ')) {
      if (currentHunk) {
        hunks.push(currentHunk);
      }
      const match = line.match(/@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@(.*)/);
      const oldStart = match ? parseInt(match[1]) : 1;
      const oldCount = match && match[2] !== undefined ? parseInt(match[2]) : 1;
      const newStart = match ? parseInt(match[3]) : 1;
      const newCount = match && match[4] !== undefined ? parseInt(match[4]) : 1;

      oldNum = oldStart;
      newNum = newStart;

      currentHunk = {
        id: `hunk-${hunkIndex++}`,
        header: line,
        oldStart,
        oldCount,
        newStart,
        newCount,
        lines: []
      };
      continue;
    }

    if (currentHunk) {
      if (line.startsWith('+')) {
        currentHunk.lines.push({ type: 'add', oldNum: '', newNum: newNum++, content: line });
      } else if (line.startsWith('-')) {
        currentHunk.lines.push({ type: 'sub', oldNum: oldNum++, newNum: '', content: line });
      } else if (line.startsWith('\\')) {
        currentHunk.lines.push({ type: 'meta', oldNum: '', newNum: '', content: line });
      } else {
        currentHunk.lines.push({ type: 'context', oldNum: oldNum++, newNum: newNum++, content: line });
      }
    }
  }

  if (currentHunk) {
    hunks.push(currentHunk);
  }

  return { fileHeaders, hunks };
}

export function buildHunkPatch(filePath: string, hunk: DiffHunkItem, selectedLineIndices?: Set<number>): string {
  let patch = `diff --git a/${filePath} b/${filePath}\n--- a/${filePath}\n+++ b/${filePath}\n`;

  if (!selectedLineIndices || selectedLineIndices.size === 0) {
    patch += `${hunk.header}\n`;
    for (const l of hunk.lines) {
      patch += `${l.content}\n`;
    }
    return patch;
  }

  const filteredLines: string[] = [];
  let oldCount = 0;
  let newCount = 0;

  for (let idx = 0; idx < hunk.lines.length; idx++) {
    const l = hunk.lines[idx];
    const isSelected = selectedLineIndices.has(idx);

    if (l.type === 'context') {
      filteredLines.push(l.content);
      oldCount++;
      newCount++;
    } else if (l.type === 'add') {
      if (isSelected) {
        filteredLines.push(l.content);
        newCount++;
      }
    } else if (l.type === 'sub') {
      if (isSelected) {
        filteredLines.push(l.content);
        oldCount++;
      } else {
        filteredLines.push(` ${l.content.startsWith('-') ? l.content.substring(1) : l.content}`);
        oldCount++;
        newCount++;
      }
    } else if (l.type === 'meta') {
      filteredLines.push(l.content);
    }
  }

  const newHeader = `@@ -${hunk.oldStart},${oldCount} +${hunk.newStart},${newCount} @@`;
  patch += `${newHeader}\n`;
  for (const fl of filteredLines) {
    patch += `${fl}\n`;
  }
  return patch;
}

export async function executeBranchAction(
  action: string, 
  path: string, 
  branchName: string,
  newName?: string,
  forceDelete?: boolean
): Promise<{ success: boolean; fetchRemote: boolean; error?: any }> {
  let success = false;
  let fetchRemote = false;

  try {
    if (action === 'checkout') {
      await invokeGit('checkout_branch', { path, branchName }, `git checkout ${branchName}`);
      success = true;
    } else if (action === 'delete_local') {
      await invokeGit('delete_branch', { path, branchName, force: !!forceDelete }, `git branch ${forceDelete ? '-D' : '-d'} ${branchName}`);
      success = true;
    } else if (action === 'delete_remote') {
      await invokeGit('delete_remote_branch', { path, branchName }, `git push origin --delete ${branchName}`);
      success = true;
      fetchRemote = true; // Fetch to update remote list
    } else if (action === 'push') {
      await invokeGit('push_branch', { path, branchName }, `git push ${branchName}`);
      success = true;
      fetchRemote = true;
    } else if (action === 'pull') {
      await invokeGit('pull_branch', { path, branchName }, `git pull ${branchName}`);
      success = true;
      fetchRemote = true;
    } else if (action === 'copy_name') {
      try {
        await navigator.clipboard.writeText(branchName);
      } catch(e) {
        console.error(e);
      }
    } else if (action === 'rename') {
      if (newName && newName.trim() !== '' && newName !== branchName) {
        await invokeGit('rename_branch', { path, old_name: branchName, new_name: newName.trim() }, `git branch -m ${branchName} ${newName.trim()}`);
        success = true;
      }
    } else if (action === 'create_branch') {
      if (newName && newName.trim() !== '') {
        await invokeGit('create_branch', { path, newBranch: newName.trim(), baseBranch: branchName }, `git checkout -b ${newName.trim()} ${branchName}`);
        success = true;
      }
    } else if (action === 'set_upstream') {
      await invokeGit('set_upstream', { path, branchName }, `git push --set-upstream origin ${branchName}`);
      success = true;
      fetchRemote = true;
    } else if (action === 'rebase') {
      await invokeGit('rebase_branch', { path, branchName }, `git rebase ${branchName}`);
      success = true;
    } else if (action === 'ff_merge') {
      await invokeGit('fast_forward_merge', { path, branchName }, `git merge --ff-only ${branchName}`);
      success = true;
    } else if (action === 'merge') {
      if (window.confirm(`Merge branch '${branchName}' into your current active branch?`)) {
        await invokeGit('merge_branch', { path, branchName }, `git merge ${branchName}`);
        success = true;
      }
    }
  } catch (e: any) {
    console.error(`Failed to ${action} branch:`, e);
    // If delete fails because unmerged, offer force delete
    if (action === 'delete_local' && e.toString().includes("not fully merged")) {
      if (window.confirm(`Branch '${branchName}' is not fully merged. Force delete locally anyway?`)) {
        try {
          await invokeGit('delete_branch', { path, branchName, force: true }, `git branch -D ${branchName}`);
          success = true;
        } catch (forceErr) {
          notify(forceErr);
        }
      }
    } else {
      if (action !== 'pull') {
        notify(e);
      }
      return { success: false, fetchRemote: false, error: e };
    }
  }

  return { success, fetchRemote };
}
