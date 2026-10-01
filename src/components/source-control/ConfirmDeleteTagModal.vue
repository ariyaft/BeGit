<script setup lang="ts">
import { ref, computed } from 'vue';
import { notify } from '../../composables/useToasts';
import { invokeGit } from '../../composables/useActivityLog';

const props = defineProps<{
  repositoryPath: string | null;
  remotes?: Array<{ name: string; url: string }>;
}>();

const emit = defineEmits<{
  refresh: [done?: () => void];
  pending: [message: string];
}>();

const visible = ref(false);
const tagName = ref('');
const deleteFromRemote = ref(false);
const selectedRemote = ref('origin');

const availableRemotes = computed(() => {
  if (props.remotes && props.remotes.length > 0) {
    return props.remotes.map(r => r.name);
  }
  return ['origin'];
});

function open(tag: string) {
  tagName.value = tag;
  deleteFromRemote.value = false;
  if (props.remotes && props.remotes.length > 0) {
    selectedRemote.value = props.remotes[0].name;
  } else {
    selectedRemote.value = 'origin';
  }
  visible.value = true;
}

function close() {
  visible.value = false;
}

async function confirm() {
  if (!tagName.value || !props.repositoryPath) return;

  const name = tagName.value;
  const remote = deleteFromRemote.value ? selectedRemote.value : null;

  visible.value = false;
  emit('pending', `Deleting tag ${name}...`);
  let awaitingRefresh = false;

  try {
    const cmdDesc = remote
      ? `git tag -d ${name} && git push ${remote} --delete ${name}`
      : `git tag -d ${name}`;

    await invokeGit(
      'delete_tag',
      {
        path: props.repositoryPath,
        name,
        deleteRemote: remote
      },
      cmdDesc
    );

    awaitingRefresh = true;
    emit('refresh', () => emit('pending', ''));
    notify(`Tag ${name} deleted successfully${remote ? ` from ${remote} as well` : ''}.`, 'success');
  } catch (error: any) {
    notify(`Failed to delete tag: ${error}`, 'error');
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.65); z-index: 9998; backdrop-filter: blur(2px);">
    <div class="modal-content" @click.stop style="background: var(--bg-card, #1e1e1e); color: var(--text-main, #fff); padding: 22px; border-radius: 8px; box-shadow: 0 8px 30px rgba(0,0,0,.5); width: 400px; max-width: 90vw; position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); border: 1px solid var(--border, #333); z-index: 9999; font-family: inherit;">
      <h3 style="margin-top: 0; margin-bottom: 12px; font-size: 16px; font-weight: 600; color: var(--accent-red, #ff5252);">
        Delete Tag
      </h3>
      <p style="margin: 0 0 16px 0; color: var(--text-muted, #aaa); font-size: 13px; line-height: 1.5;">
        Are you sure you want to delete tag <strong style="color: var(--text-main, #fff);">{{ tagName }}</strong>?
      </p>

      <div style="margin-bottom: 20px; padding: 10px; background: var(--surface-subtle, rgba(255,255,255,0.03)); border-radius: 6px; border: 1px solid var(--border, #333);">
        <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px;">
          <input type="checkbox" v-model="deleteFromRemote" />
          <span>Also delete from remote repository</span>
        </label>
        <div v-if="deleteFromRemote" style="margin-top: 8px; margin-left: 22px; display: flex; align-items: center; gap: 8px;">
          <span style="font-size: 12px; color: var(--text-muted, #aaa);">Remote:</span>
          <select
            v-model="selectedRemote"
            style="background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); border: 1px solid var(--border, #444); border-radius: 4px; padding: 3px 6px; font-size: 12px;"
          >
            <option v-for="remote in availableRemotes" :key="remote" :value="remote">
              {{ remote }}
            </option>
          </select>
        </div>
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
          Delete Tag
        </button>
      </div>
    </div>
  </div>
</template>
