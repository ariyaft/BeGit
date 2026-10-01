<script setup lang="ts">
import { nextTick, ref, computed } from 'vue';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { notify } from '../../composables/useToasts';
import { invokeGit } from '../../composables/useActivityLog';

const props = defineProps<{
  repositoryPath: string | null;
  branches?: Array<{ name: string; active?: boolean }>;
}>();

const emit = defineEmits<{
  refresh: [done?: () => void];
  pending: [message: string];
  openWorktree: [path: string];
}>();

const visible = ref(false);
const worktreePath = ref('');
const branchMode = ref<'new' | 'existing' | 'detached'>('new');
const newBranchName = ref('');
const selectedExistingBranch = ref('');
const baseBranch = ref('');
const detachedCommit = ref('');
const openInNewTab = ref(true);
const pathInput = ref<HTMLInputElement | null>(null);

const availableBranches = computed(() => {
  return (props.branches || []).map(b => b.name);
});

async function open(defaultPath?: string) {
  worktreePath.value = defaultPath || '';
  branchMode.value = 'new';
  newBranchName.value = '';
  detachedCommit.value = '';
  openInNewTab.value = true;
  
  if (availableBranches.value.length > 0) {
    selectedExistingBranch.value = availableBranches.value[0];
    baseBranch.value = availableBranches.value[0];
  } else {
    selectedExistingBranch.value = 'main';
    baseBranch.value = 'main';
  }

  visible.value = true;
  await nextTick();
  pathInput.value?.focus();
}

function close() {
  visible.value = false;
}

async function browseDirectory() {
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: 'Select Directory for New Worktree'
    });
    if (selected && typeof selected === 'string') {
      worktreePath.value = selected;
    }
  } catch (e) {
    console.error('Directory browse failed:', e);
  }
}

function normalizeBranchName(e: Event) {
  newBranchName.value = (e.target as HTMLInputElement).value.toLowerCase().replace(/ /g, '-');
}

