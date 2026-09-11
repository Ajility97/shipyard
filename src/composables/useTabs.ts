import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { useApp } from "./useApp";

export const GROUPS_TAB_ID = "groups";

export interface AppTab {
  id: string;
  title: string;
  closable: boolean;
}

interface RepoTab {
  id: string;
  title: string;
}

const repoTabs = ref<RepoTab[]>([]);
const activeId = ref(GROUPS_TAB_ID);

function folderName(path: string) {
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

export function useTabs() {
  const router = useRouter();
  const { findRepo } = useApp();

  const tabs = computed<AppTab[]>(() => [
    { id: GROUPS_TAB_ID, title: "Groups", closable: false },
    ...repoTabs.value.map((tab) => ({ ...tab, closable: true })),
  ]);

  function titleFor(id: string) {
    const match = findRepo(id);
    if (match?.status?.name) {
      return match.status.name;
    }
    if (match?.repo.path) {
      return folderName(match.repo.path);
    }
    return id;
  }

  function ensureTab(id: string) {
    if (repoTabs.value.some((tab) => tab.id === id)) {
      return;
    }
    repoTabs.value = [...repoTabs.value, { id, title: titleFor(id) }];
  }

  function routeFor(id: string) {
    return id === GROUPS_TAB_ID ? "/" : `/repo/${id}`;
  }

  function activate(id: string) {
    activeId.value = id;
    const target = routeFor(id);
    if (router.currentRoute.value.path !== target) {
      void router.push(target);
    }
  }

  function openRepo(id: string) {
    ensureTab(id);
    activate(id);
  }

  function openRepos(ids: string[], focusId = ids[ids.length - 1]) {
    for (const id of ids) {
      ensureTab(id);
    }
    if (focusId) {
      activate(focusId);
    }
  }

  function closeRepo(id: string) {
    if (id === GROUPS_TAB_ID) {
      return;
    }
    const index = repoTabs.value.findIndex((tab) => tab.id === id);
    if (index === -1) {
      return;
    }
    repoTabs.value = repoTabs.value.filter((tab) => tab.id !== id);
    if (activeId.value === id) {
      const neighbor = repoTabs.value[index] ?? repoTabs.value[index - 1];
      activate(neighbor?.id ?? GROUPS_TAB_ID);
    }
  }

  function closeRepos(ids: string[]) {
    const closing = new Set(ids);
    const index = repoTabs.value.findIndex((tab) => tab.id === activeId.value);
    repoTabs.value = repoTabs.value.filter((tab) => !closing.has(tab.id));
    if (closing.has(activeId.value)) {
      const neighbor = repoTabs.value[index] ?? repoTabs.value[index - 1];
      activate(neighbor?.id ?? GROUPS_TAB_ID);
    }
  }

  function hasTab(id: string) {
    return repoTabs.value.some((tab) => tab.id === id);
  }

  function syncFromRoute(repoId: string | undefined, isHome: boolean) {
    if (isHome) {
      activeId.value = GROUPS_TAB_ID;
      return;
    }
    if (!repoId) {
      return;
    }
    ensureTab(repoId);
    activeId.value = repoId;
  }

  function refreshTitles() {
    repoTabs.value = repoTabs.value.map((tab) => ({
      ...tab,
      title: titleFor(tab.id),
    }));
  }

  return {
    tabs,
    repoTabs,
    activeId,
    openRepo,
    openRepos,
    activate,
    closeRepo,
    closeRepos,
    hasTab,
    syncFromRoute,
    refreshTitles,
  };
}
