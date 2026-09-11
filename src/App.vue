<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { useRoute } from "vue-router";
import TabBar from "./components/TabBar.vue";
import RepoPane from "./components/RepoPane.vue";
import OutputModal from "./components/OutputModal.vue";
import SettingsModal from "./components/SettingsModal.vue";
import Toast from "./components/Toast.vue";
import GroupsView from "./views/GroupsView.vue";
import { useApp } from "./composables/useApp";
import { GROUPS_TAB_ID, useTabs } from "./composables/useTabs";

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
const settingsOpen = ref(false);
const { repoTabs, activeId, syncFromRoute, refreshTitles } = useTabs();

onMounted(() => {
  void load();
});

watch(
  () => [route.name, route.params.id],
  () => {
    const repoId = typeof route.params.id === "string" ? route.params.id : undefined;
    syncFromRoute(repoId, route.name === "home");
  },
  { immediate: true },
);

watch(statuses, () => {
  refreshTitles();
});
</script>

<template>
  <div class="app-shell">
    <TabBar @settings="settingsOpen = true" />
    <main class="main">
      <p v-if="error" class="banner">{{ error }}</p>
      <GroupsView v-show="activeId === GROUPS_TAB_ID" />
      <RepoPane
        v-for="tab in repoTabs"
        v-show="activeId === tab.id"
        :key="tab.id"
        :repo-id="tab.id"
      />
    </main>
    <Transition name="toast" :duration="{ enter: 520, leave: 280 }">
      <Toast
        v-if="toastMessage"
        :message="toastMessage"
        :kind="toastKind"
        @dismiss="onToastDismiss"
      />
    </Transition>
    <SettingsModal v-if="settingsOpen" @close="settingsOpen = false" />
    <OutputModal
      v-if="actionOutputOpen && actionOutput"
      :title="actionOutput.title"
      :results="actionOutput.results"
      @close="dismissOutput"
    />
  </div>
</template>
