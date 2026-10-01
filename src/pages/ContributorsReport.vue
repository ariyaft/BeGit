<script setup lang="ts">
import type { ContributorRange, ContributorReport } from '../types';
import ErrorIcon from '../assets/icons/error.svg?component';
import ActivityIcon from '../assets/icons/activity.svg?component';
import { computed, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';



const props = defineProps<{
  repositoryName: string;
  repositoryPath: string | null;
  active: boolean;
}>();

const report = ref<ContributorReport | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
const reportKey = ref<string | null>(null);
const visibleLimit = ref(50);
const range = ref<ContributorRange>('1w');

const rangeOptions: Array<{ value: ContributorRange; label: string }> = [
  { value: 'all', label: 'All' },
  { value: '1d', label: '1D' },
  { value: '1w', label: '1W' },
  { value: '1m', label: '1M' },
  { value: '3m', label: '3M' },
  { value: '6m', label: '6M' }
];

function formatNumber(value: number) {
  return new Intl.NumberFormat().format(value || 0);
}

function formatDate(value: string) {
  if (!value) return '—';
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString(undefined, {
    year: 'numeric', month: 'short', day: 'numeric'
  });
}

async function loadReport() {
  if (!props.active || !props.repositoryPath) return;
  visibleLimit.value = 50;
  error.value = null;
  const key = `${props.repositoryPath}:${range.value}`;
  if (reportKey.value === key && report.value?.contributors?.length) return;
  if (reportKey.value === key && loading.value) return;

  report.value = null;
  reportKey.value = key;
  loading.value = true;
  try {
    const nextReport = await invoke<ContributorReport>('get_contributor_report', { path: props.repositoryPath, range: range.value });
    if (props.repositoryPath && reportKey.value === key) report.value = nextReport;
  } catch (loadError) {
    if (reportKey.value === key) error.value = loadError instanceof Error ? loadError.message : String(loadError);
  } finally {
    if (reportKey.value === key) loading.value = false;
  }
}

function changeRange(nextRange: ContributorRange) {
  if (nextRange === range.value) return;
  range.value = nextRange;
  reportKey.value = null;
  loadReport();
}

function loadMore() {
  visibleLimit.value += 50;
}

watch(() => [props.repositoryPath, props.active] as const, loadReport, { immediate: true });

const activityPoints = computed(() => (report.value?.activity || []).slice(-60));
const maxActivity = computed(() => Math.max(1, ...activityPoints.value.map(point => point.commits)));
const activityLine = computed(() => {
  const points = activityPoints.value;
  if (!points.length) return '';
  return points.map((point, index) => {
    const x = points.length === 1 ? 300 : 20 + (index / (points.length - 1)) * 560;
    const y = 116 - (point.commits / maxActivity.value) * 88;
    return `${x.toFixed(1)},${y.toFixed(1)}`;
  }).join(' ');
});
const activityArea = computed(() => activityLine.value ? `20,116 ${activityLine.value} 580,116` : '');
const activityRange = computed(() => {
  const points = activityPoints.value;
  if (!points.length) return '';
  return points.length === 1 ? formatDate(points[0].date) : `${formatDate(points[0].date)} – ${formatDate(points[points.length - 1].date)}`;
});
const topContributors = computed(() => (report.value?.contributors || []).slice(0, 5));
const topCommitCount = computed(() => Math.max(1, ...topContributors.value.map(contributor => contributor.commits)));
const rangeDescription = computed(() => ({
  all: 'All reachable Git history',
  '1d': 'Last day',
  '1w': 'Last week',
  '1m': 'Last month',
  '3m': 'Last 3 months',
  '6m': 'Last 6 months'
})[range.value]);

function activityX(index: number) {
  const count = activityPoints.value.length;
  return count === 1 ? 300 : 20 + (index / (count - 1)) * 560;
}

function activityY(commits: number) {
  return 116 - (commits / maxActivity.value) * 88;
}

function contributorWidth(commits: number) {
  return `${Math.max(5, (commits / topCommitCount.value) * 100)}%`;
}
</script>

<template>
  <section class="contributors-report" aria-label="Contributors report">
    <header class="report-header">
      <div>
        <div class="report-eyebrow">Repository report</div>
        <h1>Contributors</h1>
        <p>{{ repositoryName }} · {{ rangeDescription }}</p>
      </div>
      <div class="range-control" aria-label="Contributor report range">
        <button
          v-for="option in rangeOptions"
          :key="option.value"
          type="button"
          :class="{ active: range === option.value }"
          :disabled="loading"
          @click="changeRange(option.value)"
        >{{ option.label }}</button>
      </div>
    </header>

    <div v-if="loading" class="report-state" role="status">
      <span class="spinner" aria-hidden="true"></span>
      Reading Git history and calculating activity…
    </div>

    <div v-else-if="error" class="report-state report-error" role="alert">
      <ErrorIcon viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="2" fill="none"  />
      <div><strong>Couldn’t create this report</strong><span>{{ error }}</span></div>
    </div>

    <div v-else-if="!report || report.contributors.length === 0" class="report-state">
      <ActivityIcon viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="1.6" fill="none"  />
      <div><strong>No Git contributions found</strong><span>This repository has no commits reachable from its current refs.</span></div>
    </div>

    <template v-else>
      <div class="report-summary">
        <article class="summary-card">
          <span>Contributors</span><strong>{{ formatNumber(report.contributors.length) }}</strong>
        </article>
        <article class="summary-card">
          <span>Commits</span><strong>{{ formatNumber(report.total_commits) }}</strong>
        </article>
        <article class="summary-card">
          <span>File changes</span><strong>{{ formatNumber(report.total_files_changed) }}</strong>
        </article>
        <article class="summary-card addition">
          <span>Lines added</span><strong>+{{ formatNumber(report.total_additions) }}</strong>
        </article>
        <article class="summary-card deletion">
          <span>Lines deleted</span><strong>−{{ formatNumber(report.total_deletions) }}</strong>
        </article>
      </div>

      <div class="report-charts">
        <article class="chart-card activity-chart">
          <div class="chart-heading">
            <div><span class="chart-eyebrow">Commit activity</span><strong>Recent active days</strong></div>
            <span>{{ activityRange }}</span>
          </div>
          <div v-if="activityPoints.length" class="activity-plot">
            <svg viewBox="0 0 600 132" preserveAspectRatio="none" role="img" aria-label="Commit activity over recent active days"  >
  <line x1="20" y1="28" x2="580" y2="28" class="chart-grid" /> <line x1="20" y1="72" x2="580" y2="72" class="chart-grid" /> <line x1="20" y1="116" x2="580" y2="116" class="chart-axis" /> <polygon :points="activityArea" class="chart-area" /> <polyline :points="activityLine" class="chart-line" /> <circle v-for="(point, index) in activityPoints" :key="point.date" :cx="activityX(index)" :cy="activityY(point.commits)" r="3" class="chart-point"> <title>{{ formatDate(point.date) }}: {{ formatNumber(point.commits) }} commits, +{{ formatNumber(point.additions) }}/−{{ formatNumber(point.deletions) }} lines</title> </circle>
</svg>
            <div class="chart-axis-labels"><span>{{ formatDate(activityPoints[0].date) }}</span><span>{{ maxActivity }} max commits/day</span><span>{{ formatDate(activityPoints[activityPoints.length - 1].date) }}</span></div>
          </div>
          <div v-else class="chart-empty">No dated contribution activity is available.</div>
        </article>

        <article class="chart-card contributors-chart">
          <div class="chart-heading">
            <div><span class="chart-eyebrow">Contribution mix</span><strong>Top contributors</strong></div>
            <span>By commits</span>
          </div>
          <div class="contributor-bars">
            <div v-for="contributor in topContributors" :key="contributor.email" class="contributor-bar-row">
              <span class="contributor-bar-name" :title="contributor.name || contributor.email">{{ contributor.name || contributor.email }}</span>
              <div class="contributor-bar-track"><span class="contributor-bar-fill" :style="{ width: contributorWidth(contributor.commits) }"></span></div>
              <strong>{{ formatNumber(contributor.commits) }}</strong>
            </div>
          </div>
        </article>
      </div>

      <div class="report-table-wrap">
        <table>
          <thead>
            <tr>
              <th>Contributor</th><th>Commits</th><th>Files changed</th><th>Lines</th><th>Active days</th><th>First contribution</th><th>Latest contribution</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="contributor in report.contributors.slice(0, Math.max(50, visibleLimit || 50))" :key="contributor.email">
              <td class="contributor-cell"><strong>{{ contributor.name || contributor.email }}</strong><span>{{ contributor.email }}</span></td>
              <td>{{ formatNumber(contributor.commits) }}</td>
              <td>{{ formatNumber(contributor.files_changed) }}</td>
              <td class="line-totals"><span class="added">+{{ formatNumber(contributor.additions) }}</span><span class="deleted">−{{ formatNumber(contributor.deletions) }}</span></td>
              <td>{{ formatNumber(contributor.active_days) }}</td>
              <td>{{ formatDate(contributor.first_commit_at) }}</td>
              <td>{{ formatDate(contributor.last_commit_at) }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <div v-if="report.contributors.length > 50" class="report-pagination" aria-live="polite">
        <span>Showing {{ formatNumber(Math.min(report.contributors.length, Math.max(50, visibleLimit || 50))) }} of {{ formatNumber(report.contributors.length) }} contributors</span>
        <button v-if="report.contributors.length > Math.max(50, visibleLimit || 50)" type="button" @click="loadMore">Show 50 more</button>
      </div>

      <p class="report-note">
        This report summarizes Git activity, not performance. Merges, rebases, generated files, bots, and multiple Git identities can affect totals.
      </p>
    </template>
  </section>
</template>

<style scoped>
.contributors-report { flex: 1; min-width: 0; overflow: auto; padding: 28px 32px 36px; background: var(--bg-color); color: var(--text-main); }
.report-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 20px; margin-bottom: 28px; }
.report-eyebrow { color: var(--accent); font-size: .76rem; font-weight: 700; letter-spacing: .08em; text-transform: uppercase; margin-bottom: 5px; }
h1 { font-size: 1.8rem; line-height: 1.15; letter-spacing: -.025em; }
.report-header p { margin-top: 7px; color: var(--text-muted); font-size: .9rem; }
.range-control { display: inline-flex; align-items: center; overflow: hidden; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-inset); }.range-control button { min-width: 39px; border: 0; border-right: 1px solid var(--border); padding: 7px 9px; background: transparent; color: var(--text-muted); cursor: pointer; font: inherit; font-size: .74rem; font-weight: 650; }.range-control button:last-child { border-right: 0; }.range-control button:hover:not(:disabled) { background: var(--surface-hover); color: var(--text-main); }.range-control button.active { background: var(--accent-blue); color: white; }.range-control button:disabled { cursor: wait; opacity: .7; }
.report-summary { display: grid; grid-template-columns: repeat(5, minmax(120px, 1fr)); gap: 12px; margin-bottom: 24px; }
.summary-card { min-height: 92px; padding: 16px; background: var(--panel-bg); border: 1px solid var(--border); border-radius: 7px; display: flex; flex-direction: column; justify-content: space-between; gap: 8px; }
.summary-card span { color: var(--text-muted); font-size: .78rem; }
.summary-card strong { font-size: 1.35rem; font-variant-numeric: tabular-nums; }
.summary-card.addition strong, .added { color: var(--success-text); }
.summary-card.deletion strong, .deleted { color: var(--danger-text); }
.report-charts { display: grid; grid-template-columns: minmax(0, 1.5fr) minmax(280px, 1fr); gap: 12px; margin-bottom: 24px; }.chart-card { min-width: 0; min-height: 190px; padding: 16px; border: 1px solid var(--border); border-radius: 7px; background: var(--panel-bg); }.chart-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 12px; color: var(--text-muted); font-size: .72rem; text-align: right; }.chart-heading strong, .chart-eyebrow { display: block; text-align: left; }.chart-heading strong { margin-top: 3px; color: var(--text-main); font-size: .92rem; }.chart-eyebrow { color: var(--accent-blue); font-size: .67rem; font-weight: 700; letter-spacing: .08em; text-transform: uppercase; }.activity-plot svg { display: block; width: 100%; height: 128px; overflow: visible; }.chart-grid { stroke: var(--border); stroke-dasharray: 3 5; opacity: .7; }.chart-axis { stroke: var(--text-muted); opacity: .45; }.chart-area { fill: var(--accent-blue); opacity: .12; }.chart-line { fill: none; stroke: var(--accent-blue); stroke-width: 2.5; stroke-linecap: round; stroke-linejoin: round; }.chart-point { fill: var(--panel-bg); stroke: var(--accent-blue); stroke-width: 2; }.chart-axis-labels { display: flex; justify-content: space-between; gap: 8px; color: var(--text-muted); font-size: .67rem; }.chart-empty { min-height: 126px; display: flex; align-items: center; color: var(--text-muted); font-size: .78rem; }.contributor-bars { display: grid; gap: 11px; padding-top: 5px; }.contributor-bar-row { display: grid; grid-template-columns: minmax(72px, 1fr) minmax(76px, 1.35fr) auto; align-items: center; gap: 9px; font-size: .74rem; }.contributor-bar-name { overflow: hidden; color: var(--text-main); text-overflow: ellipsis; white-space: nowrap; }.contributor-bar-track { height: 7px; overflow: hidden; border-radius: 999px; background: var(--surface-inset); }.contributor-bar-fill { display: block; height: 100%; border-radius: inherit; background: linear-gradient(90deg, var(--accent-blue), var(--purple-text)); }.contributor-bar-row strong { color: var(--text-muted); font-size: .72rem; font-variant-numeric: tabular-nums; }
.report-table-wrap { border: 1px solid var(--border); border-radius: 7px; overflow-x: auto; background: var(--panel-bg); }
table { width: 100%; min-width: 880px; border-collapse: collapse; font-size: .82rem; }
th { padding: 11px 14px; color: var(--text-muted); background: var(--surface-subtle); border-bottom: 1px solid var(--border); font-size: .72rem; font-weight: 650; letter-spacing: .03em; text-align: left; text-transform: uppercase; white-space: nowrap; }
td { padding: 12px 14px; border-bottom: 1px solid var(--divider); color: var(--text-main); font-variant-numeric: tabular-nums; white-space: nowrap; }
tbody tr:last-child td { border-bottom: 0; }
tbody tr:hover { background: var(--surface-subtle); }
.contributor-cell { min-width: 210px; white-space: normal; }
.contributor-cell strong, .contributor-cell span { display: block; overflow: hidden; text-overflow: ellipsis; }
.contributor-cell span { margin-top: 3px; color: var(--text-muted); font-size: .76rem; }
.line-totals { display: flex; gap: 9px; }
.report-pagination { display: flex; align-items: center; justify-content: space-between; gap: 14px; padding: 12px 2px 0; color: var(--text-muted); font-size: .78rem; }
.report-pagination button { border: 1px solid var(--border); border-radius: 5px; padding: 6px 10px; background: var(--panel-bg); color: var(--text-main); cursor: pointer; font: inherit; font-size: .78rem; }
.report-pagination button:hover { background: var(--surface-hover); }
.report-note { margin: 16px 2px 0; max-width: 830px; color: var(--text-muted); font-size: .78rem; line-height: 1.55; }
.report-state { min-height: 290px; border: 1px dashed var(--border); border-radius: 8px; color: var(--text-muted); display: flex; align-items: center; justify-content: center; gap: 12px; text-align: center; padding: 32px; }
.report-state div { display: grid; gap: 5px; text-align: left; }.report-state strong { color: var(--text-main); }.report-state span { font-size: .85rem; }.report-error { color: var(--danger-text); border-color: color-mix(in srgb, var(--danger-text) 50%, var(--border)); }.report-error strong { color: var(--danger-text); }
.spinner { width: 17px; height: 17px; border: 2px solid var(--border); border-top-color: var(--accent); border-radius: 50%; animation: spin .8s linear infinite; }
@keyframes spin { to { transform: rotate(360deg); } }
@media (max-width: 920px) { .report-summary { grid-template-columns: repeat(3, minmax(120px, 1fr)); }.report-charts { grid-template-columns: 1fr; } }
@media (max-width: 560px) { .contributors-report { padding: 20px 16px; }.report-header { flex-direction: column; }.report-summary { grid-template-columns: repeat(2, minmax(120px, 1fr)); } }
</style>
