<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { confirm } from "@tauri-apps/plugin-dialog";
import BranchList from "./BranchList.vue";
import ChangesToggle from "./ChangesToggle.vue";
import CommitFiles from "./CommitFiles.vue";
import FileHistoryList from "./FileHistoryList.vue";
import FileHistoryToggle from "./FileHistoryToggle.vue";
import FileTree from "./FileTree.vue";
import CommitContextMenu from "./CommitContextMenu.vue";
import CommitGraph from "./CommitGraph.vue";
import DiffViewer from "./DiffViewer.vue";
import Modal from "./Modal.vue";
import PathLabel from "./PathLabel.vue";
import RepoToolbar from "./RepoToolbar.vue";
import StashList from "./StashList.vue";
import TagList from "./TagList.vue";
import TerminalPane from "./TerminalPane.vue";
import WorkingTree from "./WorkingTree.vue";
import { useApp } from "../composables/useApp";
import * as api from "../api";
import type {
  BranchOverview,
  BranchTracking,
  CommitFile,
  CommitNode,
  LastCommit,
  LocalBranch,
  RepoFile,
  RepoFilesChanged,
  StashEntry,
  TagEntry,
  WorkingTreeFile,
} from "../types";
import { STANDALONE_GROUP_ID } from "../types";
import {
  abortLabel,
  absoluteFilePath,
  continueLabel,
  fileBasename,
  isConflicted,
  openInEditorLabel,
  operationNoun,
  operationTitle,
  type IgnoreKind,
} from "../gitOperation";

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
  terminalPaneHeight,
  diffMode,
  saveDiffMode,
  editor,
  refreshRepoStatus,
  patchRepoStatus,
  showToast,
  presentActionResults,
  repoDisplayName,
} = useApp();

const commits = ref<CommitNode[]>([]);
const files = ref<WorkingTreeFile[]>([]);
const branches = ref<string[]>([]);
const branchTracking = ref<BranchTracking[]>([]);
let trackingGeneration = 0;
const trackingPath = ref("");
const branchesView = ref(false);
const graphStale = ref(false);
const stashes = ref<StashEntry[]>([]);
const stashView = ref(false);
const tags = ref<TagEntry[]>([]);
const tagView = ref(false);
const terminalOpen = ref(false);
const overview = ref<BranchOverview | null>(null);
let overviewGeneration = 0;
const selectedFile = ref<WorkingTreeFile | null>(null);
const selectedCommit = ref<CommitNode | null>(null);
const selectedCommitFile = ref<CommitFile | null>(null);
const commitFiles = ref<CommitFile[]>([]);
const commitFilesLoading = ref(false);
const filesCollapsed = ref(false);
const historyOpen = ref(false);
const repoFiles = ref<RepoFile[]>([]);
const repoFilesLoading = ref(false);
const repoFilesError = ref("");
let repoFilesGeneration = 0;
const selectedHistoryFile = ref("");
const lastHistoryFile = ref("");
const historyCommits = ref<CommitNode[]>([]);
const historyCommitsLoading = ref(false);
const historyCommitsError = ref("");
const selectedHistoryCommit = ref<CommitNode | null>(null);
let historyCommitsGeneration = 0;
const diff = ref("");
const showingDiff = computed(
  () => Boolean(selectedFile.value || selectedCommitFile.value || selectedHistoryCommit.value),
);
const blameFile = computed(
  () =>
    selectedFile.value?.path ??
    selectedCommitFile.value?.path ??
    selectedHistoryCommit.value?.path ??
    selectedHistoryFile.value,
);
const blameOldPath = computed(
  () => selectedCommitFile.value?.oldPath ?? selectedHistoryCommit.value?.oldPath ?? "",
);
const blameRev = computed(() => {
  if (selectedFile.value) {
    return "";
  }
  if (selectedHistoryCommit.value) {
    return selectedHistoryCommit.value.hash;
  }
  return selectedCommit.value?.hash ?? "";
});
const blameStaged = computed(() => selectedFile.value?.staged ?? false);
const loading = ref(false);
const actionBusy = ref(false);
const actionLabel = ref("");
const actionBranch = ref("");
const message = ref("");
const creatingBranch = ref(false);
const newBranchName = ref("");
const baseBranch = ref("");
const newBranchStart = ref("");
const newBranchInput = ref<HTMLInputElement | null>(null);
const commitMenu = ref<{
  commit: CommitNode;
  hashes: string[];
  x: number;
  y: number;
} | null>(null);
const renamingBranch = ref<LocalBranch | null>(null);
const renameBranchName = ref("");
const renameBranchInput = ref<HTMLInputElement | null>(null);
const mergingBranch = ref(false);
const mergeSource = ref("");
const mergeTarget = ref("");
const committing = ref(false);
const amending = ref(false);
const lastCommit = ref<LastCommit | null>(null);
const draftTitle = ref("");
const draftDescription = ref("");
const commitTitle = ref("");
const commitDescription = ref("");
const commitTitleInput = ref<HTMLInputElement | null>(null);
const stashing = ref(false);
const stashMessage = ref("");
const stashMessageInput = ref<HTMLInputElement | null>(null);
const creatingTag = ref(false);
const newTagName = ref("");
const newTagMessage = ref("");
const newTagTarget = ref("");
const newTagInput = ref<HTMLInputElement | null>(null);
const pullingOptions = ref(false);
const pullSource = ref<"current" | "develop" | "master" | "main" | "specify">("current");
const specifyBranch = ref("");

const COMMIT_TITLE_MAX = 72;

function isDetachedBranch(name: string) {
  return name === "HEAD" || name === "detached HEAD" || name.startsWith("detached ");
}

const checkedOutBranch = computed(() => {
  const name = current.value?.status?.branch ?? "";
  return name && !isDetachedBranch(name) ? name : "";
});
const baseBranchOptions = computed(() => {
  const names = [...branches.value];
  const currentName = checkedOutBranch.value;
  if (currentName && !names.includes(currentName)) {
    names.unshift(currentName);
  }
  return names;
});
const canCreateBranch = computed(() => {
  if (!newBranchName.value.trim()) {
    return false;
  }
  if (newBranchStart.value.trim()) {
    return true;
  }
  return Boolean(baseBranch.value.trim());
});

function baseBranchLabel(name: string) {
  return name === checkedOutBranch.value ? `${name} (current)` : name;
}
const commitMenuHasMerge = computed(() => {
  const menu = commitMenu.value;
  if (!menu) {
    return false;
  }
  const selected = new Set(menu.hashes);
  return commits.value.some((commit) => selected.has(commit.hash) && commit.parents.length > 1);
});
const newBranchStartShort = computed(() => {
  const start = newBranchStart.value.trim();
  return start ? start.slice(0, 7) : "";
});
const canCreateTag = computed(() => Boolean(newTagName.value.trim()));
const canRenameBranch = computed(() => {
  const next = renameBranchName.value.trim();
  return Boolean(next) && next !== (renamingBranch.value?.name ?? "");
});
const localBranchNames = computed(() => {
  const fromOverview = overview.value?.branches.map((branch) => branch.name) ?? [];
  if (fromOverview.length) {
    return fromOverview;
  }
  return branches.value;
});
const mergeTargetHint = computed(() => {
  const raw = overview.value?.mergeTarget ?? preferredMergeTarget() ?? "";
  return raw.replace(/^origin\//, "").trim();
});
const canConfirmMerge = computed(() => {
  const source = mergeSource.value.trim();
  const target = mergeTarget.value.trim();
  return Boolean(source && target && source !== target);
});
const commitTitleLength = computed(() => [...commitTitle.value].length);
const commitTitleLeft = computed(() => Math.max(0, COMMIT_TITLE_MAX - commitTitleLength.value));
const conflictedFiles = computed(() => files.value.filter(isConflicted));
const unstagedCount = computed(
  () => files.value.filter((file) => !file.staged && !isConflicted(file)).length,
);
const stagedCount = computed(
  () => files.value.filter((file) => file.staged && !isConflicted(file)).length,
);
const canCommit = computed(
  () =>
    Boolean(commitTitle.value.trim()) &&
    commitTitleLength.value <= COMMIT_TITLE_MAX &&
    stagedCount.value > 0,
);
const conflictedCount = computed(() => conflictedFiles.value.length);
const operation = computed(() => current.value?.status?.operation ?? "");
const conflictActive = computed(() => Boolean(operation.value || conflictedCount.value));
const openEditorLabel = computed(() => openInEditorLabel(editor.value));
const conflictHeading = computed(() => operationTitle(operation.value));
const conflictCopy = computed(() => {
  const count = conflictedCount.value;
  if (count > 0) {
    const filesLabel = count === 1 ? "1 file still has conflicts" : `${count} files still have conflicts`;
    return `${filesLabel}. Open each file, fix the markers, then mark it resolved.`;
  }
  if (operation.value) {
    return `All conflicted files are marked resolved. Continue the ${operationNoun(operation.value)} or abort it.`;
  }
  return "";
});

const current = computed(() => findRepo(props.repoId));
const resizing = ref(false);
let resizeStartX = 0;
let resizeStartWidth = 320;
let resizePointerId: number | null = null;
let loadGeneration = 0;
let filesGeneration = 0;
let worktreeMutation = 0;
let watchRefresh: Promise<void> | null = null;
let watchRefreshQueued = false;
let watchRefreshRefs = false;
let watchToken = 0;
let watchClosed = false;
let stopWatch: UnlistenFn | undefined;
let watchedPath = "";

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
  document.body.classList.remove("is-resizing", "is-resizing-x");
  void saveFilesPaneWidth(filesPaneWidth.value);
}

