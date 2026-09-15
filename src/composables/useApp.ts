import { computed, nextTick, ref, watch } from "vue";
import * as api from "../api";
import type {
  AppData,
  DiffMode,
  RepoActionResult,
  RepoEntry,
  RepoGroup,
  RepoStatus,
  WindowState,
} from "../types";
import { STANDALONE_GROUP_ID } from "../types";

const groups = ref<RepoGroup[]>([]);
const standaloneRepos = ref<RepoEntry[]>([]);
const statuses = ref<Record<string, RepoStatus>>({});
const results = ref<Record<string, RepoActionResult[]>>({});
const busy = ref<Record<string, string>>({});
const error = ref("");
const loaded = ref(false);
const refreshIntervalSeconds = ref(300);
const filesPaneWidth = ref(320);
const diffMode = ref<DiffMode>("split");
const windowState = ref<WindowState | null>(null);
const FILES_PANE_MIN = 220;
const FILES_PANE_MAX = 800;
const refreshingAll = ref(false);
const lastRefreshAt = ref<Date | null>(null);
const nextRefreshAt = ref<number | null>(null);
const nowTick = ref(Date.now());
const refreshingRepos = ref<Record<string, boolean>>({});
const refreshingGroups = ref<Record<string, boolean>>({});
const refreshTotal = ref(0);
const refreshDone = ref(0);
const refreshCancelled = ref(false);
const refreshProgress = ref<Record<string, string>>({});
const toastMessage = ref("");
const toastKind = ref<"success" | "error">("success");
const actionOutput = ref<{ title: string; results: RepoActionResult[] } | null>(null);
const actionOutputOpen = ref(false);
const pullProgress = ref<Record<string, string>>({});
const pullBranchByGroup = ref<Record<string, string>>({});
const pullCancelled = ref<Record<string, boolean>>({});
const checkoutProgress = ref<Record<string, string>>({});
const checkoutCancelled = ref<Record<string, boolean>>({});
let toastTimer: ReturnType<typeof setTimeout> | null = null;
let autoRefreshTimer: ReturnType<typeof setTimeout> | null = null;
let tickTimer: ReturnType<typeof setInterval> | null = null;
let autoRefreshStarted = false;

const statusById = computed(() => statuses.value);

