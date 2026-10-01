<template>
  <section class="timeline-shell">
    <header class="timeline-header">
      <div>
        <div class="timeline-eyebrow">Repository map</div>
        <h2>Branch timeline</h2>
      </div>
      <div class="timeline-summary" aria-label="Timeline summary">
        <span>{{ lanes.length }} {{ lanes.length === 1 ? 'lane' : 'lanes' }}</span>
        <span class="summary-divider"></span>
        <span>{{ commits.length }} {{ commits.length === 1 ? 'commit' : 'commits' }}</span>
        <span class="direction-hint">oldest <span aria-hidden="true">→</span> newest</span>
        
        <div class="range-control" aria-label="Commit limit">
          <button
            v-for="limitOption in [100, 250, 500, 1000]"
            :key="limitOption"
            type="button"
            :class="{ active: commitLimit === limitOption }"
            @click="emit('change-limit', limitOption)"
          >{{ limitOption }}</button>
        </div>
      </div>
    </header>

    <div v-if="commits.length === 0" class="timeline-empty">
      <EmptyIcon viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="1.5"  />
      <span>No commits to map yet</span>
    </div>

    <div v-else ref="scrollArea" class="timeline-scroll">
      <div
        class="timeline-stage"
        :style="{ width: `${stageWidth}px`, height: `${svgHeight}px` }"
      >
        <aside class="lane-rail" :style="{ height: `${svgHeight}px` }">
          <div class="rail-title">Branches</div>
          <div
            v-for="lane in lanes"
            :key="lane.index"
            class="lane-label"
            :class="{ 'head-lane': lane.isHead }"
            :style="{ top: `${lane.y - LABEL_HEIGHT / 2}px`, '--lane-color': lane.color }"
            :title="lane.name"
          >
            <span class="lane-accent"></span>
            <span class="lane-name">{{ lane.name }}</span>
            <span class="lane-icons">
              <StarFillIcon v-if="lane.isHead" class="head-star" viewBox="0 0 24 24" width="13" height="13" aria-label="Current branch"  />
              <LaptopBranchIcon v-if="lane.isLocal" viewBox="0 0 24 24" width="13" height="13" aria-label="Local branch"  />
              <CloudBranchIcon v-if="lane.isRemote" viewBox="0 0 24 24" width="13" height="13" aria-label="Remote branch"  />
            </span>
          </div>
        </aside>

        <svg
          class="timeline-svg"
          :width="canvasWidth"
          :height="svgHeight"
          :viewBox="`0 0 ${canvasWidth} ${svgHeight}`"
          role="img"
          aria-label="Branches arranged vertically and commits arranged from oldest to newest"
          >
  <defs>
    <marker id="timeline-arrow" viewBox="0 0 10 10" refX="8.5" refY="5" markerWidth="6" markerHeight="6" orient="auto" markerUnits="strokeWidth">
      <path d="M 0 1.5 L 8.5 5 L 0 8.5 z" class="arrow-head" />
    </marker>
    <filter id="node-glow" x="-75%" y="-75%" width="250%" height="250%">
      <feGaussianBlur stdDeviation="5" result="blur" />
      <feComponentTransfer>
        <feFuncA type="linear" slope="0.6"/>
      </feComponentTransfer>
      <feMerge>
        <feMergeNode />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
    <filter id="lane-glow" x="-20%" y="-20%" width="140%" height="140%">
      <feGaussianBlur stdDeviation="3" result="blur" />
      <feMerge>
        <feMergeNode in="blur" />
        <feMergeNode in="SourceGraphic" />
      </feMerge>
    </filter>
    <linearGradient id="header-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
      <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.1" />
      <stop offset="100%" stop-color="transparent" stop-opacity="0" />
    </linearGradient>
  </defs>

  <g class="date-markers" aria-hidden="true">
    <g v-for="(marker, idx) in dateMarkers" :key="`date-${idx}`">
      <line :x1="marker.x" y1="20" :x2="marker.x" :y2="svgHeight" stroke="var(--border)" stroke-dasharray="4 4" stroke-width="1" />
      <text :x="marker.x + 8" y="32" fill="var(--text-muted)" font-size="11" font-weight="600">{{ marker.label }}</text>
    </g>
  </g>

  <g class="lane-guides" aria-hidden="true">
    <line v-for="lane in lanes" :key="`guide-${lane.index}`" x1="0" :x2="canvasWidth" :y1="lane.y" :y2="lane.y" :style="{ '--lane-color': lane.color }" />
  </g>
  
  <g class="connections" aria-hidden="true">
    <path v-for="connection in connections" :key="connection.id" :d="connection.path" class="commit-connection" :class="{ 'merge-connection': connection.isMerge }" marker-end="url(#timeline-arrow)" />
  </g>
  
  <g v-for="commit in layoutCommits" :key="commit.id" class="commit-group" :class="{ selected: selectedCommit?.id === commit.id, 'head-commit': commit.isHead }" role="button" tabindex="0" :aria-label="`${commit.hash}: ${commit.message}`" @click="emit('select-commit', commit, $event)" @keydown.enter.prevent="emit('select-commit', commit)" @keydown.space.prevent="emit('select-commit', commit)">
    <title>{{ commit.message }} — {{ commit.author }} — {{ commit.exact_time }}</title>
    <g v-if="commit.refLabel" class="ref-pill">
      <rect :x="commit.x - commit.refWidth / 2" :y="commit.y - 48" :width="commit.refWidth" height="22" rx="11" :style="{ '--node-color': commit.color }" />
      <text :x="commit.x" :y="commit.y - 33">{{ commit.refLabel }}</text>
    </g>
    <circle v-if="selectedCommit?.id === commit.id" class="selection-ring" :cx="commit.x" :cy="commit.y" :r="NODE_RADIUS + 9" />
    <circle class="commit-node" :class="{ 'node-glow': commit.isHead }" :cx="commit.x" :cy="commit.y" :r="NODE_RADIUS" :style="{ '--node-color': commit.color }" />
    <path v-if="commit.isHead" class="node-star" :transform="`translate(${commit.x - 8} ${commit.y - 8}) scale(.67)`" d="m12 2.5 2.8 5.68 6.27.91-4.54 4.43 1.07 6.25L12 16.82l-5.6 2.95 1.07-6.25-4.54-4.43 6.27-.91L12 2.5Z" />
    <circle v-else-if="commit.parents.length > 1" :cx="commit.x" :cy="commit.y" r="6" class="merge-dot" />
    <text class="commit-message" :x="commit.x" :y="commit.y + 44">{{ commitLabel(commit.message) }}</text>
  </g>
