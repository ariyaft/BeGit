<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { executeBranchAction } from '../../utils/gitHelpers';
import { notify } from '../../composables/useToasts';

const props = defineProps<{ repositoryPath: string | null }>();
const emit = defineEmits<{ refresh: [done?: () => void]; pending: [message: string] }>();

const visible = ref(false);
const baseName = ref('');
const newName = ref('');
const input = ref<HTMLInputElement | null>(null);

async function open(branch: { name: string }) {
  baseName.value = branch.name;
  newName.value = '';
  visible.value = true;
  await nextTick();
  input.value?.focus();
}

function close() {
  visible.value = false;
}

function normalizeName(event: Event) {
  newName.value = (event.target as HTMLInputElement).value.toLowerCase().replace(/ /g, '-');
}

async function confirm() {
  if (!newName.value.trim() || !props.repositoryPath) return;

  const targetName = newName.value.trim();
  visible.value = false;
  emit('pending', `Creating branch ${targetName}...`);
  let awaitingRefresh = false;
  try {
    const { success } = await executeBranchAction('create_branch', props.repositoryPath, baseName.value, targetName);
    if (success) {
      awaitingRefresh = true;
      emit('refresh', () => emit('pending', ''));
      notify(`Created branch ${targetName}.`, 'success');
    }
  } catch (error: any) {
    notify(`Failed to create branch: ${error}`);
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.6); z-index: 9998;">
    <div class="modal-content" @click.stop style="background: var(--bg-color,#1e1e1e); padding:20px; border-radius:8px; box-shadow:0 4px 15px rgba(0,0,0,.5); width:350px; position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); border:1px solid var(--border-color,#333); z-index:9999; font-family:sans-serif;">
      <h3 style="margin-top:0; color:var(--text-color,#fff);">Create Branch</h3>
      <p style="margin:10px 0; color:var(--text-muted,#aaa); font-size:14px;">Create a new branch from <strong>{{ baseName }}</strong>:</p>
      <input ref="input" :value="newName" @input="normalizeName" @keyup.enter="confirm" placeholder="new-branch-name..." style="box-sizing:border-box; width:100%; margin-bottom:15px; padding:8px; border:1px solid var(--border-color,#444); border-radius:4px; outline:none; background:var(--input-bg,#2d2d2d); color:var(--text-color,#fff);" />
      <div style="display:flex; justify-content:flex-end; gap:10px;"><button style="padding:6px 12px; border:1px solid var(--border-color,#444); border-radius:4px; background:var(--bg-light,#333); color:var(--text-color,#fff); cursor:pointer;" @click="close">Cancel</button><button style="padding:6px 12px; border:0; border-radius:4px; background:#00bcd4; color:#fff; cursor:pointer;" :disabled="!newName.trim()" @click="confirm">Create</button></div>
    </div>
  </div>
</template>
