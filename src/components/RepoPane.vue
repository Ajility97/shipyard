<script setup lang="ts">
import { computed, ref, watch } from "vue";
import CommitGraph from "./CommitGraph.vue";
import DiffViewer from "./DiffViewer.vue";
import WorkingTree from "./WorkingTree.vue";
import { useApp } from "../composables/useApp";
import * as api from "../api";
import type { CommitNode, WorkingTreeFile } from "../types";

const props = defineProps<{
  repoId: string;
}>();

const { findRepo, groups, loaded } = useApp();

const commits = ref<CommitNode[]>([]);
const files = ref<WorkingTreeFile[]>([]);
const selectedFile = ref<WorkingTreeFile | null>(null);
const diff = ref("");
const diffMode = ref<"inline" | "split">("inline");
const loading = ref(false);
const message = ref("");

const current = computed(() => findRepo(props.repoId));

async function loadRepo() {
  const match = current.value;
  if (!match) {
    commits.value = [];
    files.value = [];
    selectedFile.value = null;
    diff.value = "";
    message.value = loaded.value ? "Repository not found." : "";
    return;
  }

  loading.value = true;
  message.value = "";
  try {
    const [nextCommits, nextFiles] = await Promise.all([
      api.logGraph(match.repo.path),
      api.workingTree(match.repo.path),
    ]);
    commits.value = nextCommits;
    files.value = nextFiles;
    if (selectedFile.value && !nextFiles.some((file) => file.path === selectedFile.value?.path)) {
      selectedFile.value = null;
      diff.value = "";
    }
  } catch (err) {
    message.value = String(err);
  } finally {
    loading.value = false;
  }
}

async function selectFile(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  selectedFile.value = file;
  try {
    diff.value = await api.fileDiff(match.repo.path, file.path);
  } catch (err) {
    diff.value = String(err);
  }
}

watch(
  () => [props.repoId, groups.value, loaded.value],
  () => {
    void loadRepo();
  },
  { immediate: true },
);
</script>

<template>
  <div v-if="current" class="repo-view">
    <section class="graph-pane">
      <div class="pane-header">
        <div>
          <strong>{{ current.status?.name ?? current.repo.path }}</strong>
          <div class="commit-sub">{{ current.status?.branch ?? "" }} · {{ current.repo.path }}</div>
        </div>
        <span v-if="loading" class="muted tiny">Loading…</span>
      </div>
      <p v-if="message" class="banner">{{ message }}</p>
      <CommitGraph :commits="commits" />
    </section>
    <aside class="changes-pane">
      <WorkingTree
        :files="files"
        :selected="selectedFile?.path ?? ''"
        @select="selectFile"
      />
      <div class="diff-pane">
        <div class="pane-header">
          <span>{{ selectedFile?.path ?? "Select a changed file" }}</span>
          <div class="segmented">
            <button :class="{ primary: diffMode === 'inline' }" type="button" @click="diffMode = 'inline'">
              Inline
            </button>
            <button :class="{ primary: diffMode === 'split' }" type="button" @click="diffMode = 'split'">
              Side by side
            </button>
          </div>
        </div>
        <DiffViewer v-if="selectedFile" :raw="diff" :mode="diffMode" />
        <p v-else class="muted" style="padding: 0.85rem">
          Uncommitted files appear above. Click one to compare it with HEAD.
        </p>
      </div>
    </aside>
  </div>
  <div v-else class="empty-home">
    <p class="muted">{{ loaded ? "Repository not found." : "Loading…" }}</p>
  </div>
</template>