function startResize(event: PointerEvent) {
  event.preventDefault();
  resizeStartX = event.clientX;
  resizeStartWidth = filesPaneWidth.value;
  resizePointerId = event.pointerId;
  resizing.value = true;
  document.body.classList.add("is-resizing", "is-resizing-x");
  window.addEventListener("pointermove", onResizeMove);
  window.addEventListener("pointerup", stopResize);
  window.addEventListener("pointercancel", stopResize);
}

onUnmounted(() => {
  watchClosed = true;
  stopResize();
  stopWatch?.();
  void stopWatching(watchedPath);
});

function watchKey(path: string) {
  return path.replace(/\/+$/, "");
}

async function stopWatching(path: string) {
  if (!path) {
    return;
  }
  try {
    await api.unwatchRepo(path);
  } catch {
    /* already gone */
  }
}

async function startWatching(path: string) {
  const next = watchKey(path);
  if (watchedPath === next) {
    return;
  }
  const token = ++watchToken;
  const previous = watchedPath;
  watchedPath = next;
  await stopWatching(previous);
  if (token !== watchToken) {
    return;
  }
  if (!next) {
    return;
  }
  try {
    await api.watchRepo(next);
    if (token !== watchToken) {
      await stopWatching(next);
    }
  } catch {
    if (token === watchToken) {
      watchedPath = "";
    }
  }
}

async function refreshHistoryFile() {
  const match = current.value;
  const path = selectedHistoryFile.value;
  if (!match || !path) {
    return;
  }
  const generation = ++historyCommitsGeneration;
  try {
    const next = await api.fileLog(match.repo.path, path);
    if (generation !== historyCommitsGeneration) {
      return;
    }
    historyCommits.value = next;
  } catch {
    /* keep the list already on screen */
  }
}

async function refreshSelectedFileDiff(file: WorkingTreeFile) {
  const match = current.value;
  if (!match || selectedCommitFile.value || selectedHistoryCommit.value) {
    return;
  }
  try {
    const next = await api.fileDiff(match.repo.path, file.path, file.staged);
    if (!sameFile(file, selectedFile.value)) {
      return;
    }
    diff.value = next;
  } catch (err) {
    if (sameFile(file, selectedFile.value)) {
      diff.value = String(err);
    }
  }
}

function scheduleWatchRefresh(refs: boolean) {
  watchRefreshRefs ||= refs;
  watchRefreshQueued = true;
  if (watchRefresh || worktreeMutation > 0 || actionBusy.value) {
    return;
  }
  watchRefresh = drainWatchRefresh().finally(() => {
    watchRefresh = null;
    if (watchRefreshQueued && worktreeMutation === 0 && !actionBusy.value) {
      scheduleWatchRefresh(false);
    }
  });
}

async function drainWatchRefresh() {
  while (watchRefreshQueued && worktreeMutation === 0 && !actionBusy.value) {
    watchRefreshQueued = false;
    const refs = watchRefreshRefs;
    watchRefreshRefs = false;
    await refreshFromWatch(refs);
  }
}

async function refreshFromWatch(refs: boolean) {
  const match = current.value;
  if (!match) {
    return;
  }
  if (refs) {
    await loadRepo({
      silent: true,
      graph: true,
      overview: branchesView.value,
    });
  } else {
    await loadWorkingTree();
  }
  await refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
  if (refs && selectedHistoryFile.value) {
    void refreshHistoryFile();
  }
}

async function onRepoFilesChanged(payload: RepoFilesChanged) {
  const match = current.value;
  if (!match || watchKey(match.repo.path) !== watchKey(payload.path)) {
    return;
  }
  if (actionBusy.value) {
    return;
  }
  scheduleWatchRefresh(payload.git);
  await watchRefresh;
}

function mergeOverview(previous: BranchOverview | null, next: BranchOverview): BranchOverview {
  if (!previous || previous.mergeTarget !== next.mergeTarget) {
    return next;
  }
  const incoming = new Map(next.branches.map((branch) => [branch.name, branch]));
  const kept = previous.branches.flatMap((branch) => {
    const update = incoming.get(branch.name);
    if (!update) {
      return [];
    }
    if (update.pending && !branch.pending) {
      return [
        {
          ...update,
          merged: branch.merged,
          partial: branch.partial,
          pending: false,
        },
      ];
    }
    return [update];
  });
  const seen = new Set(kept.map((branch) => branch.name));
  const added = next.branches.filter((branch) => !seen.has(branch.name));
  return { ...next, branches: [...kept, ...added] };
}

async function classifyOverview(
  path: string,
  preferred: string | undefined,
  generation: number,
) {
  const full = await api.branchOverview(path, preferred, true);
  if (generation !== overviewGeneration || current.value?.repo.path !== path || !branchesView.value) {
    return;
  }
  overview.value = mergeOverview(overview.value, full);
}

async function loadOverview() {
  const match = current.value;
  if (!match) {
    overview.value = null;
    return;
  }
  const generation = ++overviewGeneration;
  const path = match.repo.path;
  const preferred = preferredMergeTarget();
  const snapshot = await api.branchOverview(path, preferred, false);
  if (generation !== overviewGeneration) {
    return;
  }
  const needsClassify = snapshot.branches.some((branch) => branch.pending);
  overview.value = mergeOverview(overview.value, snapshot);
  if (needsClassify) {
    await classifyOverview(path, preferred, generation);
  }
}

function applyWorkingTree(nextFiles: WorkingTreeFile[], silent: boolean) {
  const match = current.value;
  files.value = nextFiles;
  if (match && nextFiles.some(isConflicted)) {
    openChangesPane();
    void refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
  }
  if (historyOpen.value) {
    void loadRepoFiles({ silent });
  }
  const nextSelected = selectedFile.value
    ? nextFiles.find((file) => sameFile(file, selectedFile.value))
    : undefined;
  if (selectedFile.value && !nextSelected) {
    selectedFile.value = null;
    if (!selectedCommitFile.value && !selectedHistoryCommit.value) {
      diff.value = "";
    }
  } else if (nextSelected) {
    selectedFile.value = nextSelected;
    void refreshSelectedFileDiff(nextSelected);
  }
}

async function loadWorkingTree() {
  const match = current.value;
  if (!match) {
    return;
  }
  const path = match.repo.path;
  const generation = ++filesGeneration;
  try {
    const nextFiles = await api.workingTree(path);
    if (generation !== filesGeneration || current.value?.repo.path !== path) {
      return;
    }
    applyWorkingTree(nextFiles, true);
  } catch {
    /* keep the list already on screen */
  }
}

async function loadRepo(options?: { overview?: boolean; graph?: boolean; silent?: boolean }) {
  const match = current.value;
  const generation = ++loadGeneration;
  const fileGeneration = ++filesGeneration;
  if (!match) {
    overviewGeneration += 1;
    trackingGeneration += 1;
    commits.value = [];
    files.value = [];
    branches.value = [];
    branchTracking.value = [];
    trackingPath.value = "";
    stashes.value = [];
    tags.value = [];
    overview.value = null;
    selectedFile.value = null;
    closeCommitDetail();
    message.value = loaded.value ? "Repository not found." : "";
    return;
  }

  const wantOverview = options?.overview ?? branchesView.value;
  const wantGraph = options?.graph ?? true;
  const overviewGen = wantOverview ? ++overviewGeneration : overviewGeneration;
  if (!options?.silent) {
    loading.value = true;
  }
  message.value = "";
  try {
    const [nextCommits, nextFiles, nextBranches, nextStashes, nextTags, nextOverview] = await Promise.all([
      wantGraph ? api.logGraph(match.repo.path) : Promise.resolve(commits.value),
      api.workingTree(match.repo.path),
      api.listLocalBranches(match.repo.path).catch(() => [] as string[]),
      api.stashList(match.repo.path).catch(() => [] as StashEntry[]),
      api.tagList(match.repo.path).catch(() => [] as TagEntry[]),
      wantOverview
        ? api.branchOverview(match.repo.path, preferredMergeTarget(), false).catch(() => null)
        : Promise.resolve(overview.value),
    ]);
    if (generation !== loadGeneration) {
      return;
    }
    if (wantGraph) {
      commits.value = nextCommits;
      graphStale.value = false;
    }
    if (fileGeneration === filesGeneration) {
      applyWorkingTree(nextFiles, Boolean(options?.silent));
    }
    branches.value = nextBranches;
    rememberTrackingPath(match.repo.path);
    void loadBranchTracking(match.repo.path);
    stashes.value = nextStashes;
    tags.value = nextTags;
    if (wantOverview) {
      const needsClassify = nextOverview?.branches.some((branch) => branch.pending) ?? false;
      overview.value = nextOverview ? mergeOverview(overview.value, nextOverview) : nextOverview;
      if (needsClassify && overviewGen === overviewGeneration) {
        void classifyOverview(match.repo.path, preferredMergeTarget(), overviewGen);
      }
    }
    if (selectedCommit.value) {
      const nextCommit = nextCommits.find((commit) => commit.hash === selectedCommit.value?.hash);
      if (!nextCommit) {
        closeCommitDetail();
      } else {
        selectedCommit.value = nextCommit;
        await refreshCommitFiles(nextCommit);
      }
    }
  } catch (err) {
    if (generation === loadGeneration && !options?.silent) {
      message.value = String(err);
    }
  } finally {
    if (generation === loadGeneration) {
      loading.value = false;
    }
  }
}

