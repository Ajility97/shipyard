<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from "vue";
import { confirm } from "@tauri-apps/plugin-dialog";
import CommitGraph from "./CommitGraph.vue";
import DiffViewer from "./DiffViewer.vue";
import WorkingTree from "./WorkingTree.vue";
import { useApp } from "../composables/useApp";
import * as api from "../api";
import type { CommitNode, WorkingTreeFile } from "../types";
import { STANDALONE_GROUP_ID } from "../types";

const props = defineProps<{
  repoId: string;
}>();

const {
  findRepo,
  groups,
  standaloneRepos,
  loaded,
  filesPaneWidth,
  setFilesPaneWidth,
  saveFilesPaneWidth,
  diffMode,
  saveDiffMode,
  refreshStatus,
} = useApp();

const commits = ref<CommitNode[]>([]);
const files = ref<WorkingTreeFile[]>([]);
const selectedFile = ref<WorkingTreeFile | null>(null);
const filesCollapsed = ref(false);
const diff = ref("");
const loading = ref(false);
const message = ref("");

const current = computed(() => findRepo(props.repoId));
const resizing = ref(false);
let resizeStartX = 0;
let resizeStartWidth = 320;
let resizePointerId: number | null = null;

function onResizeMove(event: PointerEvent) {
  setFilesPaneWidth(resizeStartWidth + (resizeStartX - event.clientX));
}

function stopResize(event?: PointerEvent) {
  if (resizePointerId === null) {
    return;
  }
  if (event && event.pointerId !== resizePointerId) {
    return;
  }
  window.removeEventListener("pointermove", onResizeMove);
  window.removeEventListener("pointerup", stopResize);
  window.removeEventListener("pointercancel", stopResize);
  resizing.value = false;
  resizePointerId = null;
  document.body.classList.remove("is-resizing");
  void saveFilesPaneWidth(filesPaneWidth.value);
}

function startResize(event: PointerEvent) {
  event.preventDefault();
  resizeStartX = event.clientX;
  resizeStartWidth = filesPaneWidth.value;
  resizePointerId = event.pointerId;
  resizing.value = true;
  document.body.classList.add("is-resizing");
  window.addEventListener("pointermove", onResizeMove);
  window.addEventListener("pointerup", stopResize);
  window.addEventListener("pointercancel", stopResize);
}

onUnmounted(() => {
  stopResize();
});

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
    if (selectedFile.value && !nextFiles.some((file) => sameFile(file, selectedFile.value))) {
      selectedFile.value = null;
      diff.value = "";
    }
  } catch (err) {
    message.value = String(err);
  } finally {
    loading.value = false;
  }
}

function sameFile(file: WorkingTreeFile, other: WorkingTreeFile | null) {
  return !!other && file.path === other.path && file.staged === other.staged;
}

function closeDiff() {
  selectedFile.value = null;
  diff.value = "";
}

async function selectFile(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  if (sameFile(file, selectedFile.value)) {
    closeDiff();
    return;
  }
  selectedFile.value = file;
  try {
    diff.value = await api.fileDiff(match.repo.path, file.path, file.staged);
  } catch (err) {
    diff.value = String(err);
  }
}

async function reloadAfterIndexChange() {
  const match = current.value;
  if (!match) {
    return;
  }
  await loadRepo();
  await refreshStatus(match.group?.id ?? STANDALONE_GROUP_ID);
}

async function stageFile(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    await api.stageFile(match.repo.path, file.path);
    await reloadAfterIndexChange();
  } catch (err) {
    message.value = String(err);
  }
}

async function stageAll() {
  const match = current.value;
  if (!match || !files.value.some((file) => !file.staged)) {
    return;
  }
  try {
    await api.stageAll(match.repo.path);
    await reloadAfterIndexChange();
  } catch (err) {
    message.value = String(err);
  }
}

async function unstageFile(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    await api.unstageFile(match.repo.path, file.path);
    await reloadAfterIndexChange();
  } catch (err) {
    message.value = String(err);
  }
}

async function unstageAll() {
  const match = current.value;
  if (!match || !files.value.some((file) => file.staged)) {
    return;
  }
  try {
    await api.unstageAll(match.repo.path);
    await reloadAfterIndexChange();
  } catch (err) {
    message.value = String(err);
  }
}

