<template>
  <div class="graph-container" tabindex="0" style="outline: none;" @keydown="onKeyDown">
    <div class="commit-list-wrapper" :style="{ width: '100%', minWidth: `calc(${labelsColWidth + spacerWidth + msgColWidth + 230}px)` }">
      <div class="graph-header">
        <div class="commit-labels-col header-cell" :style="{ width: labelsColWidth + 'px' }">
          Branches / Tags
          <div class="labels-resizer" @mousedown.stop.prevent="startLabelsDrag"></div>
        </div>
        <div class="commit-graph-spacer header-cell" :style="{ width: spacerWidth + 'px', flexShrink: 0 }">Graph</div>
        <div class="commit-msg-col header-cell" :style="{ width: msgColWidth + 'px' }">
          Description
          <div class="labels-resizer" @mousedown.stop.prevent="startMsgDrag"></div>
        </div>
        <div class="commit-meta-col header-cell" style="padding: 0 10px;">
          <div class="commit-hash-col">Commit</div>
          <div class="commit-date-col">Date</div>
        </div>
      </div>
      
      <svg class="graph-svg" :width="spacerWidth" :height="commits.length * ROW_HEIGHT" :style="{ top: '30px', left: labelsColWidth + 'px', width: spacerWidth + 'px' }"  >
  <path v-for="(path, i) in paths" :key="i" :d="path.d" :stroke="path.color" :stroke-opacity="path.opacity" :stroke-width="path.isMain ? 3 : 1.5" fill="none" />
