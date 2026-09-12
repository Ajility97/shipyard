<script setup lang="ts">
import { computed } from "vue";
import type { WorkingTreeFile } from "../types";

const props = defineProps<{
  files: WorkingTreeFile[];
  selectedPath: string;
  selectedStaged: boolean;
}>();

const emit = defineEmits<{
  select: [file: WorkingTreeFile];
  stage: [file: WorkingTreeFile];
  unstage: [file: WorkingTreeFile];
  stageAll: [];
  unstageAll: [];
  discard: [];
}>();

const unstaged = computed(() => props.files.filter((file) => !file.staged));
const staged = computed(() => props.files.filter((file) => file.staged));

function isSelected(file: WorkingTreeFile) {
  return props.selectedPath === file.path && props.selectedStaged === file.staged;
}
</script>

<template>
  <div class="file-pane">
    <section class="file-section">
      <div class="pane-header">
        <div class="file-heading">
          <strong>Unstaged files</strong>
          <span class="file-count-badge">{{ unstaged.length }}</span>
        </div>
        <button
          class="ghost tiny file-bulk-action"
          type="button"
          :disabled="!unstaged.length"
          @click="emit('stageAll')"
        >
          Stage all files
        </button>
      </div>
      <div class="file-list">
        <p v-if="!unstaged.length" class="muted tiny empty-files">No unstaged files.</p>
        <div
          v-for="file in unstaged"
          :key="`unstaged:${file.path}`"
          class="file-item"
          :class="{ active: isSelected(file) }"
        >
          <button class="file-item-main" type="button" @click="emit('select', file)">
            <span class="file-item-path">{{ file.path }}</span>
            <span class="muted tiny file-item-status">{{ file.status }}</span>
          </button>
          <button
            class="ghost tiny file-item-action"
            type="button"
            @click.stop="emit('stage', file)"
          >
            Stage file
          </button>
        </div>
      </div>
    </section>
    <section class="file-section">
      <div class="pane-header">
        <div class="file-heading">
          <strong>Staged files</strong>
          <span class="file-count-badge">{{ staged.length }}</span>
        </div>
        <button
          class="ghost tiny file-bulk-action"
          type="button"
          :disabled="!staged.length"
          @click="emit('unstageAll')"
        >
          Unstage all files
        </button>
      </div>
      <div class="file-list">
        <p v-if="!staged.length" class="muted tiny empty-files">No staged files.</p>
        <div
          v-for="file in staged"
          :key="`staged:${file.path}`"
          class="file-item"
          :class="{ active: isSelected(file) }"
        >
          <button class="file-item-main" type="button" @click="emit('select', file)">
            <span class="file-item-path">{{ file.path }}</span>
            <span class="muted tiny file-item-status">{{ file.status }}</span>
          </button>
          <button
            class="ghost tiny file-item-action"
            type="button"
            @click.stop="emit('unstage', file)"
          >
            Unstage file
          </button>
        </div>
      </div>
    </section>
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
