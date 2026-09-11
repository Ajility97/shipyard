<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";

const lastRepoClick = ref<string | null>(null);
import { rangeIds } from "../selection";
import { useApp } from "../composables/useApp";
import { useTabs } from "../composables/useTabs";
import type { RepoEntry } from "../types";
import BranchIcon from "./BranchIcon.vue";
import FileIcon from "./FileIcon.vue";

const props = defineProps<{
  repo: RepoEntry;
  siblingIds: string[];
  flush?: boolean;
}>();

const emit = defineEmits<{
  remove: [repoId: string];
}>();

const { statuses, isRepoRefreshing } = useApp();
const { activeId, hasTab, openRepo, openRepos } = useTabs();
const menuOpen = ref(false);

function folderName(path: string) {
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function handleClick(event: MouseEvent) {
  if (event.shiftKey && lastRepoClick.value) {
    openRepos(rangeIds(props.siblingIds, lastRepoClick.value, props.repo.id), props.repo.id);
  } else {
    openRepo(props.repo.id);
  }
  lastRepoClick.value = props.repo.id;
}

function toggleMenu() {
  menuOpen.value = !menuOpen.value;
}

function closeMenu() {
  menuOpen.value = false;
}

function onRemove() {
  closeMenu();
  emit("remove", props.repo.id);
}

function onDocumentPointerDown(event: PointerEvent) {
  const target = event.target;
  if (target instanceof Element && target.closest(".overflow-menu")) {
    return;
  }
  closeMenu();
}

function onDocumentKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    closeMenu();
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
</script>

<template>
  <div
    class="repo-row"
    :class="{
      active: activeId === repo.id,
      open: hasTab(repo.id),
      refreshing: isRepoRefreshing(repo.id),
      flush,
    }"
    @click="handleClick"
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
        :aria-expanded="menuOpen"
        aria-haspopup="menu"
        title="Repository actions"
        @click.stop="toggleMenu"
      >
        ⋮
      </button>
      <div v-if="menuOpen" class="overflow-menu-dropdown" role="menu">
        <button
          class="overflow-menu-item danger"
          type="button"
          role="menuitem"
          @click.stop="onRemove"
        >
          Remove repository
        </button>
      </div>
    </div>
  </div>
</template>