</svg>
      
      <div class="commit-list">
        <div v-for="commit in commits" :key="commit.hash" 
             :id="'commit-row-' + commit.hash"
             class="commit-row" 
             :class="{ 
               'active-row': commit.isHead,
               'selected-row': isSelected(commit)
             }"
             :style="{ height: ROW_HEIGHT + 'px' }"
             @click="$emit('select-commit', commit, $event)"
             @contextmenu.prevent="$emit('contextmenu-commit', commit, $event)">
          
          <!-- Labels Column -->
          <div 
            class="commit-labels-col" 
            :style="{ width: labelsColWidth + 'px' }"
          >
            <div class="labels-wrapper">
              <div v-for="(label, lIdx) in commit.labels" :key="lIdx" 
                   class="branch-label"
                   :class="{
                     'is-draggable-branch': isBranchLabel(label),
                     'is-dragging-source': isDragging && currentDraggedBranch?.text === label.text,
                     'is-drop-target': currentHoveredTarget?.text === label.text && isDragging && currentDraggedBranch?.text !== label.text
                   }"
                   :style="getLabelStyle(label)"
                   :title="label.text + (isBranchLabel(label) ? ' (Drag onto another branch to merge or rebase)' : '')"
                   :data-branch-name="label.text"
                   :data-is-local="String(label.isLocal)"
                   :data-is-remote="String(label.isRemote)"
                   :data-is-tag="String(label.isTag)"
                   @mousedown="onBranchMouseDown($event, label)">
                <span class="label-text">{{ label.text }}</span>
                <StarIcon v-if="label.isHeadBranch" class="label-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" fill="#fbbc04" stroke-width="2" style="color: #fbbc04;"  />
                <LocalIcon v-if="label.isLocal" :style="{ color: label.isLocal && label.isRemote ? 'var(--local-label)' : '' }" class="label-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none"  />
                <CloudIcon v-if="label.isRemote" :style="{ color: label.isLocal && label.isRemote ? 'var(--remote-label)' : '' }" class="label-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none"  />
                <TagIcon v-if="label.isTag" :style="{ color: label.isLocal && label.isRemote ? 'var(--remote-label)' : '' }" class="label-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none"  />
              </div>
            </div>
            <div class="labels-resizer" @mousedown.stop.prevent="startLabelsDrag"></div>
          </div>

          <!-- Left area: Graph Spacer -->
          <div class="commit-graph-spacer" :style="{ width: spacerWidth + 'px', flexShrink: 0 }">
            <div v-if="commit.isHead" class="node head-node" 
                 :style="{ 
                   borderColor: commit.color,
                   left: (LEFT_MARGIN + (commit.col * COL_WIDTH) - 7) + 'px',
                   top: ((ROW_HEIGHT / 2) - 7) + 'px',
                   width: '14px',
                   height: '14px',
                   borderWidth: '2px'
                 }">
              <StarIcon viewBox="0 0 24 24" width="10" height="10" stroke="#fbbc04" fill="#fbbc04" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"  />
            </div>
            <div v-else class="node solid-node" 
                 :style="{ 
                   backgroundColor: commit.color, 
                   left: (LEFT_MARGIN + (commit.col * COL_WIDTH) - 4) + 'px',
                   top: ((ROW_HEIGHT / 2) - 4) + 'px'
                 }">
            </div>
          </div>

          <!-- Message Column -->
          <div class="commit-msg-col" :style="{ width: msgColWidth + 'px' }">
            <AuthorAvatar :author-name="commit.author" :size="20" style="margin-right: 8px;" />
            <div class="commit-message" :style="{ color: commit.isHead ? 'var(--text-main)' : 'var(--text-muted)' }">
              <span v-if="commit.signature" 
                    class="commit-sig-badge" 
                    :class="'sig-' + commit.signature.status"
                    :title="getSignatureTooltip(commit.signature)">
                <ShieldCheckIcon v-if="commit.signature.status === 'verified'" viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" class="sig-badge-icon" />
                <ShieldAlertIcon v-else viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" class="sig-badge-icon" />
                <span>{{ commit.signature.status === 'verified' ? 'Verified' : commit.signature.status }}</span>
              </span>
              <span v-if="parseMessage(commit.message).prefix" class="commit-type-label" :class="getTypeClass(parseMessage(commit.message).prefix || '')">
                {{ parseMessage(commit.message).prefix }}
              </span>
              {{ parseMessage(commit.message).text }}
            </div>
            <div class="labels-resizer" @mousedown.stop.prevent="startMsgDrag"></div>
          </div>

          <!-- Meta Column -->
          <div class="commit-meta-col" style="padding: 0 10px;">
            <div class="commit-hash-col">{{ commit.hash }}</div>
            <div class="commit-date-col" :title="commit.exact_time">{{ commit.time }}</div>
          </div>
          
        </div>
      </div>
    </div>

    <!-- Floating drag preview attached to cursor -->
    <Teleport to="body">
      <div 
        v-if="isDragging && currentDraggedBranch" 
        class="floating-branch-drag-preview"
        :style="{ transform: `translate3d(${dragPos.x + 14}px, ${dragPos.y + 14}px, 0)` }"
      >
        <div class="preview-badge" :class="{ 'has-target': Boolean(currentHoveredTarget) }">
          <LocalIcon v-if="currentDraggedBranch.isLocal" class="preview-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
          <CloudIcon v-else class="preview-icon" viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
          <span class="preview-source">{{ currentDraggedBranch.text }}</span>
          <template v-if="currentHoveredTarget">
            <span class="preview-arrow">➔</span>
            <span class="preview-target">{{ currentHoveredTarget.text }}</span>
          </template>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import StarIcon from '../../assets/icons/star.svg?component';
import LocalIcon from '../../assets/icons/local.svg?component';
import CloudIcon from '../../assets/icons/Cloud.svg?component';
import TagIcon from '../../assets/icons/tag.svg?component';
import ShieldCheckIcon from '../../assets/icons/shield-check.svg?component';
import ShieldAlertIcon from '../../assets/icons/shield-alert.svg?component';
import AuthorAvatar from '../Common/AuthorAvatar.vue';
import { ref, computed, type PropType } from 'vue';