function sameFile(file: WorkingTreeFile, other: WorkingTreeFile | null) {
  return !!other && file.path === other.path && file.staged === other.staged;
}

function closeDiff() {
  selectedFile.value = null;
  selectedCommitFile.value = null;
  selectedHistoryCommit.value = null;
  diff.value = "";
}

function closeCommitDetail() {
  selectedCommit.value = null;
  selectedCommitFile.value = null;
  commitFiles.value = [];
  commitFilesLoading.value = false;
  if (!selectedFile.value) {
    diff.value = "";
  }
}

async function refreshCommitFiles(commit: CommitNode) {
  const match = current.value;
  if (!match) {
    return;
  }
  commitFilesLoading.value = true;
  try {
    commitFiles.value = await api.commitFiles(match.repo.path, commit.hash);
    if (
      selectedCommitFile.value &&
      !commitFiles.value.some((file) => file.path === selectedCommitFile.value?.path)
    ) {
      selectedCommitFile.value = null;
      if (!selectedFile.value) {
        diff.value = "";
      }
    }
  } catch (err) {
    message.value = String(err);
    commitFiles.value = [];
  } finally {
    commitFilesLoading.value = false;
  }
}

async function selectCommit(commit: CommitNode) {
  const match = current.value;
  if (!match) {
    return;
  }
  if (selectedCommit.value?.hash === commit.hash) {
    closeCommitDetail();
    return;
  }
  selectedFile.value = null;
  selectedCommit.value = commit;
  selectedCommitFile.value = null;
  diff.value = "";
  openChangesPane();
  await refreshCommitFiles(commit);
  const first = commitFiles.value[0];
  if (first) {
    await selectCommitFile(first);
  }
}

async function selectCommitFile(file: CommitFile) {
  const match = current.value;
  const commit = selectedCommit.value;
  if (!match || !commit) {
    return;
  }
  if (selectedCommitFile.value?.path === file.path) {
    closeDiff();
    return;
  }
  selectedFile.value = null;
  selectedCommitFile.value = file;
  try {
    diff.value = await api.commitFileDiff(match.repo.path, commit.hash, file.path);
  } catch (err) {
    diff.value = String(err);
  }
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
  closeCommitDetail();
  selectedFile.value = file;
  try {
    diff.value = await api.fileDiff(match.repo.path, file.path, file.staged);
  } catch (err) {
    diff.value = String(err);
  }
}

async function runWorktreeMutation(work: () => Promise<void>) {
  worktreeMutation += 1;
  let succeeded = false;
  try {
    await work();
    succeeded = true;
    await loadWorkingTree();
    const match = current.value;
    if (match) {
      await refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
    }
  } finally {
    worktreeMutation -= 1;
    if (worktreeMutation > 0) {
      return;
    }
    if (watchRefreshRefs || (!succeeded && watchRefreshQueued)) {
      scheduleWatchRefresh(watchRefreshRefs);
    } else {
      watchRefreshQueued = false;
      watchRefreshRefs = false;
    }
  }
}

async function stageFile(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    await runWorktreeMutation(() => api.stageFile(match.repo.path, file.path));
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
    await runWorktreeMutation(() => api.stageAll(match.repo.path));
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
    await runWorktreeMutation(() => api.unstageFile(match.repo.path, file.path));
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
    await runWorktreeMutation(() => api.unstageAll(match.repo.path));
  } catch (err) {
    message.value = String(err);
  }
}

function markOverviewCurrent(name: string) {
  const currentOverview = overview.value;
  if (!currentOverview) {
    return;
  }
  const nextBranches = currentOverview.branches.map((branch) => ({
    ...branch,
    current: branch.name === name,
  }));
  nextBranches.sort(
    (left, right) =>
      Number(right.current) - Number(left.current) || left.name.localeCompare(right.name),
  );
  overview.value = { ...currentOverview, branches: nextBranches };
}

async function runRepoAction(label: string, work: () => Promise<string>, branch = "") {
  const match = current.value;
  if (!match || actionBusy.value) {
    return;
  }
  actionBusy.value = true;
  actionLabel.value = label;
  actionBranch.value = branch;
  message.value = "";
  try {
    const result = await work();
    showToast(result);
    await loadRepo();
    await refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
  } catch (err) {
    const text = String(err);
    message.value = text;
    showToast(text, "error");
    await loadRepo({ silent: true });
    await refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
    if (conflictActive.value) {
      openChangesPane();
      message.value = "";
      const first = conflictedFiles.value[0];
      if (first) {
        await selectFile(first);
      }
    }
  } finally {
    actionBusy.value = false;
    actionLabel.value = "";
    actionBranch.value = "";
  }
}

const pullBranch = computed(() => {
  if (pullSource.value === "current") {
    return "";
  }
  if (pullSource.value === "specify") {
    return specifyBranch.value.trim();
  }
  return pullSource.value;
});

const canConfirmPull = computed(
  () => pullSource.value === "current" || Boolean(pullBranch.value),
);

const pullRemoteLabel = computed(() => {
  if (pullSource.value === "current") {
    return "current-branch";
  }
  return pullBranch.value || "…";
});

const pullHint = computed(() =>
  pullSource.value === "current"
    ? "Use this to pick up others’ commits on the same branch."
    : "Brings that remote branch into this checkout. Conflicts appear in the files list so you can open them, mark them resolved, or abort.",
);

async function runPull(branch?: string) {
  const match = current.value;
  if (!match || actionBusy.value) {
    return;
  }
  actionBusy.value = true;
  actionLabel.value = "Pulling…";
  actionBranch.value = branch || match.status?.branch || "";
  message.value = "";
  const groupId = match.group?.id ?? STANDALONE_GROUP_ID;
  const name =
    match.status?.name ??
    match.repo.path.split("/").filter(Boolean).pop() ??
    match.repo.path;
  try {
    const result = await api.pullRepo(groupId, match.repo.id, branch);
    presentActionResults(
      branch ? `Pull ${branch}` : "Pull",
      [result],
      {
        success: branch ? `Pulled ${branch} into ${name}.` : `Pulled ${name}.`,
        error: `Pull failed for ${name}.`,
      },
    );
    await loadRepo({ silent: !result.ok });
    await refreshRepoStatus(groupId, match.repo.id);
    if (!result.ok && conflictActive.value) {
      openChangesPane();
      message.value = "";
      const first = conflictedFiles.value[0];
      if (first) {
        await selectFile(first);
      }
    }
  } catch (err) {
    const text = String(err);
    message.value = text;
    showToast(text, "error");
    await loadRepo({ silent: true });
    await refreshRepoStatus(groupId, match.repo.id);
    if (conflictActive.value) {
      openChangesPane();
      message.value = "";
      const first = conflictedFiles.value[0];
      if (first) {
        await selectFile(first);
      }
    }
  } finally {
    actionBusy.value = false;
    actionLabel.value = "";
    actionBranch.value = "";
  }
}

function fetchRepo() {
  const match = current.value;
  if (!match) {
    return;
  }
  const name = repoDisplayName(match.repo.id, match.repo.path);
  return runRepoAction("Fetching…", async () => {
    await api.refreshRepo(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id, true);
    return `Fetched ${name}.`;
  });
}

function pullRepo() {
  return runPull();
}

function openPullOptions() {
  if (actionBusy.value) {
    return;
  }
  pullSource.value = "current";
  specifyBranch.value = preferredMergeTarget() || "develop";
  pullingOptions.value = true;
}

function closePullOptions() {
  pullingOptions.value = false;
}

function confirmPull() {
  if (!canConfirmPull.value) {
    return;
  }
  const branch = pullBranch.value;
  closePullOptions();
  return runPull(branch || undefined);
}

function pushRepo() {
  const match = current.value;
  if (!match) {
    return;
  }
  return runRepoAction("Pushing…", () => api.repoPush(match.repo.path), match.status?.branch ?? "");
}

async function undoUnpushedCommits() {
  const match = current.value;
  const ahead = match?.status?.ahead ?? 0;
  if (!match || ahead < 1 || actionBusy.value) {
    return;
  }
  const countLabel = ahead === 1 ? "1 unpushed commit" : `${ahead} unpushed commits`;
  const ok = await confirm(
    `Undo ${countLabel} on this branch? The branch moves back to match the remote, and the changes stay staged. Nothing is removed from the remote.`,
    {
      title: "Undo unpushed commits",
      kind: "warning",
      okLabel: "Undo commits",
      cancelLabel: "Cancel",
    },
  );
  if (!ok) {
    return;
  }
  openChangesPane();
  return runRepoAction("Undoing commits…", () => api.resetUnpushedCommits(match.repo.path));
}

async function checkoutBranch(branch: string) {
  const match = current.value;
  if (!match || actionBusy.value) {
    return;
  }
  const previous = match.status?.branch ?? "";
  if (previous === branch) {
    return;
  }
  markOverviewCurrent(branch);
  patchRepoStatus(match.repo.id, { branch });
  actionBusy.value = true;
  actionLabel.value = "Checking out…";
  actionBranch.value = branch;
  message.value = "";
  await nextTick();
  try {
    const result = await api.checkoutLocalBranch(match.repo.path, branch);
    showToast(result);
  } catch (err) {
    markOverviewCurrent(previous);
    if (previous) {
      patchRepoStatus(match.repo.id, { branch: previous });
    }
    const text = String(err);
    message.value = text;
    showToast(text, "error");
    return;
  } finally {
    actionBusy.value = false;
    actionLabel.value = "";
    actionBranch.value = "";
  }
  graphStale.value = true;
  void loadRepo({
    overview: false,
    graph: !branchesView.value,
    silent: true,
  });
  void refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
}