</svg>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import EmptyIcon from '../../assets/icons/empty.svg?component';
import StarFillIcon from '../../assets/icons/star-fill.svg?component';
import LaptopBranchIcon from '../../assets/icons/laptop-branch.svg?component';
import CloudBranchIcon from '../../assets/icons/cloud-branch.svg?component';
import { computed, nextTick, onMounted, ref, watch, type PropType } from 'vue';

interface TimelineCommit {
  id: string;
  hash: string;
  parents: string[];
  refs: string[];
  labels?: Array<{ text: string; isHeadBranch?: boolean; isLocal?: boolean; isRemote?: boolean; isTag?: boolean }>;
  col: number;
  color?: string;
  message: string;
  author: string;
  exact_time?: string;
  isHead?: boolean;
  [key: string]: unknown;
}

interface Lane {
  index: number;
  name: string;
  color: string;
  y: number;
  isHead: boolean;
  isLocal: boolean;
  isRemote: boolean;
}

interface LayoutCommit extends TimelineCommit {
  x: number;
  y: number;
  color: string;
  refLabel: string;
  refWidth: number;
}

const props = defineProps({
  commits: {
    type: Array as PropType<TimelineCommit[]>,
    required: true
  },
  selectedCommit: {
    type: Object as PropType<TimelineCommit | null>,
    default: null
  },
  commitLimit: {
    type: Number,
    default: 500
  }
});

const emit = defineEmits<{
  (event: 'select-commit', commit: TimelineCommit, mouseEvent?: MouseEvent): void;
  (event: 'change-limit', limit: number): void;
}>();

const scrollArea = ref<HTMLElement | null>(null);
const hasPositionedInitialView = ref(false);

const LABEL_RAIL_WIDTH = 210;
const LABEL_HEIGHT = 46;
const NODE_RADIUS = 19;
const SPACING_X = 112;
const SPACING_Y = 112;
const START_X = 58;
const START_Y = 84;
const END_PADDING = 72;
const LANE_COLORS = ['#55c2ff', '#ff8a4c', '#36d399', '#a78bfa', '#f6c453', '#fb7185', '#22d3ee', '#c084fc'];

function laneIndex(commit: TimelineCommit): number {
  return Number.isFinite(commit.col) ? Math.max(0, commit.col) : 0;
}

