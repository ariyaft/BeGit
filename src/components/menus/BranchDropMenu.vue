<template>
  <div 
    v-if="visible && sourceBranch && targetBranch" 
    class="branch-drop-menu-overlay"
    @click="emit('close')"
    @contextmenu.prevent="emit('close')"
  >
    <div 
      class="branch-drop-menu"
      :style="menuStyle"
      @click.stop
    >
      <!-- Header with Flow Diagram -->
      <div class="drop-menu-header">
        <div class="drop-flow-container">
          <div class="drop-branch-chip source-chip" :title="sourceRef">
            <LocalIcon v-if="sourceBranch.isLocal" class="chip-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            <CloudIcon v-else-if="sourceBranch.isRemote" class="chip-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            <span class="chip-name">{{ sourceBranch.text }}</span>
          </div>
          
          <div class="drop-flow-arrow" title="onto / into">
            <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2.5" fill="none">
              <path d="M5 12h14M13 6l6 6-6 6" />
            </svg>
          </div>

          <div class="drop-branch-chip target-chip" :title="targetRef">
            <LocalIcon v-if="targetBranch.isLocal" class="chip-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            <CloudIcon v-else-if="targetBranch.isRemote" class="chip-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            <span class="chip-name">{{ targetBranch.text }}</span>
          </div>
        </div>

        <button class="drop-menu-close" @click="emit('close')" title="Cancel (Esc)">
          <CloseIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
        </button>
      </div>

      <!-- Action Items -->
      <div class="drop-menu-actions">
        <!-- Primary Action: Merge Source into Target -->
        <button 
          v-if="canMergeSourceIntoTarget"
          class="drop-action-btn action-merge"
          @click="onMerge(sourceRef, targetRef)"
        >
          <div class="action-icon-box merge-icon-box">
            <MergeIcon viewBox="0 0 24 24" width="15" height="15" stroke="currentColor" stroke-width="2" fill="none" />
          </div>
          <div class="action-text-content">
            <div class="action-main-line">
              <span class="action-verb">Merge</span>
              <strong class="action-branch-name">{{ sourceBranch.text }}</strong>
              <span class="action-prep">into</span>
              <strong class="action-branch-name">{{ targetBranch.text }}</strong>
            </div>
            <div class="action-sub-line">
              <span v-if="targetRef === activeBranch">Fast-forward or merge commit into checked out branch</span>
              <span v-else>Will checkout {{ targetBranch.text }} first, then merge</span>
            </div>
          </div>
        </button>

        <!-- Primary Action: Rebase Source onto Target -->
        <button 
          v-if="canRebaseSourceOntoTarget"
          class="drop-action-btn action-rebase"
          @click="onRebase(sourceRef, targetRef)"
        >
          <div class="action-icon-box rebase-icon-box">
            <RebaseIcon viewBox="0 0 24 24" width="15" height="15" stroke="currentColor" stroke-width="2" fill="none" />
          </div>
          <div class="action-text-content">
            <div class="action-main-line">
              <span class="action-verb">Rebase</span>
              <strong class="action-branch-name">{{ sourceBranch.text }}</strong>
              <span class="action-prep">onto</span>
              <strong class="action-branch-name">{{ targetBranch.text }}</strong>
            </div>
            <div class="action-sub-line">
              <span v-if="sourceRef === activeBranch">Replay checked out commits on top of {{ targetBranch.text }}</span>
              <span v-else>Will checkout {{ sourceBranch.text }} first, then rebase</span>
            </div>
          </div>
        </button>

        <!-- Reverse Actions (if applicable) -->
        <template v-if="hasReverseActions">
          <div class="drop-menu-divider">
            <span class="divider-text">Reverse Operations</span>
          </div>

          <!-- Reverse Merge: Target into Source -->
          <button 
            v-if="canMergeTargetIntoSource"
            class="drop-action-btn action-merge-rev"
            @click="onMerge(targetRef, sourceRef)"
          >
            <div class="action-icon-box merge-icon-box">
              <MergeIcon viewBox="0 0 24 24" width="15" height="15" stroke="currentColor" stroke-width="2" fill="none" />
            </div>
            <div class="action-text-content">
              <div class="action-main-line">
                <span class="action-verb">Merge</span>
                <strong class="action-branch-name">{{ targetBranch.text }}</strong>
                <span class="action-prep">into</span>
                <strong class="action-branch-name">{{ sourceBranch.text }}</strong>
              </div>
              <div class="action-sub-line">
                <span v-if="sourceRef === activeBranch">Fast-forward or merge commit into checked out branch</span>
                <span v-else>Will checkout {{ sourceBranch.text }} first, then merge</span>
              </div>
            </div>
          </button>

          <!-- Reverse Rebase: Target onto Source -->
          <button 
            v-if="canRebaseTargetOntoSource"
            class="drop-action-btn action-rebase-rev"
            @click="onRebase(targetRef, sourceRef)"
          >
            <div class="action-icon-box rebase-icon-box">
              <RebaseIcon viewBox="0 0 24 24" width="15" height="15" stroke="currentColor" stroke-width="2" fill="none" />
            </div>
            <div class="action-text-content">
              <div class="action-main-line">
                <span class="action-verb">Rebase</span>
                <strong class="action-branch-name">{{ targetBranch.text }}</strong>
                <span class="action-prep">onto</span>
                <strong class="action-branch-name">{{ sourceBranch.text }}</strong>
              </div>
              <div class="action-sub-line">
                <span v-if="targetRef === activeBranch">Replay checked out commits on top of {{ sourceBranch.text }}</span>
                <span v-else>Will checkout {{ targetBranch.text }} first, then rebase</span>
              </div>
            </div>
          </button>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, type PropType } from 'vue';
