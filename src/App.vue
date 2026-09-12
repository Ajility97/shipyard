<script setup lang="ts">
import { defineAsyncComponent, onMounted, onUnmounted, ref, watch } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useRoute } from "vue-router";
import TabBar from "./components/TabBar.vue";
import RepoPane from "./components/RepoPane.vue";
import OutputModal from "./components/OutputModal.vue";
import Toast from "./components/Toast.vue";
import GroupsView from "./views/GroupsView.vue";

const HistoryView = defineAsyncComponent(() => import("./views/HistoryView.vue"));
const SettingsView = defineAsyncComponent(() => import("./views/SettingsView.vue"));
import { useApp } from "./composables/useApp";
import { GROUPS_TAB_ID, HISTORY_TAB_ID, SETTINGS_TAB_ID, useTabs } from "./composables/useTabs";

const route = useRoute();
const {
  load,
  error,
  statuses,
  toastMessage,
  toastKind,
  actionOutput,
  actionOutputOpen,
  dismissToast,
  dismissOutput,
  openOutput,
} = useApp();

function onToastDismiss() {
  if (toastKind.value === "error" && !actionOutputOpen.value) {
    openOutput();
  }
  dismissToast();
}
const appVersion = ref("0.1.0");
const {
  repoTabs,
  historyTabOpen,
  settingsTabOpen,
  activeId,
  syncFromRoute,
  refreshTitles,
  closeActiveTab,
} = useTabs();

let stopCloseShortcut: (() => void) | undefined;

async function closeActiveTabOrWindow() {
  if (closeActiveTab()) {
    return;
  }
  await getCurrentWindow().close();
}

function onWindowKeydown(event: KeyboardEvent) {
  if (!(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey) {
    return;
  }
  if (event.key.toLowerCase() !== "w") {
    return;
  }
  if (!closeActiveTab()) {
    return;
  }
  event.preventDefault();
  event.stopPropagation();
}

onMounted(() => {
  void load();
  void getVersion()
    .then((value) => {
      appVersion.value = value;
    })
    .catch(() => {
      /* keep the bundled fallback */
    });
  window.addEventListener("keydown", onWindowKeydown, true);
  void listen("close-tab-or-window", () => {
    void closeActiveTabOrWindow();
  }).then((unlisten) => {
    stopCloseShortcut = unlisten;
  });
});

onUnmounted(() => {
  window.removeEventListener("keydown", onWindowKeydown, true);
  stopCloseShortcut?.();
});

watch(
  () => [route.name, route.params.id],
  () => {
    const repoId = typeof route.params.id === "string" ? route.params.id : undefined;
    const panel =
      route.name === "history" ? "history" : route.name === "settings" ? "settings" : undefined;
    syncFromRoute(repoId, route.name === "home", panel);
  },
  { immediate: true },
);

watch(statuses, () => {
  refreshTitles();
});
</script>

<template>
  <div class="app-shell">
    <TabBar />
    <main class="main">
      <p v-if="error" class="banner">{{ error }}</p>
      <div class="main-pane" v-show="activeId === GROUPS_TAB_ID">
        <GroupsView />
      </div>
      <div
        v-for="tab in repoTabs"
        :key="tab.id"
        class="main-pane"
        v-show="activeId === tab.id"
      >
        <RepoPane :repo-id="tab.id" />
      </div>
      <div v-if="historyTabOpen" class="main-pane" v-show="activeId === HISTORY_TAB_ID">
        <HistoryView />
      </div>
      <div v-if="settingsTabOpen" class="main-pane" v-show="activeId === SETTINGS_TAB_ID">
        <SettingsView />
      </div>
    </main>
    <footer class="status-bar">
      <span class="status-bar-version">{{ appVersion }}</span>
    </footer>
    <Transition name="toast" :duration="{ enter: 520, leave: 280 }">
      <Toast
        v-if="toastMessage"
        :message="toastMessage"
        :kind="toastKind"
        @dismiss="onToastDismiss"
      />
    </Transition>
    <OutputModal
      v-if="actionOutputOpen && actionOutput"
      :title="actionOutput.title"
      :results="actionOutput.results"
      @close="dismissOutput"
    />
  </div>
</template>
