<script setup lang="ts">
import { computed, nextTick, onUnmounted, ref, watch } from "vue";
import { confirm } from "@tauri-apps/plugin-dialog";
import BranchList from "./BranchList.vue";
import ChangesToggle from "./ChangesToggle.vue";
import CommitFiles from "./CommitFiles.vue";
import CommitGraph from "./CommitGraph.vue";
import DiffViewer from "./DiffViewer.vue";
import Modal from "./Modal.vue";
import PathLabel from "./PathLabel.vue";
import RepoToolbar from "./RepoToolbar.vue";
import StashList from "./StashList.vue";
import TerminalPane from "./TerminalPane.vue";
import WorkingTree from "./WorkingTree.vue";
import { useApp } from "../composables/useApp";
import * as api from "../api";
import type {
  BranchOverview,
  CommitFile,
  CommitNode,
  LocalBranch,
  StashEntry,
  WorkingTreeFile,
} from "../types";
import { STANDALONE_GROUP_ID } from "../types";
import {
  abortLabel,
  continueLabel,
  isConflicted,
  openInEditorLabel,
  operationNoun,
  operationTitle,
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
  refreshStatus,
  refreshRepoStatus,
  patchRepoStatus,
  showToast,
} = useApp();

const commits = ref<CommitNode[]>([]);
const files = ref<WorkingTreeFile[]>([]);
const branches = ref<string[]>([]);
const branchesView = ref(false);
const graphStale = ref(false);
const stashes = ref<StashEntry[]>([]);
const stashView = ref(false);
const terminalOpen = ref(false);
const overview = ref<BranchOverview | null>(null);
let overviewGeneration = 0;
const selectedFile = ref<WorkingTreeFile | null>(null);
const selectedCommit = ref<CommitNode | null>(null);
const selectedCommitFile = ref<CommitFile | null>(null);
const commitFiles = ref<CommitFile[]>([]);
const commitFilesLoading = ref(false);
const filesCollapsed = ref(false);
const diff = ref("");
const showingDiff = computed(() => Boolean(selectedFile.value || selectedCommitFile.value));
const loading = ref(false);
const actionBusy = ref(false);
const actionLabel = ref("");
const message = ref("");
const creatingBranch = ref(false);
const newBranchName = ref("");
const newBranchInput = ref<HTMLInputElement | null>(null);
const renamingBranch = ref<LocalBranch | null>(null);
const renameBranchName = ref("");
const renameBranchInput = ref<HTMLInputElement | null>(null);
const committing = ref(false);
const commitTitle = ref("");
const commitDescription = ref("");
const commitTitleInput = ref<HTMLInputElement | null>(null);
const stashing = ref(false);
const stashMessage = ref("");
const stashMessageInput = ref<HTMLInputElement | null>(null);
const pullingOptions = ref(false);
const pullSource = ref<"current" | "develop" | "master" | "main" | "specify">("current");
const specifyBranch = ref("");

const COMMIT_TITLE_MAX = 72;

const canCreateBranch = computed(() => Boolean(newBranchName.value.trim()));
const canRenameBranch = computed(() => {
  const next = renameBranchName.value.trim();
  return Boolean(next) && next !== (renamingBranch.value?.name ?? "");
});
const commitTitleLength = computed(() => [...commitTitle.value].length);
const commitTitleLeft = computed(() => Math.max(0, COMMIT_TITLE_MAX - commitTitleLength.value));
const canCommit = computed(
  () => Boolean(commitTitle.value.trim()) && commitTitleLength.value <= COMMIT_TITLE_MAX,
);
const conflictedFiles = computed(() => files.value.filter(isConflicted));
const unstagedCount = computed(
  () => files.value.filter((file) => !file.staged && !isConflicted(file)).length,
);
const stagedCount = computed(
  () => files.value.filter((file) => file.staged && !isConflicted(file)).length,
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
  stopResize();
});

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

