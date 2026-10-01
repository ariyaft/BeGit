<script setup lang="ts">
import { nextTick, ref } from 'vue';
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
const submoduleUrl = ref('');
const submodulePath = ref('');
const submoduleBranch = ref('');
const urlInput = ref<HTMLInputElement | null>(null);

async function open() {
  submoduleUrl.value = '';
  submodulePath.value = '';
  submoduleBranch.value = '';
  visible.value = true;
  await nextTick();
  urlInput.value?.focus();
}

function close() {
  visible.value = false;
}

function onUrlChange() {
  if (!submodulePath.value && submoduleUrl.value) {
    const trimmed = submoduleUrl.value.trim().replace(/\/+$/, '');
    const lastPart = trimmed.split('/').pop() || '';
    submodulePath.value = lastPart.replace(/\.git$/i, '');
  }
}

async function confirm() {
  const url = submoduleUrl.value.trim();
  if (!url || !props.repositoryPath) return;

  const path = submodulePath.value.trim() || null;
  const branch = submoduleBranch.value.trim() || null;

  visible.value = false;
  emit('pending', `Adding submodule from ${url}...`);
  let awaitingRefresh = false;

  try {
    const cmdDesc = `git submodule add${branch ? ' -b ' + branch : ''} ${url}${path ? ' ' + path : ''}`;

    await invokeGit(
      'add_submodule',
      {
        path: props.repositoryPath,
        url,
        subPath: path,
        branch
      },
      cmdDesc
    );

    awaitingRefresh = true;
    emit('refresh', () => emit('pending', ''));
    notify(`Submodule added successfully`, 'success');
  } catch (error: any) {
    notify(`Failed to add submodule: ${error}`, 'error');
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.65); z-index: 9998; backdrop-filter: blur(2px);">
    <div class="modal-content" @click.stop style="background: var(--bg-card, #1e1e1e); color: var(--text-main, #fff); padding: 22px; border-radius: 8px; box-shadow: 0 8px 30px rgba(0,0,0,.5); width: 440px; max-width: 90vw; position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); border: 1px solid var(--border, #333); z-index: 9999; font-family: inherit;">
      <h3 style="margin-top: 0; margin-bottom: 16px; font-size: 16px; font-weight: 600; display: flex; align-items: center; gap: 8px;">
        <span>Add Submodule</span>
      </h3>

      <div class="form-group" style="margin-bottom: 14px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Repository URL <span style="color: var(--accent-red, #ff5252);">*</span></label>
        <input
          ref="urlInput"
          v-model="submoduleUrl"
          @input="onUrlChange"
          type="text"
          placeholder="https://github.com/username/repo.git or git@github.com:..."
          style="box-sizing: border-box; width: 100%; padding: 8px 10px; border: 1px solid var(--border, #444); border-radius: 5px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 13px;"
        />
      </div>

      <div class="form-group" style="margin-bottom: 14px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Local Path / Directory (Optional)</label>
        <input
          v-model="submodulePath"
          type="text"
          placeholder="e.g. vendor/my-module or lib/core"
          style="box-sizing: border-box; width: 100%; padding: 8px 10px; border: 1px solid var(--border, #444); border-radius: 5px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 13px;"
        />
      </div>

      <div class="form-group" style="margin-bottom: 18px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Track Branch (Optional)</label>
        <input
          v-model="submoduleBranch"
          type="text"
          placeholder="e.g. main or master"
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
          :disabled="!submoduleUrl.trim()"
          @click="confirm"
        >
          Add Submodule
        </button>
      </div>
    </div>
  </div>
</template>
