import { readonly, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export type ActivityStatus = 'running' | 'success' | 'error';

export interface ActivityLogEntry {
  id: number;
  timestamp: number;
  path?: string;
  command: string;
  status: ActivityStatus;
  error?: string;
}

const storageKey = 'begitActivityLog';
const maxEntries = 250;
const entries = ref<ActivityLogEntry[]>(loadEntries());
let nextId = Math.max(0, ...entries.value.map(entry => entry.id)) + 1;

function loadEntries(): ActivityLogEntry[] {
  try {
    const stored = localStorage.getItem(storageKey);
    const savedEntries = stored ? JSON.parse(stored) as ActivityLogEntry[] : [];
    return savedEntries.map(entry => entry.status === 'running'
      ? { ...entry, status: 'error', error: 'BeGit closed before this command completed.' }
      : entry
    );
  } catch {
    return [];
  }
}

function saveEntries() {
  localStorage.setItem(storageKey, JSON.stringify(entries.value.slice(0, maxEntries)));
}

export function startActivity(command: string, path?: string) {
  const id = nextId++;
  entries.value.unshift({ id, timestamp: Date.now(), path, command, status: 'running' });
  saveEntries();
  return id;
}

export function finishActivity(id: number, error?: unknown) {
  const entry = entries.value.find(candidate => candidate.id === id);
  if (!entry) return;
  entry.status = error ? 'error' : 'success';
  entry.error = error ? String(error) : undefined;
  saveEntries();
}

export async function invokeGit<T>(command: string, args: Record<string, unknown>, activityLabel: string): Promise<T> {
  const repoPath = typeof args.path === 'string'
    ? args.path
    : (typeof args.targetPath === 'string'
      ? args.targetPath
      : (typeof args.target_path === 'string' ? args.target_path : undefined));
  const id = startActivity(activityLabel, repoPath);
  try {
    const result = await invoke<T>(command, args);
    finishActivity(id);
    return result;
  } catch (error) {
    finishActivity(id, error);
    throw error;
  }
}

export function useActivityLog() {
  function clearActivityLog(path?: string) {
    entries.value = path ? entries.value.filter(entry => entry.path !== path) : [];
    saveEntries();
  }

  return { entries: readonly(entries), clearActivityLog };
}
