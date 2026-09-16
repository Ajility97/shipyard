<script setup lang="ts">
import { computed } from "vue";
import BranchIcon from "./BranchIcon.vue";
import type { BranchOverview, LocalBranch } from "../types";

const props = defineProps<{
  overview: BranchOverview | null;
  busy: boolean;
}>();

const emit = defineEmits<{
  checkout: [branch: LocalBranch];
  rename: [branch: LocalBranch];
  delete: [branch: LocalBranch];
  deleteMerged: [];
}>();

const leftoverCount = computed(
  () =>
    props.overview?.branches.filter(
      (branch) => branch.merged && !branch.current && !branch.protected,
    ).length ?? 0,
);

function isLeftover(branch: LocalBranch) {
  return branch.merged && !branch.protected;
}

function isPartial(branch: LocalBranch) {
  return branch.partial && !branch.protected;
}
</script>

<template>
  <div class="branch-pane">
    <div class="graph-scroll branch-list">
      <p v-if="!overview" class="muted tiny branch-list-hint">Loading branches…</p>
      <p v-else-if="overview.mergeTarget" class="muted tiny branch-list-hint">
        Merged marks leftover local work already contained in
        <strong>{{ overview.mergeTarget }}</strong>. Partial means some commits are in that
        branch and some are still unique. Pull first if you want the latest remote picture.
      </p>
      <p v-else class="muted tiny branch-list-hint">
        Couldn’t find origin/develop, develop, main, or master to compare against.
      </p>
      <p v-if="overview && !overview.branches.length" class="muted tiny empty-files">
        No local branches.
      </p>
      <div
        v-for="branch in overview?.branches ?? []"
        :key="branch.name"
        class="branch-row"
        :class="{
          current: branch.current,
          leftover: isLeftover(branch),
          partial: isPartial(branch),
        }"
      >
        <BranchIcon />
        <span class="branch-row-name">{{ branch.name }}</span>
        <span v-if="branch.current" class="branch-pill">Current</span>
        <span
          v-if="isLeftover(branch)"
          class="branch-pill merged"
          :title="
            overview?.mergeTarget
              ? `Already contained in ${overview.mergeTarget}`
              : 'Already contained in the integration branch'
          "
        >
          Merged
        </span>
        <span
          v-else-if="isPartial(branch)"
          class="branch-pill partial"
          :title="
            overview?.mergeTarget
              ? `Some commits are in ${overview.mergeTarget}; others are still unique`
              : 'Some commits are already merged; others are still unique'
          "
        >
          Partial
        </span>
        <div class="branch-row-actions">
          <button
            class="ghost tiny"
            type="button"
            :disabled="busy || branch.current"
            :title="branch.current ? 'Already on this branch' : `Check out ${branch.name}`"
            @click="emit('checkout', branch)"
          >
            <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M7.5 21 3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5"
              />
            </svg>
            Checkout
          </button>
          <button
            class="ghost tiny"
            type="button"
            :disabled="busy"
            :title="`Rename ${branch.name}`"
            @click="emit('rename', branch)"
          >
            Rename
          </button>
          <button
            class="ghost tiny danger"
            type="button"
            :disabled="busy || branch.current"
            :title="branch.current ? 'Cannot delete the current branch' : `Delete ${branch.name}`"
            @click="emit('delete', branch)"
          >
            Delete
          </button>
        </div>
      </div>
    </div>
    <div class="branch-footer">
      <button
        class="ghost tiny danger"
        type="button"
        :disabled="busy || leftoverCount === 0"
        @click="emit('deleteMerged')"
      >
        Delete merged branches
        <span v-if="leftoverCount" class="file-count-badge">{{ leftoverCount }}</span>
      </button>
      <p class="muted tiny branch-footer-hint">
        Only leftover merged branches. Partial and unique work stay. Keeps develop, main, master,
        and the branch you’re on.
      </p>
    </div>
  </div>
</template>
