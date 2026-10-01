<script setup lang="ts">
import { ref } from 'vue';
import { notify } from '../../composables/useToasts';
import { invokeGit } from '../../composables/useActivityLog';

const props = defineProps<{ repositoryPath: string | null }>();

const emit = defineEmits<{
  refresh: [done?: () => void];
  pending: [message: string];
}>();

const visible = ref(false);
const stashId = ref('');
const stashMessage = ref('');

function open(stash: { id: string; message: string }) {
  stashId.value = stash.id;
  stashMessage.value = stash.message;
  visible.value = true;
}

function close() {
  visible.value = false;
}

async function confirm() {
  if (!stashId.value || !props.repositoryPath) return;

  const targetStashId = stashId.value;
  visible.value = false;
  emit('pending', `Dropping stash ${targetStashId}...`);
  let awaitingRefresh = false;
  try {
    await invokeGit('drop_stash', { path: props.repositoryPath, stashId: targetStashId }, `git stash drop ${targetStashId}`);
    awaitingRefresh = true;
    emit('refresh', () => emit('pending', ''));
    notify(`Dropped stash ${targetStashId}.`, 'success');
  } catch (error) {
    console.error('Failed to drop stash:', error);
    notify('Failed to drop stash. Check console for details.');
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.6); z-index: 9998;">
    <div class="modal-content" @click.stop style="background: var(--bg-color,#1e1e1e); padding:20px; border-radius:8px; box-shadow:0 4px 15px rgba(0,0,0,.5); width:350px; position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); border:1px solid var(--border-color,#333); z-index:9999; font-family:sans-serif;">
      <h3 style="margin-top:0; color:var(--text-danger,#f44336);">Delete Stash</h3>
      <p style="margin:10px 0; color:var(--text-color,#eee); font-size:14px;">Are you sure you want to delete <strong>{{ stashId }}</strong>?</p>
      <p style="margin-bottom:20px; color:var(--text-muted,#aaa); font-size:12px;">{{ stashMessage }}</p>
      <div style="display:flex; justify-content:flex-end; gap:10px; margin-top:20px;">
        <button style="padding:6px 12px; border:1px solid var(--border-color,#444); border-radius:4px; background:var(--bg-light,#333); color:var(--text-color,#fff); cursor:pointer;" @click="close">Cancel</button>
        <button style="padding:6px 12px; border:0; border-radius:4px; background:#f44336; color:#fff; cursor:pointer;" @click="confirm">Delete</button>
      </div>
    </div>
  </div>
</template>
