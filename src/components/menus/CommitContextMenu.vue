<template>
  <div v-if="visible && commits && commits.length > 0" 
       class="context-menu" 
       :style="{ left: x + 'px', top: y + 'px' }"
       @click.stop>
       
    <!-- Multiple Selection Actions -->
    <template v-if="commits.length > 1">
      <div v-if="canCherryPick" class="menu-item" @click="$emit('action', 'cherry_pick')">
        Cherry pick {{ commits.length }} commits
      </div>
      <div class="menu-item" @click="$emit('action', 'create_patch')">
        Create patch from commits
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item" @click="$emit('action', 'copy_sha')">
        Copy commit shas
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item text-danger" @click="$emit('action', 'drop_commit')">
        Drop {{ commits.length }} commits from history...
      </div>
    </template>

    <!-- Single Selection Actions -->
    <template v-else>
      <div v-if="canCherryPick" class="menu-item" @click="$emit('action', 'cherry_pick')">
        Cherry pick this commit
      </div>
      <div v-if="commits[0]?.isHead" class="menu-item" @click="$emit('action', 'amend_commit')">
        Amend this commit (HEAD)...
      </div>
      <div class="menu-item" @click="$emit('action', 'checkout')">
        Checkout this commit
      </div>
      <div class="menu-item" @click="$emit('action', 'create_branch')">
        Create branch here
      </div>
      <div class="menu-item" @click="$emit('action', 'create_tag')">
        Create tag here...
      </div>
      <div class="menu-item" @click="$emit('action', 'rebase')">
        Rebase active branch onto this commit
      </div>
      <div class="menu-item" @click="$emit('action', 'interactive_rebase')">
        Interactive rebase from here (squash / reword / drop)...
      </div>
      <div class="menu-item" @click="$emit('action', 'reset')">
        Reset active branch to this commit
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item" @click="$emit('action', 'revert')">
        Revert commit
      </div>
      <div class="menu-item" @click="$emit('action', 'copy_sha')">
        Copy commit sha
      </div>
      <div class="menu-divider"></div>
      <div class="menu-item text-danger" @click="$emit('action', 'drop_commit')">
        Drop this commit from history...
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { PropType } from 'vue';

const props = defineProps({
  visible: Boolean,
  x: Number,
  y: Number,
  commits: {
    type: Array as PropType<any[]>,
    default: () => []
  },
  canCherryPick: {
    type: Boolean,
    default: false
  }
});

const emit = defineEmits(['action']);
</script>

<style scoped>
/* Inherit styles from ContextMenu via App.css if possible, or add them here */
</style>
