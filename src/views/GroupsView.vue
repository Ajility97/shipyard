<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useApp } from "../composables/useApp";
import { useTabs } from "../composables/useTabs";
import RepoGroupCard from "../components/RepoGroupCard.vue";
import RepoRow from "../components/RepoRow.vue";

const {
  groups,
  standaloneRepos,
  createGroup,
  addStandaloneRepo,
  removeStandaloneRepo,
  refreshAll,
  cancelRefresh,
  refreshIntervalSeconds,
  refreshingAll,
  refreshCancelled,
  lastRefreshAt,
  countdownLabel,
  refreshProgressLabel,
  saveRefreshInterval,
} = useApp();
const { hasTab, closeRepos } = useTabs();
const creating = ref(false);
const name = ref("");
const interval = computed({
  get: () => refreshIntervalSeconds.value,
  set: (value: number) => {
    void saveRefreshInterval(value);
  },
});

const lastRefreshLabel = computed(() => {
  if (!lastRefreshAt.value) {
    return "";
  }
  return lastRefreshAt.value.toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
});

const refreshAllProgress = computed(
  () => refreshProgressLabel.value.replace(/^Refreshing\s+/, "") || "…",
);

const hasRepos = computed(
  () =>
    standaloneRepos.value.length > 0 || groups.value.some((group) => group.repos.length > 0),
);

const standaloneIds = computed(() => standaloneRepos.value.map((repo) => repo.id));

const isEmpty = computed(() => !groups.value.length && !standaloneRepos.value.length);

async function submit() {
  const value = name.value.trim();
  if (!value) {
    return;
  }
  await createGroup(value);
  name.value = "";
  creating.value = false;
}

function cancel() {
  creating.value = false;
  name.value = "";
}

async function pickStandaloneRepo() {
  const selected = await open({
    directory: true,
    multiple: true,
    title: "Add Git repositories",
  });
  const paths = Array.isArray(selected) ? selected : selected ? [selected] : [];
  const failures: string[] = [];
  for (const path of paths) {
    try {
      await addStandaloneRepo(path);
    } catch (err) {
      failures.push(`${path}: ${String(err)}`);
    }
  }
  if (failures.length) {
    window.alert(failures.join("\n"));
  }
}

async function removeStandalone(repoId: string) {
  await removeStandaloneRepo(repoId);
  if (hasTab(repoId)) {
    closeRepos([repoId]);
  }
}
</script>

<template>
  <div class="groups-page">
    <div class="groups-inner">
      <div class="groups-header">
        <div>
          <div class="brand">Krakdown</div>
          <p class="muted tiny">Click a repository to open it in a tab.</p>
        </div>
        <div class="refresh-area">
          <label class="refresh-setting">
            <span class="muted tiny">Auto-refresh</span>
            <select v-model.number="interval">
              <option :value="0">Off</option>
              <option :value="60">1 minute</option>
              <option :value="300">5 minutes</option>
              <option :value="900">15 minutes</option>
              <option :value="1800">30 minutes</option>
            </select>
          </label>
          <div class="refresh-times">
            <span v-if="countdownLabel" class="refresh-meta countdown">{{ countdownLabel }}</span>
            <span class="refresh-last">Last refresh {{ lastRefreshLabel || "—" }}</span>
          </div>
        </div>
      </div>

      <div class="groups-display">
        <div class="groups-toolbar">
          <div class="toolbar-start">
            <form v-if="creating" class="new-group" @submit.prevent="submit">
              <input
                v-model="name"
                type="text"
                placeholder="Group name"
                autofocus
                @keydown.escape="cancel"
              />
              <button class="primary" type="submit">Create</button>
              <button class="ghost" type="button" @click="cancel">Cancel</button>
            </form>
            <button v-else class="primary" type="button" @click="creating = true">New group</button>
            <button class="ghost" type="button" @click="pickStandaloneRepo">Add repository</button>
          </div>
          <div class="header-action">
            <span v-if="refreshingAll" class="action-progress">
              <span class="spinner" aria-hidden="true" />
              {{ refreshAllProgress }}
            </span>
            <button
              type="button"
              :class="{ danger: refreshingAll }"
              :disabled="!hasRepos || refreshCancelled"
              @click="refreshingAll ? cancelRefresh() : refreshAll()"
            >
              {{ refreshCancelled ? "Cancelling…" : refreshingAll ? "Cancel" : "Refresh all" }}
            </button>
          </div>
        </div>

        <p v-if="isEmpty" class="muted">
          Add a repository, or create a group for several at once.
        </p>

        <div v-if="standaloneRepos.length" class="standalone-list">
          <RepoRow
            v-for="repo in standaloneRepos"
            :key="repo.id"
            :repo="repo"
            :sibling-ids="standaloneIds"
            flush
            @remove="removeStandalone"
          />
        </div>

        <div class="groups-list">
          <RepoGroupCard v-for="group in groups" :key="group.id" :group="group" />
        </div>
      </div>
    </div>
  </div>
</template>
