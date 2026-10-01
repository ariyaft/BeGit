<script setup lang="ts">
import HorizontalGraph from '../components/timeline/HorizontalGraph.vue';
import TimelineChangesTray from '../components/timeline/TimelineChangesTray.vue';
import DiffViewer from '../components/Common/DiffViewer.vue';

defineProps<{
  commits: any[];
  selectedCommit: any;
  loading: boolean;
  selectedFilePath: string | null;
  selectedFileDiff: string | null;
  commitLimit: number;
}>();

const emit = defineEmits<{
  selectCommit: [commit: any, event?: MouseEvent];
  selectFile: [path: string];
  closeDiff: [];
  changeLimit: [limit: number];
}>();

function forwardSelectCommit(commit: any, event?: MouseEvent) {
  emit('selectCommit', commit, event);
}
</script>

<template>
  <main class="main-content">
    <HorizontalGraph
      class="timeline-graph"
      :commits="commits"
      :selected-commit="selectedCommit"
      :commit-limit="commitLimit"
      @select-commit="forwardSelectCommit"
      @change-limit="emit('changeLimit', $event)"
    />
    <TimelineChangesTray
      v-if="selectedCommit"
      :commit="selectedCommit"
      :loading="loading"
      @select-file="emit('selectFile', $event)"
    />
    <DiffViewer
      v-if="selectedFileDiff !== null"
      :file-path="selectedFilePath || ''"
      :file-diff="selectedFileDiff"
      @close="emit('closeDiff')"
    />
  </main>
</template>
