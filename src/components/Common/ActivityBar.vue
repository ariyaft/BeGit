<script setup lang="ts">
import type { AppView } from '../../types';
import SourceControlIcon from '../../assets/icons/source-control.svg?component';
import TimelineIcon from '../../assets/icons/timeline.svg?component';
import PullRequestIcon from '../../assets/icons/git-pull-request.svg?component';
import ContributorsIcon from '../../assets/icons/contributors.svg?component';
import FolderIcon from '../../assets/icons/folder.svg?component';
import ActivityLogIcon from '../../assets/icons/list.svg?component';
import UndoIcon from '../../assets/icons/undo.svg?component';

defineProps<{
  activeView: AppView;
}>();

const emit = defineEmits<{
  selectView: [view: AppView];
}>();
</script>

<template>
  <nav class="activity-bar" aria-label="Application views">
    <div class="activity-item" :class="{ active: activeView === 'source-control' }" @click="emit('selectView', 'source-control')" title="Source Control">
      <SourceControlIcon viewBox="0 0 24 24" width="24" height="24" stroke="currentColor" stroke-width="1.5" fill="none"  />
    </div>
    <div class="activity-item" :class="{ active: activeView === 'horizontal-graph' }" @click="emit('selectView', 'horizontal-graph')" title="Horizontal Timeline">
      <TimelineIcon viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="1.5" fill="none"  />
    </div>
    <div class="activity-item" :class="{ active: activeView === 'pull-requests' }" @click="emit('selectView', 'pull-requests')" title="Pull Requests">
      <PullRequestIcon viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="1.7" fill="none"  />
    </div>
    <div class="activity-item" :class="{ active: activeView === 'file-explorer' }" @click="emit('selectView', 'file-explorer')" title="File Explorer">
      <FolderIcon viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="1.6" fill="none"  />
    </div>
    <div class="activity-item" :class="{ active: activeView === 'reflog' }" @click="emit('selectView', 'reflog')" title="Git Reflog & Undo Safety Net">
      <UndoIcon viewBox="0 0 24 24" width="20" height="20" stroke="currentColor" stroke-width="1.8" fill="none"  />
    </div>
    <div class="activity-item" :class="{ active: activeView === 'contributors-report' }" @click="emit('selectView', 'contributors-report')" title="Contributors Report">
      <ContributorsIcon viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="1.6" fill="none"  />
    </div>
    <div class="activity-item" :class="{ active: activeView === 'activity-log' }" @click="emit('selectView', 'activity-log')" title="Activity Logs">
      <ActivityLogIcon viewBox="0 0 24 24" width="22" height="22" stroke="currentColor" stroke-width="1.8" fill="none"  />
    </div>
  </nav>
</template>