function cleanRef(refName: string): string {
  return refName.replace(/^HEAD -> /, '').replace(/^tag: /, '').trim();
}

function commitLabel(message: string): string {
  const subject = (message || 'Untitled commit').replace(/\s+/g, ' ').trim();
  return subject.length > 16 ? `${subject.slice(0, 15)}…` : subject;
}

const lanes = computed<Lane[]>(() => {
  const maxLane = props.commits.reduce((max, commit) => Math.max(max, laneIndex(commit)), 0);
  const result: Lane[] = [];

  for (let index = 0; index <= maxLane; index += 1) {
    const laneCommits = props.commits.filter(commit => laneIndex(commit) === index);
    const labels = laneCommits.flatMap(commit => commit.labels ?? []);
    const headLabel = labels.find(label => label.isHeadBranch);
    const localLabel = labels.find(label => label.isLocal && !label.isTag);
    const remoteLabel = labels.find(label => label.isRemote && !label.isTag);
    const fallbackRef = laneCommits
      .flatMap(commit => commit.refs ?? [])
      .find(refName => !refName.startsWith('tag: ') && !refName.includes('/HEAD'));

    const preferred = headLabel ?? localLabel ?? remoteLabel;
    const name = preferred?.text ?? (fallbackRef ? cleanRef(fallbackRef).replace(/^origin\//, '') : `branch ${index + 1}`);
    const matchingLabels = labels.filter(label => label.text === name);

    result.push({
      index,
      name,
      color: LANE_COLORS[index % LANE_COLORS.length],
      y: START_Y + index * SPACING_Y,
      isHead: matchingLabels.some(label => label.isHeadBranch) || Boolean(headLabel && headLabel.text === name),
      isLocal: matchingLabels.some(label => label.isLocal) || Boolean(localLabel && localLabel.text === name),
      isRemote: matchingLabels.some(label => label.isRemote) || Boolean(remoteLabel && remoteLabel.text === name)
    });
  }

  return result;
});

const layoutCommits = computed<LayoutCommit[]>(() => {
  return [...props.commits].reverse().map((commit, index) => {
    const lane = lanes.value[laneIndex(commit)];
    const tag = (commit.refs ?? []).find(refName => refName.startsWith('tag: '));
    const branchLabel = commit.isHead
      ? (commit.labels ?? []).find(label => label.isHeadBranch)?.text
      : undefined;
    const refLabel = tag ? cleanRef(tag) : (branchLabel ?? '');

    return {
      ...commit,
      x: START_X + index * SPACING_X,
      y: lane?.y ?? START_Y,
      color: lane?.color ?? LANE_COLORS[0],
      refLabel,
      refWidth: Math.max(48, Math.min(132, refLabel.length * 7 + 22))
    };
  });
});

const dateMarkers = computed(() => {
  const markers = [];
  let lastDateStr = '';

  for (const commit of layoutCommits.value) {
    if (!commit.exact_time) continue;
    
    const d = new Date(commit.exact_time);
    if (isNaN(d.getTime())) continue;

    const dateStr = d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
    
    if (dateStr !== lastDateStr) {
      let markerX = commit.x - SPACING_X / 2;
      
      markers.push({
        x: markerX,
        label: dateStr
      });
      lastDateStr = dateStr;
    }
  }
  return markers;
});

const connections = computed(() => {
  const byId = new Map(layoutCommits.value.map(commit => [commit.id, commit]));
  const result: Array<{ id: string; path: string; isMerge: boolean }> = [];

  for (const child of layoutCommits.value) {
    child.parents.forEach((parentId, parentIndex) => {
      const parent = byId.get(parentId);
      if (!parent) return;

      const startX = parent.x + NODE_RADIUS + 3;
      const endX = child.x - NODE_RADIUS - 8;
      const distance = Math.max(26, endX - startX);
      const controlOffset = Math.min(58, Math.max(24, distance * 0.45));
      const path = parent.y === child.y
        ? `M ${startX} ${parent.y} L ${endX} ${child.y}`
        : `M ${startX} ${parent.y} C ${startX + controlOffset} ${parent.y}, ${endX - controlOffset} ${child.y}, ${endX} ${child.y}`;

      result.push({
        id: `${parent.id}-${child.id}-${parentIndex}`,
        path,
        isMerge: child.parents.length > 1
      });
    });
  }

  return result;
});

const canvasWidth = computed(() => Math.max(460, START_X + Math.max(0, props.commits.length - 1) * SPACING_X + END_PADDING));
const stageWidth = computed(() => LABEL_RAIL_WIDTH + canvasWidth.value);
const svgHeight = computed(() => Math.max(360, START_Y + Math.max(0, lanes.value.length - 1) * SPACING_Y + 86));

async function positionAtNewestCommit() {
  if (hasPositionedInitialView.value || props.commits.length === 0) return;
  await nextTick();
  if (!scrollArea.value) return;
  scrollArea.value.scrollLeft = scrollArea.value.scrollWidth;
  hasPositionedInitialView.value = true;
}

onMounted(positionAtNewestCommit);
watch(() => props.commits.length, positionAtNewestCommit);
</script>

<style scoped>
.timeline-shell {
  width: 100%;
  height: auto;
  flex: 1;
  min-height: 0;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  color: var(--text-main);
  background:
    radial-gradient(ellipse at top right, rgba(0, 188, 212, 0.08), transparent 45%),
    radial-gradient(ellipse at bottom left, rgba(156, 39, 176, 0.05), transparent 40%),
    var(--bg-color);
}

.timeline-header {
  height: 70px;
  flex: 0 0 70px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 0 28px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  background: color-mix(in srgb, var(--panel-bg) 60%, transparent);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
  z-index: 10;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.timeline-eyebrow {
  margin-bottom: 4px;
  color: var(--text-muted);
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.15em;
  text-transform: uppercase;
  background: linear-gradient(90deg, var(--accent, #00bcd4), #9c27b0);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.timeline-header h2 {
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.015em;
  margin: 0;
  color: var(--text-main);
}

.timeline-summary {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-muted);
  font-size: 13px;
  white-space: nowrap;
  font-weight: 500;
}

.summary-divider {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: var(--border);
}

.direction-hint {
  margin-left: 10px;
  padding: 4px 12px;
  border: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
  border-radius: 999px;
  color: var(--text-main);
  background: color-mix(in srgb, var(--bg-color) 40%, transparent);
  font-size: 11px;
  font-weight: 600;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.timeline-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  scrollbar-color: var(--border) transparent;
  scrollbar-width: thin;
}

.timeline-scroll::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}
.timeline-scroll::-webkit-scrollbar-thumb {
  background-color: var(--border);
  border-radius: 4px;
}

.timeline-stage {
  min-width: 100%;
  display: grid;
  grid-template-columns: 210px auto;
  align-items: start;
  position: relative;
}

.lane-rail {
  position: sticky;
  left: 0;
  z-index: 5;
  grid-column: 1;
  overflow: hidden;
  border-right: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  background: color-mix(in srgb, var(--panel-bg) 60%, transparent);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  box-shadow: 10px 0 30px rgba(0, 0, 0, 0.2);
}

.rail-title {
  position: absolute;
  top: 24px;
  left: 24px;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.15em;
  text-transform: uppercase;
}

.lane-label {
  position: absolute;
  left: 20px;
  width: 170px;
  height: 46px;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 12px 0 0;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--lane-color) 30%, transparent);
  border-radius: 12px;
  color: var(--text-main);
  background: color-mix(in srgb, var(--lane-color) 10%, rgba(20,20,25,0.7));
  backdrop-filter: blur(10px);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15), inset 0 1px 1px rgba(255,255,255,0.05);
  transition: transform 0.2s ease, box-shadow 0.2s ease, background 0.2s ease;
}

