<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { useApp } from "../composables/useApp";
import { useOverflowMenu } from "../composables/useOverflowMenu";
import { useTabs } from "../composables/useTabs";
import type { RepoGroup } from "../types";
import Modal from "./Modal.vue";
import RepoRow from "./RepoRow.vue";

const DEFAULT_HEADER = "#16323c";
const LAST_FALLBACKS = ["develop", "master", "main"] as const;
type LastFallback = (typeof LAST_FALLBACKS)[number];

function isLastFallback(value: string): value is LastFallback {
  return (LAST_FALLBACKS as readonly string[]).includes(value);
}

const props = defineProps<{ group: RepoGroup }>();
const {
  busy,
  toggleGroup,
  renameGroup,
  deleteGroup,
  saveSettings,
  addRepo,
  removeRepo,
  pullGroup,
  pullProgress,
  pullCancelled,
  cancelPull,
  refreshGroup,
  cancelRefresh,
  refreshingAll,
  refreshCancelled,
  refreshProgress,
  isGroupRefreshing,
  checkoutGroup,
  checkoutProgress,
  checkoutCancelled,
  cancelCheckout,
} = useApp();
const { hasTab, closeRepos } = useTabs();
const {
  isOpen: groupMenuOpen,
  toggle: toggleGroupMenu,
  close: closeMenus,
} = useOverflowMenu(() => `group:${props.group.id}`);

const siblingIds = computed(() => props.group.repos.map((repo) => repo.id));

const renaming = ref(false);
const name = ref(props.group.name);
const pullFromBranch = ref(props.group.pullFromBranch);
const extraFallbacks = ref<string[]>([]);
const lastFallback = ref<LastFallback>("develop");
loadFallbacks(props.group.checkoutFallbacks);
const checkoutSource = ref<"develop" | "specify" | "master" | "main">("develop");
const checkoutTarget = ref("");
const headerColor = ref(props.group.headerColor || DEFAULT_HEADER);
const modal = ref<"pull" | "checkout" | null>(null);
const pullSource = ref<"current" | "develop" | "master" | "main" | "specify">("current");
const specifyBranch = ref("");

watch(
  () => props.group,
  (group) => {
    name.value = group.name;
    pullFromBranch.value = group.pullFromBranch;
    loadFallbacks(group.checkoutFallbacks);
    headerColor.value = group.headerColor || DEFAULT_HEADER;
  },
);

const actionLabel = computed(() => busy.value[props.group.id] ?? "");

const headerStyle = computed(() => ({
  "--group-header": headerColor.value,
  "--group-header-fg": contrastingText(headerColor.value),
}));

function loadFallbacks(list: string[]) {
  const cleaned = list.map((item) => item.trim()).filter(Boolean);
  const last = cleaned[cleaned.length - 1];
  if (last && isLastFallback(last)) {
    lastFallback.value = last;
    extraFallbacks.value = cleaned.slice(0, -1);
    return;
  }
  lastFallback.value = "develop";
  extraFallbacks.value = cleaned;
}

function fallbackList() {
  return [
    ...extraFallbacks.value.map((item) => item.trim()).filter(Boolean),
    lastFallback.value,
  ];
}