async function loadRepo(options?: { overview?: boolean; graph?: boolean; silent?: boolean }) {
  const match = current.value;
  if (!match) {
    overviewGeneration += 1;
    commits.value = [];
    files.value = [];
    branches.value = [];
    stashes.value = [];
    overview.value = null;
    selectedFile.value = null;
    closeCommitDetail();
    message.value = loaded.value ? "Repository not found." : "";
    return;
  }

  const wantOverview = options?.overview ?? branchesView.value;
  const wantGraph = options?.graph ?? true;
  const generation = wantOverview ? ++overviewGeneration : overviewGeneration;
  if (!options?.silent) {
    loading.value = true;
  }
  message.value = "";
  try {
    const [nextCommits, nextFiles, nextBranches, nextStashes, nextOverview] = await Promise.all([
      wantGraph ? api.logGraph(match.repo.path) : Promise.resolve(commits.value),
      api.workingTree(match.repo.path),
      api.listLocalBranches(match.repo.path).catch(() => [] as string[]),
      api.stashList(match.repo.path).catch(() => [] as StashEntry[]),
      wantOverview
        ? api.branchOverview(match.repo.path, preferredMergeTarget(), false).catch(() => null)
        : Promise.resolve(overview.value),
    ]);
    if (wantGraph) {
      commits.value = nextCommits;
      graphStale.value = false;
    }
    files.value = nextFiles;
    if (nextFiles.some(isConflicted)) {
      filesCollapsed.value = false;
      void refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
    }
    branches.value = nextBranches;
    stashes.value = nextStashes;
    if (wantOverview) {
      const needsClassify = nextOverview?.branches.some((branch) => branch.pending) ?? false;
      overview.value = nextOverview ? mergeOverview(overview.value, nextOverview) : nextOverview;
      if (needsClassify && generation === overviewGeneration) {
        void classifyOverview(match.repo.path, preferredMergeTarget(), generation);
      }
    }
    if (selectedFile.value && !nextFiles.some((file) => sameFile(file, selectedFile.value))) {
      selectedFile.value = null;
      if (!selectedCommitFile.value) {
        diff.value = "";
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
  selectedCommitFile.value = null;
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
  filesCollapsed.value = false;
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
    await refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
  } catch (err) {
    const text = String(err);
    message.value = text;
    showToast(text, "error");
    await loadRepo({ silent: true });
    await refreshRepoStatus(match.group?.id ?? STANDALONE_GROUP_ID, match.repo.id);
    if (conflictActive.value) {
      filesCollapsed.value = false;
      message.value = "";
      const first = conflictedFiles.value[0];
      if (first) {
        await selectFile(first);
      }
    }
  } finally {
    actionBusy.value = false;
    actionLabel.value = "";
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

function runPull(branch?: string) {
  const match = current.value;
  if (!match) {
    return;
  }
  return runRepoAction("Pulling…", async () => {
    const result = await api.pullRepo(
      match.group?.id ?? STANDALONE_GROUP_ID,
      match.repo.id,
      branch,
    );
    if (!result.ok) {
      throw result.message;
    }
    return result.message;
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
  return runRepoAction("Pushing…", () => api.repoPush(match.repo.path));
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
    return;
  }
  if (graphStale.value) {
    void loadRepo({ overview: false, silent: true });
  }
}

function toggleTerminal() {
  terminalOpen.value = !terminalOpen.value;
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
    await api.discardAllChanges(match.repo.path);
    closeDiff();
    await loadRepo();
    await refreshStatus(match.group?.id ?? STANDALONE_GROUP_ID);
  } catch (err) {
    message.value = String(err);
  }
}

watch(
  () => props.repoId,
  () => {
    branchesView.value = false;
    stashView.value = false;
    terminalOpen.value = false;
    graphStale.value = false;
    overviewGeneration += 1;
    overview.value = null;
    closeCommitDetail();
    closeDiff();
  },
);

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
        :busy="actionBusy"
        :busy-label="actionLabel || (loading ? 'Loading…' : '')"
        :branches-view="branchesView"
        :stash-view="stashView"
        :stash-count="stashes.length"
        :files-open="!filesCollapsed"
        :unstaged-count="unstagedCount"
        :staged-count="stagedCount"
        :conflicted-count="conflictedCount"
        :terminal-open="terminalOpen"
        @pull="pullRepo"
        @pull-options="openPullOptions"
        @push="pushRepo"
        @checkout="checkoutBranch"
        @create="openCreateBranch"
        @branches="toggleBranchesView"
        @stash="toggleStashView"
        @files="filesCollapsed = !filesCollapsed"
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
          @rename="openRenameBranch"
          @delete="deleteBranch"
          @delete-merged="deleteMerged"
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
            :path="selectedFile?.path ?? selectedCommitFile?.path ?? ''"
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
          <ChangesToggle
            :open="!filesCollapsed"
            :unstaged="unstagedCount"
            :staged="stagedCount"
            :conflicted="conflictedCount"
            @click="filesCollapsed = !filesCollapsed"
          />
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
      <CommitFiles
        v-if="selectedCommit"
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
        @select="selectFile"
        @stage="stageFile"
        @unstage="unstageFile"
        @stage-all="stageAll"
        @unstage-all="unstageAll"
        @discard="discardAll"
        @stash="openStash"
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
  </div>
</template>