function checkoutListedBranch(branch: LocalBranch) {
  if (branch.current) {
    return;
  }
  return checkoutBranch(branch.name);
}

async function openCreateBranch(start = "") {
  if (actionBusy.value) {
    return;
  }
  newBranchName.value = "";
  newBranchStart.value = start;
  if (!start.trim()) {
    await refreshBranches();
    baseBranch.value = checkedOutBranch.value || branches.value[0] || "";
  } else {
    baseBranch.value = "";
  }
  creatingBranch.value = true;
  await nextTick();
  newBranchInput.value?.focus();
}

function closeCreateBranch() {
  creatingBranch.value = false;
  newBranchName.value = "";
  baseBranch.value = "";
  newBranchStart.value = "";
}

function createBranch() {
  const match = current.value;
  const branch = newBranchName.value.trim();
  const start = newBranchStart.value.trim() || baseBranch.value.trim();
  if (!match || !branch || !start) {
    return;
  }
  closeCreateBranch();
  return runRepoAction(
    "Creating branch…",
    () => api.createAndCheckoutBranch(match.repo.path, branch, start),
    branch,
  );
}

async function openRenameBranch(branch: LocalBranch) {
  if (actionBusy.value) {
    return;
  }
  renamingBranch.value = branch;
  renameBranchName.value = branch.name;
  await nextTick();
  renameBranchInput.value?.focus();
  renameBranchInput.value?.select();
}

function closeRenameBranch() {
  renamingBranch.value = null;
  renameBranchName.value = "";
}

function pickMergeTarget() {
  const names = localBranchNames.value;
  const preferred = [mergeTargetHint.value, "develop", "main", "master"];
  for (const name of preferred) {
    if (name && names.includes(name)) {
      return name;
    }
  }
  return names[0] ?? "";
}

function pickMergeSource(preferred: string, target: string) {
  const names = localBranchNames.value;
  if (preferred && preferred !== target && names.includes(preferred)) {
    return preferred;
  }
  const currentName = current.value?.status?.branch ?? "";
  if (currentName && currentName !== target && names.includes(currentName)) {
    return currentName;
  }
  return names.find((name) => name !== target) ?? "";
}

async function openMergeBranch(branch?: LocalBranch) {
  if (actionBusy.value || localBranchNames.value.length < 2) {
    return;
  }
  if (!overview.value) {
    await loadOverview().catch(() => undefined);
  }
  const preferredSource = branch?.name ?? current.value?.status?.branch ?? "";
  const target = pickMergeTarget();
  mergeTarget.value = target;
  mergeSource.value = pickMergeSource(preferredSource, target);
  mergingBranch.value = true;
}

function closeMergeBranch() {
  mergingBranch.value = false;
  mergeSource.value = "";
  mergeTarget.value = "";
}

function mergeLocalBranch() {
  const match = current.value;
  const source = mergeSource.value.trim();
  const target = mergeTarget.value.trim();
  if (!match || !canConfirmMerge.value) {
    return;
  }
  closeMergeBranch();
  return runRepoAction(
    "Merging…",
    () => api.mergeLocalBranch(match.repo.path, source, target),
    target,
  );
}

function renameBranch() {
  const match = current.value;
  const from = renamingBranch.value;
  const to = renameBranchName.value.trim();
  if (!match || !from || !to || to === from.name) {
    return;
  }
  closeRenameBranch();
  return runRepoAction("Renaming…", () => api.renameLocalBranch(match.repo.path, from.name, to));
}

function clipCommitTitle(value: string) {
  return [...value].slice(0, COMMIT_TITLE_MAX).join("");
}

type CommitDraft = { title: string; description: string };
const commitDrafts = new Map<string, CommitDraft>();
const hasCommitDraft = ref(false);

function repoDraftKey() {
  return current.value?.repo.path ?? "";
}

function isMeaningfulDraft(draft: CommitDraft | undefined) {
  return Boolean(draft && (draft.title.trim() || draft.description.trim()));
}

function refreshHasCommitDraft() {
  hasCommitDraft.value = isMeaningfulDraft(commitDrafts.get(repoDraftKey()));
}

function persistCommitDraft() {
  const key = repoDraftKey();
  if (!key) {
    return;
  }
  const title = amending.value ? draftTitle.value : commitTitle.value;
  const description = amending.value ? draftDescription.value : commitDescription.value;
  const draft = { title, description };
  if (isMeaningfulDraft(draft)) {
    commitDrafts.set(key, draft);
  } else {
    commitDrafts.delete(key);
  }
  refreshHasCommitDraft();
}

function loadCommitDraft() {
  const draft = commitDrafts.get(repoDraftKey());
  commitTitle.value = draft?.title ?? "";
  commitDescription.value = draft?.description ?? "";
  refreshHasCommitDraft();
}

function clearCommitDraft() {
  const key = repoDraftKey();
  if (key) {
    commitDrafts.delete(key);
  }
  amending.value = false;
  lastCommit.value = null;
  draftTitle.value = "";
  draftDescription.value = "";
  commitTitle.value = "";
  commitDescription.value = "";
  refreshHasCommitDraft();
}

function applyLastCommitMessage() {
  const last = lastCommit.value;
  if (!last) {
    return;
  }
  commitTitle.value = clipCommitTitle(last.title);
  commitDescription.value = last.description;
}

async function openCommit() {
  if (actionBusy.value || !files.value.length) {
    return;
  }
  if (operation.value && operation.value !== "merge") {
    return;
  }
  amending.value = false;
  lastCommit.value = null;
  loadCommitDraft();
  committing.value = true;
  const match = current.value;
  if (match) {
    lastCommit.value = await api.lastCommit(match.repo.path).catch(() => null);
  }
  await nextTick();
  commitTitleInput.value?.focus();
}

function closeCommit() {
  persistCommitDraft();
  amending.value = false;
  lastCommit.value = null;
  committing.value = false;
  loadCommitDraft();
}

function onAmendChange(event: Event) {
  const checked = (event.target as HTMLInputElement).checked;
  if (checked) {
    draftTitle.value = commitTitle.value;
    draftDescription.value = commitDescription.value;
    amending.value = true;
    applyLastCommitMessage();
    return;
  }
  commitTitle.value = draftTitle.value;
  commitDescription.value = draftDescription.value;
  amending.value = false;
}

async function commitChanges() {
  const match = current.value;
  const title = commitTitle.value.trim();
  if (!match || !title || stagedCount.value === 0) {
    return;
  }
  const description = commitDescription.value;
  const amend = amending.value;
  if (amend && lastCommit.value?.published) {
    const ok = await confirm(
      "This commit is already on the remote. Amending rewrites history, and you will need to force-push.",
      {
        title: "Amend published commit",
        kind: "warning",
        okLabel: "Amend",
        cancelLabel: "Cancel",
      },
    );
    if (!ok) {
      return;
    }
  }
  closeCommit();
  return runRepoAction(amend ? "Amending…" : "Committing…", async () => {
    const result = await api.commit(match.repo.path, title, description, amend);
    clearCommitDraft();
    return result;
  });
}

async function openStash() {
  if (actionBusy.value || !files.value.length) {
    return;
  }
  stashMessage.value = "";
  stashing.value = true;
  await nextTick();
  stashMessageInput.value?.focus();
}

function closeStash() {
  stashing.value = false;
  stashMessage.value = "";
}

function stashChanges() {
  const match = current.value;
  if (!match || !files.value.length) {
    return;
  }
  const message = stashMessage.value;
  closeStash();
  closeDiff();
  return runRepoAction("Stashing…", () => api.stashPush(match.repo.path, message));
}

function preferredMergeTarget() {
  return current.value?.group?.pullFromBranch?.trim() || undefined;
}

function removeOverviewBranches(names: string[]) {
  const gone = new Set(names);
  if (!overview.value || !gone.size) {
    return;
  }
  overview.value = {
    ...overview.value,
    branches: overview.value.branches.filter((branch) => !gone.has(branch.name)),
  };
}

async function toggleBranchesView() {
  branchesView.value = !branchesView.value;
  if (!branchesView.value) {
    if (graphStale.value) {
      void loadRepo({ overview: false, silent: true });
    }
    return;
  }
  stashView.value = false;
  tagView.value = false;
  await nextTick();
  try {
    await loadOverview();
  } catch (err) {
    message.value = String(err);
  }
}

function toggleStashView() {
  stashView.value = !stashView.value;
  if (stashView.value) {
    branchesView.value = false;
    tagView.value = false;
    return;
  }
  if (graphStale.value) {
    void loadRepo({ overview: false, silent: true });
  }
}

function toggleTagView() {
  tagView.value = !tagView.value;
  if (tagView.value) {
    branchesView.value = false;
    stashView.value = false;
    return;
  }
  if (graphStale.value) {
    void loadRepo({ overview: false, silent: true });
  }
}

function toggleTerminal() {
  terminalOpen.value = !terminalOpen.value;
}

