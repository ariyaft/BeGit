<template>
  <div class="git-blame-view" @click="closePopover">
    <!-- Blame Stats & Search Toolbar -->
    <header class="blame-toolbar">
      <div class="blame-stats-section">
        <div class="stat-badge" title="Total Lines">
          <span class="stat-value">{{ lines.length }}</span>
          <span class="stat-label">lines</span>
        </div>
        <div class="stat-divider"></div>
        <div class="stat-badge" title="Unique Authors">
          <span class="stat-value">{{ authorBreakdown.length }}</span>
          <span class="stat-label">authors</span>
        </div>
        <div class="stat-divider" v-if="timeSpanText"></div>
        <div class="stat-timespan" v-if="timeSpanText" :title="`Oldest: ${timeSpanText.oldest} • Newest: ${timeSpanText.newest}`">
          <span class="timespan-label">Activity:</span>
          <span class="timespan-val">{{ timeSpanText.summary }}</span>
        </div>

        <!-- Authors Distribution Bar -->
        <div class="authors-bar-wrapper" v-if="authorBreakdown.length > 0">
          <div class="authors-bar">
            <div
              v-for="author in authorBreakdown"
              :key="author.name"
              class="author-bar-segment"
              :style="{ width: author.percent + '%', backgroundColor: author.color }"
              :title="`${author.name}: ${author.count} lines (${author.percent}%)`"
            ></div>
          </div>
          <div class="author-top-chips">
            <span
              v-for="author in authorBreakdown.slice(0, 3)"
              :key="author.name"
              class="author-chip"
              :title="`${author.name}: ${author.count} lines (${author.percent}%)`"
            >
              <span class="author-chip-dot" :style="{ backgroundColor: author.color }"></span>
              <span class="author-chip-name">{{ author.name }}</span>
              <span class="author-chip-pct">{{ author.percent }}%</span>
            </span>
            <span v-if="authorBreakdown.length > 3" class="author-chip-more">
              +{{ authorBreakdown.length - 3 }} more
            </span>
          </div>
        </div>
      </div>

      <!-- Search and View Options -->
      <div class="blame-controls-section">
        <!-- Detected Language Badge -->
        <div class="lang-badge" :title="`Detected Language: ${languageInfo.name}`">
          <svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" class="lang-icon">
            <polyline points="16 18 22 12 16 6"></polyline>
            <polyline points="8 6 2 12 8 18"></polyline>
          </svg>
          <span class="lang-name">{{ languageInfo.name }}</span>
        </div>

        <!-- Blame search input -->
        <div class="blame-search-box">
          <SearchIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search code, author, commit..."
            class="blame-search-input"
          />
          <button v-if="searchQuery" class="clear-search-btn" @click="searchQuery = ''" title="Clear search">✕</button>
          <span v-if="searchQuery" class="search-match-count">
            {{ matchingLineNumbers.size }} / {{ lines.length }}
          </span>
        </div>

        <!-- Group / Line mode toggle -->
        <div class="view-mode-toggle" title="Toggle contiguous commit grouping">
          <button
            class="mode-btn"
            :class="{ active: groupByBlock }"
            @click="groupByBlock = true"
            title="Group contiguous lines from the same commit into blocks"
          >
            Grouped
          </button>
          <button
            class="mode-btn"
            :class="{ active: !groupByBlock }"
            @click="groupByBlock = false"
            title="Show full commit details on every single line"
          >
            Line-by-Line
          </button>
        </div>

        <!-- Beautifier / Formatter Controls -->
        <div class="view-mode-toggle" title="Code beautifier options">
          <button
            class="mode-btn"
            :class="{ active: enableSyntaxHighlighting }"
            @click="toggleSyntaxHighlighting"
            title="Toggle Syntax Highlighting"
          >
            <span class="btn-icon">⚡</span>
            Syntax
          </button>
          <button
            class="mode-btn"
            :class="{ active: showIndentGuides }"
            @click="toggleIndentGuides"
            title="Toggle Indentation Guides"
          >
            <span class="btn-icon">┆</span>
            Indent
          </button>
          <button
            class="mode-btn"
            :class="{ active: wrapLines }"
            @click="toggleWrapLines"
            title="Toggle Line Wrap"
          >
            <span class="btn-icon">↩</span>
            Wrap
          </button>
        </div>

        <!-- Copy File / Refresh Buttons -->
        <button class="icon-tool-btn" @click="copyFileContent" title="Copy entire file content to clipboard">
          <CopyIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        </button>
        <button class="icon-tool-btn" @click="$emit('refresh')" title="Refresh Git Blame">
          <FetchIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
        </button>
      </div>
    </header>

    <!-- Main Content Area -->
    <div class="blame-body-container">
      <!-- Loading State -->
      <div v-if="loading" class="blame-loading-state">
        <div class="spinner"></div>
        <p>Analyzing line-by-line Git authorship...</p>
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="blame-error-state">
        <div class="error-icon-box">⚠️</div>
        <h3>Unable to load Git Blame</h3>
        <p>{{ error }}</p>
        <button class="retry-btn" @click="$emit('refresh')">Try Again</button>
      </div>

      <!-- Empty State -->
      <div v-else-if="lines.length === 0" class="blame-empty-state">
        <p>No blame data available for this file.</p>
      </div>

      <!-- Code & Blame Table -->
      <div v-else class="blame-scroll-viewport">
        <div class="blame-table">
          <!-- Render by Blocks or Raw Lines -->
          <template v-if="groupByBlock">
            <div
              v-for="block in blameBlocks"
              :key="block.id"
              class="blame-block"
              :class="{
                'commit-highlighted': hoveredCommitHash === block.commit_hash,
                'block-matches-search': blockHasSearchMatch(block)
              }"
            >
              <div
                v-for="(line, lineIdx) in block.lines"
                :key="line.line_number"
                class="blame-row"
                :class="{
                  'is-first-line': lineIdx === 0,
                  'is-matching-search': isLineMatching(line),
                  'row-hovered': hoveredCommitHash === line.commit_hash
                }"
                @mouseenter="hoveredCommitHash = line.commit_hash"
                @mouseleave="hoveredCommitHash = null"
              >
                <!-- Left Heatmap Indicator -->
                <div
                  class="heatmap-indicator"
                  :class="'age-' + block.ageCategory"
                  :style="{ backgroundColor: block.authorColor }"
                  :title="`Author: ${block.author} • Age: ${block.ageLabel}`"
                ></div>

                <!-- Blame Meta Gutter -->
                <div class="blame-gutter-cell">
                  <!-- First line of block shows rich meta -->
                  <template v-if="lineIdx === 0">
                    <button
                      class="commit-hash-pill"
                      @click.stop="openCommitPopover($event, line)"
                      title="Click for commit details"
                    >
                      {{ line.commit_hash_short }}
                    </button>
                    <AuthorAvatar :author-name="line.author" :size="16" />
                    <span class="blame-author-name" :title="`${line.author} <${line.author_mail}>`">{{ line.author }}</span>
                    <span class="blame-summary-text" :title="line.summary">{{ line.summary }}</span>
                    <span class="blame-age-text">{{ formatBlameTime(line.author_time) }}</span>
                  </template>
                  <!-- Subsequent lines show subtle block connector -->
                  <template v-else>
                    <div class="block-continuation-gutter">
                      <span class="connector-line"></span>
                    </div>
                  </template>
                </div>

                <!-- Line Number -->
                <div class="blame-lineno-cell">{{ line.line_number }}</div>

                <!-- Code Content with Beautifier & Indent Guides -->
                <div class="blame-code-cell" :class="{ 'wrap-lines': wrapLines }">
                  <span v-if="showIndentGuides && line.indentHtml" class="indent-guides-box" v-html="line.indentHtml"></span>
                  <code class="code-line-tokenized" v-html="line.renderedCode"></code>
                  <button
                    class="line-copy-btn"
                    @click.stop="copyLineCode(line)"
                    title="Copy this line"
                  >
                    <CopyIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2" fill="none" />
                  </button>
                </div>
              </div>
            </div>
          </template>

          <!-- Line-by-Line Mode -->
          <template v-else>
            <div
              v-for="line in processedLines"
              :key="line.line_number"
              class="blame-row line-mode-row"
              :class="{
                'is-matching-search': isLineMatching(line),
                'row-hovered': hoveredCommitHash === line.commit_hash
              }"
              @mouseenter="hoveredCommitHash = line.commit_hash"
              @mouseleave="hoveredCommitHash = null"
            >
              <!-- Heatmap bar -->
              <div
                class="heatmap-indicator"
                :class="'age-' + getAgeCategory(line.author_time)"
                :style="{ backgroundColor: getAuthorColor(line.author) }"
              ></div>

              <!-- Full Gutter on every line -->
              <div class="blame-gutter-cell">
                <button
                  class="commit-hash-pill"
                  @click.stop="openCommitPopover($event, line)"
                  title="Click for commit details"
                >
                  {{ line.commit_hash_short }}
                </button>
                <AuthorAvatar :author-name="line.author" :size="16" />
                <span class="blame-author-name" :title="line.author">{{ line.author }}</span>
                <span class="blame-summary-text" :title="line.summary">{{ line.summary }}</span>
                <span class="blame-age-text">{{ formatBlameTime(line.author_time) }}</span>
              </div>

              <!-- Line Number -->
              <div class="blame-lineno-cell">{{ line.line_number }}</div>

              <!-- Code Content with Beautifier & Indent Guides -->
              <div class="blame-code-cell" :class="{ 'wrap-lines': wrapLines }">
                <span v-if="showIndentGuides && line.indentHtml" class="indent-guides-box" v-html="line.indentHtml"></span>
                <code class="code-line-tokenized" v-html="line.renderedCode"></code>
                <button
                  class="line-copy-btn"
                  @click.stop="copyLineCode(line)"
                  title="Copy this line"
                >
                  <CopyIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2" fill="none" />
                </button>
              </div>
            </div>
          </template>
        </div>
      </div>
    </div>

    <!-- Floating Commit Detail Card / Popover -->
    <Teleport to="body">
      <div
        v-if="activeCommitPopover && popoverPosition"
        class="blame-commit-popover"
        :style="{ top: popoverPosition.y + 'px', left: popoverPosition.x + 'px' }"
        @click.stop
      >
        <div class="popover-header">
          <div class="popover-author-row">
            <AuthorAvatar :author-name="activeCommitPopover.author" :size="28" />
            <div class="popover-author-info">
              <span class="popover-author-name">{{ activeCommitPopover.author }}</span>
              <span class="popover-author-email" v-if="activeCommitPopover.author_mail">&lt;{{ activeCommitPopover.author_mail }}&gt;</span>
            </div>
          </div>
          <button class="popover-close-btn" @click="closePopover">✕</button>
        </div>

        <div class="popover-body">
          <p class="popover-summary">{{ activeCommitPopover.summary }}</p>
          <div class="popover-meta-grid">
            <div class="popover-meta-item">
              <span class="meta-label">Committed:</span>
              <span class="meta-val">{{ formatExactDate(activeCommitPopover.author_time) }} ({{ formatBlameTime(activeCommitPopover.author_time) }})</span>
            </div>
            <div class="popover-meta-item">
              <span class="meta-label">Line:</span>
              <span class="meta-val">Line #{{ activeCommitPopover.line_number }}</span>
            </div>
            <div class="popover-meta-item">
              <span class="meta-label">Commit SHA:</span>
              <div class="popover-sha-row">
                <code class="popover-sha">{{ activeCommitPopover.commit_hash }}</code>
                <button class="sha-copy-btn" @click="copyCommitSha(activeCommitPopover.commit_hash)" title="Copy full SHA">
                  <CopyIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="popover-actions">
          <button class="popover-primary-btn" @click="jumpToCommit(activeCommitPopover.commit_hash)">
            <TimelineIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
            View in Commit Graph
          </button>
          <button class="popover-secondary-btn" @click="copyCommitSha(activeCommitPopover.commit_hash)">
            Copy SHA
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { BlameLine } from '../../types';
import { notify } from '../../composables/useToasts';
import {
  detectLanguage,
  highlightCodeLine,
  escapeHtml,
  formatIndentation
} from '../../utils/codeHighlighter';

