<script setup lang="ts">
import { computed, ref } from "vue";
import { useApp } from "../composables/useApp";
import RepoGroupCard from "../components/RepoGroupCard.vue";

const {
  groups,
  createGroup,
  refreshAll,
  cancelRefresh,
  refreshIntervalSeconds,
  refreshingAll,
  refreshCancelled,
  lastRefreshAt,
  countdownLabel,
  saveRefreshInterval,
} = useApp();
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
</script>

<template>
  <div class="groups-page">
    <div class="groups-inner">
      <div class="groups-header">
        <div>
          <div class="brand">Krakdown</div>
          <p class="muted tiny">
            Click a repository to open it in a tab. Shift-click to open several.
            <span v-if="lastRefreshLabel"> Last refresh {{ lastRefreshLabel }}.</span>
          </p>
        </div>
        <div class="groups-toolbar">
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
          <span v-if="countdownLabel" class="countdown">{{ countdownLabel }}</span>
          <button
            type="button"
            :class="{ danger: refreshingAll }"
            :disabled="!groups.length || refreshCancelled"
            @click="refreshingAll ? cancelRefresh() : refreshAll()"
          >
            {{ refreshCancelled ? "Cancelling…" : refreshingAll ? "Cancel" : "Refresh all" }}
          </button>
          <button class="primary" type="button" @click="creating = true">New group</button>
        </div>
      </div>

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

      <p v-if="!groups.length" class="muted">
        Create a group, then add local repositories.
      </p>

      <div class="groups-list">
        <RepoGroupCard v-for="group in groups" :key="group.id" :group="group" />
      </div>
    </div>
  </div>
</template>