function openChangesPane() {
  filesCollapsed.value = false;
  historyOpen.value = false;
  closeHistoryFile();
  lastHistoryFile.value = "";
}

function toggleChangesPane() {
  if (!filesCollapsed.value && !historyOpen.value) {
    filesCollapsed.value = true;
    return;
  }
  openChangesPane();
}

function toggleHistoryPane() {
  if (!filesCollapsed.value && historyOpen.value) {
    filesCollapsed.value = true;
    historyOpen.value = false;
    closeHistoryFile();
    lastHistoryFile.value = "";
    return;
  }
  filesCollapsed.value = false;
  historyOpen.value = true;
  void loadRepoFiles();
}

function closeHistoryFile() {
  selectedHistoryFile.value = "";
  historyCommits.value = [];
  historyCommitsError.value = "";
  historyCommitsGeneration += 1;
  selectedHistoryCommit.value = null;
  if (!selectedFile.value && !selectedCommitFile.value) {
    diff.value = "";
  }
}

async function selectHistoryFile(path: string) {
  const match = current.value;
  if (!match) {
    return;
  }
  closeCommitDetail();
  selectedFile.value = null;
  selectedHistoryFile.value = path;
  lastHistoryFile.value = path;
  selectedHistoryCommit.value = null;
  diff.value = "";
  const generation = ++historyCommitsGeneration;
  historyCommitsLoading.value = true;
  historyCommitsError.value = "";
  try {
    const next = await api.fileLog(match.repo.path, path);
    if (generation !== historyCommitsGeneration) {
      return;
    }
    historyCommits.value = next;
  } catch (err) {
    if (generation !== historyCommitsGeneration) {
      return;
    }
    historyCommits.value = [];
    historyCommitsError.value = String(err);
  } finally {
    if (generation === historyCommitsGeneration) {
      historyCommitsLoading.value = false;
    }
  }
}

async function selectHistoryCommit(commit: CommitNode) {
  const match = current.value;
  const file = commit.path || selectedHistoryFile.value;
  if (!match || !file) {
    return;
  }
  if (selectedHistoryCommit.value?.hash === commit.hash) {
    closeDiff();
    return;
  }
  selectedFile.value = null;
  selectedCommitFile.value = null;
  selectedHistoryCommit.value = commit;
  try {
    diff.value = await api.commitFileDiff(match.repo.path, commit.hash, file);
  } catch (err) {
    diff.value = String(err);
  }
}

async function loadRepoFiles(options?: { silent?: boolean }) {
  const match = current.value;
  if (!match) {
    repoFiles.value = [];
    repoFilesError.value = "";
    return;
  }
  const generation = ++repoFilesGeneration;
  if (!options?.silent) {
    repoFilesLoading.value = true;
  }
  repoFilesError.value = "";
  try {
    const next = await api.repoFiles(match.repo.path);
    if (generation !== repoFilesGeneration) {
      return;
    }
    repoFiles.value = next;
  } catch (err) {
    if (generation !== repoFilesGeneration) {
      return;
    }
    if (!options?.silent) {
      repoFiles.value = [];
      repoFilesError.value = String(err);
    }
  } finally {
    if (generation === repoFilesGeneration) {
      repoFilesLoading.value = false;
    }
  }
}

function stashRef(index: number) {
  return `stash@{${index}}`;
}

async function applyStash(stash: StashEntry) {
  const match = current.value;
  if (!match) {
    return;
  }
  if (files.value.length) {
    const ok = await confirm(
      `You have uncommitted changes. Applying ${stashRef(stash.index)} may cause conflicts.`,
      {
        title: "Apply stash",
        kind: "warning",
        okLabel: "Apply",
        cancelLabel: "Cancel",
      },
    );
    if (!ok) {
      return;
    }
  }
  return runRepoAction("Applying stash…", () => api.stashApply(match.repo.path, stash.index));
}

async function popStash(stash: StashEntry) {
  const match = current.value;
  if (!match) {
    return;
  }
  const ok = await confirm(
    files.value.length
      ? `You have uncommitted changes. Pop ${stashRef(stash.index)} anyway? It will be removed if it applies cleanly.`
      : `Apply ${stashRef(stash.index)} and remove it from the stash list?`,
    {
      title: "Pop stash",
      kind: "warning",
      okLabel: "Pop",
      cancelLabel: "Cancel",
    },
  );
  if (!ok) {
    return;
  }
  return runRepoAction("Popping stash…", () => api.stashPop(match.repo.path, stash.index));
}

async function dropStash(stash: StashEntry) {
  const match = current.value;
  if (!match) {
    return;
  }
  const ok = await confirm(
    `Permanently delete ${stashRef(stash.index)}? This cannot be undone.`,
    {
      title: "Drop stash",
      kind: "warning",
      okLabel: "Drop",
      cancelLabel: "Cancel",
    },
  );
  if (!ok) {
    return;
  }
  return runRepoAction("Dropping stash…", () => api.stashDrop(match.repo.path, stash.index));
}

async function openCreateTag(target = "") {
  if (actionBusy.value) {
    return;
  }
  newTagName.value = "";
  newTagMessage.value = "";
    newTagTarget.value = target || selectedCommit.value?.hash.slice(0, 12) || "";
  creatingTag.value = true;
  await nextTick();
  newTagInput.value?.focus();
}

function openCommitMenu(commit: CommitNode, hashes: string[], x: number, y: number) {
  commitMenu.value = { commit, hashes, x, y };
}

function closeCommitMenu() {
  commitMenu.value = null;
}

function menuHashes(oldestFirst: boolean) {
  const hashes = commitMenu.value?.hashes ?? [];
  const index = new Map(commits.value.map((commit, i) => [commit.hash, i]));
  return [...hashes].sort((left, right) => {
    const a = index.get(left) ?? 0;
    const b = index.get(right) ?? 0;
    return oldestFirst ? b - a : a - b;
  });
}

function checkoutMenuCommit() {
  const match = current.value;
  const commit = commitMenu.value?.commit;
  closeCommitMenu();
  if (!match || !commit) {
    return;
  }
  return runRepoAction("Checking out…", () => api.checkoutCommit(match.repo.path, commit.hash));
}

function createBranchFromMenu() {
  const hash = commitMenu.value?.commit.hash ?? "";
  closeCommitMenu();
  if (!hash) {
    return;
  }
  return openCreateBranch(hash);
}

function cherryPickMenuCommits() {
  const match = current.value;
  const hashes = menuHashes(true);
  closeCommitMenu();
  if (!match || !hashes.length) {
    return;
  }
  return runRepoAction("Cherry-picking…", () => api.cherryPickCommits(match.repo.path, hashes));
}

function revertMenuCommits() {
  const match = current.value;
  const hashes = menuHashes(false);
  closeCommitMenu();
  if (!match || !hashes.length) {
    return;
  }
  return runRepoAction("Reverting…", () => api.revertCommits(match.repo.path, hashes));
}

async function copyMenuShas() {
  const hashes = commitMenu.value?.hashes ?? [];
  closeCommitMenu();
  if (!hashes.length) {
    return;
  }
  try {
    await navigator.clipboard.writeText(hashes.join("\n"));
    showToast(hashes.length === 1 ? "Copied commit SHA" : `Copied ${hashes.length} commit SHAs`);
  } catch (err) {
    showToast(String(err), "error");
  }
}

async function copyMenuLink() {
  const match = current.value;
  const hash = commitMenu.value?.commit.hash ?? "";
  closeCommitMenu();
  if (!match || !hash) {
    return;
  }
  try {
    const url = await api.commitRemoteUrl(match.repo.path, hash);
    await navigator.clipboard.writeText(url);
    showToast("Copied commit link");
  } catch (err) {
    showToast(String(err), "error");
  }
}

function createTagFromMenu() {
  const hash = commitMenu.value?.commit.hash ?? "";
  closeCommitMenu();
  if (!hash) {
    return;
  }
  return openCreateTag(hash);
}

function closeCreateTag() {
  creatingTag.value = false;
  newTagName.value = "";
  newTagMessage.value = "";
  newTagTarget.value = "";
}

function createTag() {
  const match = current.value;
  const name = newTagName.value.trim();
  if (!match || !name) {
    return;
  }
  const message = newTagMessage.value;
  const target = newTagTarget.value.trim();
  closeCreateTag();
  return runRepoAction("Creating tag…", () => api.createTag(match.repo.path, name, message, target));
}

async function deleteTag(tag: TagEntry) {
  const match = current.value;
  if (!match) {
    return;
  }
  const ok = await confirm(`Permanently delete tag ${tag.name}? This cannot be undone.`, {
    title: "Delete tag",
    kind: "warning",
    okLabel: "Delete",
    cancelLabel: "Cancel",
  });
  if (!ok) {
    return;
  }
  return runRepoAction("Deleting tag…", () => api.deleteTag(match.repo.path, tag.name));
}

async function refreshBranches() {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    branches.value = await api.listLocalBranches(match.repo.path);
    rememberTrackingPath(match.repo.path);
    void loadBranchTracking(match.repo.path);
  } catch {
    /* keep the last successful list */
  }
}

function rememberTrackingPath(path: string) {
  if (trackingPath.value === path) {
    return;
  }
  trackingPath.value = path;
  trackingGeneration += 1;
  branchTracking.value = [];
}

