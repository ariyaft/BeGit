<template>
  <aside class="left-sidebar" :style="{ width: width + 'px' }" v-show="width > 0">
    <!-- Search / Filter Header -->
    <div class="sidebar-header">
      <div class="search-container" :class="{ 'has-query': Boolean(searchQuery) }">
        <SearchIcon class="search-icon" viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
        <input 
          type="text" 
          v-model="searchQuery" 
          placeholder="Filter branches, tags..."
          class="sidebar-search-input"
          @keydown.esc="searchQuery = ''"
        />
        <span v-if="searchQuery" class="search-count-tag" title="Matching items">
          {{ totalMatchCount }}
        </span>
        <button v-if="searchQuery" class="clear-search-btn-sidebar" @click="searchQuery = ''" title="Clear filter (Esc)">
          <CloseIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" />
        </button>
      </div>
    </div>

    <div class="sidebar-scrollable">
      <!-- Segmented View Mode Switcher -->
      <div class="view-mode-container">
        <div class="segmented-control">
          <button 
            class="segmented-btn" 
            :class="{ active: branchViewMode === 'flat' }" 
            @click.stop="branchViewMode = 'flat'"
            title="Flat list mode"
          >
            <ListIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
            <span>Flat</span>
          </button>
          <button 
            class="segmented-btn" 
            :class="{ active: branchViewMode === 'tree' }" 
            @click.stop="branchViewMode = 'tree'"
            title="Hierarchical tree mode"
          >
            <TimelineIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
            <span>Tree</span>
          </button>
        </div>
      </div>

      <!-- 1. LOCAL BRANCHES SECTION -->
      <div class="sidebar-section section-local" :class="{ 'is-collapsed': collapsedSections.has('local') }">
        <div class="section-title" @click="toggleSection('local')">
          <div class="title-left">
            <div class="section-icon-box icon-box-local">
              <BranchIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </div>
            <span class="section-name">LOCAL</span>
            <span class="count-badge count-local">{{ (project.localBranches || []).length }}</span>
          </div>

          <div class="title-right">
            <button class="header-action-btn" @click.stop="$emit('create-branch')" title="Create New Branch (+)">
              <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            </button>
            <span class="chevron-wrapper" :class="{ 'is-collapsed': collapsedSections.has('local') }">
              <ChevronDownIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </span>
          </div>
        </div>
        
        <div class="section-body" v-show="!collapsedSections.has('local')">
          <ul v-if="branchViewMode === 'flat'" class="branch-list">
            <li v-for="branch in filteredLocalBranches" :key="branch.name" 
                class="branch-item"
                :class="{ 'active-branch': branch.active }" 
                @click="$emit('item-click', { type: 'branch', id: branch.name })"
                @dblclick="$emit('branch-dblclick', branch)"
                :title="branch.name + (branch.active ? ' (Checked Out)' : '')">
              <div v-if="branch.active" class="active-indicator-bar"></div>
              <BranchIcon class="branch-icon branch-local-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              <span class="branch-name" v-html="highlightMatch(branch.name)"></span>
              
              <!-- Ahead / Behind Badges -->
              <span class="branch-sync" v-if="branch.ahead > 0 || branch.behind > 0">
                <span v-if="branch.ahead > 0" class="sync-badge sync-ahead" :title="`${branch.ahead} commits ahead of upstream`">↑{{ branch.ahead }}</span>
                <span v-if="branch.behind > 0" class="sync-badge sync-behind" :title="`${branch.behind} commits behind upstream`">↓{{ branch.behind }}</span>
              </span>

              <!-- Active Status Badge -->
              <span v-if="branch.active" class="active-pill" title="Current HEAD Branch">
                <CheckIcon viewBox="0 0 24 24" width="10" height="10" stroke="currentColor" stroke-width="3" fill="none" />
              </span>

              <!-- Context Menu Trigger -->
              <div class="branch-menu-dots" @click.stop="$emit('branch-context', $event, branch)" title="Branch actions">
                <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              </div>
            </li>
          </ul>

          <ul v-else class="branch-list tree-list">
            <li v-for="node in treeLocalBranches" :key="node.path" 
                class="branch-item"
                :class="{ 'active-branch': node.branch?.active, 'is-dir': node.isDir }"
                :style="{ paddingLeft: (node.depth * 13 + 8) + 'px' }"
                @click="node.isDir ? toggleBranchDir(node.path) : $emit('item-click', { type: 'branch', id: node.branch.name })"
                @dblclick="!node.isDir ? $emit('branch-dblclick', node.branch) : null"
                :title="node.branch ? node.branch.name : node.name">
              
              <div v-if="node.branch?.active" class="active-indicator-bar"></div>

              <div class="tree-toggle" v-if="node.isDir">
                <span class="chevron-wrapper" :class="{ 'is-collapsed': isDirCollapsed(node.path) }">
                  <ChevronDownIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" />
                </span>
              </div>
              <div class="tree-toggle placeholder" v-else></div>
              
              <FolderIcon v-if="node.isDir" class="branch-icon folder-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              <BranchIcon v-else class="branch-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              
              <span class="branch-name">
                <span v-html="highlightMatch(node.name)"></span>
                <span v-if="node.isDir" class="dir-count-pill">{{ node.leafCount }}</span>
              </span>
              
              <span class="branch-sync" v-if="!node.isDir && (node.branch.ahead > 0 || node.branch.behind > 0)">
                <span v-if="node.branch.ahead > 0" class="sync-badge sync-ahead">↑{{ node.branch.ahead }}</span>
                <span v-if="node.branch.behind > 0" class="sync-badge sync-behind">↓{{ node.branch.behind }}</span>
              </span>

              <span v-if="!node.isDir && node.branch.active" class="active-pill" title="Current HEAD Branch">
                <CheckIcon viewBox="0 0 24 24" width="10" height="10" stroke="currentColor" stroke-width="3" fill="none" />
              </span>

              <div v-if="!node.isDir" class="branch-menu-dots" @click.stop="$emit('branch-context', $event, node.branch)" title="Branch actions">
                <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              </div>
            </li>
          </ul>
        </div>
      </div>

      <!-- 2. REMOTE BRANCHES SECTION -->
      <div class="sidebar-section section-remote" :class="{ 'is-collapsed': collapsedSections.has('remote') }">
        <div class="section-title" @click="toggleSection('remote')">
          <div class="title-left">
            <div class="section-icon-box icon-box-remote">
              <CloudIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </div>
            <span class="section-name">REMOTE</span>
            <span class="count-badge count-remote">{{ (project.remoteBranches || []).length }}</span>
          </div>

          <div class="title-right">
            <button class="header-action-btn" @click.stop="$emit('add-remote')" title="Add Remote Repository (+)">
              <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            </button>
            <span class="chevron-wrapper" :class="{ 'is-collapsed': collapsedSections.has('remote') }">
              <ChevronDownIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </span>
          </div>
        </div>
        
        <div class="section-body" v-show="!collapsedSections.has('remote')">
          <!-- Remote Dropdown Bar -->
          <div class="remote-selector-bar" v-if="project.remotes && project.remotes.length > 0">
            <div class="remote-pill-label">
              <CloudIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" style="color: #4caf50;" />
              <select v-model="selectedRemoteName" class="remote-select-native">
                <option v-for="remote in project.remotes" :key="remote.name" :value="remote.name">
                  {{ remote.name }}
                </option>
              </select>
            </div>
            
            <div class="remote-actions-group">
              <button class="remote-sub-btn" @click.stop="openSelectedRemoteContext($event)" title="Remote Options">
                <MoreVerticalIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
              </button>
            </div>
          </div>

          <ul v-if="branchViewMode === 'flat'" class="branch-list">
            <li v-for="branch in filteredRemoteBranches" :key="branch.name" 
                class="branch-item"
                @click="$emit('item-click', { type: 'branch', id: branch.name })"
                @dblclick="$emit('branch-dblclick', branch)"
                :title="branch.name">
              <CloudIcon class="branch-icon branch-remote-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              <span class="branch-name" v-html="highlightMatch(branch.name.startsWith('origin/') ? branch.name.substring(7) : branch.name)"></span>
              <div class="branch-menu-dots" @click.stop="$emit('branch-context', $event, branch)" title="Remote branch actions">
                <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              </div>
            </li>
          </ul>

          <ul v-else class="branch-list tree-list">
            <li v-for="node in treeRemoteBranches" :key="node.path" 
                class="branch-item"
                :class="{ 'is-dir': node.isDir }"
                :style="{ paddingLeft: (node.depth * 13 + 8) + 'px' }"
                @click="node.isDir ? toggleBranchDir(node.path) : $emit('item-click', { type: 'branch', id: node.branch.name })"
                @dblclick="!node.isDir ? $emit('branch-dblclick', node.branch) : null"
                :title="node.branch ? node.branch.name : node.name">
              
              <div class="tree-toggle" v-if="node.isDir">
                <span class="chevron-wrapper" :class="{ 'is-collapsed': isDirCollapsed(node.path) }">
                  <ChevronDownIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" />
                </span>
              </div>
              <div class="tree-toggle placeholder" v-else></div>
              
              <FolderIcon v-if="node.isDir" class="branch-icon folder-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              <CloudIcon v-else class="branch-icon branch-remote-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              
              <span class="branch-name">
                <span v-html="highlightMatch(node.name)"></span>
                <span v-if="node.isDir" class="dir-count-pill">{{ node.leafCount }}</span>
              </span>
              <div v-if="!node.isDir" class="branch-menu-dots" @click.stop="$emit('branch-context', $event, node.branch)" title="Remote branch actions">
                <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              </div>
            </li>
          </ul>
        </div>
      </div>

      <!-- 3. TAGS SECTION -->
      <div class="sidebar-section section-tags" :class="{ 'is-collapsed': collapsedSections.has('tags') }">
        <div class="section-title" @click="toggleSection('tags')">
          <div class="title-left">
            <div class="section-icon-box icon-box-tags">
              <TagIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </div>
            <span class="section-name">TAGS</span>
            <span class="count-badge count-tags">{{ (project.tags || []).length }}</span>
          </div>

          <div class="title-right">
            <button class="header-action-btn" @click.stop="$emit('create-tag')" title="Create Tag (+)">
              <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            </button>
            <span class="chevron-wrapper" :class="{ 'is-collapsed': collapsedSections.has('tags') }">
              <ChevronDownIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </span>
          </div>
        </div>
        
        <div class="section-body" v-show="!collapsedSections.has('tags')">
          <template v-if="(project.tags || []).length > 0">
            <ul v-if="branchViewMode === 'flat'" class="branch-list">
              <li v-for="tag in filteredTags" :key="tag" 
                  class="branch-item"
                  @click="$emit('item-click', { type: 'tag', id: tag })"
                  :title="tag">
                <TagIcon class="branch-icon branch-tag-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
                <span class="branch-name" v-html="highlightMatch(tag)"></span>
                <div class="branch-menu-dots" @click.stop="$emit('tag-context', $event, tag)" title="Tag actions">
                  <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
                </div>
              </li>
            </ul>

            <ul v-else class="branch-list tree-list">
              <li v-for="node in treeTags" :key="node.path" 
                  class="branch-item"
                  :class="{ 'is-dir': node.isDir }"
                  :style="{ paddingLeft: (node.depth * 13 + 8) + 'px' }"
                  @click="node.isDir ? toggleBranchDir(node.path) : $emit('item-click', { type: 'tag', id: node.branch.originalName })"
                  :title="node.branch ? node.branch.originalName : node.name">
                
                <div class="tree-toggle" v-if="node.isDir">
                  <span class="chevron-wrapper" :class="{ 'is-collapsed': isDirCollapsed(node.path) }">
                    <ChevronDownIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" />
                  </span>
                </div>
                <div class="tree-toggle placeholder" v-else></div>
                
                <FolderIcon v-if="node.isDir" class="branch-icon folder-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
                <TagIcon v-else class="branch-icon branch-tag-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
                
                <span class="branch-name">
                  <span v-html="highlightMatch(node.name)"></span>
                  <span v-if="node.isDir" class="dir-count-pill">{{ node.leafCount }}</span>
                </span>
                <div v-if="!node.isDir" class="branch-menu-dots" @click.stop="$emit('tag-context', $event, node.branch.originalName)" title="Tag actions">
                  <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
                </div>
              </li>
            </ul>
          </template>
          
          <div v-else class="sidebar-empty-state">
            <span>No tags found</span>
            <button class="empty-action-link" @click="$emit('create-tag')">+ Create tag</button>
          </div>
        </div>
      </div>

      <!-- 4. WORKTREES SECTION -->
      <div class="sidebar-section section-worktrees" :class="{ 'is-collapsed': collapsedSections.has('worktrees') }">
        <div class="section-title" @click="toggleSection('worktrees')">
          <div class="title-left">
            <div class="section-icon-box icon-box-worktrees">
              <WorktreeIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </div>
            <span class="section-name">WORKTREES</span>
            <span class="count-badge count-worktrees">{{ (project.worktrees || []).length }}</span>
          </div>

          <div class="title-right">
            <button class="header-action-btn" @click.stop="$emit('create-worktree')" title="Add Worktree (+)">
              <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            </button>
            <button v-if="(project.worktrees || []).length > 1" class="header-action-btn" @click.stop="$emit('prune-worktrees')" title="Prune Stale Worktrees">
              <FetchIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
            </button>
            <span class="chevron-wrapper" :class="{ 'is-collapsed': collapsedSections.has('worktrees') }">
              <ChevronDownIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </span>
          </div>
        </div>

        <div class="section-body" v-show="!collapsedSections.has('worktrees')">
          <ul v-if="filteredWorktrees.length > 0" class="branch-list">
            <li v-for="wt in filteredWorktrees" :key="wt.path"
                class="branch-item"
                :class="{ 'active-branch': wt.is_main }"
                @click="$emit('open-worktree', wt.path)"
                :title="wt.path">
              <WorktreeIcon class="branch-icon branch-worktree-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              <span class="branch-name">
                <span v-if="wt.branch" v-html="highlightMatch(wt.branch)"></span>
                <span v-else style="font-style: italic; opacity: 0.8;">detached @ {{ wt.commit_hash.substring(0, 7) }}</span>
              </span>
              <span v-if="wt.is_main" class="worktree-main-badge">main</span>
              <span v-if="wt.is_locked" class="worktree-lock-badge" :title="wt.lock_reason ? `Locked: ${wt.lock_reason}` : 'Locked'">
                <LockIcon viewBox="0 0 24 24" width="11" height="11" stroke="currentColor" stroke-width="2.5" fill="none" />
              </span>
              <div class="branch-menu-dots" @click.stop="$emit('worktree-context', $event, wt)" title="Worktree actions">
                <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              </div>
            </li>
          </ul>
          <div v-else class="sidebar-empty-state">
            <span>No extra worktrees</span>
            <button class="empty-action-link" @click="$emit('create-worktree')">+ Add worktree</button>
          </div>
        </div>
      </div>

      <!-- 5. SUBMODULES SECTION -->
      <div class="sidebar-section section-submodules" :class="{ 'is-collapsed': collapsedSections.has('submodules') }">
        <div class="section-title" @click="toggleSection('submodules')">
          <div class="title-left">
            <div class="section-icon-box icon-box-submodules">
              <SubmoduleIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </div>
            <span class="section-name">SUBMODULES</span>
            <span class="count-badge count-submodules">{{ (project.submodules || []).length }}</span>
          </div>

          <div class="title-right">
            <button class="header-action-btn" @click.stop="$emit('add-submodule')" title="Add Submodule (+)">
              <PlusIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2.5" fill="none" />
            </button>
            <button v-if="(project.submodules || []).length > 0" class="header-action-btn" @click.stop="$emit('sync-all-submodules')" title="Sync All Submodules">
              <FetchIcon viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" />
            </button>
            <span class="chevron-wrapper" :class="{ 'is-collapsed': collapsedSections.has('submodules') }">
              <ChevronDownIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </span>
          </div>
        </div>

        <div class="section-body" v-show="!collapsedSections.has('submodules')">
          <ul v-if="filteredSubmodules.length > 0" class="branch-list">
            <li v-for="sm in filteredSubmodules" :key="sm.path"
                class="branch-item"
                @click="$emit('open-submodule', sm.full_path)"
                :title="`${sm.path} (${sm.url})`">
              <SubmoduleIcon class="branch-icon branch-submodule-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              <span class="branch-name" v-html="highlightMatch(sm.name || sm.path)"></span>
              <span :class="['submodule-status-badge', `status-${sm.status}`]" :title="`Status: ${sm.status}`">
                {{ sm.status }}
              </span>
              <div class="branch-menu-dots" @click.stop="$emit('submodule-context', $event, sm)" title="Submodule actions">
                <MoreVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              </div>
            </li>
          </ul>
          <div v-else class="sidebar-empty-state">
            <span>No submodules configured</span>
            <button class="empty-action-link" @click="$emit('add-submodule')">+ Add submodule</button>
          </div>
        </div>
      </div>

      <!-- 6. STASHES SECTION (Optional) -->
      <div class="sidebar-section section-stashes" v-if="project.stashes && project.stashes.length > 0" :class="{ 'is-collapsed': collapsedSections.has('stashes') }">
        <div class="section-title" @click="toggleSection('stashes')">
          <div class="title-left">
            <div class="section-icon-box icon-box-stashes">
              <StashIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </div>
            <span class="section-name">STASHES</span>
            <span class="count-badge count-stashes">{{ (project.stashes || []).length }}</span>
          </div>

          <div class="title-right">
            <span class="chevron-wrapper" :class="{ 'is-collapsed': collapsedSections.has('stashes') }">
              <ChevronDownIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2.2" fill="none" />
            </span>
          </div>
        </div>
        
        <div class="section-body" v-show="!collapsedSections.has('stashes')">
          <ul class="branch-list">
            <li v-for="stash in filteredStashes" :key="stash.id" 
                class="branch-item"
                @click="$emit('item-click', { type: 'stash', id: stash.id })">
              <StashIcon class="branch-icon branch-stash-icon" viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" />
              <span class="branch-name" :title="stash.message" v-html="highlightMatch(`${stash.id}: ${stash.message}`)"></span>
              <div class="branch-menu-dots" @click.stop="$emit('stash-context', $event, stash)" title="Stash actions">
                <EllipsisVerticalIcon viewBox="0 0 24 24" width="13" height="13" stroke="currentColor" stroke-width="2" fill="none" />
              </div>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import SearchIcon from '../../assets/icons/search.svg?component';
