<script setup lang="ts">
import { computed } from "vue";
import type { CommitNode } from "../types";
import { formatCommitDate } from "../graphLayout";
import PathLabel from "./PathLabel.vue";

const props = defineProps<{
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

function fileName(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.slice(normalized.lastIndexOf("/") + 1);
}

type HistoryMarker = {
  kind: "marker";
  key: string;
  label: string;
  tone: "added" | "renamed" | "moved" | "copied";
  from?: string;
  to?: string;
};

type HistoryRow =
  | { kind: "commit"; commit: CommitNode }
  | HistoryMarker;

const rows = computed(() => {
  const next: HistoryRow[] = [];
  const lastIndex = props.commits.length - 1;
  props.commits.forEach((commit, index) => {
    const status = (commit.status ?? "").toLowerCase();
    const path = commit.path ?? props.file;
    next.push({ kind: "commit", commit });
    if ((status === "renamed" || status === "copied") && commit.oldPath) {
      const sameName = fileName(commit.oldPath) === fileName(path);
      const tone = status === "copied" ? "copied" : sameName ? "moved" : "renamed";
      next.push({
        kind: "marker",
        key: `${commit.hash}-rename`,
        label: status === "copied" ? "Copied" : sameName ? "Moved" : "Renamed",
        tone,
        from: commit.oldPath,
        to: path,
      });
    }
    if (index === lastIndex && status === "added") {
      next.push({
        kind: "marker",
        key: `${commit.hash}-added`,
        label: "Added",
        tone: "added",
        to: path !== props.file ? path : undefined,
      });
    }
  });
  return next;
});
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
          <template v-for="row in rows" :key="row.kind === 'commit' ? row.commit.hash : row.key">
            <div v-if="row.kind === 'marker'" class="file-history-marker" :class="row.tone">
              <span class="file-history-marker-label">{{ row.label }}</span>
              <span v-if="row.from && row.to" class="file-history-marker-paths">
                <PathLabel :path="row.from" />
                <span class="file-history-marker-arrow" aria-hidden="true">→</span>
                <PathLabel :path="row.to" />
              </span>
              <PathLabel v-else-if="row.to" :path="row.to" />
            </div>
            <button
              v-else
              class="file-item file-history-commit"
              :class="{ active: selectedHash === row.commit.hash }"
              type="button"
              :title="row.commit.subject"
              @click="emit('select', row.commit)"
            >
              <span class="file-history-subject">{{ row.commit.subject || row.commit.hash.slice(0, 7) }}</span>
              <span class="muted tiny file-history-meta">
                {{ row.commit.hash.slice(0, 7) }} · {{ row.commit.author }} ·
                {{ formatCommitDate(row.commit.date) }}
              </span>
            </button>
          </template>
        </template>
      </div>
    </section>
  </div>
</template>