async function loadBranchTracking(path: string) {
  const generation = ++trackingGeneration;
  try {
    const next = await api.listBranchTracking(path);
    if (generation !== trackingGeneration) {
      return;
    }
    branchTracking.value = next;
  } catch {
    /* keep the last successful list */
  }
}

function notFullyMerged(err: unknown) {
  return String(err).toLowerCase().includes("not fully merged");
}

async function deleteBranch(branch: LocalBranch) {
  const match = current.value;
  if (!match || branch.current) {
    return;
  }
  const target = overview.value?.mergeTarget ?? "the integration branch";
  const force = !branch.merged;
  const ok = await confirm(
    force
      ? branch.partial
        ? `${branch.name} is only partially merged into ${target}. Some commits are still unique. Delete this local branch anyway?`
        : `${branch.name} is not fully merged into ${target}. Delete this local branch anyway?`
      : branch.protected
        ? `Delete local branch ${branch.name}? This is a protected integration branch.`
        : `Delete local branch ${branch.name}? It is already merged into ${target}.`,
    {
      title: force
        ? branch.partial
          ? "Delete partial branch"
          : "Delete unmerged branch"
        : "Delete branch",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    },
  );
  if (!ok) {
    return;
  }
  return runRepoAction("Deleting…", async () => {
    try {
      return await api.deleteLocalBranch(match.repo.path, branch.name, force);
    } catch (err) {
      if (
        force ||
        !notFullyMerged(err) ||
        !(await confirm(
          `${branch.name} is marked merged into ${target}, but git will not delete it safely. Force delete this local branch?`,
          {
            title: "Force delete branch",
            kind: "warning",
            okLabel: "Force delete",
            cancelLabel: "Keep",
          },
        ))
      ) {
        throw err;
      }
      return api.deleteLocalBranch(match.repo.path, branch.name, true);
    }
  });
}

function selectedDeleteCopy(branches: LocalBranch[], target: string) {
  const merged = branches.filter((branch) => branch.merged).length;
  const partial = branches.filter((branch) => !branch.merged && branch.partial).length;
  const unique = branches.filter((branch) => !branch.merged && !branch.partial).length;
  const bits: string[] = [];
  if (merged) {
    bits.push(
      `${merged} ${merged === 1 ? "is" : "are"} already merged into ${target}`,
    );
  }
  if (partial) {
    bits.push(`${partial} ${partial === 1 ? "is" : "are"} only partially merged`);
  }
  if (unique) {
    bits.push(`${unique} ${unique === 1 ? "has" : "have"} unique work`);
  }
  const noun = branches.length === 1 ? "branch" : "branches";
  return `Delete ${branches.length} local ${noun}?${bits.length ? ` ${bits.join(". ")}.` : ""}`;
}

async function deleteSelectedBranches(branches: LocalBranch[]) {
  const match = current.value;
  const victims = branches.filter((branch) => !branch.current && !branch.protected);
  if (!match || !victims.length) {
    return;
  }
  const target = overview.value?.mergeTarget ?? "the integration branch";
  const ok = await confirm(selectedDeleteCopy(victims, target), {
    title: "Delete selected branches",
    kind: "warning",
    okLabel: "Delete",
    cancelLabel: "Cancel",
  });
  if (!ok) {
    return;
  }
  if (actionBusy.value) {
    return;
  }
  actionBusy.value = true;
  actionLabel.value = "Deleting selected branches…";
  message.value = "";
  try {
    const safe = victims.filter((branch) => branch.merged).map((branch) => branch.name);
    const forced = victims.filter((branch) => !branch.merged).map((branch) => branch.name);
    const deleted: string[] = [];
    const errors: string[] = [];
    const notes: string[] = [];

    if (safe.length) {
      let result = await api.deleteMergedBranches(
        match.repo.path,
        preferredMergeTarget(),
        false,
        safe,
      );
      deleted.push(...result.deleted);
      errors.push(...result.errors);
      if (result.message) {
        notes.push(result.message);
      }
      if (result.refused.length) {
        const refused = result.refused.join(", ");
        const forceOk = await confirm(
          result.deleted.length
            ? `Deleted ${result.deleted.length}. Git would not safely delete ${refused}. Those branches are already contained in ${target}, but not fully merged into the branch you're on (squash merges and unmerged remotes do this). Force delete them?`
            : `Git would not safely delete ${refused}. Those leftover branches are already contained in ${target}, but not fully merged into the branch you're on (squash merges and unmerged remotes do this). Force delete them?`,
          {
            title: "Force delete leftover branches",
            kind: "warning",
            okLabel: "Force delete",
            cancelLabel: "Keep",
          },
        );
        if (forceOk) {
          const extra = await api.deleteMergedBranches(
            match.repo.path,
            preferredMergeTarget(),
            true,
            result.refused,
          );
          deleted.push(...extra.deleted);
          errors.push(...extra.errors);
          if (extra.message) {
            notes.push(extra.message);
          }
        }
      }
    }

    if (forced.length) {
      const result = await api.deleteMergedBranches(
        match.repo.path,
        preferredMergeTarget(),
        true,
        forced,
      );
      deleted.push(...result.deleted);
      errors.push(...result.errors);
      if (result.message) {
        notes.push(result.message);
      }
    }

    const text = notes.filter(Boolean).join(" ") || `Deleted ${deleted.length} local branches.`;
    const failed = deleted.length === 0 && errors.length > 0;
    if (failed) {
      message.value = text;
    }
    showToast(text, failed ? "error" : "success");
    overviewGeneration += 1;
    removeOverviewBranches(deleted);
    await loadRepo({ overview: false, graph: !branchesView.value, silent: true });
    if (branchesView.value && overview.value?.branches.some((branch) => branch.pending)) {
      void loadOverview();
    }
    await refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
  } catch (err) {
    const text = String(err);
    message.value = text;
    showToast(text, "error");
  } finally {
    actionBusy.value = false;
    actionLabel.value = "";
  }
}

async function deleteMerged() {
  const match = current.value;
  const count =
    overview.value?.branches.filter(
      (branch) => branch.merged && !branch.current && !branch.protected && !branch.pending,
    ).length ?? 0;
  if (!match || count === 0) {
    return;
  }
  const target = overview.value?.mergeTarget ?? "the integration branch";
  const ok = await confirm(
    `Delete ${count} leftover local ${count === 1 ? "branch" : "branches"} already merged into ${target}? Partial and unique branches stay. This never deletes develop, main, master, or the current branch.`,
    {
      title: "Delete merged branches",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    },
  );
  if (!ok) {
    return;
  }
  if (actionBusy.value) {
    return;
  }
  actionBusy.value = true;
  actionLabel.value = "Deleting merged branches…";
  message.value = "";
  try {
    const victims =
      overview.value?.branches
        .filter((branch) => branch.merged && !branch.current && !branch.protected && !branch.pending)
        .map((branch) => branch.name) ?? [];
    let result = await api.deleteMergedBranches(
      match.repo.path,
      preferredMergeTarget(),
      false,
      victims,
    );
    if (result.refused.length) {
      const refused = result.refused.join(", ");
      const forceOk = await confirm(
        result.deleted.length
          ? `Deleted ${result.deleted.length}. Git would not safely delete ${refused}. Those branches are already contained in ${target}, but not fully merged into the branch you're on (squash merges and unmerged remotes do this). Force delete them?`
          : `Git would not safely delete ${refused}. Those leftover branches are already contained in ${target}, but not fully merged into the branch you're on (squash merges and unmerged remotes do this). Force delete them?`,
        {
          title: "Force delete leftover branches",
          kind: "warning",
          okLabel: "Force delete",
          cancelLabel: "Keep",
        },
      );
      if (forceOk) {
        const forced = await api.deleteMergedBranches(
          match.repo.path,
          preferredMergeTarget(),
          true,
          result.refused,
        );
        result = {
          deleted: [...result.deleted, ...forced.deleted],
          refused: forced.refused,
          errors: [...result.errors, ...forced.errors],
          message: [result.message, forced.message].filter(Boolean).join(" "),
        };
      }
    }
    const failed = result.deleted.length === 0 && result.errors.length > 0;
    const text = result.message;
    if (failed) {
      message.value = text;
    }
    showToast(text, failed ? "error" : "success");
    overviewGeneration += 1;
    removeOverviewBranches(result.deleted);
    await loadRepo({ overview: false, graph: !branchesView.value, silent: true });
    if (branchesView.value && overview.value?.branches.some((branch) => branch.pending)) {
      void loadOverview();
    }
    await refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
  } catch (err) {
    const text = String(err);
    message.value = text;
    showToast(text, "error");
  } finally {
    actionBusy.value = false;
    actionLabel.value = "";
  }
}

async function openInEditor(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    await api.openInEditor(match.repo.path, file.path);
  } catch (err) {
    message.value = String(err);
    showToast(String(err), "error");
  }
}

async function ignoreFile(file: WorkingTreeFile, kind: IgnoreKind) {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    await runWorktreeMutation(async () => {
      await api.ignoreWorkingTreePath(match.repo.path, file.path, kind);
      if (selectedFile.value?.path === file.path) {
        closeDiff();
      }
    });
  } catch (err) {
    message.value = String(err);
    showToast(String(err), "error");
  }
}

function stashFile(file: WorkingTreeFile) {
  const match = current.value;
  if (!match || actionBusy.value) {
    return;
  }
  closeDiff();
  return runRepoAction("Stashing…", () => api.stashFile(match.repo.path, file.path));
}

