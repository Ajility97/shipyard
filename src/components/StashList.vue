<script setup lang="ts">
import type { StashEntry } from "../types";
import { formatCommitDate } from "../graphLayout";

defineProps<{
  stashes: StashEntry[];
  busy: boolean;
}>();

const emit = defineEmits<{
  apply: [stash: StashEntry];
  pop: [stash: StashEntry];
  drop: [stash: StashEntry];
}>();

function stashRef(index: number) {
  return `stash@{${index}}`;
}
</script>

<template>
  <div class="branch-pane">
    <div class="graph-scroll branch-list">
      <p class="muted tiny branch-list-hint">
        Apply keeps the stash. Pop applies it and removes it. Drop deletes it without applying.
      </p>
      <p v-if="!stashes.length" class="muted tiny empty-files">No stashes.</p>
      <div v-for="stash in stashes" :key="stash.index" class="branch-row stash-row">
        <span class="branch-pill">{{ stashRef(stash.index) }}</span>
        <span class="branch-row-name" :title="stash.message">{{ stash.message }}</span>
        <span class="stash-row-date">{{ formatCommitDate(stash.date) }}</span>
        <div class="stash-row-actions">
          <button
            class="ghost tiny"
            type="button"
            :disabled="busy"
            :title="`Apply ${stashRef(stash.index)}`"
            @click="emit('apply', stash)"
          >
            Apply
          </button>
          <button
            class="ghost tiny"
            type="button"
            :disabled="busy"
            :title="`Pop ${stashRef(stash.index)}`"
            @click="emit('pop', stash)"
          >
            Pop
          </button>
          <button
            class="ghost tiny danger"
            type="button"
            :disabled="busy"
            :title="`Drop ${stashRef(stash.index)}`"
            @click="emit('drop', stash)"
          >
            Drop
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