.lane-label:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.25), inset 0 1px 1px rgba(255,255,255,0.1);
  background: color-mix(in srgb, var(--lane-color) 15%, rgba(20,20,25,0.8));
}

.lane-label.head-lane {
  border-color: color-mix(in srgb, #fbbc04 50%, var(--lane-color));
  box-shadow: 0 0 15px rgba(251, 188, 4, 0.15), 0 4px 16px rgba(0, 0, 0, 0.2);
}

.lane-accent {
  width: 6px;
  height: 100%;
  flex: 0 0 6px;
  background: linear-gradient(180deg, var(--lane-color), color-mix(in srgb, var(--lane-color) 60%, black));
}

.lane-name {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  font-size: 13px;
  font-weight: 700;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.lane-icons {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 6px;
  color: color-mix(in srgb, var(--lane-color) 85%, white);
}

.lane-icons svg:not(.head-star) {
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  filter: drop-shadow(0 0 2px rgba(255,255,255,0.2));
}

.head-star {
  fill: #fbbc04;
  filter: drop-shadow(0 0 4px rgba(251, 188, 4, 0.6));
}

.timeline-svg {
  grid-column: 2;
  display: block;
  overflow: visible;
}

.lane-guides line {
  stroke: var(--lane-color);
  stroke-width: 1.5;
  stroke-dasharray: 4 12;
  opacity: 0.1;
}

.commit-connection {
  fill: none;
  stroke: color-mix(in srgb, var(--text-muted) 50%, #475569);
  stroke-width: 3.5;
  stroke-linecap: round;
  opacity: 0.85;
  transition: opacity 0.3s ease, stroke-width 0.3s ease, stroke 0.3s ease;
}

.commit-group:hover ~ .connections .commit-connection, /* Requires reordering SVG or handling via JS for real hover paths, kept for CSS validity */
.commit-connection:hover {
  stroke: color-mix(in srgb, var(--accent) 70%, white);
  opacity: 1;
}

.merge-connection {
  stroke-width: 4;
  opacity: 0.95;
}

.arrow-head {
  fill: color-mix(in srgb, var(--text-muted) 50%, #475569);
}

.commit-group {
  cursor: pointer;
  outline: none;
}

.commit-node {
  fill: color-mix(in srgb, var(--node-color) 15%, var(--bg-color));
  stroke: var(--node-color);
  stroke-width: 4;
  transition: all 0.2s ease;
}

.commit-group:hover .commit-node,
.commit-group:focus-visible .commit-node {
  stroke-width: 6;
  fill: var(--node-color);
  filter: url(#node-glow);
}

.node-glow {
  filter: url(#node-glow);
  fill: var(--node-color);
  animation: pulse-glow 2.5s infinite alternate;
}

@keyframes pulse-glow {
  0% { filter: drop-shadow(0 0 4px var(--node-color)); }
  100% { filter: drop-shadow(0 0 12px var(--node-color)); }
}

.selection-ring {
  fill: none;
  stroke: color-mix(in srgb, var(--node-color, var(--accent)) 60%, white);
  stroke-width: 2.5;
  stroke-dasharray: 4 4;
  opacity: 0.9;
  animation: spin-ring 12s linear infinite;
  transform-origin: center;
}

@keyframes spin-ring {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.node-star {
  fill: #fbbc04;
  stroke: #fff;
  stroke-width: 1;
  pointer-events: none;
  filter: drop-shadow(0 2px 4px rgba(0,0,0,0.4));
}

.merge-dot {
  fill: var(--node-color);
  stroke: var(--bg-color);
  stroke-width: 2;
  pointer-events: none;
}

.commit-message {
  fill: var(--text-main);
  font-size: 11px;
  font-weight: 600;
  text-anchor: middle;
  pointer-events: none;
  text-shadow: 0 1px 4px var(--bg-color), 0 1px 8px var(--bg-color);
  opacity: 0.85;
}

.commit-group:hover .commit-message {
  opacity: 1;
  font-weight: 700;
}

.ref-pill rect {
  fill: color-mix(in srgb, var(--node-color) 20%, rgba(20,20,25,0.9));
  stroke: color-mix(in srgb, var(--node-color) 50%, transparent);
  stroke-width: 1.5;
  rx: 12px;
  filter: drop-shadow(0 4px 8px rgba(0,0,0,0.3));
}

.ref-pill text {
  fill: #fff;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.02em;
  text-anchor: middle;
  pointer-events: none;
}

.timeline-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  color: var(--text-muted);
  font-size: 15px;
  font-weight: 600;
}

@media (prefers-reduced-motion: reduce) {
  .commit-node,
  .commit-connection,
  .selection-ring,
  .node-glow {
    transition: none;
    animation: none;
  }
}

.range-control {
  display: inline-flex;
  align-items: center;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--border) 40%, transparent);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-color) 50%, rgba(0,0,0,0.2));
  margin-left: 16px;
  box-shadow: inset 0 1px 3px rgba(0,0,0,0.1);
}
.range-control button {
  min-width: 44px;
  border: 0;
  border-right: 1px solid color-mix(in srgb, var(--border) 40%, transparent);
  padding: 6px 12px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  font: inherit;
  font-size: 11px;
  font-weight: 700;
  transition: all 0.2s ease;
}
.range-control button:last-child { border-right: 0; }
.range-control button:hover:not(:disabled) { 
  background: color-mix(in srgb, var(--accent) 15%, transparent); 
  color: var(--text-main); 
}
.range-control button.active { 
  background: var(--accent); 
  color: white; 
  box-shadow: 0 0 10px color-mix(in srgb, var(--accent) 40%, transparent);
}
.range-control button:disabled { cursor: wait; opacity: 0.5; }

</style>