async function revealFile(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    await api.revealFileInFinder(match.repo.path, file.path);
  } catch (err) {
    message.value = String(err);
    showToast(String(err), "error");
  }
}

async function copyFilePath(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  try {
    await navigator.clipboard.writeText(absoluteFilePath(match.repo.path, file.path));
  } catch (err) {
    message.value = String(err);
    showToast(String(err), "error");
  }
}

async function deleteFile(file: WorkingTreeFile) {
  const match = current.value;
  if (!match) {
    return;
  }
  const ok = await confirm(`Delete ${fileBasename(file.path)}? This cannot be undone.`, {
    title: "Delete file",
    kind: "warning",
    okLabel: "Delete",
    cancelLabel: "Cancel",
  });
  if (!ok) {
    return;
  }
  try {
    await runWorktreeMutation(async () => {
      await api.deleteWorkingTreeFile(match.repo.path, file.path);
      if (selectedFile.value?.path === file.path) {
        closeDiff();
      }
    });
  } catch (err) {
    message.value = String(err);
    showToast(String(err), "error");
  }
}

async function abortCurrentOperation() {
  const match = current.value;
  if (!match || !operation.value || actionBusy.value) {
    return;
  }
  const noun = operationNoun(operation.value);
  const ok = await confirm(
    `Abort this ${noun}? The repository returns to the state before the ${noun} started.`,
    {
      title: abortLabel(operation.value),
      kind: "warning",
      okLabel: abortLabel(operation.value),
      cancelLabel: "Cancel",
    },
  );
  if (!ok) {
    return;
  }
  closeDiff();
  return runRepoAction("Aborting…", () => api.abortOperation(match.repo.path));
}

async function continueCurrentOperation() {
  const match = current.value;
  if (!match || !operation.value || actionBusy.value || conflictedCount.value) {
    return;
  }
  closeDiff();
  return runRepoAction("Continuing…", () => api.continueOperation(match.repo.path));
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
    await runWorktreeMutation(async () => {
      await api.discardAllChanges(match.repo.path);
      closeDiff();
    });
  } catch (err) {
    message.value = String(err);
  }
}

watch(
  () => props.repoId,
  () => {
    branchesView.value = false;
    stashView.value = false;
    tagView.value = false;
    historyOpen.value = false;
    repoFiles.value = [];
    repoFilesError.value = "";
    repoFilesGeneration += 1;
    closeHistoryFile();
    lastHistoryFile.value = "";
    terminalOpen.value = false;
    graphStale.value = false;
    overviewGeneration += 1;
    overview.value = null;
    trackingGeneration += 1;
    branchTracking.value = [];
    trackingPath.value = "";
    closeCommitDetail();
    closeDiff();
    closeCommitMenu();
  },
);

watch(
  () => [props.repoId, groups.value, standaloneRepos.value, loaded.value],
  () => {
    void loadRepo();
  },
  { immediate: true },
);

watch(
  () => current.value?.repo.path ?? "",
  (path) => {
    void startWatching(path);
    refreshHasCommitDraft();
  },
  { immediate: true },
);

watch(
  () => files.value.length,
  (count, previous) => {
    if (count === 0 && (previous ?? 0) > 0 && !committing.value) {
      clearCommitDraft();
    }
  },
);

void listen<RepoFilesChanged>("repo-files-changed", (event) => {
  void onRepoFilesChanged(event.payload);
}).then((unlisten) => {
  if (watchClosed) {
    unlisten();
    return;
  }
  stopWatch = unlisten;
});
</script>