import LocalIcon from '../../assets/icons/local.svg?component';
import CloudIcon from '../../assets/icons/Cloud.svg?component';
import CloseIcon from '../../assets/icons/close.svg?component';
import MergeIcon from '../../assets/icons/context-2.svg?component';
import RebaseIcon from '../../assets/icons/timeline.svg?component';

export interface BranchDropPayload {
  source: { text: string; isLocal: boolean; isRemote: boolean; isHeadBranch?: boolean };
  target: { text: string; isLocal: boolean; isRemote: boolean; isHeadBranch?: boolean };
  x: number;
  y: number;
}

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  x: {
    type: Number,
    default: 0
  },
  y: {
    type: Number,
    default: 0
  },
  sourceBranch: {
    type: Object as PropType<{ text: string; isLocal: boolean; isRemote: boolean; isHeadBranch?: boolean } | null>,
    default: null
  },
  targetBranch: {
    type: Object as PropType<{ text: string; isLocal: boolean; isRemote: boolean; isHeadBranch?: boolean } | null>,
    default: null
  },
  activeBranch: {
    type: String,
    default: ''
  }
});

const emit = defineEmits<{
  merge: [source: string, target: string];
  rebase: [source: string, target: string];
  close: [];
}>();

const sourceRef = computed(() => {
  if (!props.sourceBranch) return '';
  if (!props.sourceBranch.isLocal && props.sourceBranch.isRemote) {
    return `origin/${props.sourceBranch.text}`;
  }
  return props.sourceBranch.text;
});

const targetRef = computed(() => {
  if (!props.targetBranch) return '';
  if (!props.targetBranch.isLocal && props.targetBranch.isRemote) {
    return `origin/${props.targetBranch.text}`;
  }
  return props.targetBranch.text;
});

// Can merge into target if target has a local branch
const canMergeSourceIntoTarget = computed(() => {
  if (!props.targetBranch) return false;
  return props.targetBranch.isLocal;
});

// Can rebase source onto target if source has a local branch
const canRebaseSourceOntoTarget = computed(() => {
  if (!props.sourceBranch) return false;
  return props.sourceBranch.isLocal;
});

// Can merge target into source if source has a local branch
const canMergeTargetIntoSource = computed(() => {
  if (!props.sourceBranch) return false;
  return props.sourceBranch.isLocal;
});

// Can rebase target onto source if target has a local branch
const canRebaseTargetOntoSource = computed(() => {
  if (!props.targetBranch) return false;
  return props.targetBranch.isLocal;
});

const hasReverseActions = computed(() => {
  return canMergeTargetIntoSource.value || canRebaseTargetOntoSource.value;
});