async function persistSettings() {
  extraFallbacks.value = extraFallbacks.value.map((item) => item.trim()).filter(Boolean);
  const nextFallbacks = fallbackList();
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

function startRename() {
  closeMenus();
  renaming.value = true;
}

function cancelRename() {
  renaming.value = false;
  name.value = props.group.name;
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
  closeMenus();
  const ok = await confirm(
    `Delete group “${props.group.name}”? Repositories on disk are not deleted.`,
    {
      title: "Delete group",
      kind: "warning",
      okLabel: "Delete",
      cancelLabel: "Cancel",
    },
  );
  if (!ok) {
    return;
  }
  const selected = props.group.repos.filter((repo) => hasTab(repo.id)).map((repo) => repo.id);
  await deleteGroup(props.group.id);
  if (selected.length) {
    closeRepos(selected);
  }
}

async function pickRepo() {
  closeMenus();
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
  extraFallbacks.value = [...extraFallbacks.value, ""];
}

function removeFallback() {
  extraFallbacks.value = extraFallbacks.value.slice(0, -1);
}

async function removeAndLeave(repoId: string) {
  await removeRepo(props.group.id, repoId);
  if (hasTab(repoId)) {
    closeRepos([repoId]);
  }
}

const pulling = computed(() => Boolean(pullProgress.value[props.group.id]));
const checkingOut = computed(() => Boolean(checkoutProgress.value[props.group.id]));
const groupRefreshing = computed(() => isGroupRefreshing(props.group.id));
const canCancelRefresh = computed(() => groupRefreshing.value && !refreshingAll.value);

const pullLabel = computed(() => {
  if (pullCancelled.value[props.group.id]) {
    return "Cancelling…";
  }
  return pulling.value ? "Cancel" : "Pull";
});

const pullBranch = computed(() => {
  if (pullSource.value === "current") {
    return "";
  }
  if (pullSource.value === "specify") {
    return specifyBranch.value.trim();
  }
  return pullSource.value;
});

const canConfirmPull = computed(
  () => pullSource.value === "current" || Boolean(pullBranch.value),
);

const pullRemoteLabel = computed(() => {
  if (pullSource.value === "current") {
    return "current-branch";
  }
  return pullBranch.value || "…";
});

const pullHint = computed(() =>
  pullSource.value === "current"
    ? "Use this to pick up others’ commits on the same branch."
    : "Brings that remote branch into this checkout. If Git hits conflicts, resolve them in your local files.",
);

const checkoutLabel = computed(() => {
  if (checkoutCancelled.value[props.group.id]) {
    return "Cancelling…";
  }
  return checkingOut.value ? "Cancel" : "Checkout";
});

const checkoutBranch = computed(() => {
  if (checkoutSource.value === "specify") {
    return checkoutTarget.value.trim();
  }
  return checkoutSource.value;
});

const canConfirmCheckout = computed(() => Boolean(checkoutBranch.value));

const refreshLabel = computed(() => {
  if (canCancelRefresh.value && refreshCancelled.value) {
    return "Cancelling…";
  }
  return canCancelRefresh.value ? "Cancel" : "Refresh";
});

function openPull() {
  if (pulling.value) {
    cancelPull(props.group.id);
    return;
  }
  pullSource.value = "current";
  specifyBranch.value = props.group.pullFromBranch;
  modal.value = "pull";
}

function openCheckout() {
  if (checkingOut.value) {
    cancelCheckout(props.group.id);
    return;
  }
  checkoutSource.value = "develop";
  checkoutTarget.value = "";
  loadFallbacks(props.group.checkoutFallbacks);
  modal.value = "checkout";
}

function closeModal() {
  modal.value = null;
}

async function confirmPull() {
  if (!canConfirmPull.value) {
    return;
  }
  const branch = pullBranch.value;
  modal.value = null;
  if (branch) {
    pullFromBranch.value = branch;
    await persistSettings();
  }
  return pullGroup(props.group.id, branch || undefined);
}

async function checkoutAll() {
  if (!canConfirmCheckout.value) {
    return;
  }
  const target = checkoutBranch.value;
  const fallbacks = checkoutSource.value === "specify" ? fallbackList() : [];
  modal.value = null;
  if (checkoutSource.value === "specify") {
    await persistSettings();
  }
  return checkoutGroup(props.group.id, target, fallbacks);
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
      <button
        class="chevron"
        type="button"
        :class="{ open: group.expanded }"
        :aria-expanded="group.expanded"
        @click="toggleGroup(group.id)"
      >
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path d="M4 6l4 4 4-4" />
        </svg>
      </button>
      <div class="group-heading">
        <input
          v-if="renaming"
          v-model="name"
          type="text"
          @keydown.enter="finishRename"
          @keydown.escape="cancelRename"
        />
        <span v-else class="group-title">{{ group.name }}</span>
        <span class="group-count">{{ group.repos.length }}</span>
        <button
          v-if="renaming"
          class="ghost tiny"
          type="button"
          @mousedown.prevent="finishRename"
        >
          Save
        </button>
        <button
          v-if="renaming"
          class="ghost tiny"
          type="button"
          @mousedown.prevent="cancelRename"
        >
          Cancel
        </button>
        <label v-if="renaming" class="color-picker" title="Header color">
          <input type="color" :value="headerColor" @input="onHeaderColor" />
        </label>
      </div>
      <div class="group-actions">
        <template v-if="group.repos.length">
          <div class="header-action">
            <span v-if="pulling" class="action-progress">
              <span class="spinner" aria-hidden="true" />
              {{ pullProgress[group.id] }}
            </span>
            <button
              class="ghost tiny"
              type="button"
              :class="{ danger: pulling }"
              :disabled="(!!actionLabel && !pulling) || pullCancelled[group.id]"
              @click="openPull"
            >
              <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path
                  d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5M16.5 12 12 16.5m0 0L7.5 12m4.5 4.5V3"
                />
              </svg>
              {{ pullLabel }}
            </button>
          </div>
          <div class="header-action">
            <span v-if="checkingOut" class="action-progress">
              <span class="spinner" aria-hidden="true" />
              Checking out {{ checkoutProgress[group.id] }}
            </span>
            <button
              class="ghost tiny"
              type="button"
              :class="{ danger: checkingOut }"
              :disabled="(!!actionLabel && !checkingOut) || checkoutCancelled[group.id]"
              @click="openCheckout"
            >
              <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path
                  d="M7.5 21 3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5"
                />
              </svg>
              {{ checkoutLabel }}
            </button>
          </div>
          <div class="header-action">
            <span v-if="groupRefreshing" class="action-progress">
              <span class="spinner" aria-hidden="true" />
              {{ refreshProgress[group.id] || "…" }}
            </span>
            <button
              class="ghost tiny"
              type="button"
              :class="{ danger: canCancelRefresh }"
              :disabled="
                refreshingAll ||
                refreshCancelled ||
                (!!actionLabel && !groupRefreshing)
              "
              @click="groupRefreshing ? cancelRefresh() : refreshGroup(group.id)"
            >
              <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
                <path
                  d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
                />
              </svg>
              {{ refreshLabel }}
            </button>
          </div>
        </template>
        <div class="overflow-menu group-menu">
          <button
            class="ghost tiny overflow-menu-trigger"
            type="button"
            :aria-expanded="groupMenuOpen"
            aria-haspopup="menu"
            title="Group actions"
            @click.stop="toggleGroupMenu"
          >
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <circle cx="8" cy="3.25" r="1.25" />
              <circle cx="8" cy="8" r="1.25" />
              <circle cx="8" cy="12.75" r="1.25" />
            </svg>
          </button>
          <div v-if="groupMenuOpen" class="overflow-menu-dropdown" role="menu">
            <button
              class="overflow-menu-item"
              type="button"
              role="menuitem"
              @click.stop="pickRepo"
            >
              Add repository
            </button>
            <button
              class="overflow-menu-item"
              type="button"
              role="menuitem"
              @click.stop="startRename"
            >
              Edit group
            </button>
            <button
              class="overflow-menu-item danger"
              type="button"
              role="menuitem"
              @click.stop="confirmDelete"
            >
              Delete group
            </button>
          </div>
        </div>
      </div>
    </div>

    <div v-if="group.expanded && group.repos.length" class="group-body">
      <RepoRow
        v-for="repo in group.repos"
        :key="repo.id"
        :repo="repo"
        :sibling-ids="siblingIds"
        @remove="removeAndLeave"
      />
    </div>

    <Modal v-if="modal === 'pull'" title="Pull from remote" @close="closeModal">
      <p class="pull-summary">
        Merges <code>origin/{{ pullRemoteLabel }}</code> into the currently checked-out branch.
        Checkout does not change.
      </p>
      <fieldset class="radio-list">
        <legend class="muted tiny">Remote branch</legend>
        <label class="radio-option">
          <input v-model="pullSource" type="radio" value="current" />
          origin/current-branch
        </label>
        <label class="radio-option">
          <input v-model="pullSource" type="radio" value="develop" />
          origin/develop
        </label>
        <label class="radio-option">
          <input v-model="pullSource" type="radio" value="master" />
          origin/master
        </label>
        <label class="radio-option">
          <input v-model="pullSource" type="radio" value="main" />
          origin/main
        </label>
        <label class="radio-option">
          <input v-model="pullSource" type="radio" value="specify" />
          Specify
        </label>
        <input
          v-if="pullSource === 'specify'"
          v-model="specifyBranch"
          type="text"
          placeholder="branch name"
          autofocus
          @keydown.enter="confirmPull"
        />
      </fieldset>
      <p class="muted tiny pull-hint">{{ pullHint }}</p>
      <template #actions>
        <button class="ghost" type="button" @click="closeModal">Cancel</button>
        <button class="primary" type="button" :disabled="!canConfirmPull" @click="confirmPull">
          Pull
        </button>
      </template>
    </Modal>

    <Modal v-if="modal === 'checkout'" title="Checkout" @close="closeModal">
      <fieldset class="radio-list">
        <legend class="muted tiny">Branch</legend>
        <label class="radio-option">
          <input v-model="checkoutSource" type="radio" value="develop" />
          develop
        </label>
        <label class="radio-option">
          <input v-model="checkoutSource" type="radio" value="specify" />
          Specify
        </label>
        <div v-if="checkoutSource === 'specify'" class="specify-panel">
          <label class="modal-label">
            <span class="muted tiny">Check out this branch</span>
            <input
              v-model="checkoutTarget"
              type="text"
              placeholder="feature/JIRA-123"
              autofocus
              @keydown.enter="checkoutAll"
            />
          </label>
          <div class="specify-fallbacks">
            <p class="muted tiny specify-fallbacks-title">Fallbacks</p>
            <div v-if="extraFallbacks.length" class="fallback-editor">
              <input
                v-for="(_fallback, index) in extraFallbacks"
                :key="index"
                v-model="extraFallbacks[index]"
                type="text"
                placeholder="branch name"
              />
            </div>
            <div class="fallback-editor-actions">
              <button class="ghost tiny" type="button" @click="addFallback">
                {{ extraFallbacks.length ? "Add another fallback" : "Add a fallback" }}
              </button>
              <button
                v-if="extraFallbacks.length"
                class="ghost tiny"
                type="button"
                @click="removeFallback"
              >
                Remove
              </button>
            </div>
            <p class="muted tiny specify-fallbacks-title">Final fallback</p>
            <div class="specify-last-fallback">
              <label class="radio-option">
                <input v-model="lastFallback" type="radio" value="develop" />
                develop
              </label>
              <label class="radio-option">
                <input v-model="lastFallback" type="radio" value="master" />
                master
              </label>
              <label class="radio-option">
                <input v-model="lastFallback" type="radio" value="main" />
                main
              </label>
            </div>
          </div>
        </div>
        <label class="radio-option">
          <input v-model="checkoutSource" type="radio" value="master" />
          master
        </label>
        <label class="radio-option">
          <input v-model="checkoutSource" type="radio" value="main" />
          main
        </label>
      </fieldset>
      <template #actions>
        <button class="ghost" type="button" @click="closeModal">Cancel</button>
        <button class="primary" type="button" :disabled="!canConfirmCheckout" @click="checkoutAll">
          Checkout
        </button>
      </template>
    </Modal>
  </section>
</template>
