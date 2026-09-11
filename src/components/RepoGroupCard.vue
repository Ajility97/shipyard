<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import { rangeIds } from "../selection";
import { useApp } from "../composables/useApp";
import { useTabs } from "../composables/useTabs";
import type { RepoGroup } from "../types";

const DEFAULT_HEADER = "#16323c";

const props = defineProps<{ group: RepoGroup }>();
const {
  statuses,
  results,
  busy,
  toggleGroup,
  renameGroup,
  deleteGroup,
  saveSettings,
  addRepo,
  removeRepo,
  runAction,
  refreshGroup,
  cancelRefresh,
  refreshingAll,
  refreshCancelled,
  isRepoRefreshing,
  isGroupRefreshing,
  clearResults,
} = useApp();
const { activeId, hasTab, openRepo, openRepos, closeRepos } = useTabs();
const lastClickedId = ref<string | null>(null);

const siblingIds = computed(() => props.group.repos.map((repo) => repo.id));

const renaming = ref(false);
const name = ref(props.group.name);
const pullFromBranch = ref(props.group.pullFromBranch);
const fallbacks = ref([...props.group.checkoutFallbacks]);
const checkoutTarget = ref("");
const headerColor = ref(props.group.headerColor || DEFAULT_HEADER);

watch(
  () => props.group,
  (group) => {
    name.value = group.name;
    pullFromBranch.value = group.pullFromBranch;
    fallbacks.value = [...group.checkoutFallbacks];
    headerColor.value = group.headerColor || DEFAULT_HEADER;
  },
);

const actionLabel = computed(() => busy.value[props.group.id] ?? "");
const groupResults = computed(() => results.value[props.group.id] ?? []);

const headerStyle = computed(() => ({
  "--group-header": headerColor.value,
  "--group-header-fg": contrastingText(headerColor.value),
}));

async function persistSettings() {
  const nextFallbacks = fallbacks.value.map((item) => item.trim()).filter(Boolean);
  if (!nextFallbacks.length) {
    nextFallbacks.push("develop");
    fallbacks.value = ["develop"];
  }
  await saveSettings(
    props.group.id,
    pullFromBranch.value.trim() || "develop",
    nextFallbacks,
    headerColor.value,
  );
}

function onHeaderColor(event: Event) {
  const value = (event.target as HTMLInputElement).value;
  headerColor.value = value;
  void persistSettings();
}

async function finishRename() {
  const value = name.value.trim();
  renaming.value = false;
  if (!value || value === props.group.name) {
    name.value = props.group.name;
    return;
  }
  await renameGroup(props.group.id, value);
}

async function confirmDelete() {
  if (window.confirm(`Delete group “${props.group.name}”? Repositories on disk are not deleted.`)) {
    const selected = props.group.repos.filter((repo) => hasTab(repo.id)).map((repo) => repo.id);
    await deleteGroup(props.group.id);
    if (selected.length) {
      closeRepos(selected);
    }
  }
}

async function pickRepo() {
  const selected = await open({
    directory: true,
    multiple: true,
    title: "Add Git repositories",
  });
  const paths = Array.isArray(selected) ? selected : selected ? [selected] : [];
  const failures: string[] = [];
  for (const path of paths) {
    try {
      await addRepo(props.group.id, path);
    } catch (err) {
      failures.push(`${path}: ${String(err)}`);
    }
  }
  if (failures.length) {
    window.alert(failures.join("\n"));
  }
}

function addFallback() {
  const next = [...fallbacks.value];
  if (next.length && next[next.length - 1] === "develop") {
    next.splice(next.length - 1, 0, "");
  } else {
    next.push("");
  }
  fallbacks.value = next;
}

function removeFallback(index: number) {
  fallbacks.value = fallbacks.value.filter((_, i) => i !== index);
  if (!fallbacks.value.length) {
    fallbacks.value = ["develop"];
  }
  void persistSettings();
}

async function removeAndLeave(repoId: string) {
  await removeRepo(props.group.id, repoId);
  if (hasTab(repoId)) {
    closeRepos([repoId]);
  }
}

function handleRepoClick(event: MouseEvent, repoId: string) {
  if (event.shiftKey && lastClickedId.value) {
    openRepos(rangeIds(siblingIds.value, lastClickedId.value, repoId), repoId);
  } else {
    openRepo(repoId);
  }
  lastClickedId.value = repoId;
}

function pullAll() {
  return runAction(props.group.id, "Pulling current branches…", () => api.pullCurrent(props.group.id));
}

function pullNamed() {
  return persistSettings().then(() =>
    runAction(props.group.id, `Pulling ${pullFromBranch.value}…`, () => api.pullFromBranch(props.group.id)),
  );
}

function checkoutAll() {
  return persistSettings().then(() =>
    runAction(props.group.id, "Checking out branches…", () =>
      api.checkoutAll(props.group.id, checkoutTarget.value, fallbacks.value),
    ),
  );
}