async function discardAll() {
  const match = current.value;
  if (!match || !files.value.length) {
    return;
  }
  const ok = await confirm(
    "Discard all uncommitted changes? Tracked files will be reset and untracked files will be deleted.",
    {
      title: "Discard all changes",
      kind: "warning",
      okLabel: "Discard",
      cancelLabel: "Cancel",
    },
  );
  if (!ok) {
    return;
  }
  try {
    await api.discardAllChanges(match.repo.path);
    closeDiff();
    await loadRepo();
    await refreshStatus(match.group?.id ?? STANDALONE_GROUP_ID);
  } catch (err) {
    message.value = String(err);
  }
}

watch(
  () => [props.repoId, groups.value, standaloneRepos.value, loaded.value],
  () => {
    void loadRepo();
  },
  { immediate: true },
);
</script>

<template>
  <div
    v-if="current"
    class="repo-view"
    :class="{ 'files-collapsed': filesCollapsed, resizing }"
    :style="{ '--files-pane-width': `${filesPaneWidth}px` }"
  >
    <section v-if="!selectedFile" class="graph-pane">
      <div class="pane-header">
        <div>
          <strong>{{ current.status?.name ?? current.repo.path }}</strong>
          <div class="commit-sub">{{ current.status?.branch ?? "" }} · {{ current.repo.path }}</div>
        </div>
        <div class="pane-header-end">
          <span v-if="loading" class="muted tiny">Loading…</span>
          <button
            v-if="filesCollapsed"
            class="files-float"
            type="button"
            title="Show files panel"
            aria-label="Show files panel"
            @click="filesCollapsed = false"
          >
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <rect x="1.75" y="2.25" width="12.5" height="11.5" rx="1.5" />
              <path d="M10.25 2.25v11.5" />
              <path d="M8.85 5.6L6.6 8l2.25 2.4" />
            </svg>
          </button>
        </div>
      </div>
      <p v-if="message" class="banner">{{ message }}</p>
      <div class="graph-scroll">
        <CommitGraph :commits="commits" />
      </div>
    </section>
    <section v-else class="diff-main">
      <div class="pane-header">
        <div class="diff-heading">
          <button class="ghost tiny" type="button" @click="closeDiff">← Back</button>
          <span class="diff-path">{{ selectedFile.path }}</span>
          <span class="muted tiny">{{ selectedFile.staged ? "Staged" : "Unstaged" }}</span>
        </div>
        <div class="pane-header-end">
          <div class="segmented" role="group" aria-label="Diff layout">
            <button
              type="button"
              :class="{ active: diffMode === 'inline' }"
              :aria-pressed="diffMode === 'inline'"
              @click="saveDiffMode('inline')"
            >
              Inline
            </button>
            <button
              type="button"
              :class="{ active: diffMode === 'split' }"
              :aria-pressed="diffMode === 'split'"
              @click="saveDiffMode('split')"
            >
              Side by side
            </button>
          </div>
          <button
            v-if="filesCollapsed"
            class="files-float"
            type="button"
            title="Show files panel"
            aria-label="Show files panel"
            @click="filesCollapsed = false"
          >
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <rect x="1.75" y="2.25" width="12.5" height="11.5" rx="1.5" />
              <path d="M10.25 2.25v11.5" />
              <path d="M8.85 5.6L6.6 8l2.25 2.4" />
            </svg>
          </button>
        </div>
      </div>
      <div class="diff-scroll">
        <DiffViewer :raw="diff" :mode="diffMode" />
      </div>
    </section>
    <aside v-if="!filesCollapsed" class="changes-pane">
      <button
        class="pane-resize"
        type="button"
        aria-label="Resize files panel"
        @pointerdown="startResize"
      />
      <button
        class="files-float dock-left"
        type="button"
        title="Collapse files panel"
        aria-label="Collapse files panel"
        @click="filesCollapsed = true"
      >
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <rect x="1.75" y="2.25" width="12.5" height="11.5" rx="1.5" />
          <path d="M10.25 2.25v11.5" />
          <path d="M7.15 5.6L9.4 8l-2.25 2.4" />
        </svg>
      </button>
      <WorkingTree
        :files="files"
        :selected-path="selectedFile?.path ?? ''"
        :selected-staged="selectedFile?.staged ?? false"
        @select="selectFile"
        @stage="stageFile"
        @unstage="unstageFile"
        @stage-all="stageAll"
        @unstage-all="unstageAll"
        @discard="discardAll"
      />
    </aside>
  </div>
  <div v-else class="empty-home">
    <p class="muted">{{ loaded ? "Repository not found." : "Loading…" }}</p>
  </div>
</template>
