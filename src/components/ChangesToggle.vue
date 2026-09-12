<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  open: boolean;
  unstaged: number;
  staged: number;
}>();

const emit = defineEmits<{
  click: [];
}>();

const title = computed(() => {
  const parts = [];
  if (props.unstaged) {
    parts.push(`${props.unstaged} unstaged`);
  }
  if (props.staged) {
    parts.push(`${props.staged} staged`);
  }
  const counts = parts.length ? parts.join(" · ") : "No local changes";
  return props.open ? `Hide changes · ${counts}` : `Show changes · ${counts}`;
});
</script>

<template>
  <button
    class="ghost tiny"
    :class="{ active: open }"
    type="button"
    :aria-pressed="open"
    :title="title"
    @click="emit('click')"
  >
    <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
      <rect x="3.75" y="4.5" width="16.5" height="15" rx="2" />
      <path d="M14.25 4.5v15" />
    </svg>
    Changes
    <span v-if="unstaged" class="file-count-badge unstaged" title="Unstaged">{{ unstaged }}</span>
    <span v-if="staged" class="file-count-badge staged" title="Staged">{{ staged }}</span>
  </button>
</template>
