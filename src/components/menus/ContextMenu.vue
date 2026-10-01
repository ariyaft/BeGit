<template>
  <!-- Context Menu Overlay -->
  <div v-if="visible && branch" 
       class="context-menu" 
       :style="{ left: x + 'px', top: y + 'px' }"
       @click.stop>
       
    <!-- Universal Items -->
    <div class="menu-item" @click="$emit('action', 'copy_name')">
      <CopyIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Copy Branch Name
    </div>
    <div class="menu-item" @click="$emit('action', 'rename')">
      <Context1Icon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Rename Branch
    </div>
    <div class="menu-item" @click="$emit('action', 'create_branch')">
      <PlusIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Create Branch Here
    </div>
    
    <div class="menu-divider"></div>
    
    <!-- Inactive only -->
    <div class="menu-item" @click="$emit('action', 'checkout')" v-if="!branch.active">
      <ChevronDownIcon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Checkout Branch
    </div>
    <div class="menu-item" @click="$emit('action', 'merge')" v-if="!branch.active">
      <Context2Icon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Merge into active branch
    </div>
    
    <div class="menu-item" @click="$emit('action', 'push')" v-if="!branch.isRemote">
      <Context3Icon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Push to origin
    </div>
    <div class="menu-item" @click="$emit('action', 'pull')" v-if="!branch.isRemote">
      <Context4Icon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Pull from origin
    </div>
    <!-- Set upstream for local branches -->
    <div class="menu-item" @click="$emit('action', 'set_upstream')" v-if="!branch.name?.startsWith('origin/')">
      <Context5Icon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Set Upstream (origin)
    </div>

    <div class="menu-divider" v-if="!branch.active"></div>
    
    <div class="menu-item text-danger" @click="$emit('action', 'delete_local')" v-if="!branch.active && !branch.isRemote">
      <ContextDanger1Icon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Delete Local Branch
    </div>
    
    <div class="menu-item text-danger" @click="$emit('action', 'delete_remote')" v-if="!branch.active && branch.isRemote">
      <ContextDanger2Icon viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none"  />
      Delete Remote Branch
    </div>
  </div>
</template>

<script setup lang="ts">
import CopyIcon from '../../assets/icons/copy.svg?component';
import Context1Icon from '../../assets/icons/context-1.svg?component';
import PlusIcon from '../../assets/icons/plus.svg?component';
import ChevronDownIcon from '../../assets/icons/chevron-down.svg?component';
import Context2Icon from '../../assets/icons/context-2.svg?component';
import Context3Icon from '../../assets/icons/context-3.svg?component';
import Context4Icon from '../../assets/icons/context-4.svg?component';
import Context5Icon from '../../assets/icons/context-5.svg?component';
import ContextDanger1Icon from '../../assets/icons/context-danger-1.svg?component';
import ContextDanger2Icon from '../../assets/icons/context-danger-2.svg?component';


const props = defineProps({
  visible: Boolean,
  x: Number,
  y: Number,
  branch: Object
})

const emit = defineEmits(['action'])
</script>