const props = defineProps({
  commits: {
    type: Array as PropType<any[]>,
    required: true
  },
  selectedCommits: {
    type: Array as PropType<any[]>,
    default: () => []
  },
  selectedCommit: {
    type: Object as PropType<any>,
    default: null
  }
});

const emit = defineEmits(['select-commit', 'contextmenu-commit', 'branch-drop']);

const isDragging = ref(false);
const currentDraggedBranch = ref<any | null>(null);
const currentHoveredTarget = ref<any | null>(null);
const dragPos = ref({ x: 0, y: 0 });

function isBranchLabel(label: any): boolean {
  if (!label) return false;
  if (label.isTag) return false;
  if (label.text === 'HEAD') return false;
  return true;
}

function onBranchMouseDown(e: MouseEvent, label: any) {
  if (e.button !== 0) return; // Only primary left click
  if (!isBranchLabel(label)) return;

  const startX = e.clientX;
  const startY = e.clientY;
  let dragStarted = false;

  function onMouseMove(moveEvent: MouseEvent) {
    const dx = moveEvent.clientX - startX;
    const dy = moveEvent.clientY - startY;

    if (!dragStarted) {
      if (Math.hypot(dx, dy) > 4) {
        dragStarted = true;
        isDragging.value = true;
        currentDraggedBranch.value = label;
        document.body.classList.add('is-branch-dragging');
      } else {
        return;
      }
    }

    dragPos.value = { x: moveEvent.clientX, y: moveEvent.clientY };

    // Hit-test element under cursor
    const el = document.elementFromPoint(moveEvent.clientX, moveEvent.clientY);
    if (!el) {
      currentHoveredTarget.value = null;
      return;
    }

    const branchEl = el.closest('.branch-label') as HTMLElement | null;
    if (branchEl && branchEl.dataset.branchName) {
      const name = branchEl.dataset.branchName;
      const isTag = branchEl.dataset.isTag === 'true';
      if (!isTag && name !== 'HEAD' && name !== label.text) {
        currentHoveredTarget.value = {
          text: name,
          isLocal: branchEl.dataset.isLocal === 'true',
          isRemote: branchEl.dataset.isRemote === 'true'
        };
        return;
      }
    }

    const cellEl = el.closest('.commit-labels-col') as HTMLElement | null;
    if (cellEl) {
      const labels = Array.from(cellEl.querySelectorAll('.branch-label')) as HTMLElement[];
      const validLabels = labels.filter(l => l.dataset.isTag !== 'true' && l.dataset.branchName !== 'HEAD' && l.dataset.branchName !== label.text);
      if (validLabels.length === 1) {
        const targetEl = validLabels[0];
        currentHoveredTarget.value = {
          text: targetEl.dataset.branchName!,
          isLocal: targetEl.dataset.isLocal === 'true',
          isRemote: targetEl.dataset.isRemote === 'true'
        };
        return;
      }
    }

    currentHoveredTarget.value = null;
  }

  function onMouseUp(upEvent: MouseEvent) {
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
    document.body.classList.remove('is-branch-dragging');

    if (dragStarted) {
      upEvent.preventDefault();
      upEvent.stopPropagation();

      const source = currentDraggedBranch.value;
      const target = currentHoveredTarget.value;

      isDragging.value = false;
      currentDraggedBranch.value = null;
      currentHoveredTarget.value = null;

      if (source && target && source.text !== target.text) {
        emit('branch-drop', {
          source,
          target,
          x: upEvent.clientX,
          y: upEvent.clientY
        });
      }
    } else {
      isDragging.value = false;
      currentDraggedBranch.value = null;
      currentHoveredTarget.value = null;
    }
  }

  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
}

function isSelected(commit: any) {
  return props.selectedCommits.some(c => c.id === commit.id);
}

