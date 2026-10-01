<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue';
import CloseIcon from '../../assets/icons/close.svg?component';
import ActionIcon from '../../assets/icons/action.svg?component';
import { invokeGit } from '../../composables/useActivityLog';
import { notify } from '../../composables/useToasts';

const props = defineProps<{
  repositoryPath: string | null;
}>();

const emit = defineEmits<{
  close: [];
  refresh: [];
  pending: [message: string];
}>();

interface TerminalEntry {
  command: string;
  output: string;
  failed: boolean;
}

const command = ref('');
const entries = ref<TerminalEntry[]>([]);
const running = ref(false);
const inputRef = ref<HTMLInputElement | null>(null);
const height = ref(Number(localStorage.getItem('begitGitTerminalHeight')) || 240);
const panelStyle = computed(() => ({ height: `${height.value}px` }));

onMounted(() => nextTick(() => inputRef.value?.focus()));

async function runCommand() {
  const value = command.value.trim();
  if (!value || !props.repositoryPath || running.value) return;

  command.value = '';
  running.value = true;
  emit('pending', 'Running Git command...');
  try {
    const result = await invokeGit<{ output: string }>(
      'run_git_terminal_command',
      { path: props.repositoryPath, command: value },
      `git ${value.replace(/^git\s+/i, '')}`
    );
    entries.value.push({ command: value, output: result.output, failed: false });
    emit('refresh');
    notify('Git command completed', 'success');
  } catch (error) {
    entries.value.push({ command: value, output: String(error), failed: true });
    notify('Git command failed; see the terminal output.', 'error');
  } finally {
    running.value = false;
    emit('pending', '');
    await nextTick();
    inputRef.value?.focus();
  }
}

function clear() {
  entries.value = [];
  inputRef.value?.focus();
}

function startResize(event: MouseEvent) {
  const startY = event.clientY;
  const startHeight = height.value;
  const onMove = (moveEvent: MouseEvent) => {
    height.value = Math.max(140, Math.min(520, startHeight + startY - moveEvent.clientY));
  };
  const onUp = () => {
    localStorage.setItem('begitGitTerminalHeight', String(height.value));
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  };
  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}
</script>

<template>
  <section class="git-terminal" :style="panelStyle" aria-label="Git terminal">
    <div class="terminal-resizer" title="Drag to resize terminal" @mousedown.prevent="startResize" />
    <header class="terminal-header">
      <div class="terminal-title">
        <ActionIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        <span>Git Terminal</span>
        <span class="terminal-path" :title="repositoryPath || ''">{{ repositoryPath || 'No repository selected' }}</span>
      </div>
      <div class="terminal-actions">
        <button class="terminal-action" @click="clear" title="Clear terminal output">Clear</button>
        <button class="terminal-icon-action" @click="emit('close')" title="Close terminal">
          <CloseIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        </button>
      </div>
    </header>

    <div class="terminal-output" aria-live="polite">
      <div v-if="entries.length === 0" class="terminal-hint">
        Run Git commands in this repository. Examples: <code>status</code>, <code>log --oneline -10</code>, or <code>git diff</code>.
      </div>
      <article v-for="(entry, index) in entries" :key="index" class="terminal-entry" :class="{ failed: entry.failed }">
        <div class="terminal-command"><span>$</span> git {{ entry.command.replace(/^git\s+/i, '') }}</div>
        <pre>{{ entry.output }}</pre>
      </article>
    </div>

    <form class="terminal-input-row" @submit.prevent="runCommand">
      <span class="terminal-prompt">git</span>
      <input
        ref="inputRef"
        v-model="command"
        class="terminal-input"
        :disabled="running || !repositoryPath"
        autocomplete="off"
        autocapitalize="off"
        spellcheck="false"
        placeholder="status"
        aria-label="Git command"
      />
      <button class="terminal-run" type="submit" :disabled="running || !repositoryPath || !command.trim()">
        {{ running ? 'Running…' : 'Run' }}
      </button>
    </form>
  </section>
</template>

<style scoped>
.git-terminal { min-height: 140px; max-height: 520px; display: flex; flex-direction: column; flex-shrink: 0; background: #0d1117; border-top: 1px solid var(--border); color: #c9d1d9; position: relative; font-family: ui-monospace, SFMono-Regular, Consolas, monospace; }
.terminal-resizer { position: absolute; top: -5px; left: 0; right: 0; height: 9px; cursor: row-resize; z-index: 2; }
.terminal-resizer:hover::after { content: ''; position: absolute; left: 0; right: 0; top: 4px; border-top: 2px solid var(--accent); }
.terminal-header { height: 34px; padding: 0 10px 0 14px; display: flex; align-items: center; justify-content: space-between; gap: 12px; border-bottom: 1px solid #30363d; background: #161b22; flex-shrink: 0; font-family: inherit; }
.terminal-title, .terminal-actions { display: flex; align-items: center; gap: 8px; min-width: 0; }
.terminal-title { color: #f0f6fc; font-size: .76rem; font-weight: 700; }
.terminal-title :deep(svg) { color: #58a6ff; flex-shrink: 0; }
.terminal-path { color: #8b949e; font-weight: 400; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.terminal-action, .terminal-icon-action { border: 0; background: transparent; color: #8b949e; cursor: pointer; border-radius: 4px; font-size: .72rem; padding: 4px 6px; display: inline-flex; align-items: center; }
.terminal-action:hover, .terminal-icon-action:hover { color: #f0f6fc; background: #30363d; }
.terminal-output { flex: 1; overflow: auto; padding: 10px 14px; font-size: .78rem; line-height: 1.45; min-height: 0; }
.terminal-hint { color: #8b949e; padding-top: 2px; }
.terminal-hint code { color: #79c0ff; }
.terminal-entry + .terminal-entry { margin-top: 13px; }
.terminal-command { color: #7ee787; white-space: pre-wrap; }
.terminal-command span { color: #58a6ff; margin-right: 7px; }
.terminal-entry pre { margin: 4px 0 0; white-space: pre-wrap; word-break: break-word; color: #c9d1d9; font: inherit; }
.terminal-entry.failed pre { color: #ff7b72; }
.terminal-input-row { display: flex; align-items: center; gap: 8px; padding: 8px 10px 8px 14px; border-top: 1px solid #30363d; flex-shrink: 0; }
.terminal-prompt { color: #7ee787; font-size: .78rem; font-weight: 700; }
.terminal-input { min-width: 0; flex: 1; border: 1px solid #30363d; border-radius: 4px; background: #010409; color: #f0f6fc; padding: 6px 8px; outline: none; font: inherit; font-size: .78rem; }
.terminal-input:focus { border-color: #58a6ff; box-shadow: 0 0 0 2px rgba(56, 139, 253, .18); }
.terminal-run { border: 1px solid #238636; border-radius: 4px; padding: 6px 10px; background: #238636; color: white; cursor: pointer; font: inherit; font-size: .76rem; font-weight: 700; }
.terminal-run:disabled { cursor: not-allowed; opacity: .55; }
</style>
