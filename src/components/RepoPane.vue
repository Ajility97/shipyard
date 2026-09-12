<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from "vue";
import { confirm } from "@tauri-apps/plugin-dialog";
import CommitGraph from "./CommitGraph.vue";
import DiffViewer from "./DiffViewer.vue";
import Modal from "./Modal.vue";
import PathLabel from "./PathLabel.vue";
import RepoToolbar from "./RepoToolbar.vue";
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
  showToast,
} = useApp();

const commits = ref<CommitNode[]>([]);
const files = ref<WorkingTreeFile[]>([]);
const branches = ref<string[]>([]);
const selectedFile = ref<WorkingTreeFile | null>(null);
const filesCollapsed = ref(false);
const diff = ref("");
const loading = ref(false);
const actionBusy = ref(false);
const actionLabel = ref("");
const message = ref("");
const creatingBranch = ref(false);
const newBranchName = ref("");
const newBranchInput = ref<HTMLInputElement | null>(null);
const committing = ref(false);
const commitTitle = ref("");
const commitDescription = ref("");
const commitTitleInput = ref<HTMLInputElement | null>(null);

const COMMIT_TITLE_MAX = 72;

const canCreateBranch = computed(() => Boolean(newBranchName.value.trim()));
const commitTitleLength = computed(() => [...commitTitle.value].length);
const commitTitleLeft = computed(() => Math.max(0, COMMIT_TITLE_MAX - commitTitleLength.value));
const canCommit = computed(
  () => Boolean(commitTitle.value.trim()) && commitTitleLength.value <= COMMIT_TITLE_MAX,
);

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
    branches.value = [];
    selectedFile.value = null;
    diff.value = "";
    message.value = loaded.value ? "Repository not found." : "";
    return;
  }

  loading.value = true;
  message.value = "";
  try {
    const [nextCommits, nextFiles, nextBranches] = await Promise.all([
      api.logGraph(match.repo.path),
      api.workingTree(match.repo.path),
      api.listLocalBranches(match.repo.path).catch(() => [] as string[]),
    ]);
    commits.value = nextCommits;
    files.value = nextFiles;
    branches.value = nextBranches;
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

async function runRepoAction(label: string, work: () => Promise<string>) {
  const match = current.value;
  if (!match || actionBusy.value) {
    return;
  }
  actionBusy.value = true;
  actionLabel.value = label;
  message.value = "";
  try {
    const result = await work();
    showToast(result);
    await loadRepo();
    await refreshStatus(match.group?.id ?? STANDALONE_GROUP_ID);
  } catch (err) {
    const text = String(err);
    message.value = text;
    showToast(text, "error");
  } finally {
    actionBusy.value = false;
    actionLabel.value = "";
  }
}

function pullRepo() {
  const match = current.value;
  if (!match) {
    return;
  }
  return runRepoAction("Pulling…", () => api.repoPull(match.repo.path));
}

function pushRepo() {
  const match = current.value;
  if (!match) {
    return;
  }
  return runRepoAction("Pushing…", () => api.repoPush(match.repo.path));
}

function checkoutBranch(branch: string) {
  const match = current.value;
  if (!match) {
    return;
  }
  return runRepoAction("Checking out…", () => api.checkoutLocalBranch(match.repo.path, branch));
}

async function openCreateBranch() {
  if (actionBusy.value) {
    return;
  }
  newBranchName.value = "";
  creatingBranch.value = true;
  await nextTick();
  newBranchInput.value?.focus();
}

function closeCreateBranch() {
  creatingBranch.value = false;
  newBranchName.value = "";
}

function createBranch() {
  const match = current.value;
  const branch = newBranchName.value.trim();
  if (!match || !branch) {
    return;
  }
  closeCreateBranch();
  return runRepoAction("Creating branch…", () =>
    api.createAndCheckoutBranch(match.repo.path, branch),
  );
}

async function openCommit() {
  if (actionBusy.value || !files.value.some((file) => file.staged)) {
    return;
  }
  commitTitle.value = "";
  commitDescription.value = "";
  committing.value = true;
  await nextTick();
  commitTitleInput.value?.focus();
}

function closeCommit() {
  committing.value = false;
  commitTitle.value = "";
  commitDescription.value = "";
}

function commitChanges() {
  const match = current.value;
  const title = commitTitle.value.trim();
  if (!match || !title) {
    return;
  }
  const description = commitDescription.value;
  closeCommit();
  return runRepoAction("Committing…", () => api.commit(match.repo.path, title, description));
}

async function refreshBranches() {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    branches.value = await api.listLocalBranches(match.repo.path);
  } catch {
    /* keep the last successful list */
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
  <div class="repo-pane">
  <div
    v-if="current"
    class="repo-view"
    :class="{ 'files-collapsed': filesCollapsed, resizing }"
    :style="{ '--files-pane-width': `${filesPaneWidth}px` }"
  >
    <section v-if="!selectedFile" class="graph-pane">
      <RepoToolbar
        :repo-id="current.repo.id"
        :name="current.status?.name ?? current.repo.path"
        :branch="current.status?.branch ?? ''"
        :path="current.repo.path"
        :branches="branches"
        :busy="actionBusy"
        :busy-label="actionLabel || (loading ? 'Loading…' : '')"
        @pull="pullRepo"
        @push="pushRepo"
        @checkout="checkoutBranch"
        @create="openCreateBranch"
        @refresh-branches="refreshBranches"
      >
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
      </RepoToolbar>
      <p v-if="message" class="banner">{{ message }}</p>
      <div class="graph-scroll">
        <CommitGraph :commits="commits" />
      </div>
    </section>
    <section v-else class="diff-main">
      <div class="pane-header">
        <div class="diff-heading">
          <button class="ghost tiny" type="button" @click="closeDiff">← Back</button>
          <PathLabel class="diff-path" :path="selectedFile.path" />
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
        @commit="openCommit"
      />
    </aside>
  </div>
  <div v-else class="empty-home">
    <p class="muted">{{ loaded ? "Repository not found." : "Loading…" }}</p>
  </div>
  <Modal v-if="committing" title="Commit" medium @close="closeCommit">
    <label class="modal-label">
      <span class="modal-label-row">
        <span class="muted tiny">Title</span>
        <span
          class="muted tiny char-count"
          :class="{ warn: commitTitleLeft <= 12, bad: commitTitleLeft === 0 }"
        >
          {{ commitTitleLength }}/{{ COMMIT_TITLE_MAX }} · {{ commitTitleLeft }} left
        </span>
      </span>
      <input
        ref="commitTitleInput"
        v-model="commitTitle"
        type="text"
        :maxlength="COMMIT_TITLE_MAX"
        placeholder="Short summary of the change"
        @keydown.enter.prevent="commitChanges"
      />
    </label>
    <label class="modal-label">
      <span class="muted tiny">Description</span>
      <textarea
        v-model="commitDescription"
        rows="5"
        placeholder="Optional details"
      />
    </label>
    <template #actions>
      <button class="ghost" type="button" @click="closeCommit">Cancel</button>
      <button class="ghost commit" type="button" :disabled="!canCommit" @click="commitChanges">
        Commit
      </button>
    </template>
  </Modal>
  <Modal v-if="creatingBranch" title="New branch" @close="closeCreateBranch">
    <label class="modal-label">
      <span class="muted tiny">Branch name</span>
      <input
        ref="newBranchInput"
        v-model="newBranchName"
        type="text"
        placeholder="feature/JIRA-123"
        @keydown.enter="createBranch"
      />
    </label>
    <template #actions>
      <button class="ghost" type="button" @click="closeCreateBranch">Cancel</button>
      <button class="primary" type="button" :disabled="!canCreateBranch" @click="createBranch">
        Create and switch
      </button>
    </template>
  </Modal>
  </div>
</template>