function parseMessage(msg: string) {
  if (!msg) return { prefix: null, text: '' };
  
  const match = msg.match(/^([a-zA-Z]+(?:\[.*?\]|\(.*?\))?!?:)\s+(.*)/);
  if (match) {
    return { prefix: match[1], text: match[2] };
  }
  
  if (msg.startsWith('Merge ') || msg.startsWith('merge ')) {
    return { prefix: 'Merge', text: msg.substring(6) };
  }
  
  return { prefix: null, text: msg };
}

function getTypeClass(prefix: string) {
  if (!prefix) return 'label-default';
  const type = prefix.split(/[\(\:\!]/)[0].toLowerCase();
  if (['feat', 'feature'].includes(type)) return 'label-feat';
  if (['fix', 'bug'].includes(type)) return 'label-fix';
  if (['chore', 'refactor', 'style', 'test', 'docs', 'build', 'ci', 'perf'].includes(type)) return 'label-chore';
  if (type === 'merge') return 'label-merge';
  return 'label-default';
}

function getSignatureTooltip(sig: any) {
  if (!sig) return '';
  const statusUpper = sig.status.charAt(0).toUpperCase() + sig.status.slice(1);
  let tip = `${statusUpper} signature`;
  if (sig.signer) tip += ` by ${sig.signer}`;
  if (sig.key) tip += ` (Key: ${sig.key})`;
  return tip;
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    e.preventDefault();
    if (props.commits.length === 0) return;
    
    const currentIndex = props.commits.findIndex((c: any) => c.id === props.selectedCommit?.id);
    
    let nextCommit = null;
    if (currentIndex === -1) {
      nextCommit = props.commits[0];
    } else if (e.key === 'ArrowDown' && currentIndex < props.commits.length - 1) {
      nextCommit = props.commits[currentIndex + 1];
    } else if (e.key === 'ArrowUp' && currentIndex > 0) {
      nextCommit = props.commits[currentIndex - 1];
    }
    
    if (nextCommit) {
      emit('select-commit', nextCommit);
      
      setTimeout(() => {
        const el = document.getElementById(`commit-row-${nextCommit.hash}`);
        const container = document.querySelector('.graph-container');
        if (el && container) {
          const elTop = el.offsetTop;
          const elBottom = elTop + el.offsetHeight;
          const containerTop = container.scrollTop;
          const containerBottom = containerTop + container.clientHeight;
          
          if (elTop < containerTop + 30) {
            container.scrollTo({ top: elTop - 30 });
          } else if (elBottom > containerBottom) {
            container.scrollTo({ top: elBottom - container.clientHeight });
          }
        }
      }, 0);
    }
  }
}

// Constants for layout
const ROW_HEIGHT = 36;
const COL_WIDTH = 16;
const LEFT_MARGIN = 20; 
const START_Y = ROW_HEIGHT / 2;

const labelsColWidth = ref(180);
const msgColWidth = ref(480);

function getLabelStyle(label: any) {
  const GREEN = 'var(--remote-label)';
  const YELLOW = 'var(--local-label)';
  
  if (label.isLocal && label.isRemote) {
    return {
      borderTopColor: YELLOW,
      borderLeftColor: YELLOW,
      borderBottomColor: GREEN,
      borderRightColor: GREEN,
      color: 'var(--text-main)'
    };
  } else if (label.isRemote || label.isTag) {
    return {
      borderColor: GREEN,
      color: GREEN
    };
  } else {
    return {
      borderColor: YELLOW,
      color: YELLOW
    };
  }
}

function getZoomFactor(): number {
  const rootZoom = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--app-zoom'));
  if (!isNaN(rootZoom) && rootZoom > 0) return rootZoom;
  const docZoom = parseFloat((document.documentElement.style as any).zoom);
  if (!isNaN(docZoom) && docZoom > 0) return docZoom / 100;
  return 1;
}