import AuthorAvatar from './AuthorAvatar.vue';
import SearchIcon from '../../assets/icons/search.svg?component';
import CopyIcon from '../../assets/icons/copy.svg?component';
import FetchIcon from '../../assets/icons/fetch.svg?component';
import TimelineIcon from '../../assets/icons/timeline.svg?component';

export interface ProcessedBlameLine extends BlameLine {
  renderedCode: string;
  indentHtml: string;
}

export interface BlameBlock {
  id: string;
  commit_hash: string;
  commit_hash_short: string;
  author: string;
  author_mail: string;
  author_time: number;
  summary: string;
  lines: ProcessedBlameLine[];
  ageCategory: 'recent' | 'month' | 'quarter' | 'year' | 'ancient';
  ageLabel: string;
  authorColor: string;
}

const props = defineProps<{
  lines: BlameLine[];
  loading: boolean;
  error: string | null;
  filePath: string;
  repositoryPath?: string | null;
}>();

const emit = defineEmits<{
  (e: 'select-commit', hash: string): void;
  (e: 'refresh'): void;
}>();

// --- State ---
const searchQuery = ref('');
const groupByBlock = ref(true);
const hoveredCommitHash = ref<string | null>(null);
const activeCommitPopover = ref<BlameLine | null>(null);
const popoverPosition = ref<{ x: number; y: number } | null>(null);

