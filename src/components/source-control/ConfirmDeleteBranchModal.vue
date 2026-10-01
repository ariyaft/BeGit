<script setup lang="ts">
import { ref } from 'vue';
import { executeBranchAction } from '../../utils/gitHelpers';
import { notify } from '../../composables/useToasts';

const props = defineProps<{ repositoryPath: string | null }>();
const emit = defineEmits<{ refresh: [fetchRemote: boolean, done?: () => void]; pending: [message: string] }>();

const visible = ref(false);
const branchName = ref('');
const isRemote = ref(false);

function open(branch: { name: string }, remote: boolean) {
  branchName.value = branch.name;
  isRemote.value = remote;
  visible.value = true;
}

function close() {
  visible.value = false;
}

async function confirm() {
  if (!props.repositoryPath || !branchName.value) return;

  const targetBranch = branchName.value;
  const targetIsRemote = isRemote.value;
  visible.value = false;
  emit('pending', `Deleting ${targetIsRemote ? 'remote' : 'local'} branch ${targetBranch}...`);
  let awaitingRefresh = false;
  try {
    const { success, fetchRemote } = await executeBranchAction(
      targetIsRemote ? 'delete_remote' : 'delete_local',
      props.repositoryPath,
      targetBranch
    );
    if (success) {
      awaitingRefresh = true;
      emit('refresh', fetchRemote, () => emit('pending', ''));
      notify(`Deleted ${targetIsRemote ? 'remote' : 'local'} branch ${targetBranch}.`, 'success');
    }
  } catch (error: any) {
    if (!targetIsRemote && error.toString().includes('not fully merged')) {
      if (window.confirm(`Branch '${targetBranch}' is not fully merged. Force delete locally anyway?`)) {
        try {
          const { success, fetchRemote } = await executeBranchAction('delete_local', props.repositoryPath, targetBranch, undefined, true);
          if (success) {
            awaitingRefresh = true;
            emit('refresh', fetchRemote, () => emit('pending', ''));
            notify(`Deleted local branch ${targetBranch}.`, 'success');
          }
        } catch (forceError) {
          notify(forceError);
        }
      }
    } else {
      notify(error);
    }
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.6); z-index: 9998;">
    <div class="modal-content" @click.stop style="background: var(--bg-color,#1e1e1e); padding:20px; border-radius:8px; box-shadow:0 4px 15px rgba(0,0,0,.5); width:350px; position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); border:1px solid var(--border-color,#333); z-index:9999; font-family:sans-serif;">
      <h3 style="margin-top:0; color:var(--text-danger,#f44336);">Delete Branch</h3>
      <p style="margin:10px 0; color:var(--text-color,#eee); font-size:14px;">Are you sure you want to delete the {{ isRemote ? 'remote' : 'local' }} branch <strong>{{ branchName }}</strong>?</p>
      <p v-if="isRemote" style="margin-bottom:20px; color:var(--text-muted,#aaa); font-size:12px;">This action cannot be undone and will permanently remove the branch from the remote server.</p>
      <div style="display:flex; justify-content:flex-end; gap:10px; margin-top:20px;">
        <button style="padding:6px 12px; border:1px solid var(--border-color,#444); border-radius:4px; background:var(--bg-light,#333); color:var(--text-color,#fff); cursor:pointer;" @click="close">Cancel</button>
        <button style="padding:6px 12px; border:0; border-radius:4px; background:#f44336; color:#fff; cursor:pointer;" @click="confirm">Delete</button>
      </div>
    </div>
  </div>
</template>
