<script setup lang="ts">
import { nextTick, ref } from 'vue';
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
const reason = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

async function open(wt: WorktreeInfo) {
  worktree.value = wt;
  reason.value = '';
  visible.value = true;
  await nextTick();
  inputRef.value?.focus();
}

function close() {
  visible.value = false;
}

async function confirm() {
  if (!worktree.value || !props.repositoryPath) return;

  const wtPath = worktree.value.path;
  const lockReason = reason.value.trim() || null;

  visible.value = false;
  emit('pending', `Locking worktree...`);
  let awaitingRefresh = false;

  try {
    const cmdDesc = lockReason
      ? `git worktree lock --reason "${lockReason}" ${wtPath}`
      : `git worktree lock ${wtPath}`;

    await invokeGit(
      'lock_worktree',
      {
        path: props.repositoryPath,
        worktreePath: wtPath,
        reason: lockReason
      },
      cmdDesc
    );

    awaitingRefresh = true;
    emit('refresh', () => emit('pending', ''));
    notify(`Worktree locked`, 'success');
  } catch (error: any) {
    notify(`Failed to lock worktree: ${error}`, 'error');
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible && worktree" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.65); z-index: 9998; backdrop-filter: blur(2px);">
    <div class="modal-content" @click.stop style="background: var(--bg-card, #1e1e1e); color: var(--text-main, #fff); padding: 22px; border-radius: 8px; box-shadow: 0 8px 30px rgba(0,0,0,.5); width: 400px; max-width: 90vw; position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); border: 1px solid var(--border, #333); z-index: 9999; font-family: inherit;">
      <h3 style="margin-top: 0; margin-bottom: 12px; font-size: 16px; font-weight: 600;">
        Lock Worktree
      </h3>
      <p style="margin: 0 0 12px 0; color: var(--text-muted, #aaa); font-size: 13px;">
        Prevent automatic pruning or deletion of this worktree:
      </p>

      <div style="margin-bottom: 16px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Reason for lock (Optional)</label>
        <input
          ref="inputRef"
          v-model="reason"
          type="text"
          placeholder="e.g. Work in progress or stored on portable drive"
          @keyup.enter="confirm"
          style="box-sizing: border-box; width: 100%; padding: 8px 10px; border: 1px solid var(--border, #444); border-radius: 5px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 13px;"
        />
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
          @click="confirm"
        >
          Lock
        </button>
      </div>
    </div>
  </div>
</template>