import CloseIcon from '../../assets/icons/close.svg?component';
import ListIcon from '../../assets/icons/list.svg?component';
import TimelineIcon from '../../assets/icons/timeline.svg?component';
import ChevronDownIcon from '../../assets/icons/chevron-down.svg?component';
import BranchIcon from '../../assets/icons/branch.svg?component';
import CheckIcon from '../../assets/icons/check.svg?component';
import MoreVerticalIcon from '../../assets/icons/more-vertical.svg?component';
import FolderIcon from '../../assets/icons/folder.svg?component';
import PlusIcon from '../../assets/icons/plus.svg?component';
import CloudIcon from '../../assets/icons/Cloud.svg?component';
import TagIcon from '../../assets/icons/tag.svg?component';
import StashIcon from '../../assets/icons/stash.svg?component';
import EllipsisVerticalIcon from '../../assets/icons/EllipsisVertical.svg?component';
import FetchIcon from '../../assets/icons/fetch.svg?component';
import LockIcon from '../../assets/icons/lock.svg?component';
import WorktreeIcon from '../../assets/icons/worktree.svg?component';
import SubmoduleIcon from '../../assets/icons/submodule.svg?component';
import { ref, computed, watch } from 'vue';

const props = defineProps({
  project: {
    type: Object,
    required: true
  },
  width: {
    type: Number,
    required: true
  }
});