// Beautifier & Formatter preferences with localStorage persistence
const enableSyntaxHighlighting = ref(localStorage.getItem('begit_blame_syntax') !== 'false');
const showIndentGuides = ref(localStorage.getItem('begit_blame_indent') !== 'false');
const wrapLines = ref(localStorage.getItem('begit_blame_wrap') === 'true');

function toggleSyntaxHighlighting() {
  enableSyntaxHighlighting.value = !enableSyntaxHighlighting.value;
  localStorage.setItem('begit_blame_syntax', enableSyntaxHighlighting.value.toString());
}

function toggleIndentGuides() {
  showIndentGuides.value = !showIndentGuides.value;
  localStorage.setItem('begit_blame_indent', showIndentGuides.value.toString());
}

function toggleWrapLines() {
  wrapLines.value = !wrapLines.value;
  localStorage.setItem('begit_blame_wrap', wrapLines.value.toString());
}

// --- Language Detection ---
const languageInfo = computed(() => detectLanguage(props.filePath));

// --- Author Colors Helper ---
const authorColorMap = new Map<string, string>();
function getAuthorColor(author: string): string {
  if (authorColorMap.has(author)) {
    return authorColorMap.get(author)!;
  }
  let hash = 0;
  for (let i = 0; i < author.length; i++) {
    hash = author.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = Math.abs(hash) % 360;
  const color = `hsl(${hue}, 65%, 55%)`;
  authorColorMap.set(author, color);
  return color;
}

// --- Age Category Helper ---
function getAgeCategory(epochSec?: number): 'recent' | 'month' | 'quarter' | 'year' | 'ancient' {
  if (!epochSec) return 'ancient';
  const nowSec = Math.floor(Date.now() / 1000);
  const diff = nowSec - epochSec;
  if (diff < 7 * 86400) return 'recent';       // < 7 days
  if (diff < 30 * 86400) return 'month';       // < 30 days
  if (diff < 90 * 86400) return 'quarter';     // < 3 months
  if (diff < 365 * 86400) return 'year';       // < 1 year
  return 'ancient';                            // > 1 year
}

function getAgeLabel(epochSec?: number): string {
  const cat = getAgeCategory(epochSec);
  switch (cat) {
    case 'recent': return 'Less than 7 days ago';
    case 'month': return 'Less than 30 days ago';
    case 'quarter': return '1 to 3 months ago';
    case 'year': return '3 to 12 months ago';
    default: return 'Over a year ago';
  }
}

// --- Formatted Time Helpers ---
function formatBlameTime(epochSec?: number) {
  if (!epochSec) return '';
  const date = new Date(epochSec * 1000);
  const now = new Date();
  const diffSec = Math.floor((now.getTime() - date.getTime()) / 1000);
  if (diffSec < 60) return 'just now';
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
  if (diffSec < 2592000) return `${Math.floor(diffSec / 86400)}d ago`;
  if (diffSec < 31536000) return `${Math.floor(diffSec / 2592000)}mo ago`;
  return `${Math.floor(diffSec / 31536000)}y ago`;
}

function formatExactDate(epochSec?: number) {
  if (!epochSec) return '';
  const d = new Date(epochSec * 1000);
  return d.toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  });
}

