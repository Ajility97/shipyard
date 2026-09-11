<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import { rangeIds } from "../selection";
import { useApp } from "../composables/useApp";
import { useTabs } from "../composables/useTabs";
import type { RepoGroup } from "../types";
import BranchIcon from "./BranchIcon.vue";
import FileIcon from "./FileIcon.vue";
import Modal from "./Modal.vue";

const DEFAULT_HEADER = "#16323c";
const LAST_FALLBACKS = ["develop", "master", "main"] as const;
type LastFallback = (typeof LAST_FALLBACKS)[number];

function isLastFallback(value: string): value is LastFallback {
  return (LAST_FALLBACKS as readonly string[]).includes(value);
}

const props = defineProps<{ group: RepoGroup }>();
const {
  statuses,
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
  isRepoRefreshing,
  isGroupRefreshing,
  checkoutGroup,
  checkoutProgress,
  checkoutCancelled,
  cancelCheckout,
} = useApp();
const { activeId, hasTab, openRepo, openRepos, closeRepos } = useTabs();
const lastClickedId = ref<string | null>(null);
const menuRepoId = ref<string | null>(null);
const groupMenuOpen = ref(false);

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
  const last = cleaned.at(-1);
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
  menuRepoId.value = null;
  await removeRepo(props.group.id, repoId);
  if (hasTab(repoId)) {
    closeRepos([repoId]);
  }
}

function toggleRepoMenu(repoId: string) {
  groupMenuOpen.value = false;
  menuRepoId.value = menuRepoId.value === repoId ? null : repoId;
}

function toggleGroupMenu() {
  menuRepoId.value = null;
  groupMenuOpen.value = !groupMenuOpen.value;
}

function closeMenus() {
  menuRepoId.value = null;
  groupMenuOpen.value = false;
}

function onDocumentPointerDown(event: PointerEvent) {
  const target = event.target;
  if (target instanceof Element && target.closest(".overflow-menu")) {
    return;
  }
  closeMenus();
}

function onDocumentKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    closeMenus();
  }
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocumentPointerDown);
  document.addEventListener("keydown", onDocumentKeydown);
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocumentPointerDown);
  document.removeEventListener("keydown", onDocumentKeydown);
});

function handleRepoClick(event: MouseEvent, repoId: string) {
  if (event.shiftKey && lastClickedId.value) {
    openRepos(rangeIds(siblingIds.value, lastClickedId.value, repoId), repoId);
  } else {
    openRepo(repoId);
  }
  lastClickedId.value = repoId;
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
            ⋮
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

    <div v-if="group.expanded" class="group-body">
      <div
        v-for="repo in group.repos"
        :key="repo.id"
        class="repo-row"
        :class="{ active: activeId === repo.id, open: hasTab(repo.id), refreshing: isRepoRefreshing(repo.id) }"
        @click="handleRepoClick($event, repo.id)"
      >
        <span class="repo-name">{{ statuses[repo.id]?.name ?? folderName(repo.path) }}</span>
        <span class="branch">
          <span v-if="isRepoRefreshing(repo.id)" class="spinner" aria-label="Refreshing repository" />
          <span class="branch-name">
            <BranchIcon />
            {{ statuses[repo.id]?.branch ?? "…" }}
          </span>
          <span
            v-if="(statuses[repo.id]?.behind ?? 0) > 0 || (statuses[repo.id]?.ahead ?? 0) > 0"
            class="sync-counts"
          >
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
          </span>
          <span
            v-if="(statuses[repo.id]?.changedFiles ?? 0) > 0"
            class="diff-stat"
            :title="`${statuses[repo.id]?.changedFiles} uncommitted files`"
          >
            <span class="file-count">
              <FileIcon />
              {{ statuses[repo.id]?.changedFiles }}
            </span>
            <span class="line-changes">
              <span v-if="(statuses[repo.id]?.insertions ?? 0) > 0" class="line-ins">
                +{{ statuses[repo.id]?.insertions }}
              </span>
              <span v-if="(statuses[repo.id]?.deletions ?? 0) > 0" class="line-del">
                −{{ statuses[repo.id]?.deletions }}
              </span>
            </span>
          </span>
        </span>
        <div class="overflow-menu repo-menu" @click.stop>
          <button
            class="ghost tiny overflow-menu-trigger"
            type="button"
            :aria-expanded="menuRepoId === repo.id"
            aria-haspopup="menu"
            title="Repository actions"
            @click.stop="toggleRepoMenu(repo.id)"
          >
            ⋮
          </button>
          <div v-if="menuRepoId === repo.id" class="overflow-menu-dropdown" role="menu">
            <button
              class="overflow-menu-item danger"
              type="button"
              role="menuitem"
              @click.stop="removeAndLeave(repo.id)"
            >
              Remove repository
            </button>
          </div>
        </div>
      </div>

      <p v-if="actionLabel" class="muted tiny" style="padding: 0.35rem 0.9rem">{{ actionLabel }}</p>
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