export function useApp() {
  async function load() {
    error.value = "";
    try {
      applyState(await api.getState());
      loaded.value = true;
      startAutoRefresh();
      await Promise.all([
        ...groups.value.filter((group) => group.expanded).map((group) => refreshStatus(group.id)),
        refreshStatus(STANDALONE_GROUP_ID),
      ]);
    } catch (err) {
      error.value = String(err);
    }
  }

  function applyState(data: AppData) {
    groups.value = data.groups;
    standaloneRepos.value = data.repos ?? [];
    refreshIntervalSeconds.value = data.refreshIntervalSeconds ?? 300;
    filesPaneWidth.value = clampFilesPaneWidth(data.filesPaneWidth ?? 320);
    diffMode.value = data.diffMode === "inline" ? "inline" : "split";
    windowState.value = data.window ?? null;
  }

  function applyStatus(status: RepoStatus) {
    statuses.value = { ...statuses.value, [status.id]: status };
    lastRefreshAt.value = new Date();
  }

  async function refreshStatus(groupId: string, fetch = false) {
    try {
      const list =
        groupId === STANDALONE_GROUP_ID
          ? await api.standaloneStatus(fetch)
          : await api.groupStatus(groupId, fetch);
      const next = { ...statuses.value };
      for (const status of list) {
        next[status.id] = status;
      }
      statuses.value = next;
      lastRefreshAt.value = new Date();
    } catch (err) {
      error.value = String(err);
    }
  }

  function cancelRefresh() {
    refreshCancelled.value = true;
  }

  function cancelPull(groupId: string) {
    pullCancelled.value = { ...pullCancelled.value, [groupId]: true };
  }

  function cancelCheckout(groupId: string) {
    checkoutCancelled.value = { ...checkoutCancelled.value, [groupId]: true };
  }

  function dismissToast() {
    if (toastTimer) {
      clearTimeout(toastTimer);
      toastTimer = null;
    }
    toastMessage.value = "";
  }

  function showToast(message: string, kind: "success" | "error" = "success") {
    dismissToast();
    toastKind.value = kind;
    toastMessage.value = message;
    toastTimer = setTimeout(
      () => {
        toastMessage.value = "";
        toastTimer = null;
      },
      kind === "error" ? 5600 : 3200,
    );
  }

  function dismissOutput() {
    actionOutputOpen.value = false;
  }

  function openOutput() {
    if (actionOutput.value) {
      actionOutputOpen.value = true;
    }
  }

  function presentActionResults(
    title: string,
    outcomes: RepoActionResult[],
    messages: { success: string; error: string },
  ) {
    actionOutput.value = { title, results: outcomes };
    const failed = outcomes.filter((item) => !item.ok).length;
    if (failed) {
      actionOutputOpen.value = true;
      showToast(messages.error, "error");
      return;
    }
    showToast(messages.success);
  }

  function refreshDoneMessage(count: number, groupName?: string) {
    const repos = count === 1 ? "1 repository" : `${count} repositories`;
    return groupName
      ? `Refreshed ${groupName} (${repos}).`
      : `Refresh complete. Updated ${repos}.`;
  }

  type RefreshJob = { groupId: string; repoId: string };

  // Fetch is network-bound; a small pool feels like GitKraken without
  // launching every `git fetch` at once.
  const REFRESH_CONCURRENCY = 8;

  async function runPool<T>(items: T[], worker: (item: T) => Promise<void>) {
    if (!items.length) {
      return;
    }
    let next = 0;
    const workers = Math.min(REFRESH_CONCURRENCY, items.length);
    await Promise.all(
      Array.from({ length: workers }, async () => {
        while (!refreshCancelled.value) {
          const index = next++;
          if (index >= items.length) {
            return;
          }
          await worker(items[index]);
        }
      }),
    );
  }

  function interleaveRefreshJobs(jobs: RefreshJob[]) {
    const buckets = new Map<string, RefreshJob[]>();
    for (const job of jobs) {
      const list = buckets.get(job.groupId) ?? [];
      list.push(job);
      buckets.set(job.groupId, list);
    }
    const queues = [...buckets.values()];
    const ordered: RefreshJob[] = [];
    let added = true;
    while (added) {
      added = false;
      for (const queue of queues) {
        const job = queue.shift();
        if (job) {
          ordered.push(job);
          added = true;
        }
      }
    }
    return ordered;
  }

  function markReposRefreshing(repoIds: string[]) {
    refreshingRepos.value = {
      ...refreshingRepos.value,
      ...Object.fromEntries(repoIds.map((id) => [id, true])),
    };
  }

  function unmarkRepoRefreshing(repoId: string) {
    if (!refreshingRepos.value[repoId]) {
      return;
    }
    const next = { ...refreshingRepos.value };
    delete next[repoId];
    refreshingRepos.value = next;
  }

  function clearReposRefreshing(repoIds: string[]) {
    if (!repoIds.length) {
      return;
    }
    const next = { ...refreshingRepos.value };
    for (const id of repoIds) {
      delete next[id];
    }
    refreshingRepos.value = next;
  }

  async function refreshRepoJobs(jobs: RefreshJob[], groupTotals: Record<string, number>) {
    const completed: Record<string, number> = {};
    const progress = { ...refreshProgress.value };
    for (const [groupId, total] of Object.entries(groupTotals)) {
      completed[groupId] = 0;
      progress[groupId] = `0/${total}`;
    }
    refreshProgress.value = progress;
    markReposRefreshing(jobs.map((job) => job.repoId));
    await nextTick();

    const bumpProgress = (groupId: string) => {
      completed[groupId] = (completed[groupId] ?? 0) + 1;
      const total = groupTotals[groupId];
      if (total) {
        refreshProgress.value = {
          ...refreshProgress.value,
          [groupId]: `${completed[groupId]}/${total}`,
        };
      }
      if (refreshTotal.value > 0) {
        refreshDone.value += 1;
      }
    };

    try {
      await runPool(jobs, async (job) => {
        if (refreshCancelled.value) {
          unmarkRepoRefreshing(job.repoId);
          return;
        }
        try {
          applyStatus(await api.refreshRepo(job.groupId, job.repoId, true));
        } catch (err) {
          error.value = String(err);
        } finally {
          unmarkRepoRefreshing(job.repoId);
          bumpProgress(job.groupId);
        }
      });
    } finally {
      clearReposRefreshing(jobs.map((job) => job.repoId));
    }
  }

  async function refreshGroup(groupId: string) {
    const group = groups.value.find((item) => item.id === groupId);
    if (!group) {
      return;
    }
    refreshingGroups.value = { ...refreshingGroups.value, [groupId]: true };
    busy.value = { ...busy.value, [groupId]: "Refreshing…" };
    error.value = "";
    const notify = !refreshingAll.value;
    if (!refreshingAll.value) {
      refreshCancelled.value = false;
      refreshDone.value = 0;
      refreshTotal.value = group.repos.length;
    }
    try {
      await refreshRepoJobs(
        group.repos.map((repo) => ({ groupId, repoId: repo.id })),
        { [groupId]: group.repos.length },
      );
      if (notify && !refreshCancelled.value) {
        showToast(refreshDoneMessage(group.repos.length, group.name));
      }
    } finally {
      const groupsBusy = { ...refreshingGroups.value };
      delete groupsBusy[groupId];
      refreshingGroups.value = groupsBusy;
      const next = { ...busy.value };
      delete next[groupId];
      busy.value = next;
      const progress = { ...refreshProgress.value };
      delete progress[groupId];
      refreshProgress.value = progress;
      if (!refreshingAll.value) {
        refreshCancelled.value = false;
        refreshDone.value = 0;
        refreshTotal.value = 0;
      }
    }
  }

  async function refreshAll(options?: { notify?: boolean }) {
    const standaloneCount = standaloneRepos.value.length;
    const groupedCount = groups.value.reduce((sum, group) => sum + group.repos.length, 0);
    if (refreshingAll.value || !(standaloneCount || groupedCount)) {
      return;
    }
    const notify = options?.notify ?? true;
    refreshingAll.value = true;
    refreshCancelled.value = false;
    error.value = "";
    refreshDone.value = 0;
    refreshTotal.value = standaloneCount + groupedCount;
    const groupTotals: Record<string, number> = {};
    if (standaloneCount) {
      groupTotals[STANDALONE_GROUP_ID] = standaloneCount;
    }
    const groupsBusy = { ...refreshingGroups.value };
    const progress = { ...refreshProgress.value };
    for (const group of groups.value) {
      if (!group.repos.length) {
        continue;
      }
      groupTotals[group.id] = group.repos.length;
      groupsBusy[group.id] = true;
      progress[group.id] = `0/${group.repos.length}`;
    }
    refreshingGroups.value = groupsBusy;
    refreshProgress.value = progress;
    if (autoRefreshTimer) {
      clearTimeout(autoRefreshTimer);
      autoRefreshTimer = null;
    }
    try {
      const jobs = interleaveRefreshJobs([
        ...standaloneRepos.value.map((repo) => ({
          groupId: STANDALONE_GROUP_ID,
          repoId: repo.id,
        })),
        ...groups.value.flatMap((group) =>
          group.repos.map((repo) => ({ groupId: group.id, repoId: repo.id })),
        ),
      ]);
      await refreshRepoJobs(jobs, groupTotals);
      if (notify && !refreshCancelled.value) {
        showToast(refreshDoneMessage(refreshTotal.value));
      }
    } finally {
      refreshingAll.value = false;
      refreshCancelled.value = false;
      refreshDone.value = 0;
      refreshTotal.value = 0;
      const leftoverGroups = { ...refreshingGroups.value };
      const leftoverProgress = { ...refreshProgress.value };
      for (const groupId of Object.keys(groupTotals)) {
        delete leftoverGroups[groupId];
        delete leftoverProgress[groupId];
      }
      refreshingGroups.value = leftoverGroups;
      refreshProgress.value = leftoverProgress;
      startAutoRefresh();
    }
  }

  function stopAutoRefresh() {
    if (autoRefreshTimer) {
      clearTimeout(autoRefreshTimer);
      autoRefreshTimer = null;
    }
    nextRefreshAt.value = null;
  }

  function startAutoRefresh() {
    stopAutoRefresh();
    const seconds = refreshIntervalSeconds.value;
    if (seconds <= 0) {
      return;
    }
    nextRefreshAt.value = Date.now() + seconds * 1000;
    autoRefreshTimer = setTimeout(() => {
      void refreshAll({ notify: false });
    }, seconds * 1000);
  }

  function clampFilesPaneWidth(width: number) {
    return Math.round(Math.min(FILES_PANE_MAX, Math.max(FILES_PANE_MIN, width)));
  }

  function setFilesPaneWidth(width: number) {
    filesPaneWidth.value = clampFilesPaneWidth(width);
  }

  async function saveFilesPaneWidth(width: number) {
    filesPaneWidth.value = await api.updateFilesPaneWidth(clampFilesPaneWidth(width));
  }

  async function saveDiffMode(mode: DiffMode) {
    diffMode.value = await api.updateDiffMode(mode);
  }

  async function replaceSettings(data: AppData) {
    const next = await api.replaceAppData(data);
    applyState(next);
    startAutoRefresh();
    return next;
  }

  async function saveRefreshInterval(seconds: number) {
    refreshIntervalSeconds.value = await api.updateAppSettings(seconds);
    startAutoRefresh();
  }

  async function saveWindowState(next: WindowState) {
    windowState.value = await api.updateWindowState(next);
    return windowState.value;
  }

  function isRepoRefreshing(repoId: string) {
    return Boolean(refreshingRepos.value[repoId]);
  }

  function isGroupRefreshing(groupId: string) {
    return Boolean(refreshingGroups.value[groupId]);
  }

  const refreshProgressLabel = computed(() => {
    if (!refreshingAll.value && !Object.keys(refreshingGroups.value).length) {
      return "";
    }
    if (!refreshTotal.value) {
      return "Refreshing…";
    }
    return `Refreshing ${refreshDone.value}/${refreshTotal.value}`;
  });

  const countdownLabel = computed(() => {
    if (!nextRefreshAt.value || refreshIntervalSeconds.value <= 0) {
      return "";
    }
    const remaining = Math.max(0, Math.ceil((nextRefreshAt.value - nowTick.value) / 1000));
    const minutes = Math.floor(remaining / 60);
    const seconds = remaining % 60;
    return `Next refresh in ${minutes}:${String(seconds).padStart(2, "0")}`;
  });

  async function createGroup(name: string) {
    const group = await api.createGroup(name);
    groups.value = [group, ...groups.value];
    return group;
  }

  async function renameGroup(groupId: string, name: string) {
    await api.renameGroup(groupId, name);
    patchGroup(groupId, { name });
  }

  async function deleteGroup(groupId: string) {
    await api.deleteGroup(groupId);
    groups.value = groups.value.filter((group) => group.id !== groupId);
    const next = { ...results.value };
    delete next[groupId];
    results.value = next;
  }

  function groupNeedsStatus(group: RepoGroup) {
    return group.repos.some((repo) => !statuses.value[repo.id]);
  }

  function toggleGroup(groupId: string) {
    const group = groups.value.find((item) => item.id === groupId);
    if (!group) {
      return;
    }
    const expanded = !group.expanded;
    patchGroup(groupId, { expanded });
    void api.toggleGroup(groupId).catch((err) => {
      patchGroup(groupId, { expanded: !expanded });
      error.value = String(err);
    });
    if (expanded && groupNeedsStatus(group)) {
      void refreshStatus(groupId);
    }
  }

  function setAllGroupsExpanded(expanded: boolean) {
    if (!groups.value.length) {
      return;
    }
    const previous = groups.value.map((group) => group.expanded);
    groups.value = groups.value.map((group) =>
      group.expanded === expanded ? group : { ...group, expanded },
    );
    void api.setAllGroupsExpanded(expanded).catch((err) => {
      groups.value = groups.value.map((group, index) => ({
        ...group,
        expanded: previous[index] ?? group.expanded,
      }));
      error.value = String(err);
    });
    if (expanded) {
      for (const group of groups.value) {
        if (groupNeedsStatus(group)) {
          void refreshStatus(group.id);
        }
      }
    }
  }

  async function saveSettings(
    groupId: string,
    pullFromBranch: string,
    checkoutFallbacks: string[],
    headerColor?: string,
  ) {
    await api.updateGroupSettings(groupId, pullFromBranch, checkoutFallbacks, headerColor);
    patchGroup(groupId, { pullFromBranch, checkoutFallbacks, headerColor });
  }

  async function addStandaloneRepo(path: string) {
    const repo = await api.addStandaloneRepo(path);
    standaloneRepos.value = [...standaloneRepos.value, repo];
    applyStatus(await api.refreshRepo(STANDALONE_GROUP_ID, repo.id, false));
    return repo;
  }

  async function removeStandaloneRepo(repoId: string) {
    await api.removeStandaloneRepo(repoId);
    standaloneRepos.value = standaloneRepos.value.filter((repo) => repo.id !== repoId);
    const next = { ...statuses.value };
    delete next[repoId];
    statuses.value = next;
  }

  async function addRepo(groupId: string, path: string) {
    const repo = await api.addRepo(groupId, path);
    const group = groups.value.find((item) => item.id === groupId);
    if (group) {
      patchGroup(groupId, { repos: [...group.repos, repo] });
    }
    await refreshStatus(groupId);
  }

  async function removeRepo(groupId: string, repoId: string) {
    await api.removeRepo(groupId, repoId);
    const group = groups.value.find((item) => item.id === groupId);
    if (group) {
      patchGroup(groupId, { repos: group.repos.filter((repo) => repo.id !== repoId) });
    }
    const next = { ...statuses.value };
    delete next[repoId];
    statuses.value = next;
  }

  function repoDisplayName(repoId: string, path: string) {
    return (
      statuses.value[repoId]?.name ??
      path.split("/").filter(Boolean).pop() ??
      path
    );
  }

  async function pullGroup(groupId: string, branch?: string) {
    const group = groups.value.find((item) => item.id === groupId);
    if (!group || busy.value[groupId] || !group.repos.length) {
      return;
    }
    error.value = "";
    const outcomes: RepoActionResult[] = [];
    pullCancelled.value = { ...pullCancelled.value, [groupId]: false };
    try {
      for (const [index, repo] of group.repos.entries()) {
        if (pullCancelled.value[groupId]) {
          break;
        }
        const name = repoDisplayName(repo.id, repo.path);
        const progress = `${index + 1}/${group.repos.length}`;
        const activeBranch = branch || statuses.value[repo.id]?.branch || "…";
        pullProgress.value = { ...pullProgress.value, [groupId]: progress };
        pullBranchByGroup.value = { ...pullBranchByGroup.value, [groupId]: activeBranch };
        busy.value = {
          ...busy.value,
          [groupId]: `Pulling ${name} (${progress})…`,
        };
        refreshingRepos.value = { ...refreshingRepos.value, [repo.id]: true };
        await nextTick();
        try {
          outcomes.push(await api.pullRepo(groupId, repo.id, branch));
          applyStatus(await api.refreshRepo(groupId, repo.id, false));
        } catch (err) {
          outcomes.push({
            path: repo.path,
            ok: false,
            message: String(err),
          });
        } finally {
          const next = { ...refreshingRepos.value };
          delete next[repo.id];
          refreshingRepos.value = next;
          await nextTick();
        }
      }
      if (outcomes.length && !pullCancelled.value[groupId]) {
        results.value = { ...results.value, [groupId]: outcomes };
        const failed = outcomes.filter((item) => !item.ok).length;
        const repos =
          group.repos.length === 1 ? "1 repository" : `${group.repos.length} repositories`;
        presentActionResults(branch ? `Pull ${branch} — ${group.name}` : `Pull — ${group.name}`, outcomes, {
          success: branch
            ? `Pulled ${branch} into ${group.name} (${repos}).`
            : `Pulled ${group.name} (${repos}).`,
          error:
            failed === 1
              ? `Pull failed for 1 repository in ${group.name}.`
              : `Pull failed for ${failed} repositories in ${group.name}.`,
        });
      }
    } finally {
      const next = { ...busy.value };
      delete next[groupId];
      busy.value = next;
      const progress = { ...pullProgress.value };
      delete progress[groupId];
      pullProgress.value = progress;
      const branches = { ...pullBranchByGroup.value };
      delete branches[groupId];
      pullBranchByGroup.value = branches;
      const cancelled = { ...pullCancelled.value };
      delete cancelled[groupId];
      pullCancelled.value = cancelled;
    }
  }

  async function checkoutGroup(groupId: string, target: string, fallbacks: string[]) {
    const group = groups.value.find((item) => item.id === groupId);
    if (!group || busy.value[groupId] || !group.repos.length) {
      return;
    }
    const branch = target.trim();
    if (!branch) {
      return;
    }
    error.value = "";
    const outcomes: RepoActionResult[] = [];
    checkoutCancelled.value = { ...checkoutCancelled.value, [groupId]: false };
    try {
      for (const [index, repo] of group.repos.entries()) {
        if (checkoutCancelled.value[groupId]) {
          break;
        }
        const name = repoDisplayName(repo.id, repo.path);
        const progress = `${index + 1}/${group.repos.length}`;
        checkoutProgress.value = { ...checkoutProgress.value, [groupId]: progress };
        busy.value = {
          ...busy.value,
          [groupId]: `Checking out ${name} (${progress})…`,
        };
        const alreadyOn = statuses.value[repo.id]?.branch === branch;
        if (alreadyOn) {
          outcomes.push({
            path: repo.path,
            ok: true,
            message: `Already on ${branch}`,
          });
          continue;
        }
        refreshingRepos.value = { ...refreshingRepos.value, [repo.id]: true };
        await nextTick();
        try {
          outcomes.push(await api.checkoutRepo(groupId, repo.id, branch, fallbacks));
          applyStatus(await api.refreshRepo(groupId, repo.id, false));
        } catch (err) {
          outcomes.push({
            path: repo.path,
            ok: false,
            message: String(err),
          });
        } finally {
          const next = { ...refreshingRepos.value };
          delete next[repo.id];
          refreshingRepos.value = next;
          await nextTick();
        }
      }
      if (outcomes.length && !checkoutCancelled.value[groupId]) {
        results.value = { ...results.value, [groupId]: outcomes };
        const failed = outcomes.filter((item) => !item.ok).length;
        const repos =
          group.repos.length === 1 ? "1 repository" : `${group.repos.length} repositories`;
        presentActionResults(`Checkout — ${group.name}`, outcomes, {
          success: `Checked out ${group.name} (${repos}).`,
          error:
            failed === 1
              ? `Checkout failed for 1 repository in ${group.name}.`
              : `Checkout failed for ${failed} repositories in ${group.name}.`,
        });
      }
    } finally {
      const next = { ...busy.value };
      delete next[groupId];
      busy.value = next;
      const progress = { ...checkoutProgress.value };
      delete progress[groupId];
      checkoutProgress.value = progress;
      const cancelled = { ...checkoutCancelled.value };
      delete cancelled[groupId];
      checkoutCancelled.value = cancelled;
    }
  }

  async function runAction(
    groupId: string,
    label: string,
    action: () => Promise<RepoActionResult[]>,
    title?: string,
  ) {
    busy.value = { ...busy.value, [groupId]: label };
    error.value = "";
    try {
      const outcome = await action();
      results.value = { ...results.value, [groupId]: outcome };
      const group = groups.value.find((item) => item.id === groupId);
      const failed = outcome.filter((item) => !item.ok).length;
      const heading = title ?? label.replace(/…$/, "");
      presentActionResults(heading, outcome, {
        success: group ? `Finished ${heading.toLowerCase()} for ${group.name}.` : "Done.",
        error:
          failed === 1
            ? `${heading} failed for 1 repository.`
            : `${heading} failed for ${failed} repositories.`,
      });
      await refreshStatus(groupId);
    } catch (err) {
      error.value = String(err);
    } finally {
      const next = { ...busy.value };
      delete next[groupId];
      busy.value = next;
    }
  }

  function clearResults(groupId: string) {
    const next = { ...results.value };
    delete next[groupId];
    results.value = next;
  }

  function findRepo(repoId: string) {
    const standalone = standaloneRepos.value.find((item) => item.id === repoId);
    if (standalone) {
      return { group: null, repo: standalone, status: statuses.value[standalone.id] };
    }
    for (const group of groups.value) {
      const repo = group.repos.find((item) => item.id === repoId);
      if (repo) {
        return { group, repo, status: statuses.value[repo.id] };
      }
    }
    return null;
  }

  function patchGroup(groupId: string, patch: Partial<RepoGroup>) {
    groups.value = groups.value.map((group) =>
      group.id === groupId ? { ...group, ...patch } : group,
    );
  }

  if (!autoRefreshStarted) {
    autoRefreshStarted = true;
    watch(refreshIntervalSeconds, () => {
      startAutoRefresh();
    });
    if (!tickTimer) {
      tickTimer = setInterval(() => {
        nowTick.value = Date.now();
      }, 250);
    }
  }

  return {
    groups,
    standaloneRepos,
    statuses: statusById,
    results,
    busy,
    error,
    loaded,
    refreshIntervalSeconds,
    filesPaneWidth,
    setFilesPaneWidth,
    saveFilesPaneWidth,
    diffMode,
    saveDiffMode,
    windowState,
    saveWindowState,
    replaceSettings,
    refreshingAll,
    lastRefreshAt,
    countdownLabel,
    refreshProgressLabel,
    refreshCancelled,
    refreshProgress,
    toastMessage,
    toastKind,
    actionOutput,
    actionOutputOpen,
    showToast,
    dismissToast,
    dismissOutput,
    openOutput,
    cancelRefresh,
    cancelPull,
    cancelCheckout,
    load,
    refreshStatus,
    refreshGroup,
    refreshAll,
    pullGroup,
    pullProgress,
    pullBranchByGroup,
    pullCancelled,
    checkoutGroup,
    checkoutProgress,
    checkoutCancelled,
    isRepoRefreshing,
    isGroupRefreshing,
    saveRefreshInterval,
    createGroup,
    renameGroup,
    deleteGroup,
    toggleGroup,
    setAllGroupsExpanded,
    saveSettings,
    addRepo,
    addStandaloneRepo,
    removeStandaloneRepo,
    removeRepo,
    runAction,
    clearResults,
    findRepo,
  };
}
