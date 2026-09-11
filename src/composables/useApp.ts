import { computed, nextTick, ref, watch } from "vue";
import * as api from "../api";
import { ensureNotificationPermission, notifyRefreshComplete } from "../notify";
import type { RepoActionResult, RepoGroup, RepoStatus } from "../types";

const groups = ref<RepoGroup[]>([]);
const statuses = ref<Record<string, RepoStatus>>({});
const results = ref<Record<string, RepoActionResult[]>>({});
const busy = ref<Record<string, string>>({});
const error = ref("");
const loaded = ref(false);
const refreshIntervalSeconds = ref(300);
const refreshingAll = ref(false);
const lastRefreshAt = ref<Date | null>(null);
const nextRefreshAt = ref<number | null>(null);
const nowTick = ref(Date.now());
const refreshingRepos = ref<Record<string, boolean>>({});
const refreshingGroups = ref<Record<string, boolean>>({});
const refreshTotal = ref(0);
const refreshDone = ref(0);
const refreshCancelled = ref(false);
let autoRefreshTimer: ReturnType<typeof setTimeout> | null = null;
let tickTimer: ReturnType<typeof setInterval> | null = null;
let autoRefreshStarted = false;

const statusById = computed(() => statuses.value);

export function useApp() {
  async function load() {
    error.value = "";
    try {
      const data = await api.getState();
      groups.value = data.groups;
      refreshIntervalSeconds.value = data.refreshIntervalSeconds ?? 300;
      loaded.value = true;
      startAutoRefresh();
      await Promise.all(
        groups.value.filter((group) => group.expanded).map((group) => refreshStatus(group.id)),
      );
    } catch (err) {
      error.value = String(err);
    }
  }

  function applyStatus(status: RepoStatus) {
    statuses.value = { ...statuses.value, [status.id]: status };
    lastRefreshAt.value = new Date();
  }

  async function refreshStatus(groupId: string, fetch = false) {
    try {
      const list = await api.groupStatus(groupId, fetch);
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

  async function refreshRepo(groupId: string, repoId: string) {
    if (refreshCancelled.value) {
      return;
    }
    refreshingRepos.value = { ...refreshingRepos.value, [repoId]: true };
    if (refreshTotal.value > 0) {
      refreshDone.value += 1;
    }
    await nextTick();
    try {
      applyStatus(await api.refreshRepo(groupId, repoId, true));
    } catch (err) {
      error.value = String(err);
    } finally {
      const next = { ...refreshingRepos.value };
      delete next[repoId];
      refreshingRepos.value = next;
      await nextTick();
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
      void ensureNotificationPermission();
    }
    try {
      for (const repo of group.repos) {
        if (refreshCancelled.value) {
          break;
        }
        await refreshRepo(groupId, repo.id);
      }
      if (notify && !refreshCancelled.value) {
        void notifyRefreshComplete(group.repos.length, group.name);
      }
    } finally {
      const groupsBusy = { ...refreshingGroups.value };
      delete groupsBusy[groupId];
      refreshingGroups.value = groupsBusy;
      const next = { ...busy.value };
      delete next[groupId];
      busy.value = next;
      if (!refreshingAll.value) {
        refreshCancelled.value = false;
        refreshDone.value = 0;
        refreshTotal.value = 0;
      }
    }
  }

  async function refreshAll(options?: { notify?: boolean }) {
    if (refreshingAll.value || !groups.value.length) {
      return;
    }
    const notify = options?.notify ?? true;
    refreshingAll.value = true;
    refreshCancelled.value = false;
    error.value = "";
    refreshDone.value = 0;
    refreshTotal.value = groups.value.reduce((sum, group) => sum + group.repos.length, 0);
    if (notify) {
      void ensureNotificationPermission();
    }
    if (autoRefreshTimer) {
      clearTimeout(autoRefreshTimer);
      autoRefreshTimer = null;
    }
    try {
      for (const group of groups.value) {
        if (refreshCancelled.value) {
          break;
        }
        await refreshGroup(group.id);
      }
      if (notify && !refreshCancelled.value) {
        void notifyRefreshComplete(refreshTotal.value);
      }
    } finally {
      refreshingAll.value = false;
      refreshCancelled.value = false;
      refreshDone.value = 0;
      refreshTotal.value = 0;
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

  async function saveRefreshInterval(seconds: number) {
    refreshIntervalSeconds.value = await api.updateAppSettings(seconds);
    startAutoRefresh();
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
    if (refreshingAll.value) {
      return refreshProgressLabel.value || "Refreshing…";
    }
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
    groups.value = [...groups.value, group];
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

  async function toggleGroup(groupId: string) {
    const expanded = await api.toggleGroup(groupId);
    patchGroup(groupId, { expanded });
    if (expanded) {
      await refreshStatus(groupId);
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

  async function runAction(
    groupId: string,
    label: string,
    action: () => Promise<RepoActionResult[]>,
  ) {
    busy.value = { ...busy.value, [groupId]: label };
    error.value = "";
    try {
      const outcome = await action();
      results.value = { ...results.value, [groupId]: outcome };
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
    statuses: statusById,
    results,
    busy,
    error,
    loaded,
    refreshIntervalSeconds,
    refreshingAll,
    lastRefreshAt,
    countdownLabel,
    refreshProgressLabel,
    refreshCancelled,
    cancelRefresh,
    load,
    refreshStatus,
    refreshGroup,
    refreshAll,
    isRepoRefreshing,
    isGroupRefreshing,
    saveRefreshInterval,
    createGroup,
    renameGroup,
    deleteGroup,
    toggleGroup,
    saveSettings,
    addRepo,
    removeRepo,
    runAction,
    clearResults,
    findRepo,
  };
}
