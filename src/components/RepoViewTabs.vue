<script lang="ts">
export type RepoViewTab = "commits" | "terminal" | "branches" | "remotes" | "tags" | "stashes";
</script>

<script setup lang="ts">
import BranchIcon from "./BranchIcon.vue";

defineProps<{
  active: RepoViewTab;
  busy: boolean;
  branchCount: number;
  remoteCount: number;
  tagCount: number;
  stashCount: number;
}>();

const emit = defineEmits<{
  select: [tab: RepoViewTab];
}>();
</script>

<template>
  <div class="view-tabs repo-view-tabs" role="tablist" aria-label="Repository views">
    <button
      class="view-tab"
      :class="{ active: active === 'commits' }"
      type="button"
      role="tab"
      :aria-selected="active === 'commits'"
      @click="emit('select', 'commits')"
    >
      <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="3" />
        <path d="M12 3v6m0 6v6" />
      </svg>
      Commits
    </button>
    <button
      class="view-tab"
      :class="{ active: active === 'branches' }"
      type="button"
      role="tab"
      :disabled="busy"
      :aria-selected="active === 'branches'"
      @click="emit('select', 'branches')"
    >
      <BranchIcon />
      Branches
      <span v-if="branchCount" class="file-count-badge">{{ branchCount }}</span>
    </button>
    <button
      class="view-tab"
      :class="{ active: active === 'remotes' }"
      type="button"
      role="tab"
      :disabled="busy"
      :aria-selected="active === 'remotes'"
      title="Remotes and their branches"
      @click="emit('select', 'remotes')"
    >
      <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M12 21a9.004 9.004 0 008.716-6.747M12 21a9.004 9.004 0 01-8.716-6.747M12 21c2.485 0 4.5-4.03 4.5-9S14.485 3 12 3m0 18c-2.485 0-4.5-4.03-4.5-9S9.515 3 12 3m0 0a8.997 8.997 0 017.843 4.582M12 3a8.997 8.997 0 00-7.843 4.582m15.686 0A11.953 11.953 0 0112 10.5c-2.998 0-5.74-1.1-7.843-2.918m15.686 0A8.959 8.959 0 0121 12c0 .778-.099 1.533-.284 2.253m0 0A17.919 17.919 0 0112 16.5c-3.162 0-6.133-.815-8.716-2.247m0 0A9.015 9.015 0 013 12c0-1.605.42-3.113 1.157-4.418"
        />
      </svg>
      Remotes
      <span v-if="remoteCount" class="file-count-badge">{{ remoteCount }}</span>
    </button>
    <button
      class="view-tab"
      :class="{ active: active === 'tags' }"
      type="button"
      role="tab"
      :disabled="busy"
      :aria-selected="active === 'tags'"
      @click="emit('select', 'tags')"
    >
      <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M9.568 3H5.25A2.25 2.25 0 003 5.25v4.318c0 .597.237 1.17.659 1.591l9.581 9.581c.699.699 1.78.872 2.607.33a18.095 18.095 0 005.223-5.223c.542-.827.369-1.908-.33-2.607L11.16 3.66A2.25 2.25 0 009.568 3Z"
        />
        <path d="M6 6h.01" />
      </svg>
      Tags
      <span v-if="tagCount" class="file-count-badge">{{ tagCount }}</span>
    </button>
    <button
      class="view-tab"
      :class="{ active: active === 'stashes' }"
      type="button"
      role="tab"
      :disabled="busy"
      :aria-selected="active === 'stashes'"
      @click="emit('select', 'stashes')"
    >
      <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125Z"
        />
      </svg>
      Stashes
      <span v-if="stashCount" class="file-count-badge">{{ stashCount }}</span>
    </button>
    <button
      class="view-tab view-tab-end"
      :class="{ active: active === 'terminal' }"
      type="button"
      role="tab"
      :aria-selected="active === 'terminal'"
      title="Terminal in this repository"
      @click="emit('select', 'terminal')"
    >
      <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M6.75 7.5l3 2.25-3 2.25m4.5 0h3m-9 8.25h13.5A2.25 2.25 0 0021 18V6a2.25 2.25 0 00-2.25-2.25H3.75A2.25 2.25 0 001.5 6v12a2.25 2.25 0 002.25 2.25z"
        />
      </svg>
      Terminal
    </button>
  </div>
</template>
