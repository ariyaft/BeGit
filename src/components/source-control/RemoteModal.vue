<script setup lang="ts">
import type { RemoteModalMode as Mode } from '../../types';
import { ref } from 'vue';
import { notify } from '../../composables/useToasts';
import { invokeGit } from '../../composables/useActivityLog';

const props = defineProps<{ repositoryPath: string | null }>();
const emit = defineEmits<{ refresh: [done?: () => void]; pending: [message: string] }>();

const visible = ref(false);
const mode = ref<Mode>('add');
const name = ref('');
const url = ref('');
const oldUrl = ref('');

function openAdd() {
  mode.value = 'add';
  name.value = 'origin';
  url.value = '';
  oldUrl.value = '';
  visible.value = true;
}

function openEdit(remote: { name: string; url: string }) {
  mode.value = 'edit';
  name.value = remote.name;
  url.value = remote.url;
  oldUrl.value = remote.url;
  visible.value = true;
}

function openRemove(remote: { name: string; url: string }) {
  mode.value = 'remove';
  name.value = remote.name;
  url.value = remote.url;
  oldUrl.value = remote.url;
  visible.value = true;
}

function close() {
  visible.value = false;
}

async function confirm() {
  if (!props.repositoryPath) return;
  if (mode.value === 'add' && (!name.value.trim() || !url.value.trim())) return;
  if (mode.value === 'edit' && (!url.value.trim() || url.value === oldUrl.value)) return;

  const verb = mode.value === 'add' ? 'Adding' : mode.value === 'edit' ? 'Updating' : 'Removing';
  visible.value = false;
  emit('pending', `${verb} remote ${name.value}...`);
  let awaitingRefresh = false;
  try {
    if (mode.value === 'remove') {
      await invokeGit('remove_remote', { path: props.repositoryPath, name: name.value }, `git remote remove ${name.value}`);
    } else if (mode.value === 'edit') {
      await invokeGit('remove_remote', { path: props.repositoryPath, name: name.value }, `git remote remove ${name.value}`);
      await invokeGit('add_remote', { path: props.repositoryPath, name: name.value, url: url.value }, `git remote add ${name.value}`);
    } else {
      await invokeGit('add_remote', { path: props.repositoryPath, name: name.value.trim(), url: url.value.trim() }, `git remote add ${name.value.trim()}`);
    }
    awaitingRefresh = true;
    emit('refresh', () => emit('pending', ''));
    notify(`${verb} remote ${name.value} complete.`, 'success');
  } catch (error: any) {
    notify(`Failed to ${verb.toLowerCase()} remote: ${error}`);
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ openAdd, openEdit, openRemove });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.6); z-index: 9998;">
    <div class="modal-content" @click.stop style="background: var(--bg-color,#1e1e1e); padding:20px; border-radius:8px; box-shadow:0 4px 15px rgba(0,0,0,.5); width:350px; position:absolute; top:50%; left:50%; transform:translate(-50%,-50%); border:1px solid var(--border-color,#333); z-index:9999; font-family:sans-serif;">
      <h3 style="margin-top:0; color:var(--text-color,#fff);">{{ mode === 'add' ? 'Add Remote' : mode === 'edit' ? 'Edit Remote URL' : 'Remove Remote' }}</h3>
      <template v-if="mode === 'add'"><p>Remote Name:</p><input v-model="name" type="text" /><p>Remote URL:</p><input v-model="url" type="text" @keyup.enter="confirm" /></template>
      <template v-else-if="mode === 'edit'"><p>New URL for remote <strong>{{ name }}</strong>:</p><input v-model="url" type="text" @keyup.enter="confirm" /></template>
      <p v-else>Are you sure you want to remove remote <strong>{{ name }}</strong>?</p>
      <div style="display:flex; justify-content:flex-end; gap:10px; margin-top:20px;"><button style="padding:6px 12px; border:1px solid var(--border-color,#444); border-radius:4px; background:var(--bg-light,#333); color:var(--text-color,#fff); cursor:pointer;" @click="close">Cancel</button><button style="padding:6px 12px; border:0; border-radius:4px; background:#00bcd4; color:#fff; cursor:pointer;" @click="confirm" :disabled="mode === 'add' ? (!name.trim() || !url.trim()) : mode === 'edit' ? (!url.trim() || url === oldUrl) : false">{{ mode === 'add' ? 'Add' : mode === 'edit' ? 'Save' : 'Remove' }}</button></div>
    </div>
  </div>
</template>

<style scoped>
p { margin: 10px 0; color: var(--text-muted, #aaa); font-size: 14px; } input { box-sizing: border-box; width: 100%; margin-bottom: 12px; padding: 8px; border: 1px solid var(--border-color, #444); border-radius: 4px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-color, #fff); }
</style>
