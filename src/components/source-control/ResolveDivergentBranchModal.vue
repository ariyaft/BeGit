<script setup lang="ts">
import { ref } from 'vue';
import { notify } from '../../composables/useToasts';
import { invokeGit } from '../../composables/useActivityLog';

const props = defineProps<{ repositoryPath: string | null }>();
const emit = defineEmits<{ refresh: [fetchRemote: boolean, done?: () => void]; pending: [message: string] }>();

const visible = ref(false);
const branchName = ref('');

function open(branch: string) {
  branchName.value = branch;
  visible.value = true;
}

function close() {
  visible.value = false;
}

async function handleAction(action: 'rebase' | 'merge') {
  if (!props.repositoryPath || !branchName.value) return;

  const targetBranch = branchName.value;
  visible.value = false;
  
  const actionLabel = action === 'rebase' ? 'Rebasing' : 'Merging';
  emit('pending', `${actionLabel} ${targetBranch} onto remote...`);
  
  let awaitingRefresh = false;
  try {
    if (action === 'rebase') {
      await invokeGit('rebase_branch', { path: props.repositoryPath, branchName: `origin/${targetBranch}` }, `git rebase origin/${targetBranch}`);
    } else {
      await invokeGit('merge_branch', { path: props.repositoryPath, branchName: `origin/${targetBranch}` }, `git merge origin/${targetBranch}`);
    }
    awaitingRefresh = true;
    emit('refresh', true, () => emit('pending', ''));
    notify(`${actionLabel} completed successfully.`, 'success');
  } catch (error: any) {
    notify(`${actionLabel} failed: ${error}`);
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.6); z-index: 9998;">
    <div class="modal-content" @click.stop style="background: var(--bg-color,#1e1e1e); padding:20px; border-radius:8px; box-shadow:0 4px 15px rgba(0,0,0,.5); width:450px; position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); border:1px solid var(--border-color,#333); z-index:9999; font-family:sans-serif;">
      <h3 style="margin-top:0; color:var(--text-color,#fff);">Branch Diverged</h3>
      <p style="margin:10px 0; color:var(--text-color,#eee); font-size:14px; line-height:1.5;">
        Cannot fast-forward pull because the local branch <strong>{{ branchName }}</strong> has diverged from its remote counterpart.
      </p>
      <p style="margin-bottom:20px; color:var(--text-muted,#aaa); font-size:13px; line-height:1.5;">
        How would you like to reconcile these changes?
      </p>
      
      <div style="display:flex; flex-direction: column; gap:12px; margin-top:20px;">
        <button style="padding:10px 12px; text-align:left; border:1px solid var(--border-color,#444); border-radius:4px; background:var(--bg-light,#333); color:var(--text-color,#fff); cursor:pointer; display:flex; flex-direction:column; gap:4px;" @click="handleAction('rebase')">
          <strong style="color:#2196f3;">Rebase (Recommended)</strong>
          <span style="font-size:12px; color:var(--text-muted,#aaa);">Reapply your local commits on top of the remote changes, keeping a linear history.</span>
        </button>
        
        <button style="padding:10px 12px; text-align:left; border:1px solid var(--border-color,#444); border-radius:4px; background:var(--bg-light,#333); color:var(--text-color,#fff); cursor:pointer; display:flex; flex-direction:column; gap:4px;" @click="handleAction('merge')">
          <strong>Merge</strong>
          <span style="font-size:12px; color:var(--text-muted,#aaa);">Create a new merge commit combining both local and remote changes.</span>
        </button>
      </div>

      <div style="display:flex; justify-content:flex-end; gap:10px; margin-top:20px; padding-top:16px; border-top:1px solid var(--border-color,#333);">
        <button style="padding:6px 12px; border:1px solid var(--border-color,#444); border-radius:4px; background:transparent; color:var(--text-color,#fff); cursor:pointer;" @click="close">Cancel</button>
      </div>
    </div>
  </div>
</template>