function startLabelsDrag(e: MouseEvent) {
  e?.preventDefault?.();
  e?.stopPropagation?.();
  window.getSelection()?.removeAllRanges();
  const startX = e.clientX;
  const startWidth = labelsColWidth.value;
  const zoom = getZoomFactor();
  document.body.classList.add('is-resizing');

  function onMouseMove(e: MouseEvent) {
    let newWidth = startWidth + ((e.clientX - startX) / zoom);
    if (newWidth < 60) newWidth = 60;
    if (newWidth > 600) newWidth = 600;
    labelsColWidth.value = Math.round(newWidth);
  }

  function onMouseUp() {
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
    document.body.classList.remove('is-resizing');
    window.getSelection()?.removeAllRanges();
  }

  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
}

function startMsgDrag(e: MouseEvent) {
  e?.preventDefault?.();
  e?.stopPropagation?.();
  window.getSelection()?.removeAllRanges();
  const startX = e.clientX;
  const startWidth = msgColWidth.value;
  const zoom = getZoomFactor();
  document.body.classList.add('is-resizing');

  function onMouseMove(e: MouseEvent) {
    let newWidth = startWidth + ((e.clientX - startX) / zoom);
    if (newWidth < 150) newWidth = 150;
    if (newWidth > 1200) newWidth = 1200;
    msgColWidth.value = Math.round(newWidth);
  }

  function onMouseUp() {
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
    document.body.classList.remove('is-resizing');
    window.getSelection()?.removeAllRanges();
  }

  document.addEventListener('mousemove', onMouseMove);
  document.addEventListener('mouseup', onMouseUp);
}

const maxCol = computed(() => {
  let m = 0;
  for (const c of props.commits as any[]) {
    if (c.col > m) m = c.col;
  }
  return m;
});

const spacerWidth = computed(() => Math.max(60, 20 + (maxCol.value + 2.5) * COL_WIDTH));

// Calculate SVG paths (GitKraken orthogonal style)
const paths = computed(() => {
  const result: Array<{ d: string; color: string; opacity: number; zIndex: number; isMain: boolean }> = [];
  
  (props.commits as any[]).forEach((commit, index) => {
    const nodeX = LEFT_MARGIN + (commit.col * COL_WIDTH);
    const nodeY = START_Y + (index * ROW_HEIGHT);

    commit.parents.forEach((parentId: string, parentIdx: number) => {
      const parentIndex = props.commits.findIndex((c: any) => c.id === parentId);
      if (parentIndex !== -1) {
        const parentCommit = props.commits[parentIndex] as any;
        const endX = LEFT_MARGIN + (parentCommit.col * COL_WIDTH);
        const endY = START_Y + (parentIndex * ROW_HEIGHT);
        
        let d = '';
        if (nodeX === endX) {
          // Straight vertical
          d = `M ${nodeX} ${nodeY} L ${endX} ${endY}`;
        } else {
          // Orthogonal L-shape
          const r = 6;
          const dirX = nodeX < endX ? 1 : -1;
          const dirY = nodeY < endY ? 1 : -1;
          
          if (Math.abs(endY - nodeY) <= r) {
            d = `M ${nodeX} ${nodeY} L ${endX} ${endY}`;
          } else {
            d = `M ${nodeX} ${nodeY} 
                 L ${endX - (r * dirX)} ${nodeY}
                 Q ${endX} ${nodeY} ${endX} ${nodeY + (r * dirY)}
                 L ${endX} ${endY}`;
          }
        }
        
        const isMainPath = commit.isMain && parentCommit.isMain && nodeX === endX;

        result.push({
          d,
          color: parentIdx === 0 ? commit.color : parentCommit.color,
          opacity: 1,
          zIndex: 1,
          isMain: isMainPath
        });
      } else {
        // Parent not in current loaded commits, draw line to bottom
        const endY = props.commits.length * ROW_HEIGHT;
        result.push({
          d: `M ${nodeX} ${nodeY} L ${nodeX} ${endY}`,
          color: parentIdx === 0 ? commit.color : '#888',
          opacity: 1,
          zIndex: 1,
          isMain: commit.isMain && parentIdx === 0
        });
      }
    });
  });
  
  return result;
});
</script>

