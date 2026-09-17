<script setup lang="ts">
import type { CommitNode } from "../types";
import { formatCommitDate } from "../graphLayout";
import PathLabel from "./PathLabel.vue";

defineProps<{
  file: string;
  commits: CommitNode[];
  selectedHash: string;
  loading?: boolean;
  error?: string;
}>();

const emit = defineEmits<{
  select: [commit: CommitNode];
  close: [];
}>();
</script>

<template>
  <div class="file-pane">
    <section class="file-section">
      <div class="pane-header file-history-header">
        <div class="file-heading file-history-title">
          <strong>
            <PathLabel :path="file" />
          </strong>
        </div>
        <button class="ghost tiny" type="button" @click="emit('close')">Back</button>
      </div>
      <div class="file-list">
        <p v-if="loading" class="muted tiny empty-files">Loading history…</p>
        <p v-else-if="error" class="muted tiny empty-files">{{ error }}</p>
        <p v-else-if="!commits.length" class="muted tiny empty-files">No history for this file.</p>
        <template v-else>
          <button
            v-for="commit in commits"
            :key="commit.hash"
            class="file-item file-history-commit"
            :class="{ active: selectedHash === commit.hash }"
            type="button"
            :title="commit.subject"
            @click="emit('select', commit)"
          >
            <span class="file-history-subject">{{ commit.subject || commit.hash.slice(0, 7) }}</span>
            <span class="muted tiny file-history-meta">
              {{ commit.hash.slice(0, 7) }} · {{ commit.author }} · {{ formatCommitDate(commit.date) }}
            </span>
          </button>
        </template>
      </div>
    </section>
  </div>
</template>
