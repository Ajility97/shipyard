<script setup lang="ts">
import { onMounted, watch } from "vue";
import { useRoute } from "vue-router";
import TabBar from "./components/TabBar.vue";
import RepoPane from "./components/RepoPane.vue";
import GroupsView from "./views/GroupsView.vue";
import { useApp } from "./composables/useApp";
import { GROUPS_TAB_ID, useTabs } from "./composables/useTabs";

const route = useRoute();
const { load, error, statuses } = useApp();
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
    </main>
  </div>
</template>
