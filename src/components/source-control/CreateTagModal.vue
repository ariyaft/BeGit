<script setup lang="ts">
import { nextTick, ref, computed } from 'vue';
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
const targetCommit = ref('');
const tagType = ref<'lightweight' | 'annotated'>('annotated');
const message = ref('');
const pushToRemote = ref(false);
const selectedRemote = ref('origin');
const nameInput = ref<HTMLInputElement | null>(null);

const availableRemotes = computed(() => {
  if (props.remotes && props.remotes.length > 0) {
    return props.remotes.map(r => r.name);
  }
  return ['origin'];
});

async function open(options?: { targetCommit?: string; name?: string }) {
  tagName.value = options?.name || '';
  targetCommit.value = options?.targetCommit || 'HEAD';
  tagType.value = 'annotated';
  message.value = '';
  pushToRemote.value = false;
  if (props.remotes && props.remotes.length > 0) {
    selectedRemote.value = props.remotes[0].name;
  } else {
    selectedRemote.value = 'origin';
  }
  visible.value = true;
  await nextTick();
  nameInput.value?.focus();
}

function close() {
  visible.value = false;
}

async function confirm() {
  const name = tagName.value.trim();
  if (!name || !props.repositoryPath) return;

  const target = targetCommit.value.trim() && targetCommit.value.trim() !== 'HEAD' ? targetCommit.value.trim() : null;
  const isAnnotated = tagType.value === 'annotated';
  const tagMsg = isAnnotated ? (message.value.trim() || name) : null;
  const pushRemote = pushToRemote.value ? selectedRemote.value : null;

  visible.value = false;
  emit('pending', `Creating tag ${name}...`);
  let awaitingRefresh = false;

  try {
    const cmdDesc = isAnnotated
      ? `git tag -a ${name} -m "${tagMsg}"${target ? ' ' + target : ''}${pushRemote ? ` && git push ${pushRemote} ${name}` : ''}`
      : `git tag ${name}${target ? ' ' + target : ''}${pushRemote ? ` && git push ${pushRemote} ${name}` : ''}`;

    await invokeGit(
      'create_tag',
      {
        path: props.repositoryPath,
        name,
        targetCommit: target,
        message: tagMsg,
        annotate: isAnnotated,
        pushRemote: pushRemote
      },
      cmdDesc
    );

    awaitingRefresh = true;
    emit('refresh', () => emit('pending', ''));
    notify(`Tag ${name} created successfully${pushRemote ? ` and pushed to ${pushRemote}` : ''}.`, 'success');
  } catch (error: any) {
    notify(`Failed to create tag: ${error}`, 'error');
  } finally {
    if (!awaitingRefresh) emit('pending', '');
  }
}

defineExpose({ open });
</script>

<template>
  <div v-if="visible" class="modal-overlay" @click="close" style="position: fixed; inset: 0; background: rgba(0,0,0,.65); z-index: 9998; backdrop-filter: blur(2px);">
    <div class="modal-content" @click.stop style="background: var(--bg-card, #1e1e1e); color: var(--text-main, #fff); padding: 22px; border-radius: 8px; box-shadow: 0 8px 30px rgba(0,0,0,.5); width: 420px; max-width: 90vw; position: absolute; top: 50%; left: 50%; transform: translate(-50%,-50%); border: 1px solid var(--border, #333); z-index: 9999; font-family: inherit;">
      <h3 style="margin-top: 0; margin-bottom: 16px; font-size: 16px; font-weight: 600; display: flex; align-items: center; gap: 8px;">
        <span>Create Tag</span>
      </h3>

      <div class="form-group" style="margin-bottom: 14px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Tag Name <span style="color: var(--accent-red, #ff5252);">*</span></label>
        <input
          ref="nameInput"
          v-model="tagName"
          type="text"
          placeholder="e.g. v1.0.0 or release-2026"
          @keyup.enter="confirm"
          style="box-sizing: border-box; width: 100%; padding: 8px 10px; border: 1px solid var(--border, #444); border-radius: 5px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 13px;"
        />
      </div>

      <div class="form-group" style="margin-bottom: 14px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Target Commit / Ref</label>
        <input
          v-model="targetCommit"
          type="text"
          placeholder="HEAD, branch name, or commit SHA"
          style="box-sizing: border-box; width: 100%; padding: 8px 10px; border: 1px solid var(--border, #444); border-radius: 5px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 13px;"
        />
      </div>

      <div class="form-group" style="margin-bottom: 14px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Tag Type</label>
        <div style="display: flex; gap: 16px; font-size: 13px;">
          <label style="display: flex; align-items: center; gap: 6px; cursor: pointer;">
            <input type="radio" value="annotated" v-model="tagType" />
            <span>Annotated (<code>git tag -a</code>)</span>
          </label>
          <label style="display: flex; align-items: center; gap: 6px; cursor: pointer;">
            <input type="radio" value="lightweight" v-model="tagType" />
            <span>Lightweight</span>
          </label>
        </div>
      </div>

      <div v-if="tagType === 'annotated'" class="form-group" style="margin-bottom: 14px;">
        <label style="display: block; font-size: 12px; font-weight: 600; margin-bottom: 6px; color: var(--text-muted, #aaa);">Annotation Message</label>
        <textarea
          v-model="message"
          rows="3"
          placeholder="Tag release notes or description..."
          style="box-sizing: border-box; width: 100%; padding: 8px 10px; border: 1px solid var(--border, #444); border-radius: 5px; outline: none; background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); font-size: 12px; resize: vertical;"
        ></textarea>
      </div>

      <div class="form-group" style="margin-bottom: 18px; padding-top: 4px; border-top: 1px solid var(--border, #333);">
        <label style="display: flex; align-items: center; gap: 8px; cursor: pointer; font-size: 13px; margin-top: 8px;">
          <input type="checkbox" v-model="pushToRemote" />
          <span>Push tag to remote repository</span>
        </label>
        <div v-if="pushToRemote" style="margin-top: 8px; margin-left: 22px; display: flex; align-items: center; gap: 8px;">
          <span style="font-size: 12px; color: var(--text-muted, #aaa);">Remote:</span>
          <select
            v-model="selectedRemote"
            style="background: var(--input-bg, #2d2d2d); color: var(--text-main, #fff); border: 1px solid var(--border, #444); border-radius: 4px; padding: 4px 8px; font-size: 12px;"
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
          style="padding: 7px 16px; border: 0; border-radius: 5px; background: var(--accent, #00bcd4); color: #fff; font-size: 12px; font-weight: 600; cursor: pointer; display: flex; align-items: center; gap: 6px;"
          :disabled="!tagName.trim()"
          @click="confirm"
        >
          <span>Create Tag</span>
        </button>
      </div>
    </div>
  </div>
</template>