async function confirm() {
  const wtPath = worktreePath.value.trim();
  if (!wtPath || !props.repositoryPath) return;

  const mode = branchMode.value;
  let newB: string | null = null;
  let existB: string | null = null;
  let commitOrBase: string | null = null;
  let detach = false;

  if (mode === 'new') {
    if (!newBranchName.value.trim()) return;
    newB = newBranchName.value.trim();
    commitOrBase = baseBranch.value || null;
  } else if (mode === 'existing') {
    if (!selectedExistingBranch.value) return;
    existB = selectedExistingBranch.value;
  } else if (mode === 'detached') {
    if (!detachedCommit.value.trim()) return;
    commitOrBase = detachedCommit.value.trim();
    detach = true;
  }

  visible.value = false;
  emit('pending', `Adding worktree at ${wtPath}...`);
  let awaitingRefresh = false;

  try {
    const cmdDesc = newB
      ? `git worktree add -b ${newB} ${wtPath}${commitOrBase ? ' ' + commitOrBase : ''}`
      : existB
      ? `git worktree add ${wtPath} ${existB}`
      : `git worktree add --detach ${wtPath} ${commitOrBase}`;

    await invokeGit(
      'add_worktree',
      {
        path: props.repositoryPath,
        worktreePath: wtPath,
        branch: existB,
        newBranch: newB,
        commitOrBranch: commitOrBase,
        detach
      },
      cmdDesc
    );

    awaitingRefresh = true;
    emit('refresh', () => emit('pending', ''));
    notify(`Worktree created successfully at ${wtPath}`, 'success');

    if (openInNewTab.value) {
      emit('openWorktree', wtPath);
    }
  } catch (error: any) {
    notify(`Failed to add worktree: ${error}`, 'error');
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.65); z-index: 9998; backdrop-filter: blur(2px);">
    <div class="modal-content" @click.stop style="background: var(--bg-card, #1e1e1e); color: var(--text-main, #fff); padding: 22px; border-radius: 8px; box-shadow: 0 8px 30px rgba(0,0,0,.5); width: 460px; max-width: 90vw; position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); border: 1px solid var(--border, #333); z-index: 9999; font-family: inherit;">
      <h3 style="margin-top: 0; margin-bottom: 16px; font-size: 16px; font-weight: 600; display: flex; align-items: center; gap: 8px;">
        <span>Create Git Worktree</span>
      </h3>

      <div class="form-group" style="margin-bottom: 14px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Worktree Directory Location <span style="color: var(--accent-red, #ff5252);">*</span></label>
        <div style="display: flex; gap: 8px;">
          <input
            ref="pathInput"
            v-model="worktreePath"
            type="text"
            placeholder="/path/to/worktree or ../feature-dir"
            style="flex: 1; padding: 8px 10px; border: 1px solid var(--border, #444); border-radius: 5px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 13px;"
          />
          <button
            type="button"
            @click="browseDirectory"
            style="padding: 8px 12px; border: 1px solid var(--border, #444); border-radius: 5px; background: var(--surface-subtle, #333); color: var(--text-main, #fff); font-size: 12px; cursor: pointer; white-space: nowrap;"
          >
            Browse...
          </button>
        </div>
      </div>

      <div class="form-group" style="margin-bottom: 14px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 8px; color: var(--text-muted, #aaa);">Branch Configuration</label>
        <div style="display: flex; flex-direction: column; gap: 8px; font-size: 13px;">
          <label style="display: flex; align-items: center; gap: 8px; cursor: pointer;">
            <input type="radio" value="new" v-model="branchMode" />
            <span>Create new branch</span>
          </label>
          <label style="display: flex; align-items: center; gap: 8px; cursor: pointer;">
            <input type="radio" value="existing" v-model="branchMode" />
            <span>Checkout existing branch</span>
          </label>
          <label style="display: flex; align-items: center; gap: 8px; cursor: pointer;">
            <input type="radio" value="detached" v-model="branchMode" />
            <span>Detached HEAD / Specific commit</span>
          </label>
        </div>
      </div>

      <!-- New Branch Fields -->
      <div v-if="branchMode === 'new'" style="background: var(--surface-subtle, rgba(255,255,255,0.03)); padding: 12px; border-radius: 6px; border: 1px solid var(--border, #333); margin-bottom: 14px;">
        <div style="margin-bottom: 10px;">
          <label style="display: block; font-size: 11px; font-weight: 600; margin-bottom: 4px; color: var(--text-muted, #aaa);">New Branch Name <span style="color: var(--accent-red, #ff5252);">*</span></label>
          <input
            :value="newBranchName"
            @input="normalizeBranchName"
            type="text"
            placeholder="e.g. feature/my-work"
            style="box-sizing: border-box; width: 100%; padding: 6px 8px; border: 1px solid var(--border, #444); border-radius: 4px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 12px;"
          />
        </div>
        <div>
          <label style="display: block; font-size: 11px; font-weight: 600; margin-bottom: 4px; color: var(--text-muted, #aaa);">Base Branch / Commit</label>
          <select
            v-model="baseBranch"
            style="box-sizing: border-box; width: 100%; padding: 6px 8px; border: 1px solid var(--border, #444); border-radius: 4px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 12px;"
          >
            <option v-for="b in availableBranches" :key="b" :value="b">{{ b }}</option>
          </select>
        </div>
      </div>

      <!-- Existing Branch Fields -->
      <div v-else-if="branchMode === 'existing'" style="background: var(--surface-subtle, rgba(255,255,255,0.03)); padding: 12px; border-radius: 6px; border: 1px solid var(--border, #333); margin-bottom: 14px;">
        <label style="display: block; font-size: 11px; font-weight: 600; margin-bottom: 4px; color: var(--text-muted, #aaa);">Select Existing Branch</label>
        <select
          v-model="selectedExistingBranch"
          style="box-sizing: border-box; width: 100%; padding: 6px 8px; border: 1px solid var(--border, #444); border-radius: 4px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 12px;"
        >
          <option v-for="b in availableBranches" :key="b" :value="b">{{ b }}</option>
        </select>
      </div>

      <!-- Detached Commit Fields -->
      <div v-else-if="branchMode === 'detached'" style="background: var(--surface-subtle, rgba(255,255,255,0.03)); padding: 12px; border-radius: 6px; border: 1px solid var(--border, #333); margin-bottom: 14px;">
        <label style="display: block; font-size: 11px; font-weight: 600; margin-bottom: 4px; color: var(--text-muted, #aaa);">Commit SHA / Tag / Ref</label>
        <input
          v-model="detachedCommit"
          type="text"
          placeholder="e.g. HEAD, 4a1b2c3, v1.0.0"
          style="box-sizing: border-box; width: 100%; padding: 6px 8px; border: 1px solid var(--border, #444); border-radius: 4px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 12px;"
        />
      </div>

      <div class="form-group" style="margin-bottom: 18px;">
        <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px;">
          <input type="checkbox" v-model="openInNewTab" />
          <span>Open new worktree as a tab in BeGit immediately</span>
        </label>
      </div>

      <div style="display: flex; justify-content: flex-end; gap: 10px;">
        <button
          style="padding: 7px 14px; border: 1px solid var(--border, #444); border-radius: 5px; background: var(--surface-subtle, #333); color: var(--text-main, #fff); font-size: 12px; font-weight: 500; cursor: pointer;"
          @click="close"
        >
          Cancel
        </button>
        <button
          style="padding: 7px 16px; border: 0; border-radius: 5px; background: var(--accent, #00bcd4); color: #fff; font-size: 12px; font-weight: 600; cursor: pointer;"
          :disabled="!worktreePath.trim() || (branchMode === 'new' && !newBranchName.trim()) || (branchMode === 'detached' && !detachedCommit.trim())"
          @click="confirm"
        >
          Add Worktree
        </button>
      </div>
    </div>
  </div>
</template>
