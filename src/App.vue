<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { getVersion } from "@tauri-apps/api/app";
import { useRoute } from "vue-router";
import TabBar from "./components/TabBar.vue";
import RepoPane from "./components/RepoPane.vue";
import OutputModal from "./components/OutputModal.vue";
import Toast from "./components/Toast.vue";
import GroupsView from "./views/GroupsView.vue";
import SettingsView from "./views/SettingsView.vue";
import { useApp } from "./composables/useApp";
import { GROUPS_TAB_ID, SETTINGS_TAB_ID, useTabs } from "./composables/useTabs";

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
const { repoTabs, settingsTabOpen, activeId, syncFromRoute, refreshTitles } = useTabs();

onMounted(() => {
  void load();
  void getVersion()
    .then((value) => {
      appVersion.value = value;
    })
    .catch(() => {
      /* keep the bundled fallback */
    });
});

watch(
  () => [route.name, route.params.id],
  () => {
    const repoId = typeof route.params.id === "string" ? route.params.id : undefined;
    syncFromRoute(repoId, route.name === "home", route.name === "settings");
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
      <GroupsView v-show="activeId === GROUPS_TAB_ID" />
      <RepoPane
        v-for="tab in repoTabs"
        v-show="activeId === tab.id"
        :key="tab.id"
        :repo-id="tab.id"
      />
      <SettingsView v-if="settingsTabOpen" v-show="activeId === SETTINGS_TAB_ID" />
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
