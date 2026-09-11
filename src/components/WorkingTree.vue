<script setup lang="ts">
import type { WorkingTreeFile } from "../types";

defineProps<{
  files: WorkingTreeFile[];
  selected: string;
}>();

const emit = defineEmits<{
  select: [file: WorkingTreeFile];
  collapse: [];
}>();
</script>

<template>
  <div class="file-pane">
    <div class="pane-header">
      <div class="file-heading">
        <strong>Uncommitted changes</strong>
        <span class="file-count-badge">{{ files.length }}</span>
      </div>
      <button
        class="files-float"
        type="button"
        title="Collapse files panel"
        aria-label="Collapse files panel"
        @click="emit('collapse')"
      >
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <rect x="1.75" y="2.25" width="12.5" height="11.5" rx="1.5" />
          <path d="M10.25 2.25v11.5" />
          <path d="M7.15 5.6L9.4 8l-2.25 2.4" />
        </svg>
      </button>
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
  </div>
</template>
