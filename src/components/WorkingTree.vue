<script setup lang="ts">
import type { WorkingTreeFile } from "../types";

defineProps<{
  files: WorkingTreeFile[];
  selected: string;
}>();

const emit = defineEmits<{
  select: [file: WorkingTreeFile];
  discard: [];
}>();
</script>

<template>
  <div class="file-pane">
    <div class="pane-header">
      <div class="file-heading">
        <strong>Uncommitted changes</strong>
        <span class="file-count-badge">{{ files.length }}</span>
      </div>
    </div>
    <div class="file-list">
      <p v-if="!files.length" class="muted tiny" style="padding: 0.75rem 0.85rem">
        Working tree is clean.
      </p>
      <button
        v-for="file in files"
        :key="file.path"
        class="file-item"
        :class="{ active: selected === file.path }"
        type="button"
        @click="emit('select', file)"
      >
        <span>{{ file.path }}</span>
        <span class="muted tiny">{{ file.status }}</span>
      </button>
    </div>
    <div class="file-footer">
      <button
        class="ghost tiny danger"
        type="button"
        :disabled="!files.length"
        @click="emit('discard')"
      >
        Discard all
      </button>
    </div>
  </div>
</template>