const emit = defineEmits([
  'create-branch',
  'branch-dblclick',
  'branch-context',
  'item-click',
  'remote-context',
  'add-remote',
  'stash-context',
  'create-tag',
  'tag-context',
  'create-worktree',
  'worktree-context',
  'open-worktree',
  'prune-worktrees',
  'add-submodule',
  'submodule-context',
  'open-submodule',
  'sync-all-submodules'
]);

const branchViewMode = ref<'flat' | 'tree'>('tree');
const collapsedBranchDirs = ref<Set<string>>(new Set());
const expandedBranchDirs = ref<Set<string>>(new Set());
const collapsedSections = ref<Set<string>>(new Set(['worktrees', 'submodules']));
const searchQuery = ref('');

function escapeHtml(text: any) {
  if (text == null) return '';
  return String(text).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function highlightMatch(text: any) {
  if (text == null) return '';
  const str = String(text);
  if (!searchQuery.value) return escapeHtml(str);
  const q = searchQuery.value;
  const escapedText = escapeHtml(str);
  const escapedQuery = q.replace(/[-\/\\^$*+?.()|[\]{}]/g, '\\$&');
  const regex = new RegExp(`(${escapedQuery})`, 'gi');
  return escapedText.replace(regex, '<span class="highlight-match">$1</span>');
}

const filteredLocalBranches = computed(() => {
  if (!searchQuery.value) return props.project.localBranches || [];
  const q = searchQuery.value.toLowerCase();
  return (props.project.localBranches || []).filter((b: any) => b.name.toLowerCase().includes(q));
});

const filteredRemoteBranches = computed(() => {
  if (!searchQuery.value) return props.project.remoteBranches || [];
  const q = searchQuery.value.toLowerCase();
  return (props.project.remoteBranches || []).filter((b: any) => b.name.toLowerCase().includes(q));
});

const filteredTags = computed(() => {
  if (!searchQuery.value) return props.project.tags || [];
  const q = searchQuery.value.toLowerCase();
  return (props.project.tags || []).filter((t: string) => t.toLowerCase().includes(q));
});

const filteredWorktrees = computed(() => {
  if (!searchQuery.value) return props.project.worktrees || [];
  const q = searchQuery.value.toLowerCase();
  return (props.project.worktrees || []).filter((wt: any) =>
    (wt.branch && wt.branch.toLowerCase().includes(q)) ||
    (wt.path && wt.path.toLowerCase().includes(q))
  );
});

const filteredSubmodules = computed(() => {
  if (!searchQuery.value) return props.project.submodules || [];
  const q = searchQuery.value.toLowerCase();
  return (props.project.submodules || []).filter((sm: any) =>
    (sm.name && sm.name.toLowerCase().includes(q)) ||
    (sm.path && sm.path.toLowerCase().includes(q)) ||
    (sm.url && sm.url.toLowerCase().includes(q))
  );
});

const filteredStashes = computed(() => {
  if (!searchQuery.value) return props.project.stashes || [];
  const q = searchQuery.value.toLowerCase();
  return (props.project.stashes || []).filter((s: any) => s.message.toLowerCase().includes(q) || s.id.toLowerCase().includes(q));
});

const totalMatchCount = computed(() => {
  if (!searchQuery.value) return 0;
  return (
    filteredLocalBranches.value.length +
    filteredRemoteBranches.value.length +
    filteredTags.value.length +
    filteredWorktrees.value.length +
    filteredSubmodules.value.length +
    filteredStashes.value.length
  );
});

const selectedRemoteName = ref('');

watch(() => props.project.remotes, (remotes) => {
  if (remotes && remotes.length > 0 && !selectedRemoteName.value) {
    selectedRemoteName.value = remotes[0].name;
  } else if (!remotes || remotes.length === 0) {
    selectedRemoteName.value = '';
  }
}, { immediate: true });

function openSelectedRemoteContext(event: MouseEvent) {
  if (!selectedRemoteName.value || !props.project.remotes) return;
  const remote = props.project.remotes.find((r: any) => r.name === selectedRemoteName.value);
  if (remote) {
    emit('remote-context', event, remote);
  }
}

function toggleSection(section: string) {
  if (collapsedSections.value.has(section)) {
    collapsedSections.value.delete(section);
  } else {
    collapsedSections.value.add(section);
  }
}

function isDirCollapsed(path: string) {
  if (path === 'tags/' || path === 'local/' || path === 'remote/') return false;
  if (path.startsWith('tags/')) {
    return !expandedBranchDirs.value.has(path);
  }
  return collapsedBranchDirs.value.has(path);
}

function toggleBranchDir(path: string) {
  if (path.startsWith('tags/')) {
    if (expandedBranchDirs.value.has(path)) {
      expandedBranchDirs.value.delete(path);
    } else {
      expandedBranchDirs.value.add(path);
    }
  } else {
    if (collapsedBranchDirs.value.has(path)) {
      collapsedBranchDirs.value.delete(path);
    } else {
      collapsedBranchDirs.value.add(path);
    }
  }
}

function buildBranchTree(branchesList: Array<any>, prefix: string, stripOrigin: boolean = false, sortDescending: boolean = false) {
  const root = { name: '', path: prefix, isDir: true, children: [] as any[] };
  
  branchesList.forEach(b => {
    let bName = b.name;
    if (stripOrigin && bName.startsWith('origin/')) {
      bName = bName.substring(7);
    }
    
    const parts = bName.split('/');
    let current = root;
    
    parts.forEach((part: string, index: number) => {
      const isLast = index === parts.length - 1;
      
      if (!current.children) {
        current.children = [];
        current.isDir = true;
      }

      let child = current.children.find((c: any) => c.name === part);
      
      if (!child) {
        child = {
          name: part,
          path: prefix + parts.slice(0, index + 1).join('/'),
          isDir: !isLast,
          children: isLast ? undefined : [],
          branch: isLast ? b : undefined,
        };
        current.children.push(child);
      } else if (isLast) {
        child.branch = b;

      }
      
      current = child;
    });
  });
  
  const countLeaves = (node: any) => {
    if (!node.isDir) return 1;
    let count = 0;
    if (node.children) {
      node.children.forEach((c: any) => {
        count += countLeaves(c);
      });
    }
    node.leafCount = count;
    return count;
  };
  countLeaves(root);
  
  const sortTree = (node: any) => {
    if (node.children) {
      node.children.sort((a: any, b: any) => {
        if (a.isDir !== b.isDir) return b.isDir ? -1 : 1;
        const cmp = a.name.localeCompare(b.name, undefined, { numeric: true });
        return sortDescending ? -cmp : cmp;
      });
      node.children.forEach(sortTree);
    }
  };
  sortTree(root);
  
  const result: any[] = [];
  const flatten = (node: any, depth = -1) => {
    if (depth >= 0) {
      result.push({ ...node, depth });
    }
    if (node.isDir && !isDirCollapsed(node.path) && node.children) {
      node.children.forEach((child: any) => flatten(child, depth + 1));
    }
  };
  
  flatten(root);
  return result;
}

const treeLocalBranches = computed(() => {
  return buildBranchTree(filteredLocalBranches.value, 'local/');
});

const treeRemoteBranches = computed(() => {
  return buildBranchTree(filteredRemoteBranches.value, 'remote/', true);
});

const treeTags = computed(() => {
  const tagObjects = filteredTags.value.map((t: string) => {
    let simulatedPath = t;
    if (t.includes('@')) {
      const parts = t.split('@');
      simulatedPath = parts[0] + '/' + parts.slice(1).join('@');
    } else if (t.includes('-')) {
      const lastIndex = t.lastIndexOf('-');
      if (lastIndex !== -1) {
        simulatedPath = t.substring(0, lastIndex) + '/' + t.substring(lastIndex + 1);
      }
    }
    return { name: simulatedPath, originalName: t };
  });
  return buildBranchTree(tagObjects, 'tags/', false, true);
});
</script>

<style scoped>
/* Sidebar Container */
.left-sidebar {
  background-color: var(--panel-bg, #161b22);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  border-right: 1px solid var(--border, rgba(128, 128, 128, 0.18));
  user-select: none;
}

/* Header & Search */
.sidebar-header {
  padding: 10px 10px 8px 10px;
  border-bottom: 1px solid var(--border, rgba(128, 128, 128, 0.15));
}

.search-container {
  position: relative;
  display: flex;
  align-items: center;
  width: 100%;
}

.sidebar-search-input {
  width: 100%;
  background: var(--bg-main, #0d1117);
  border: 1px solid var(--border, rgba(128, 128, 128, 0.22));
  border-radius: 6px;
  padding: 5px 24px 5px 26px;
  color: var(--text-main, #e6edf3);
  font-size: 0.78rem;
  outline: none;
  transition: all 0.15s ease;
}

.sidebar-search-input:focus {
  border-color: var(--accent-blue, #58a6ff);
  box-shadow: 0 0 0 2px rgba(88, 166, 255, 0.18);
  background: var(--bg-main, #0d1117);
}

.sidebar-search-input::placeholder {
  color: var(--text-muted, #8b949e);
  font-size: 0.76rem;
}

.search-container .search-icon {
  position: absolute;
  left: 8px;
  color: var(--text-muted, #8b949e);
  pointer-events: none;
  transition: color 0.15s ease;
}

.search-container:focus-within .search-icon {
  color: var(--accent-blue, #58a6ff);
}

.search-count-tag {
  position: absolute;
  right: 24px;
  font-size: 10px;
  font-weight: 600;
  padding: 1px 5px;
  border-radius: 4px;
  background: rgba(88, 166, 255, 0.15);
  color: var(--accent-blue, #58a6ff);
  pointer-events: none;
}

.clear-search-btn-sidebar {
  position: absolute;
  right: 5px;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  background: transparent;
  border: none;
  border-radius: 50%;
  color: var(--text-muted, #8b949e);
  cursor: pointer;
  transition: all 0.15s ease;
}

.clear-search-btn-sidebar:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

/* Scrollable area */
.sidebar-scrollable {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding-bottom: 24px;
  scrollbar-width: thin;
}

/* Segmented Control */
.view-mode-container {
  padding: 8px 10px 8px 10px;
}

.segmented-control {
  display: flex;
  background: var(--surface-inset, var(--bg-main));
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px;
  gap: 2px;
}

.segmented-btn {
  flex: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  padding: 4px 8px;
  border-radius: 4px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 0.74rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
}

.segmented-btn:hover:not(.active) {
  color: var(--text-main);
}

.segmented-btn.active {
  background: var(--bg-card);
  color: var(--text-main);
  font-weight: 600;
  box-shadow: 0 1px 3px var(--shadow-color, rgba(0, 0, 0, 0.15));
}

/* ==========================================================================
   NATIVE SECTION ACCORDION WITH THEMATIC HEADER BACKGROUNDS
   ========================================================================== */
.sidebar-section {
  margin: 0;
  padding: 0;
  border: none;
  border-top: 1px solid var(--border, rgba(128, 128, 128, 0.15));
  background: transparent;
}

/* Themed Section Header Bars */
.section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 12px;
  cursor: pointer;
  user-select: none;
  position: sticky;
  top: 0;
  z-index: 3;
  backdrop-filter: blur(8px);
  border-bottom: 1px solid var(--border, rgba(128, 128, 128, 0.12));
  transition: filter 0.15s ease, background 0.15s ease;
}

.sidebar-section.is-collapsed .section-title {
  border-bottom: 1px solid transparent;
}

.section-title:hover {
  filter: brightness(1.2);
}

/* Thematic Background Gradients per Section */
.section-local .section-title {
  background: linear-gradient(90deg, rgba(0, 188, 212, 0.12) 0%, rgba(0, 188, 212, 0.03) 60%, var(--panel-bg) 100%);
  border-bottom-color: rgba(0, 188, 212, 0.2);
}

.section-remote .section-title {
  background: linear-gradient(90deg, rgba(76, 175, 80, 0.12) 0%, rgba(76, 175, 80, 0.03) 60%, var(--panel-bg) 100%);
  border-bottom-color: rgba(76, 175, 80, 0.2);
}

.section-tags .section-title {
  background: linear-gradient(90deg, rgba(255, 152, 0, 0.12) 0%, rgba(255, 152, 0, 0.03) 60%, var(--panel-bg) 100%);
  border-bottom-color: rgba(255, 152, 0, 0.2);
}

.section-worktrees .section-title {
  background: linear-gradient(90deg, rgba(38, 198, 218, 0.12) 0%, rgba(38, 198, 218, 0.03) 60%, var(--panel-bg) 100%);
  border-bottom-color: rgba(38, 198, 218, 0.2);
}

.section-submodules .section-title {
  background: linear-gradient(90deg, rgba(171, 71, 188, 0.12) 0%, rgba(171, 71, 188, 0.03) 60%, var(--panel-bg) 100%);
  border-bottom-color: rgba(171, 71, 188, 0.2);
}

.section-stashes .section-title {
  background: linear-gradient(90deg, rgba(236, 64, 122, 0.12) 0%, rgba(236, 64, 122, 0.03) 60%, var(--panel-bg) 100%);
  border-bottom-color: rgba(236, 64, 122, 0.2);
}

.title-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

/* Thematic Icon Boxes */
.section-icon-box {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  flex-shrink: 0;
}

.icon-box-local {
  background: rgba(0, 188, 212, 0.2);
  color: #00bcd4;
}

.icon-box-remote {
  background: rgba(76, 175, 80, 0.2);
  color: #4caf50;
}

.icon-box-tags {
  background: rgba(255, 152, 0, 0.2);
  color: #ff9800;
}

.icon-box-worktrees {
  background: rgba(38, 198, 218, 0.2);
  color: #26c6da;
}

.icon-box-submodules {
  background: rgba(171, 71, 188, 0.2);
  color: #ab47bc;
}

.icon-box-stashes {
  background: rgba(236, 64, 122, 0.2);
  color: #ec407a;
}

.section-name {
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: var(--text-main, #e6edf3);
}

.count-badge {
  font-size: 10px;
  font-weight: 700;
  padding: 1px 6px;
  border-radius: 10px;
  line-height: 1.2;
}

.count-local {
  background: rgba(0, 188, 212, 0.15);
  color: #00bcd4;
  border: 1px solid rgba(0, 188, 212, 0.3);
}

.count-remote {
  background: rgba(76, 175, 80, 0.15);
  color: #4caf50;
  border: 1px solid rgba(76, 175, 80, 0.3);
}

.count-tags {
  background: rgba(255, 152, 0, 0.15);
  color: #ff9800;
  border: 1px solid rgba(255, 152, 0, 0.3);
}

.count-worktrees {
  background: rgba(38, 198, 218, 0.15);
  color: #26c6da;
  border: 1px solid rgba(38, 198, 218, 0.3);
}

.count-submodules {
  background: rgba(171, 71, 188, 0.15);
  color: #ab47bc;
  border: 1px solid rgba(171, 71, 188, 0.3);
}

.count-stashes {
  background: rgba(236, 64, 122, 0.15);
  color: #ec407a;
  border: 1px solid rgba(236, 64, 122, 0.3);
}

.title-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.chevron-wrapper {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--text-muted, #8b949e);
}

.chevron-wrapper.is-collapsed {
  transform: rotate(-90deg);
}

.section-title:hover .chevron-wrapper {
  color: var(--text-main, #ffffff);
}

.header-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  border-radius: 4px;
  border: none;
  background: var(--surface-subtle);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s ease;
}

.header-action-btn:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

/* Section Body */
.section-body {
  padding: 4px 6px 10px 6px;
  background: transparent;
}

/* Remote Selector Bar */
.remote-selector-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin: 4px 4px 6px 4px;
  padding: 4px 8px;
  background: var(--input-bg);
  border: 1px solid var(--border);
  border-radius: 6px;
}

.remote-pill-label {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.remote-select-native {
  flex: 1;
  background: transparent;
  color: var(--text-main);
  border: none;
  font-size: 11px;
  font-weight: 600;
  outline: none;
  cursor: pointer;
}

.remote-actions-group {
  display: flex;
  align-items: center;
  gap: 2px;
}

.remote-sub-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  border-radius: 3px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.remote-sub-btn:hover {
  background: var(--surface-hover);
  color: var(--text-main);
}

/* Branch List & Items */
.branch-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.branch-item {
  position: relative;
  display: flex;
  align-items: center;
  padding: 5px 8px;
  gap: 8px;
  font-size: 0.82rem;
  border-radius: 5px;
  color: var(--text-main);
  cursor: pointer;
  transition: background-color 0.12s ease;
  margin-bottom: 2px;
}

.branch-item:hover {
  background-color: var(--surface-hover);
}

.branch-item.active-branch {
  background-color: var(--row-selected, rgba(0, 188, 212, 0.14));
  font-weight: 600;
  color: var(--text-main);
}

.active-indicator-bar {
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 3px;
  background: var(--accent, #00bcd4);
  border-radius: 0 3px 3px 0;
  box-shadow: 0 0 6px var(--accent, #00bcd4);
}

.branch-icon {
  flex-shrink: 0;
  color: var(--text-muted);
  transition: color 0.12s ease;
}

.active-branch .branch-icon {
  color: var(--accent, #00bcd4);
}

.branch-local-icon {
  color: #00bcd4;
  opacity: 0.85;
}

.branch-remote-icon {
  color: #4caf50;
  opacity: 0.85;
}

.branch-tag-icon {
  color: #ff9800;
  opacity: 0.85;
}

.branch-worktree-icon {
  color: #26c6da;
  opacity: 0.85;
}

.branch-submodule-icon {
  color: #ab47bc;
  opacity: 0.85;
}

.branch-stash-icon {
  color: #ec407a;
  opacity: 0.85;
}

.folder-icon {
  color: #90a4ae;
  opacity: 0.9;
}

.branch-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
}

:deep(.highlight-match) {
  background: rgba(255, 193, 7, 0.25);
  color: #ffeb3b;
  border-radius: 2px;
  padding: 0 1px;
}

/* Sync Badges */
.branch-sync {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 0.68rem;
  font-family: var(--font-mono, monospace);
  font-weight: 700;
  flex-shrink: 0;
}

.sync-badge {
  padding: 1px 5px;
  border-radius: 4px;
  line-height: 1.2;
}

.sync-ahead {
  color: #4caf50;
  background: rgba(76, 175, 80, 0.15);
  border: 1px solid rgba(76, 175, 80, 0.3);
}

.sync-behind {
  color: #f44336;
  background: rgba(244, 67, 54, 0.15);
  border: 1px solid rgba(244, 67, 54, 0.3);
}

.active-pill {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: rgba(76, 175, 80, 0.2);
  color: #4caf50;
  border: 1px solid rgba(76, 175, 80, 0.4);
  flex-shrink: 0;
}

/* Menu Dots Trigger */
.branch-menu-dots {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  opacity: 0;
  color: var(--text-muted, #8b949e);
  transition: all 0.15s ease;
  margin-left: auto;
  flex-shrink: 0;
}

.branch-item:hover .branch-menu-dots {
  opacity: 0.7;
}

.branch-menu-dots:hover {
  opacity: 1 !important;
  background: var(--surface-hover);
  color: var(--text-main);
}

/* Tree specific */
.tree-toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 12px;
  flex-shrink: 0;
}

.tree-toggle.placeholder {
  width: 4px;
}

.dir-count-pill {
  font-size: 9px;
  font-weight: 600;
  padding: 1px 5px;
  border-radius: 8px;
  background: var(--surface-subtle);
  color: var(--text-muted);
  margin-left: 6px;
}

/* Worktree and Submodule Badges */
.worktree-main-badge {
  font-size: 9px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #26c6da;
  background: rgba(38, 198, 218, 0.12);
  border: 1px solid rgba(38, 198, 218, 0.3);
  padding: 1px 5px;
  border-radius: 4px;
}

.worktree-lock-badge {
  color: #ff9800;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.submodule-status-badge {
  font-size: 9px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 10px;
  text-transform: capitalize;
}

.submodule-status-badge.status-initialized {
  color: #4caf50;
  background: rgba(76, 175, 80, 0.12);
  border: 1px solid rgba(76, 175, 80, 0.25);
}

.submodule-status-badge.status-uninitialized {
  color: #ff9800;
  background: rgba(255, 152, 0, 0.12);
  border: 1px solid rgba(255, 152, 0, 0.25);
}

.submodule-status-badge.status-modified {
  color: #2196f3;
  background: rgba(33, 150, 243, 0.12);
  border: 1px solid rgba(33, 150, 243, 0.25);
}

.submodule-status-badge.status-conflict {
  color: #ff5252;
  background: rgba(255, 82, 82, 0.12);
  border: 1px solid rgba(255, 82, 82, 0.25);
}

/* Empty states */
.sidebar-empty-state {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  margin: 2px 4px;
  font-size: 0.74rem;
  color: var(--text-muted);
  font-style: italic;
  border-radius: 4px;
  background: var(--surface-subtle);
}

.empty-action-link {
  font-size: 0.72rem;
  font-weight: 600;
  font-style: normal;
  border: none;
  background: transparent;
  color: var(--accent, #00bcd4);
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 4px;
  transition: all 0.12s ease;
}

.empty-action-link:hover {
  background: rgba(0, 188, 212, 0.12);
  text-decoration: underline;
}
</style>