// --- Search in HTML Helper ---
function highlightSearchInHtml(html: string, query: string): string {
  if (!query) return html;
  const escapedQuery = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(?<=>|^)([^<]*?)(${escapedQuery})([^<]*?)(?=<|$)`, 'gi');
  return html.replace(regex, (_match, prefix, hit, suffix) => {
    return `${prefix}<mark class="blame-search-mark">${hit}</mark>${suffix}`;
  });
}

// --- Processed Lines with Syntax Highlighting & Beautifier ---
const processedLines = computed<ProcessedBlameLine[]>(() => {
  if (!props.lines || props.lines.length === 0) return [];
  const lang = languageInfo.value;
  const highlight = enableSyntaxHighlighting.value;
  const indent = showIndentGuides.value;
  const q = searchQuery.value.trim();

  return props.lines.map((line) => {
    let indentHtml = '';
    let codeToHighlight = line.content;

    if (indent) {
      const formatted = formatIndentation(line.content, 2);
      indentHtml = formatted.indentHtml;
      codeToHighlight = formatted.restHtml;
    }

    let codeHtml = highlight
      ? highlightCodeLine(codeToHighlight, lang)
      : escapeHtml(codeToHighlight);

    if (q) {
      codeHtml = highlightSearchInHtml(codeHtml, q);
    }

    return {
      ...line,
      renderedCode: codeHtml,
      indentHtml
    };
  });
});

// --- Grouped Blocks Calculation ---
const blameBlocks = computed<BlameBlock[]>(() => {
  const pLines = processedLines.value;
  if (!pLines || pLines.length === 0) return [];
  const blocks: BlameBlock[] = [];
  let currentBlock: BlameBlock | null = null;

  for (const line of pLines) {
    if (!currentBlock || currentBlock.commit_hash !== line.commit_hash) {
      currentBlock = {
        id: `block-${line.commit_hash}-${line.line_number}`,
        commit_hash: line.commit_hash,
        commit_hash_short: line.commit_hash_short,
        author: line.author,
        author_mail: line.author_mail,
        author_time: line.author_time,
        summary: line.summary,
        lines: [line],
        ageCategory: getAgeCategory(line.author_time),
        ageLabel: getAgeLabel(line.author_time),
        authorColor: getAuthorColor(line.author)
      };
      blocks.push(currentBlock);
    } else {
      currentBlock.lines.push(line);
    }
  }

  return blocks;
});

// --- Author Statistics Breakdown ---
const authorBreakdown = computed(() => {
  if (!props.lines || props.lines.length === 0) return [];
  const counts = new Map<string, number>();
  for (const line of props.lines) {
    counts.set(line.author, (counts.get(line.author) || 0) + 1);
  }

  const total = props.lines.length;
  const result: { name: string; count: number; percent: number; color: string }[] = [];
  for (const [name, count] of counts.entries()) {
    const percent = Math.round((count / total) * 100);
    result.push({
      name,
      count,
      percent,
      color: getAuthorColor(name)
    });
  }

  result.sort((a, b) => b.count - a.count);
  return result;
});

// --- Activity Time Span ---
const timeSpanText = computed(() => {
  if (!props.lines || props.lines.length === 0) return null;
  const timestamps = props.lines.map(l => l.author_time).filter(Boolean);
  if (timestamps.length === 0) return null;

  const minTs = Math.min(...timestamps);
  const maxTs = Math.max(...timestamps);

  const oldest = formatBlameTime(minTs);
  const newest = formatBlameTime(maxTs);

  return {
    oldest,
    newest,
    summary: `${oldest} → ${newest}`
  };
});

// --- Search Filter Logic ---
const matchingLineNumbers = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return new Set<number>();

  const matches = new Set<number>();
  for (const line of props.lines) {
    if (
      line.content.toLowerCase().includes(q) ||
      line.author.toLowerCase().includes(q) ||
      line.summary.toLowerCase().includes(q) ||
      line.commit_hash_short.toLowerCase().includes(q) ||
      line.commit_hash.toLowerCase().includes(q) ||
      String(line.line_number).includes(q)
    ) {
      matches.add(line.line_number);
    }
  }
  return matches;
});

function isLineMatching(line: BlameLine): boolean {
  if (!searchQuery.value.trim()) return false;
  return matchingLineNumbers.value.has(line.line_number);
}

function blockHasSearchMatch(block: BlameBlock): boolean {
  if (!searchQuery.value.trim()) return false;
  return block.lines.some(l => matchingLineNumbers.value.has(l.line_number));
}

// --- Popover Handlers ---
function openCommitPopover(event: MouseEvent, line: BlameLine) {
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  
  const x = Math.min(rect.right + 10, window.innerWidth - 380);
  const y = Math.min(rect.top - 10, window.innerHeight - 300);

  activeCommitPopover.value = line;
  popoverPosition.value = { x: Math.max(10, x), y: Math.max(10, y) };
}

function closePopover() {
  activeCommitPopover.value = null;
  popoverPosition.value = null;
}

function jumpToCommit(hash: string) {
  closePopover();
  emit('select-commit', hash);
}

async function copyCommitSha(sha: string) {
  try {
    await navigator.clipboard.writeText(sha);
    notify('Commit SHA copied to clipboard', 'success');
  } catch (e) {
    notify('Failed to copy commit SHA', 'error');
  }
}

async function copyFileContent() {
  if (!props.lines || props.lines.length === 0) return;
  const fullText = props.lines.map(l => l.content).join('\n');
  try {
    await navigator.clipboard.writeText(fullText);
    notify('File content copied to clipboard', 'success');
  } catch (e) {
    notify('Failed to copy file content', 'error');
  }
}

async function copyLineCode(line: BlameLine) {
  try {
    await navigator.clipboard.writeText(line.content);
    notify(`Copied line #${line.line_number} to clipboard`, 'success');
  } catch (e) {
    notify('Failed to copy line', 'error');
  }
}