const menuStyle = computed(() => {
  const width = 340;
  const height = hasReverseActions.value ? 280 : 180;
  const padding = 12;

  let left = props.x;
  let top = props.y;

  if (left + width > window.innerWidth - padding) {
    left = Math.max(padding, window.innerWidth - width - padding);
  }
  if (top + height > window.innerHeight - padding) {
    top = Math.max(padding, window.innerHeight - height - padding);
  }

  return {
    left: `${left}px`,
    top: `${top}px`
  };
});

function onMerge(source: string, target: string) {
  emit('merge', source, target);
}

function onRebase(source: string, target: string) {
  emit('rebase', source, target);
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.visible) {
    emit('close');
  }
}

onMounted(() => {
  window.addEventListener('keydown', onKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
});
</script>

<style scoped>
.branch-drop-menu-overlay {
  position: fixed;
  inset: 0;
  z-index: 10000;
  background-color: transparent;
}

.branch-drop-menu {
  position: fixed;
  width: 340px;
  background-color: var(--panel-bg);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 10px 30px var(--shadow-color), 0 0 0 1px rgba(255, 255, 255, 0.05);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  animation: dropMenuPop 0.14s cubic-bezier(0.16, 1, 0.3, 1);
  user-select: none;
}

@keyframes dropMenuPop {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-4px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.drop-menu-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border);
}

.drop-flow-container {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;
  max-width: calc(100% - 24px);
}

.drop-branch-chip {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  background-color: var(--surface-subtle);
  border: 1px solid var(--border);
  color: var(--text-main);
  max-width: 120px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-chip {
  border-color: rgba(99, 102, 241, 0.4);
  background-color: rgba(99, 102, 241, 0.1);
  color: var(--text-main);
}

.target-chip {
  border-color: rgba(16, 185, 129, 0.4);
  background-color: rgba(16, 185, 129, 0.1);
  color: var(--text-main);
}

.chip-icon {
  flex-shrink: 0;
  color: var(--accent);
}

.chip-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.drop-flow-arrow {
  display: flex;
  align-items: center;
  color: var(--text-muted);
  flex-shrink: 0;
}

.drop-menu-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 4px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.drop-menu-close:hover {
  background-color: var(--surface-hover);
  color: var(--text-main);
}

.drop-menu-actions {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.drop-action-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid transparent;
  background: transparent;
  color: var(--text-main);
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
  width: 100%;
}

.drop-action-btn:hover {
  background-color: var(--surface-hover);
  border-color: var(--border);
}

.action-icon-box {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  flex-shrink: 0;
}

.merge-icon-box {
  background-color: rgba(168, 85, 247, 0.15);
  color: #a855f7;
  border: 1px solid rgba(168, 85, 247, 0.3);
}

.rebase-icon-box {
  background-color: rgba(14, 165, 233, 0.15);
  color: #0ea5e9;
  border: 1px solid rgba(14, 165, 233, 0.3);
}

.drop-action-btn:hover .merge-icon-box {
  background-color: rgba(168, 85, 247, 0.25);
  box-shadow: 0 0 8px rgba(168, 85, 247, 0.3);
}

.drop-action-btn:hover .rebase-icon-box {
  background-color: rgba(14, 165, 233, 0.25);
  box-shadow: 0 0 8px rgba(14, 165, 233, 0.3);
}

.action-text-content {
  display: flex;
  flex-direction: column;
  gap: 2px;
  overflow: hidden;
  flex: 1;
}

.action-main-line {
  font-size: 0.82rem;
  display: flex;
  align-items: center;
  gap: 4px;
  flex-wrap: wrap;
}

.action-verb {
  font-weight: 600;
}

.action-branch-name {
  color: var(--accent);
  max-width: 110px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.action-prep {
  color: var(--text-muted);
}

.action-sub-line {
  font-size: 0.7rem;
  color: var(--text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.drop-menu-divider {
  display: flex;
  align-items: center;
  justify-content: center;
  margin: 4px 0 2px 0;
  position: relative;
  text-align: center;
}

.drop-menu-divider::before {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  top: 50%;
  height: 1px;
  background-color: var(--border);
}

.divider-text {
  position: relative;
  background-color: var(--panel-bg);
  padding: 0 8px;
  font-size: 0.68rem;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