<template>
  <div class="repo-pane">
  <div
    v-if="current"
    class="repo-view"
    :class="{ 'files-collapsed': filesCollapsed, resizing }"
    :style="{
      '--files-pane-width': `${filesPaneWidth}px`,
      '--terminal-pane-height': `${terminalPaneHeight}px`,
    }"
  >
    <section v-show="!showingDiff" class="graph-pane">
      <RepoToolbar
        :repo-id="current.repo.id"
        :name="current.status?.name ?? current.repo.path"
        :branch="current.status?.branch ?? ''"
        :path="current.repo.path"
        :branches="branches"
        :branch-tracking="branchTracking"
        :busy="actionBusy"
        :busy-label="actionLabel || (loading ? 'Loading…' : '')"
        :busy-branch="actionBranch"
        :branches-view="branchesView"
        :tag-view="tagView"
        :tag-count="tags.length"
        :stash-view="stashView"
        :stash-count="stashes.length"
        :files-open="!filesCollapsed && !historyOpen"
        :history-open="!filesCollapsed && historyOpen"
        :unstaged-count="unstagedCount"
        :staged-count="stagedCount"
        :conflicted-count="conflictedCount"
        :terminal-open="terminalOpen"
        @fetch="fetchRepo"
        @pull="pullRepo"
        @pull-options="openPullOptions"
        @push="pushRepo"
        @undo-unpushed="undoUnpushedCommits"
        @checkout="checkoutBranch"
        @create="openCreateBranch"
        @merge="openMergeBranch()"
        @branches="toggleBranchesView"
        @tags="toggleTagView"
        @stash="toggleStashView"
        @files="toggleChangesPane"
        @history="toggleHistoryPane"
        @refresh-branches="refreshBranches"
        @terminal="toggleTerminal"
      />
      <div v-if="conflictActive" class="conflict-banner">
        <div class="conflict-banner-copy">
          <strong>{{ conflictHeading }}</strong>
          <p class="tiny">{{ conflictCopy }}</p>
        </div>
        <div class="conflict-banner-actions">
          <button
            class="ghost tiny danger"
            type="button"
            :disabled="actionBusy || !operation"
            @click="abortCurrentOperation"
          >
            {{ abortLabel(operation) }}
          </button>
          <button
            class="ghost tiny commit"
            type="button"
            :disabled="actionBusy || !operation || conflictedCount > 0"
            @click="continueCurrentOperation"
          >
            <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M4.5 12.75l6 6 9-13.5" />
            </svg>
            {{ continueLabel(operation) }}
          </button>
        </div>
      </div>
      <p v-if="message" class="banner">{{ message }}</p>
      <div class="graph-body" :class="{ 'with-terminal': terminalOpen }">
        <BranchList
          v-if="branchesView"
          :overview="overview"
          :busy="actionBusy"
          @checkout="checkoutListedBranch"
          @merge="openMergeBranch"
          @rename="openRenameBranch"
          @delete="deleteBranch"
          @delete-merged="deleteMerged"
          @delete-selected="deleteSelectedBranches"
        />
        <TagList
          v-else-if="tagView"
          :tags="tags"
          :busy="actionBusy"
          @create="openCreateTag"
          @delete="deleteTag"
        />
        <StashList
          v-else-if="stashView"
          :stashes="stashes"
          :busy="actionBusy"
          :can-stash="files.length > 0"
          @apply="applyStash"
          @pop="popStash"
          @drop="dropStash"
          @push="openStash"
        />
        <div v-else class="graph-scroll">
          <CommitGraph
            :commits="commits"
            :selected-hash="selectedCommit?.hash ?? ''"
            @select="selectCommit"
            @menu="openCommitMenu"
          />
        </div>
        <TerminalPane
          v-if="terminalOpen"
          :cwd="current.repo.path"
          @close="terminalOpen = false"
        />
      </div>
    </section>
    <section v-if="showingDiff" class="diff-main">
      <div class="pane-header">
        <div class="diff-heading">
          <button class="ghost tiny" type="button" @click="closeDiff">← Back</button>
          <PathLabel
            class="diff-path"
            :path="selectedFile?.path ?? selectedCommitFile?.path ?? selectedHistoryCommit?.path ?? selectedHistoryFile"
          />
          <span class="muted tiny">{{
            selectedFile
              ? isConflicted(selectedFile)
                ? "Conflicted"
                : selectedFile.staged
                  ? "Staged"
                  : "Unstaged"
              : selectedCommitFile
                ? `${selectedCommitFile.status} · ${selectedCommit?.hash.slice(0, 7)}`
                : selectedHistoryCommit
                  ? selectedHistoryCommit.status
                    ? `${selectedHistoryCommit.status} · ${selectedHistoryCommit.hash.slice(0, 7)}`
                    : `${selectedHistoryCommit.hash.slice(0, 7)}`
                  : ""
          }}</span>
          <template v-if="selectedFile && isConflicted(selectedFile)">
            <button class="ghost tiny" type="button" @click="openInEditor(selectedFile)">
              {{ openEditorLabel }}
            </button>
            <button class="ghost tiny stage" type="button" @click="stageFile(selectedFile)">
              Mark resolved
            </button>
          </template>
          <button
            v-if="operation"
            class="ghost tiny danger"
            type="button"
            :disabled="actionBusy"
            @click="abortCurrentOperation"
          >
            {{ abortLabel(operation) }}
          </button>
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
          <FileHistoryToggle
            :open="!filesCollapsed && historyOpen"
            @click="toggleHistoryPane"
          />
          <ChangesToggle
            :open="!filesCollapsed && !historyOpen"
            :unstaged="unstagedCount"
            :staged="stagedCount"
            :conflicted="conflictedCount"
            @click="toggleChangesPane"
          />
        </div>
      </div>
      <div class="diff-scroll">
        <DiffViewer
          :raw="diff"
          :mode="diffMode"
          :repo-path="current.repo.path"
          :file="blameFile"
          :rev="blameRev"
          :staged="blameStaged"
          :old-path="blameOldPath"
        />
      </div>
    </section>
    <aside v-if="!filesCollapsed" class="changes-pane">
      <button
        class="pane-resize"
        type="button"
        aria-label="Resize files panel"
        @pointerdown="startResize"
      />
      <div v-if="historyOpen" class="file-history-host">
        <FileTree
          class="file-history-tree"
          :class="{ parked: Boolean(selectedHistoryFile) }"
          :files="repoFiles"
          :loading="repoFilesLoading"
          :error="repoFilesError"
          :parked="Boolean(selectedHistoryFile)"
          :selected-path="lastHistoryFile"
          @select="selectHistoryFile"
        />
        <FileHistoryList
          v-if="selectedHistoryFile"
          :file="selectedHistoryFile"
          :commits="historyCommits"
          :selected-hash="selectedHistoryCommit?.hash ?? ''"
          :loading="historyCommitsLoading"
          :error="historyCommitsError"
          @select="selectHistoryCommit"
          @close="closeHistoryFile"
        />
      </div>
      <CommitFiles
        v-else-if="selectedCommit"
        :commit="selectedCommit"
        :files="commitFiles"
        :selected-path="selectedCommitFile?.path ?? ''"
        :loading="commitFilesLoading"
        @select="selectCommitFile"
        @close="closeCommitDetail"
      />
      <WorkingTree
        v-else
        :files="files"
        :selected-path="selectedFile?.path ?? ''"
        :selected-staged="selectedFile?.staged ?? false"
        :operation="operation"
        :has-draft="hasCommitDraft"
        @select="selectFile"
        @stage="stageFile"
        @unstage="unstageFile"
        @stage-all="stageAll"
        @unstage-all="unstageAll"
        @discard="discardAll"
        @stash="openStash"
        @stash-file="stashFile"
        @ignore="ignoreFile"
        @reveal="revealFile"
        @copy-path="copyFilePath"
        @delete-file="deleteFile"
        @commit="openCommit"
        @open-editor="openInEditor"
      />
    </aside>
  </div>
  <div v-else class="empty-home">
    <p class="muted">{{ loaded ? "Repository not found." : "Loading…" }}</p>
  </div>
  <Modal v-if="pullingOptions" title="Pull from remote" @close="closePullOptions">
    <p class="pull-summary">
      Merges <code>origin/{{ pullRemoteLabel }}</code> into the currently checked-out branch.
      Checkout does not change.
    </p>
    <fieldset class="radio-list">
      <legend class="muted tiny">Remote branch</legend>
      <label class="radio-option">
        <input v-model="pullSource" type="radio" value="current" />
        origin/current-branch
      </label>
      <label class="radio-option">
        <input v-model="pullSource" type="radio" value="develop" />
        origin/develop
      </label>
      <label class="radio-option">
        <input v-model="pullSource" type="radio" value="master" />
        origin/master
      </label>
      <label class="radio-option">
        <input v-model="pullSource" type="radio" value="main" />
        origin/main
      </label>
      <label class="radio-option">
        <input v-model="pullSource" type="radio" value="specify" />
        Specify
      </label>
      <input
        v-if="pullSource === 'specify'"
        v-model="specifyBranch"
        type="text"
        placeholder="branch name"
        autofocus
        @keydown.enter="confirmPull"
      />
    </fieldset>
    <p class="muted tiny pull-hint">{{ pullHint }}</p>
    <template #actions>
      <button class="ghost" type="button" @click="closePullOptions">Cancel</button>
      <button class="primary" type="button" :disabled="!canConfirmPull" @click="confirmPull">
        Pull
      </button>
    </template>
  </Modal>
  <Modal v-if="committing" :title="amending ? 'Amend last commit' : 'Commit'" medium @close="closeCommit">
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
    <label v-if="lastCommit" class="radio-option">
      <input type="checkbox" :checked="amending" @change="onAmendChange" />
      Amend last commit
    </label>
    <p v-if="amending && lastCommit?.published" class="muted tiny">
      This commit is already on the remote. Amending rewrites it, and you will need to force-push.
    </p>
    <p v-if="!stagedCount" class="muted tiny">Stage a file to commit.</p>
    <template #actions>
      <button class="ghost" type="button" @click="closeCommit">Close</button>
      <button class="ghost commit" type="button" :disabled="!canCommit" @click="commitChanges">
        {{ amending ? "Amend" : "Commit" }}
      </button>
    </template>
  </Modal>
  <Modal v-if="stashing" title="Stash changes" @close="closeStash">
    <label class="modal-label">
      <span class="muted tiny">Message</span>
      <input
        ref="stashMessageInput"
        v-model="stashMessage"
        type="text"
        placeholder="Optional summary of this work"
        @keydown.enter.prevent="stashChanges"
      />
    </label>
    <p class="muted tiny">Saves staged, unstaged, and untracked files, then clears the working tree.</p>
    <template #actions>
      <button class="ghost" type="button" @click="closeStash">Cancel</button>
      <button class="primary" type="button" :disabled="!files.length" @click="stashChanges">
        Stash
      </button>
    </template>
  </Modal>
  <Modal v-if="creatingTag" title="New tag" @close="closeCreateTag">
    <label class="modal-label">
      <span class="muted tiny">Tag name</span>
      <input
        ref="newTagInput"
        v-model="newTagName"
        type="text"
        placeholder="v1.0.0"
        @keydown.enter.prevent="createTag"
      />
    </label>
    <label class="modal-label">
      <span class="muted tiny">Message</span>
      <input
        v-model="newTagMessage"
        type="text"
        placeholder="Optional. Makes an annotated tag"
      />
    </label>
    <label class="modal-label">
      <span class="muted tiny">Commit</span>
      <input
        v-model="newTagTarget"
        type="text"
        placeholder="Current commit (HEAD)"
        @keydown.enter.prevent="createTag"
      />
    </label>
    <p class="muted tiny">Leave commit blank to tag HEAD. A message makes an annotated tag.</p>
    <template #actions>
      <button class="ghost" type="button" @click="closeCreateTag">Cancel</button>
      <button class="primary" type="button" :disabled="!canCreateTag" @click="createTag">
        Create tag
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
    <label v-if="!newBranchStart" class="modal-label">
      <span class="muted tiny">Base branch</span>
      <select v-model="baseBranch" :disabled="!baseBranchOptions.length">
        <option v-if="!baseBranchOptions.length" value="" disabled>No local branches</option>
        <option v-for="item in baseBranchOptions" :key="item" :value="item">
          {{ baseBranchLabel(item) }}
        </option>
      </select>
    </label>
    <p v-if="newBranchStartShort" class="muted tiny">Starts at {{ newBranchStartShort }}.</p>
    <p v-else class="muted tiny">The new branch starts at the tip of the base branch, then checks it out.</p>
    <template #actions>
      <button class="ghost" type="button" @click="closeCreateBranch">Cancel</button>
      <button class="primary" type="button" :disabled="!canCreateBranch" @click="createBranch">
        Create and switch
      </button>
    </template>
  </Modal>
  <Modal v-if="renamingBranch" title="Rename branch" @close="closeRenameBranch">
    <label class="modal-label">
      <span class="muted tiny">Branch name</span>
      <input
        ref="renameBranchInput"
        v-model="renameBranchName"
        type="text"
        placeholder="feature/JIRA-123"
        @keydown.enter="renameBranch"
      />
    </label>
    <template #actions>
      <button class="ghost" type="button" @click="closeRenameBranch">Cancel</button>
      <button class="primary" type="button" :disabled="!canRenameBranch" @click="renameBranch">
        Rename
      </button>
    </template>
  </Modal>
  <Modal v-if="mergingBranch" title="Merge local branch" @close="closeMergeBranch">
    <p class="pull-summary">
      Merges <code>{{ mergeSource || "…" }}</code> into <code>{{ mergeTarget || "…" }}</code>.
      Checkout switches to the target first if needed.
    </p>
    <label class="modal-label">
      <span class="muted tiny">From</span>
      <select v-model="mergeSource">
        <option v-for="name in localBranchNames" :key="`from-${name}`" :value="name">
          {{ name }}
        </option>
      </select>
    </label>
    <label class="modal-label">
      <span class="muted tiny">Into</span>
      <select v-model="mergeTarget">
        <option v-for="name in localBranchNames" :key="`into-${name}`" :value="name">
          {{ name }}
        </option>
      </select>
    </label>
    <p class="muted tiny pull-hint">
      Conflicts appear in the files list so you can open them, mark them resolved, or abort.
    </p>
    <template #actions>
      <button class="ghost" type="button" @click="closeMergeBranch">Cancel</button>
      <button class="primary" type="button" :disabled="!canConfirmMerge" @click="mergeLocalBranch">
        Merge
      </button>
    </template>
  </Modal>
  <CommitContextMenu
    v-if="commitMenu"
    :commit="commitMenu.commit"
    :hashes="commitMenu.hashes"
    :x="commitMenu.x"
    :y="commitMenu.y"
    :busy="actionBusy"
    :operation="operation"
    :has-merge="commitMenuHasMerge"
    @checkout="checkoutMenuCommit"
    @create-branch="createBranchFromMenu"
    @cherry-pick="cherryPickMenuCommits"
    @revert="revertMenuCommits"
    @copy-sha="copyMenuShas"
    @copy-link="copyMenuLink"
    @create-tag="createTagFromMenu"
    @close="closeCommitMenu"
  />
  </div>
</template>
