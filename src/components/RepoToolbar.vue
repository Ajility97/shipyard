<script setup lang="ts">
import { computed } from "vue";
import BranchIcon from "./BranchIcon.vue";
import ChangesToggle from "./ChangesToggle.vue";
import FileHistoryToggle from "./FileHistoryToggle.vue";
import SplitAction from "./SplitAction.vue";
import { useApp } from "../composables/useApp";
import { useOverflowMenu } from "../composables/useOverflowMenu";

const props = defineProps<{
  repoId: string;
  name: string;
  branch: string;
  path: string;
  branches: string[];
  busy: boolean;
  busyLabel: string;
  busyBranch?: string;
  branchesView: boolean;
  stashView: boolean;
  stashCount: number;
  filesOpen: boolean;
  historyOpen: boolean;
  unstagedCount: number;
  stagedCount: number;
  conflictedCount?: number;
  terminalOpen: boolean;
}>();

const emit = defineEmits<{
  pull: [];
  pullOptions: [];
  push: [];
  checkout: [branch: string];
  create: [];
  branches: [];
  stash: [];
  files: [];
  history: [];
  refreshBranches: [];
  terminal: [];
}>();

const { statuses } = useApp();
const { isOpen, toggle, close } = useOverflowMenu(() => `branch-${props.repoId}`);

const currentBranch = computed(
  () => statuses.value[props.repoId]?.branch || props.branch,
);

const pullTitle = computed(() =>
  currentBranch.value ? `Pull from ${currentBranch.value}` : "Pull from current branch",
);

const pushTitle = computed(() =>
  currentBranch.value ? `Push to ${currentBranch.value}` : "Push current branch",
);

const progressLabel = computed(() => props.busyLabel.replace(/…$/, "").trim());
const progressBranch = computed(() => props.busyBranch?.trim() || "");

function selectBranch(branch: string) {
  close();
  if (branch === props.branch) {
    return;
  }
  emit("checkout", branch);
}

async function toggleBranches() {
  if (!isOpen.value) {
    emit("refreshBranches");
  }
  toggle();
}
</script>

<template>
  <div class="pane-header repo-toolbar">
    <div class="repo-toolbar-meta">
      <strong class="repo-toolbar-name">{{ name }}</strong>
      <div class="overflow-menu branch-menu">
        <button
          class="branch-switch"
          type="button"
          :disabled="busy"
          :aria-expanded="isOpen"
          aria-haspopup="listbox"
          :title="branch ? `Switch branch from ${branch}` : 'Switch branch'"
          @click="toggleBranches"
        >
          <span class="branch-switch-name">{{ branch || "No branch" }}</span>
          <svg viewBox="0 0 16 16" aria-hidden="true">
            <path d="M4.2 6.2L8 10l3.8-3.8" />
          </svg>
        </button>
        <div
          v-if="isOpen"
          class="overflow-menu-dropdown branch-menu-dropdown"
          role="listbox"
          aria-label="Local branches"
        >
          <p v-if="!branches.length" class="muted tiny empty-branches">No local branches.</p>
          <button
            v-for="item in branches"
            :key="item"
            class="overflow-menu-item"
            :class="{ active: item === branch }"
            type="button"
            role="option"
            :aria-selected="item === branch"
            @click="selectBranch(item)"
          >
            {{ item }}
          </button>
        </div>
      </div>
      <span class="repo-path" :title="path">{{ path }}</span>
    </div>
    <div class="repo-toolbar-bar repo-toolbar-actions">
      <div class="repo-toolbar-work">
        <button class="ghost tiny" type="button" :disabled="busy" @click="emit('create')">
          <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M6 3v12a3 3 0 0 0 3 3h4.5" />
            <circle cx="6" cy="5" r="2" />
            <circle cx="6" cy="19" r="2" />
            <path d="M15 6h6M18 3v6" />
          </svg>
          New branch
        </button>
        <SplitAction
          :primary-title="pullTitle"
          more-title="Pull from another branch"
          :disabled="busy"
          @primary="emit('pull')"
          @more="emit('pullOptions')"
        >
          <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3"
            />
          </svg>
          Pull
        </SplitAction>
        <button
          class="ghost tiny"
          type="button"
          :disabled="busy"
          :title="pushTitle"
          @click="emit('push')"
        >
          <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M7.5 7.5 12 3m0 0 4.5 4.5M12 3v13.5"
            />
          </svg>
          Push
        </button>
        <span v-if="busyLabel" class="action-progress">
          {{ progressLabel }}
          <span v-if="progressBranch" class="action-branch-badge" :title="progressBranch">
            <BranchIcon />
            <span class="action-branch-name">{{ progressBranch }}</span>
          </span>
          <span class="spinner" aria-hidden="true" />
        </span>
      </div>
      <div class="repo-toolbar-views">
        <button
          class="ghost tiny"
          :class="{ active: terminalOpen }"
          type="button"
          :aria-pressed="terminalOpen"
          title="Open a terminal in this repository"
          @click="emit('terminal')"
        >
          <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H3.75A2.25 2.25 0 001.5 6v12a2.25 2.25 0 002.25 2.25z"
            />
          </svg>
          Terminal
        </button>
        <button
          class="ghost tiny"
          :class="{ active: branchesView }"
          type="button"
          :disabled="busy"
          :aria-pressed="branchesView"
          @click="emit('branches')"
        >
          <BranchIcon />
          Branches
          <span v-if="branches.length" class="file-count-badge">{{ branches.length }}</span>
        </button>
        <button
          class="ghost tiny"
          :class="{ active: stashView }"
          type="button"
          :disabled="busy"
          :aria-pressed="stashView"
          @click="emit('stash')"
        >
          <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path
              d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125Z"
            />
          </svg>
          Stashes
          <span v-if="stashCount" class="file-count-badge">{{ stashCount }}</span>
        </button>
        <FileHistoryToggle :open="historyOpen" @click="emit('history')" />
        <ChangesToggle
          :open="filesOpen"
          :unstaged="unstagedCount"
          :staged="stagedCount"
          :conflicted="conflictedCount"
          @click="emit('files')"
        />
        <slot />
      </div>
    </div>
  </div>
</template>
