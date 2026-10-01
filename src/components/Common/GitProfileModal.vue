<script setup lang="ts">
import ProfileIcon from '../../assets/icons/profile.svg?component';
import { ref, watch } from 'vue';

interface Profile {
  name: string;
  email: string;
  scope: 'repository' | 'global';
}

const props = defineProps<{
  visible: boolean;
  profile: Profile;
  loading: boolean;
  saving: boolean;
  error: string | null;
  hasRepository: boolean;
}>();

const name = ref('');
const email = ref('');
const scope = ref<'repository' | 'global'>('global');

watch(() => [props.visible, props.profile] as const, () => {
  if (!props.visible) return;
  name.value = props.profile.name || '';
  email.value = props.profile.email || '';
  scope.value = props.profile.scope || (props.hasRepository ? 'repository' : 'global');
}, { immediate: true, deep: true });

const emit = defineEmits<{
  close: [];
  save: [profile: Profile];
}>();

function submit() {
  emit('save', { name: name.value, email: email.value, scope: scope.value });
}
</script>

<template>
  <div v-if="visible" class="profile-modal-backdrop" @mousedown.self="!saving && $emit('close')">
    <form class="profile-modal" @submit.prevent="submit">
      <header>
        <div class="profile-icon"><ProfileIcon viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="1.8" fill="none"  /></div>
        <div><h2>Git profile</h2><p>Used as the author identity for new commits.</p></div>
        <button type="button" class="profile-close" :disabled="saving" @click="$emit('close')" aria-label="Close Git profile dialog">×</button>
      </header>

      <div v-if="loading" class="profile-loading">Reading Git configuration…</div>
      <template v-else>
        <label>Display name<input v-model.trim="name" autocomplete="name" placeholder="Your name" :disabled="saving" required /></label>
        <label>Email address<input v-model.trim="email" type="email" autocomplete="email" placeholder="you@example.com" :disabled="saving" required /></label>
        <fieldset>
          <legend>Apply profile to</legend>
          <label class="scope-option" :class="{ disabled: !hasRepository }"><input v-model="scope" type="radio" value="repository" :disabled="saving || !hasRepository" /> This repository only</label>
          <label class="scope-option"><input v-model="scope" type="radio" value="global" :disabled="saving" /> All Git repositories</label>
        </fieldset>
        <p v-if="error" class="profile-error">{{ error }}</p>
        <footer><button type="button" class="profile-cancel" :disabled="saving" @click="$emit('close')">Cancel</button><button type="submit" class="profile-save" :disabled="saving || !name || !email">{{ saving ? 'Saving…' : 'Save profile' }}</button></footer>
      </template>
    </form>
  </div>
</template>

<style scoped>
.profile-modal-backdrop { position: fixed; inset: 0; z-index: 10000; display: grid; place-items: center; padding: 20px; background: var(--overlay-bg); }.profile-modal { width: min(430px, 100%); padding: 20px; border: 1px solid var(--border); border-radius: 9px; background: var(--panel-bg); box-shadow: 0 18px 52px var(--shadow-color); color: var(--text-main); }.profile-modal header { display: flex; align-items: center; gap: 11px; margin-bottom: 20px; }.profile-icon { width: 36px; height: 36px; display: grid; place-items: center; border-radius: 50%; background: var(--row-selected); color: var(--accent-blue); }.profile-modal h2 { font-size: 1.05rem; }.profile-modal header p { margin-top: 3px; color: var(--text-muted); font-size: .78rem; }.profile-close { margin-left: auto; border: 0; background: transparent; color: var(--text-muted); cursor: pointer; font-size: 1.5rem; line-height: 1; }.profile-modal > label { display: grid; gap: 6px; margin-bottom: 14px; color: var(--text-muted); font-size: .78rem; font-weight: 650; }.profile-modal input[type='text'], .profile-modal input[type='email'] { width: 100%; border: 1px solid var(--border); border-radius: 5px; padding: 8px 9px; background: var(--input-bg); color: var(--text-main); font: inherit; font-size: .88rem; }.profile-modal input:focus { outline: 2px solid color-mix(in srgb, var(--accent-blue) 45%, transparent); border-color: var(--accent-blue); }.profile-modal fieldset { margin: 18px 0 0; padding: 10px; border: 1px solid var(--border); border-radius: 5px; }.profile-modal legend { padding: 0 4px; color: var(--text-muted); font-size: .74rem; }.scope-option { display: flex; align-items: center; gap: 7px; padding: 4px 0; color: var(--text-main); font-size: .8rem; }.scope-option.disabled { color: var(--text-muted); }.profile-error { margin-top: 12px; color: var(--danger-text); font-size: .8rem; }.profile-modal footer { display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px; }.profile-modal footer button { border-radius: 5px; padding: 7px 11px; cursor: pointer; font: inherit; font-size: .8rem; }.profile-cancel { border: 1px solid var(--border); background: transparent; color: var(--text-main); }.profile-save { border: 1px solid var(--accent-blue); background: var(--accent-blue); color: white; }.profile-modal button:disabled { cursor: default; opacity: .55; }.profile-loading { min-height: 144px; display: grid; place-items: center; color: var(--text-muted); font-size: .84rem; }
</style>
