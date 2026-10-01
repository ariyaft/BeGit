<script setup lang="ts">
import { ref } from 'vue';
import type { WorktreeInfo } from '../../types';
import { notify } from '../../composables/useToasts';
import { invokeGit } from '../../composables/useActivityLog';

const props = defineProps<{
  repositoryPath: string | null;
}>();

const emit = defineEmits<{
  refresh: [done?: () => void];
  pending: [message: string];
}>();

const visible = ref(false);
const worktree = ref<WorktreeInfo | null>(null);
const force = ref(false);

function open(wt: WorktreeInfo) {
  worktree.value = wt;
  force.value = false;
  visible.value = true;
}

function close() {
  visible.value = false;
}

async function confirm() {
  if (!worktree.value || !props.repositoryPath) return;

  const wtPath = worktree.value.path;
  const isForce = force.value;

  visible.value = false;
  emit('pending', `Removing worktree at ${wtPath}...`);
  let awaitingRefresh = false;

  try {
    const cmdDesc = `git worktree remove${isForce ? ' --force' : ''} ${wtPath}`;

    await invokeGit(
      'remove_worktree',
      {
        path: props.repositoryPath,
        worktreePath: wtPath,
        force: isForce
      },
      cmdDesc
    );

    awaitingRefresh = true;
    emit('refresh', () => emit('pending', ''));
    notify(`Worktree removed successfully`, 'success');
  } catch (error: any) {
    notify(`Failed to remove worktree: ${error}`, 'error');
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible && worktree" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.65); z-index: 9998; backdrop-filter: blur(2px);">
    <div class="modal-content" @click.stop style="background: var(--bg-card, #1e1e1e); color: var(--text-main, #fff); padding: 22px; border-radius: 8px; box-shadow: 0 8px 30px rgba(0,0,0,.5); width: 420px; max-width: 90vw; position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); border: 1px solid var(--border, #333); z-index: 9999; font-family: inherit;">
      <h3 style="margin-top: 0; margin-bottom: 12px; font-size: 16px; font-weight: 600; color: var(--accent-red, #ff5252);">
        Remove Worktree
      </h3>
      <p style="margin: 0 0 8px 0; color: var(--text-muted, #aaa); font-size: 13px; line-height: 1.5;">
        Are you sure you want to remove the worktree at:
      </p>
      <div style="background: var(--input-bg, #2d2d2d); padding: 8px 10px; border-radius: 4px; font-size: 12px; font-family: monospace; word-break: break-all; margin-bottom: 14px; border: 1px solid var(--border, #444);">
        {{ worktree.path }}
      </div>

      <div style="margin-bottom: 20px;">
        <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px;">
          <input type="checkbox" v-model="force" />
          <span>Force removal (even if uncommitted changes or untracked files exist)</span>
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
          style="padding: 7px 16px; border: 0; border-radius: 5px; background: var(--accent-red, #ff5252); color: #fff; font-size: 12px; font-weight: 600; cursor: pointer;"
          @click="confirm"
        >
          Remove Worktree
        </button>
      </div>
    </div>
  </div>
</template>