<style scoped>
.commit-type-label {
  display: inline-block;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  margin-right: 6px;
  text-transform: uppercase;
  vertical-align: middle;
}
.label-feat {
  background: rgba(76, 175, 80, 0.15);
  color: var(--success-text);
  border: 1px solid rgba(76, 175, 80, 0.3);
}
.label-fix {
  background: rgba(244, 67, 54, 0.15);
  color: var(--danger-text);
  border: 1px solid rgba(244, 67, 54, 0.3);
}
.label-chore {
  background: rgba(33, 150, 243, 0.15);
  color: var(--info-text);
  border: 1px solid rgba(33, 150, 243, 0.3);
}
.label-merge {
  background: rgba(156, 39, 176, 0.15);
  color: #9c27b0;
  border: 1px solid rgba(156, 39, 176, 0.3);
}
.label-default {
  background: rgba(158, 158, 158, 0.15);
  color: var(--text-muted);
  border: 1px solid rgba(158, 158, 158, 0.3);
}

.commit-sig-badge {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 0.7rem;
  font-weight: 600;
  margin-right: 6px;
  vertical-align: middle;
  text-transform: capitalize;
  user-select: none;
}

.commit-sig-badge.sig-verified {
  background: rgba(46, 160, 67, 0.15);
  color: #3fb950;
  border: 1px solid rgba(46, 160, 67, 0.35);
}

.commit-sig-badge.sig-unverified {
  background: rgba(210, 153, 34, 0.15);
  color: #d29922;
  border: 1px solid rgba(210, 153, 34, 0.35);
}

.commit-sig-badge.sig-bad,
.commit-sig-badge.sig-expired,
.commit-sig-badge.sig-revoked {
  background: rgba(248, 81, 73, 0.15);
  color: #f85149;
  border: 1px solid rgba(248, 81, 73, 0.35);
}

.sig-badge-icon {
  flex-shrink: 0;
}

/* Branch Drag and Drop styles */
.branch-label * {
  pointer-events: none;
}

.branch-label.is-draggable-branch {
  cursor: grab;
  user-select: none;
  transition: transform 0.12s ease, opacity 0.12s ease, box-shadow 0.12s ease;
}

.branch-label.is-draggable-branch:active {
  cursor: grabbing;
}

.branch-label.is-dragging,
.branch-label.is-dragging-source {
  opacity: 0.35;
  transform: scale(0.95);
  border-style: dashed !important;
}

.branch-label.is-drop-target {
  outline: 2px dashed var(--accent, #6366f1) !important;
  outline-offset: 2px !important;
  background-color: rgba(99, 102, 241, 0.3) !important;
  box-shadow: 0 0 12px rgba(99, 102, 241, 0.6) !important;
  transform: scale(1.08) !important;
  z-index: 10;
}

/* Floating drag preview */
.floating-branch-drag-preview {
  position: fixed;
  top: 0;
  left: 0;
  pointer-events: none;
  z-index: 99999;
  user-select: none;
  will-change: transform;
}

.preview-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 6px;
  font-size: 0.76rem;
  font-weight: 600;
  background-color: var(--panel-bg, #1e1e2e);
  color: var(--text-main, #ffffff);
  border: 1px solid var(--accent, #6366f1);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 0 12px rgba(99, 102, 241, 0.5);
  transition: all 0.15s ease;
}

.preview-badge.has-target {
  border-color: #10b981;
  background-color: var(--panel-bg, #1e1e2e);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 0 16px rgba(16, 185, 129, 0.6);
}

.preview-icon {
  color: var(--accent, #6366f1);
  flex-shrink: 0;
}

.preview-badge.has-target .preview-icon {
  color: #10b981;
}

.preview-source {
  color: var(--text-main);
}

.preview-arrow {
  color: var(--text-muted);
  font-weight: 700;
}

.preview-target {
  color: #10b981;
  font-weight: 700;
}
</style>