watch(() => props.filePath, () => {
  closePopover();
  searchQuery.value = '';
});
</script>

<style scoped>
.git-blame-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
  overflow: hidden;
  background-color: var(--bg-main, #0d1117);
  color: var(--text-main, #c9d1d9);
  position: relative;
}

/* --- Blame Toolbar --- */
.blame-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 16px;
  background: var(--surface-subtle, rgba(255, 255, 255, 0.03));
  border-bottom: 1px solid var(--border, rgba(128, 128, 128, 0.15));
  flex-wrap: wrap;
  flex-shrink: 0;
  user-select: none;
}

.blame-stats-section {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.stat-badge {
  display: inline-flex;
  align-items: baseline;
  gap: 4px;
}

.stat-value {
  font-size: 0.88rem;
  font-weight: 700;
  color: var(--text-main, #e6edf3);
}

.stat-label {
  font-size: 0.72rem;
  color: var(--text-muted, #8b949e);
  text-transform: lowercase;
}

.stat-divider {
  width: 1px;
  height: 14px;
  background: var(--border, rgba(128, 128, 128, 0.2));
}

.stat-timespan {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 0.72rem;
  color: var(--text-muted, #8b949e);
}

.timespan-val {
  font-weight: 500;
  color: var(--text-main, #c9d1d9);
}

/* Authors Distribution Bar */
.authors-bar-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 6px;
}

.authors-bar {
  display: flex;
  width: 90px;
  height: 6px;
  border-radius: 3px;
  overflow: hidden;
  background: rgba(128, 128, 128, 0.15);
}

.author-bar-segment {
  height: 100%;
  transition: width 0.2s ease;
}

.author-top-chips {
  display: flex;
  align-items: center;
  gap: 6px;
}

.author-chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 0.7rem;
  color: var(--text-muted, #8b949e);
  padding: 1px 5px;
  border-radius: 3px;
  background: var(--bg-card, rgba(255, 255, 255, 0.04));
  border: 1px solid var(--border, rgba(128, 128, 128, 0.12));
}

.author-chip-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.author-chip-name {
  max-width: 75px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.author-chip-pct {
  font-size: 0.65rem;
  color: var(--text-muted, #8b949e);
  font-weight: 600;
}

.author-chip-more {
  font-size: 0.68rem;
  color: var(--text-muted, #8b949e);
}

/* Controls Section */
.blame-controls-section {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
  flex-wrap: wrap;
}

/* Language Badge */
.lang-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 3px 8px;
  background: rgba(88, 166, 255, 0.1);
  border: 1px solid rgba(88, 166, 255, 0.25);
  border-radius: 5px;
  color: var(--accent-blue, #58a6ff);
  font-size: 0.72rem;
  font-weight: 600;
}

.lang-icon {
  flex-shrink: 0;
}

.blame-search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.blame-search-box .search-icon {
  position: absolute;
  left: 8px;
  color: var(--text-muted, #8b949e);
  pointer-events: none;
}

.blame-search-input {
  width: 190px;
  padding: 4px 28px 4px 26px;
  border-radius: 5px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  background: var(--bg-main, #0d1117);
  color: var(--text-main, #c9d1d9);
  font-size: 0.75rem;
  outline: none;
  transition: all 0.15s ease;
}

.blame-search-input:focus {
  border-color: var(--accent-blue, #58a6ff);
  width: 230px;
}

.clear-search-btn {
  position: absolute;
  right: 6px;
  border: none;
  background: none;
  color: var(--text-muted, #8b949e);
  cursor: pointer;
  font-size: 0.72rem;
  padding: 2px;
}

.clear-search-btn:hover {
  color: var(--text-main, #e6edf3);
}

.search-match-count {
  position: absolute;
  right: 22px;
  font-size: 0.68rem;
  color: var(--accent-blue, #58a6ff);
  font-weight: 600;
  pointer-events: none;
}

.view-mode-toggle {
  display: flex;
  background: var(--bg-main, #0d1117);
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  border-radius: 5px;
  padding: 2px;
  gap: 2px;
}

.mode-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  border: none;
  background: transparent;
  color: var(--text-muted, #8b949e);
  font-size: 0.72rem;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.12s ease;
}

.mode-btn .btn-icon {
  font-size: 0.75rem;
  opacity: 0.85;
}

.mode-btn:hover {
  color: var(--text-main, #e6edf3);
}

.mode-btn.active {
  background: var(--surface-hover, rgba(255, 255, 255, 0.12));
  color: var(--accent-blue, #58a6ff);
  font-weight: 600;
}

.icon-tool-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 5px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  background: var(--bg-main, #0d1117);
  color: var(--text-muted, #8b949e);
  cursor: pointer;
  transition: all 0.15s ease;
}

.icon-tool-btn:hover {
  background: var(--surface-hover, rgba(255, 255, 255, 0.1));
  color: var(--text-main, #e6edf3);
  border-color: var(--accent-blue, #58a6ff);
}

/* --- Blame Body Container --- */
.blame-body-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.blame-scroll-viewport {
  flex: 1;
  overflow: auto;
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, "Liberation Mono", "JetBrains Mono", monospace;
  font-size: 0.8rem;
  line-height: 22px;
  -webkit-font-smoothing: antialiased;
}

.blame-table {
  display: flex;
  flex-direction: column;
  min-width: 100%;
  width: max-content;
}

/* --- Blame Blocks & Rows --- */
.blame-block {
  border-bottom: 1px solid rgba(128, 128, 128, 0.1);
  transition: background-color 0.1s ease;
}

.blame-block.commit-highlighted {
  background: rgba(88, 166, 255, 0.08);
}

.blame-row {
  display: flex;
  align-items: stretch;
  min-height: 22px;
  line-height: 22px;
  position: relative;
  transition: background-color 0.08s ease;
}

.blame-row:hover {
  background: var(--surface-hover, rgba(255, 255, 255, 0.05));
}

.blame-row.row-hovered {
  background: rgba(88, 166, 255, 0.09);
}

.blame-row.is-matching-search {
  background: rgba(210, 153, 34, 0.15);
}

.blame-row.is-matching-search .blame-code-cell {
  background: rgba(210, 153, 34, 0.18);
  border-left: 2px solid #d29922;
}

/* Heatmap Indicator Strip */
.heatmap-indicator {
  width: 4px;
  min-width: 4px;
  flex-shrink: 0;
  opacity: 0.85;
}

.blame-row.row-hovered .heatmap-indicator {
  opacity: 1;
  width: 5px;
}

/* Blame Gutter Cell */
.blame-gutter-cell {
  display: flex;
  align-items: center;
  gap: 7px;
  width: 320px;
  min-width: 320px;
  padding: 0 10px;
  background: var(--bg-card, #161b22);
  border-right: 1px solid var(--border, rgba(128, 128, 128, 0.15));
  font-size: 0.73rem;
  user-select: none;
  overflow: hidden;
  flex-shrink: 0;
}

.commit-hash-pill {
  font-family: inherit;
  font-size: 0.72rem;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--surface-subtle, rgba(255, 255, 255, 0.05));
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  color: var(--accent-blue, #58a6ff);
  cursor: pointer;
  transition: all 0.15s ease;
  flex-shrink: 0;
}

.commit-hash-pill:hover {
  background: var(--accent-blue, #58a6ff);
  color: #ffffff;
  border-color: var(--accent-blue, #58a6ff);
  box-shadow: 0 1px 4px rgba(88, 166, 255, 0.4);
}

.blame-author-name {
  font-weight: 600;
  color: var(--text-main, #e6edf3);
  max-width: 80px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
}

.blame-summary-text {
  flex: 1;
  min-width: 0;
  color: var(--text-muted, #8b949e);
  font-size: 0.72rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.blame-age-text {
  margin-left: auto;
  color: var(--text-muted, #8b949e);
  font-size: 0.69rem;
  white-space: nowrap;
  flex-shrink: 0;
}

/* Block continuation line */
.block-continuation-gutter {
  display: flex;
  align-items: center;
  justify-content: flex-start;
  padding-left: 12px;
  width: 100%;
  height: 100%;
}

.connector-line {
  display: block;
  width: 1px;
  height: 100%;
  background: rgba(128, 128, 128, 0.12);
}

/* Line Number Cell */
.blame-lineno-cell {
  width: 48px;
  min-width: 48px;
  text-align: right;
  padding: 0 10px;
  color: var(--text-muted, #6e7681);
  background: var(--bg-card, #161b22);
  border-right: 1px solid var(--border, rgba(128, 128, 128, 0.15));
  user-select: none;
  font-size: 0.74rem;
  flex-shrink: 0;
}

/* Code Content Cell */
.blame-code-cell {
  flex: 1;
  padding: 0 14px;
  color: var(--text-main, #c9d1d9);
  font-size: 0.8rem;
  user-select: text;
  display: flex;
  align-items: center;
  position: relative;
}

.blame-code-cell.wrap-lines {
  white-space: pre-wrap;
  word-break: break-all;
}

.blame-code-cell:not(.wrap-lines) {
  white-space: pre;
}

.blame-code-cell code {
  font-family: inherit;
  tab-size: 2;
}

.indent-guides-box {
  display: inline-flex;
  align-items: stretch;
  height: 100%;
  vertical-align: top;
  user-select: none;
  pointer-events: none;
}

:deep(.indent-guide) {
  display: inline-block;
  position: relative;
  height: 22px;
  vertical-align: top;
  user-select: none;
  pointer-events: none;
}

:deep(.indent-line) {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 1px;
  background: rgba(128, 128, 128, 0.16);
}

.blame-row:hover :deep(.indent-line) {
  background: rgba(128, 128, 128, 0.32);
}

/* Line Quick Copy Button */
.line-copy-btn {
  display: none;
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  background: var(--bg-card, #161b22);
  color: var(--text-muted, #8b949e);
  border-radius: 4px;
  padding: 2px 4px;
  cursor: pointer;
  z-index: 5;
  transition: all 0.15s ease;
}

.blame-row:hover .line-copy-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.line-copy-btn:hover {
  background: var(--surface-hover, rgba(255, 255, 255, 0.1));
  color: var(--text-main, #e6edf3);
  border-color: var(--accent-blue, #58a6ff);
}

/* --- Floating Popover Card --- */
.blame-commit-popover {
  position: fixed;
  z-index: 10000;
  width: 360px;
  max-width: 90vw;
  background: var(--bg-card, #161b22);
  border: 1px solid var(--border, rgba(128, 128, 128, 0.25));
  border-radius: 10px;
  box-shadow: 0 12px 36px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  animation: popoverFadeIn 0.15s ease-out;
}

@keyframes popoverFadeIn {
  from {
    opacity: 0;
    transform: translateY(-6px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.popover-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  background: var(--surface-subtle, rgba(255, 255, 255, 0.03));
  border-bottom: 1px solid var(--border, rgba(128, 128, 128, 0.15));
}

.popover-author-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.popover-author-info {
  display: flex;
  flex-direction: column;
}

.popover-author-name {
  font-weight: 700;
  font-size: 0.88rem;
  color: var(--text-main, #e6edf3);
}

.popover-author-email {
  font-size: 0.72rem;
  color: var(--text-muted, #8b949e);
}

.popover-close-btn {
  border: none;
  background: transparent;
  color: var(--text-muted, #8b949e);
  cursor: pointer;
  font-size: 0.8rem;
  padding: 4px;
  border-radius: 4px;
}

.popover-close-btn:hover {
  color: var(--text-main, #e6edf3);
  background: var(--surface-hover, rgba(255, 255, 255, 0.1));
}

.popover-body {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.popover-summary {
  margin: 0;
  font-size: 0.85rem;
  font-weight: 600;
  line-height: 1.4;
  color: var(--text-main, #e6edf3);
}

.popover-meta-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
  background: var(--bg-main, #0d1117);
  padding: 10px;
  border-radius: 6px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.12));
}

.popover-meta-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 0.74rem;
}

.meta-label {
  color: var(--text-muted, #8b949e);
  font-weight: 500;
}

.meta-val {
  color: var(--text-main, #c9d1d9);
  font-weight: 600;
}

.popover-sha-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.popover-sha {
  font-family: ui-monospace, monospace;
  font-size: 0.7rem;
  color: var(--accent-blue, #58a6ff);
  background: rgba(88, 166, 255, 0.1);
  padding: 1px 5px;
  border-radius: 3px;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sha-copy-btn {
  border: none;
  background: transparent;
  color: var(--text-muted, #8b949e);
  cursor: pointer;
  padding: 2px;
  border-radius: 3px;
  display: flex;
  align-items: center;
}

.sha-copy-btn:hover {
  color: var(--text-main, #e6edf3);
  background: var(--surface-hover, rgba(255, 255, 255, 0.1));
}

.popover-actions {
  display: flex;
  gap: 8px;
  padding: 10px 14px;
  background: var(--surface-subtle, rgba(255, 255, 255, 0.02));
  border-top: 1px solid var(--border, rgba(128, 128, 128, 0.15));
}

.popover-primary-btn {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid var(--accent-blue, #58a6ff);
  background: var(--accent-blue, #58a6ff);
  color: #ffffff;
  font-size: 0.78rem;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s ease;
}

.popover-primary-btn:hover {
  opacity: 0.9;
  box-shadow: 0 2px 6px rgba(88, 166, 255, 0.35);
}

.popover-secondary-btn {
  padding: 6px 12px;
  border-radius: 6px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.25));
  background: transparent;
  color: var(--text-main, #c9d1d9);
  font-size: 0.78rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.popover-secondary-btn:hover {
  background: var(--surface-hover, rgba(255, 255, 255, 0.08));
  color: var(--text-main, #ffffff);
}

/* --- Loading / Error / Empty States --- */
.blame-loading-state,
.blame-error-state,
.blame-empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 32px;
  gap: 12px;
  color: var(--text-muted, #8b949e);
  text-align: center;
}

.blame-error-state h3 {
  margin: 0;
  color: #f85149;
  font-size: 1rem;
}

.error-icon-box {
  font-size: 2rem;
}

.retry-btn {
  padding: 6px 16px;
  border-radius: 6px;
  border: 1px solid var(--border, rgba(128, 128, 128, 0.2));
  background: var(--surface-subtle, rgba(255, 255, 255, 0.05));
  color: var(--text-main, #e6edf3);
  font-size: 0.8rem;
  cursor: pointer;
  transition: all 0.15s ease;
}

.retry-btn:hover {
  background: var(--surface-hover, rgba(255, 255, 255, 0.1));
}

.spinner {
  width: 28px;
  height: 28px;
  border: 3px solid rgba(88, 166, 255, 0.2);
  border-top-color: var(--accent-blue, #58a6ff);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* --- Syntax Highlighting Styles (Highlight.js + Prism) --- */
:deep(.hljs-comment),
:deep(.hljs-quote),
:deep(.hljs-doctag),
:deep(.token.comment),
:deep(.token.prolog),
:deep(.token.doctype),
:deep(.token.cdata) {
  color: #8b949e !important;
  font-style: italic;
}

:deep(.hljs-punctuation),
:deep(.token.punctuation) {
  color: #e6edf3 !important;
}

:deep(.hljs-attr),
:deep(.hljs-attribute),
:deep(.hljs-tag),
:deep(.hljs-name),
:deep(.token.property),
:deep(.token.tag),
:deep(.token.constant),
:deep(.token.symbol),
:deep(.token.deleted) {
  color: #7ee787 !important;
}

:deep(.hljs-number),
:deep(.token.number) {
  color: #79c0ff !important;
}

:deep(.hljs-literal),
:deep(.hljs-boolean),
:deep(.token.boolean) {
  color: #ff7b72 !important;
}

:deep(.hljs-built_in),
:deep(.hljs-builtin-name) {
  color: #79c0ff !important;
}

:deep(.hljs-string),
:deep(.hljs-regexp),
:deep(.token.selector),
:deep(.token.attr-name),
:deep(.token.string),
:deep(.token.char),
:deep(.token.builtin),
:deep(.token.inserted) {
  color: #a5d6ff !important;
}

:deep(.hljs-operator),
:deep(.token.operator),
:deep(.token.entity),
:deep(.token.url) {
  color: #79c0ff !important;
}

:deep(.hljs-keyword),
:deep(.hljs-selector-tag),
:deep(.hljs-meta .hljs-keyword),
:deep(.token.atrule),
:deep(.token.attr-value),
:deep(.token.keyword) {
  color: #ff7b72 !important;
  font-weight: 600;
}

:deep(.hljs-title),
:deep(.hljs-title.function_),
:deep(.hljs-function),
:deep(.token.function) {
  color: #d2a8ff !important;
}

:deep(.hljs-title.class_),
:deep(.hljs-type),
:deep(.hljs-class),
:deep(.token.class-name) {
  color: #ffa657 !important;
}

:deep(.hljs-variable),
:deep(.hljs-template-variable),
:deep(.token.regex),
:deep(.token.important),
:deep(.token.variable) {
  color: #ffa657 !important;
}

:deep(.hljs-params) {
  color: #c9d1d9 !important;
}

:deep(.token.important),
:deep(.token.bold),
:deep(.hljs-strong) {
  font-weight: bold;
}

:deep(.token.italic),
:deep(.hljs-emphasis) {
  font-style: italic;
}

:deep(mark.blame-search-mark) {
  background: rgba(210, 153, 34, 0.45);
  color: inherit;
  border-radius: 2px;
  padding: 0 1px;
  box-shadow: 0 0 4px rgba(210, 153, 34, 0.5);
}
</style>
