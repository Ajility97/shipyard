<script setup lang="ts">
import { computed } from "vue";
import { conflictCountLabel } from "../gitOperation";

const props = defineProps<{
  historyOpen: boolean;
  unstaged: number;
  staged: number;
  conflicted?: number;
}>();

const emit = defineEmits<{
  changes: [];
  history: [];
}>();

const changesTitle = computed(() => {
  const parts = [];
  if (props.conflicted) {
    parts.push(conflictCountLabel(props.conflicted));
  }
  if (props.unstaged) {
    parts.push(`${props.unstaged} unstaged`);
  }
  if (props.staged) {
    parts.push(`${props.staged} staged`);
  }
  return parts.length ? parts.join(" · ") : "No local changes";
});
</script>

<template>
  <div class="view-tabs changes-panel-tabs" role="tablist" aria-label="Side panel">
    <button
      class="view-tab"
      :class="{ active: !historyOpen }"
      type="button"
      role="tab"
      :aria-selected="!historyOpen"
      :title="changesTitle"
      @click="emit('changes')"
    >
      <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
        <rect x="3.75" y="4.5" width="16.5" height="15" rx="2" />
        <path d="M14.25 4.5v15" />
      </svg>
      Changes
      <span
        v-if="conflicted"
        class="file-count-badge conflicted"
        :title="conflictCountLabel(conflicted)"
      >{{ conflicted }}</span>
      <span v-if="unstaged" class="file-count-badge unstaged" title="Unstaged">{{ unstaged }}</span>
      <span v-if="staged" class="file-count-badge staged" title="Staged">{{ staged }}</span>
    </button>
    <button
      class="view-tab"
      :class="{ active: historyOpen }"
      type="button"
      role="tab"
      :aria-selected="historyOpen"
      title="Browse the history of any file"
      @click="emit('history')"
    >
      <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
      </svg>
      File History
    </button>
  </div>
</template>