function folderName(path: string) {
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function contrastingText(color: string) {
  const hex = color.replace("#", "");
  const normalized =
    hex.length === 3
      ? hex
          .split("")
          .map((part) => part + part)
          .join("")
      : hex;
  if (normalized.length < 6) {
    return "#e8edf5";
  }
  const red = Number.parseInt(normalized.slice(0, 2), 16);
  const green = Number.parseInt(normalized.slice(2, 4), 16);
  const blue = Number.parseInt(normalized.slice(4, 6), 16);
  const luma = red * 0.299 + green * 0.587 + blue * 0.114;
  return luma > 150 ? "#14161b" : "#e8edf5";
}
</script>

<template>
  <section class="group">
    <div class="group-header" :style="headerStyle">
      <button class="chevron" type="button" :aria-expanded="group.expanded" @click="toggleGroup(group.id)">
        {{ group.expanded ? "▾" : "▸" }}
      </button>
      <input
        v-if="renaming"
        v-model="name"
        type="text"
        @blur="finishRename"
        @keydown.enter="finishRename"
        @keydown.escape="renaming = false; name = group.name"
      />
      <button v-else class="group-title" type="button" @dblclick="renaming = true">
        {{ group.name }}
      </button>
      <span class="group-count">{{ group.repos.length }}</span>
      <span v-if="isGroupRefreshing(group.id)" class="spinner" aria-label="Refreshing group" />
      <div class="group-actions">
        <label class="color-picker" title="Header color">
          <input type="color" :value="headerColor" @input="onHeaderColor" />
        </label>
        <button
          class="ghost tiny"
          type="button"
          :class="{ danger: isGroupRefreshing(group.id) && !refreshingAll }"
          :disabled="
            refreshingAll ||
            refreshCancelled ||
            (!!actionLabel && !isGroupRefreshing(group.id))
          "
          @click="isGroupRefreshing(group.id) ? cancelRefresh() : refreshGroup(group.id)"
        >
          {{
            isGroupRefreshing(group.id) && !refreshingAll
              ? refreshCancelled
                ? "Cancelling…"
                : "Cancel"
              : "Refresh"
          }}
        </button>
        <button class="ghost tiny" type="button" @click="renaming = true">Rename</button>
        <button class="ghost danger tiny" type="button" @click="confirmDelete">Delete</button>
      </div>
    </div>

    <div v-if="group.expanded" class="group-body">
      <div class="toolbar">
        <button class="primary" type="button" :disabled="!!actionLabel" @click="pullAll">
          Pull all
        </button>
        <input
          v-model="pullFromBranch"
          type="text"
          class="grow"
          title="Branch to pull into the current checkout"
          @change="persistSettings"
        />
        <button type="button" :disabled="!!actionLabel" @click="pullNamed">
          Pull from branch
        </button>
        <input
          v-model="checkoutTarget"
          type="text"
          class="grow"
          placeholder="Check out all to, e.g. MERP-123"
        />
        <button type="button" :disabled="!!actionLabel || !checkoutTarget.trim()" @click="checkoutAll">
          Checkout all
        </button>
      </div>
      <div class="fallback-row wrap">
        <span class="muted tiny">Fallbacks</span>
        <input
          v-for="(_fallback, index) in fallbacks"
          :key="index"
          v-model="fallbacks[index]"
          type="text"
          @change="persistSettings"
        />
        <button class="ghost tiny" type="button" @click="addFallback">+</button>
        <button
          v-if="fallbacks.length > 1"
          class="ghost tiny"
          type="button"
          @click="removeFallback(fallbacks.length - 1)"
        >
          −
        </button>
      </div>

      <div
        v-for="repo in group.repos"
        :key="repo.id"
        class="repo-row"
        :class="{ active: activeId === repo.id, open: hasTab(repo.id), refreshing: isRepoRefreshing(repo.id) }"
      >
        <button class="repo-name" type="button" @click="handleRepoClick($event, repo.id)">
          {{ statuses[repo.id]?.name ?? folderName(repo.path) }}
        </button>
        <span class="branch">
          <span v-if="isRepoRefreshing(repo.id)" class="spinner" aria-label="Refreshing repository" />
          {{ statuses[repo.id]?.branch ?? "…" }}
          <span
            v-if="(statuses[repo.id]?.behind ?? 0) > 0"
            class="sync-count behind"
            :title="`${statuses[repo.id]?.behind} commits behind`"
          >↓{{ statuses[repo.id]?.behind }}</span>
          <span
            v-if="(statuses[repo.id]?.ahead ?? 0) > 0"
            class="sync-count ahead"
            :title="`${statuses[repo.id]?.ahead} commits ahead`"
          >↑{{ statuses[repo.id]?.ahead }}</span>
          <span v-if="statuses[repo.id]?.dirty" class="dirty-chip" title="Uncommitted changes">*</span>
        </span>
        <button class="ghost danger tiny" type="button" @click="removeAndLeave(repo.id)">×</button>
      </div>

      <button class="ghost add-row" type="button" @click="pickRepo">+ Add repository</button>
      <p v-if="actionLabel" class="muted tiny" style="padding: 0.35rem 0.9rem">{{ actionLabel }}</p>
      <div v-if="groupResults.length" class="results">
        <div
          v-for="result in groupResults"
          :key="result.path"
          class="result"
          :class="result.ok ? 'ok' : 'bad'"
        >
          <strong>{{ folderName(result.path) }}</strong>
          — {{ result.message }}
        </div>
        <button class="ghost tiny" type="button" @click="clearResults(group.id)">Dismiss</button>
      </div>
    </div>
  </section>
</template>
