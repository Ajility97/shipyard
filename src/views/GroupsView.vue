<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { useApp } from "../composables/useApp";
import { useTabs } from "../composables/useTabs";
import type { RepoGroup } from "../types";
import RepoGroupCard from "../components/RepoGroupCard.vue";
import RepoRow from "../components/RepoRow.vue";

const DRAFT_GROUP: RepoGroup = {
  id: "__draft__",
  name: "",
  expanded: true,
  pullFromBranch: "develop",
  checkoutFallbacks: ["develop"],
  headerColor: "#16323c",
  repos: [],
};

const {
  groups,
  standaloneRepos,
  setAllGroupsExpanded,
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

const canExpandAll = computed(
  () => groups.value.length > 0 && groups.value.some((group) => !group.expanded),
);

const canCollapseAll = computed(
  () => groups.value.length > 0 && groups.value.some((group) => group.expanded),
);

function startCreate() {
  creating.value = true;
}

function cancelCreate() {
  creating.value = false;
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
            <button
              class="primary"
              type="button"
              :disabled="creating"
              @click="startCreate"
            >
              New group
            </button>
            <button class="ghost" type="button" @click="pickStandaloneRepo">Add repository</button>
          </div>
          <div class="toolbar-end">
            <button
              class="ghost"
              type="button"
              :disabled="!canExpandAll"
              @click="setAllGroupsExpanded(true)"
            >
              <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="M19.5 5.25 12 12.75 4.5 5.25m15 6L12 18.75l-7.5-7.5" />
              </svg>
              Expand
            </button>
            <button
              class="ghost"
              type="button"
              :disabled="!canCollapseAll"
              @click="setAllGroupsExpanded(false)"
            >
              <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path d="m4.5 18.75 7.5-7.5 7.5 7.5m-15-6 7.5-7.5 7.5 7.5" />
              </svg>
              Collapse
            </button>
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
                <svg
                  v-if="!refreshingAll"
                  class="button-icon"
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <path
                    d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
                  />
                </svg>
                {{ refreshCancelled ? "Cancelling…" : refreshingAll ? "Cancel" : "Refresh" }}
              </button>
            </div>
          </div>
        </div>

        <p v-if="isEmpty && !creating" class="muted">
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
          <RepoGroupCard
            v-if="creating"
            :group="DRAFT_GROUP"
            draft
            @cancel="cancelCreate"
            @created="cancelCreate"
          />
          <RepoGroupCard v-for="group in groups" :key="group.id" :group="group" />
        </div>
      </div>
    </div>
  </div>
</template>
