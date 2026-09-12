<script setup lang="ts">
import { computed } from "vue";
import type { WorkingTreeFile } from "../types";
import FileStatusIcon from "./FileStatusIcon.vue";
import PathLabel from "./PathLabel.vue";

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
  commit: [];
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
          class="ghost tiny file-bulk-action stage"
          type="button"
          :disabled="!unstaged.length"
          @click="emit('stageAll')"
        >
          <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 4.5v15m7.5-7.5h-15" />
          </svg>
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
            <FileStatusIcon :status="file.status" />
            <PathLabel class="file-item-path" :path="file.path" />
          </button>
          <button
            class="tiny file-item-action stage"
            type="button"
            @click.stop="emit('stage', file)"
          >
            <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M12 4.5v15m7.5-7.5h-15" />
            </svg>
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
          class="ghost tiny file-bulk-action unstage"
          type="button"
          :disabled="!staged.length"
          @click="emit('unstageAll')"
        >
          <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M5 12h14" />
          </svg>
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
            <FileStatusIcon :status="file.status" />
            <PathLabel class="file-item-path" :path="file.path" />
          </button>
          <button
            class="tiny file-item-action unstage"
            type="button"
            @click.stop="emit('unstage', file)"
          >
            <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M5 12h14" />
            </svg>
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
        Discard
      </button>
      <button
        class="ghost tiny commit"
        type="button"
        :disabled="!staged.length"
        @click="emit('commit')"
      >
        <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
          <path d="M4.5 12.75l6 6 9-13.5" />
        </svg>
        Commit
      </button>
    </div>
  </div>
</template>
