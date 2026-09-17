<script setup lang="ts">
import { computed, nextTick, ref, watch } from "vue";
import { buildFileTree, filterFileTree, searchExpandedPaths, treeHasFolders } from "../fileTree";
import type { RepoFile } from "../types";
import FileTreeNode from "./FileTreeNode.vue";

const props = defineProps<{
  files: RepoFile[];
  loading?: boolean;
  error?: string;
  parked?: boolean;
  selectedPath?: string;
}>();

const emit = defineEmits<{
  select: [path: string];
}>();

const searchOpen = ref(false);
const query = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const fileList = ref<HTMLElement | null>(null);
const expanded = ref<Set<string>>(new Set());
let savedScroll = 0;

const tree = computed(() => buildFileTree(props.files));
const visible = computed(() => filterFileTree(tree.value, query.value));
const searching = computed(() => Boolean(query.value.trim()));
const searchOpenFolders = computed(() => searchExpandedPaths(visible.value, query.value));
const hasFolders = computed(() => treeHasFolders(visible.value));
const canCollapse = computed(() => !searching.value && expanded.value.size > 0);
const openFolders = computed(() => (searching.value ? searchOpenFolders.value : expanded.value));

function rememberScroll() {
  savedScroll = fileList.value?.scrollTop ?? savedScroll;
}

async function restoreScroll() {
  await nextTick();
  requestAnimationFrame(() => {
    if (fileList.value) {
      fileList.value.scrollTop = savedScroll;
    }
  });
}

watch(
  () => props.parked,
  (hidden, wasHidden) => {
    if (hidden) {
      rememberScroll();
      return;
    }
    if (wasHidden) {
      void restoreScroll();
    }
  },
);

async function toggleSearch() {
  searchOpen.value = !searchOpen.value;
  if (!searchOpen.value) {
    query.value = "";
    return;
  }
  await nextTick();
  searchInput.value?.focus();
}

function collapseAll() {
  expanded.value = new Set();
}

async function clearSearch() {
  query.value = "";
  await nextTick();
  searchInput.value?.focus();
}

function toggleFolder(path: string) {
  if (searching.value) {
    return;
  }
  const next = new Set(expanded.value);
  if (next.has(path)) {
    next.delete(path);
  } else {
    next.add(path);
  }
  expanded.value = next;
}

function selectFile(path: string) {
  rememberScroll();
  emit("select", path);
}

function onSearchKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") {
    return;
  }
  if (query.value) {
    query.value = "";
    return;
  }
  searchOpen.value = false;
}
</script>

<template>
  <div class="file-pane">
    <section class="file-section">
      <div class="pane-header file-tree-header">
        <label v-if="searchOpen" class="file-tree-search">
          <span class="sr-only">Filter files</span>
          <input
            ref="searchInput"
            v-model="query"
            type="text"
            class="file-tree-query"
            placeholder="Filter files"
            @keydown="onSearchKeydown"
          />
          <button
            v-if="query"
            class="file-tree-clear"
            type="button"
            title="Clear search"
            @click="clearSearch"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M6 18 18 6M6 6l12 12" />
            </svg>
          </button>
        </label>
        <div v-else class="file-heading">
          <strong>File History</strong>
        </div>
        <div class="file-tree-actions">
          <button
            class="ghost tiny icon-action"
            :class="{ active: searchOpen }"
            type="button"
            :aria-pressed="searchOpen"
            :title="searchOpen ? 'Close search' : 'Search files'"
            @click="toggleSearch"
          >
            <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path
                d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"
              />
            </svg>
          </button>
          <button
            class="ghost tiny icon-action"
            type="button"
            :disabled="!hasFolders || !canCollapse"
            title="Collapse all folders"
            @click="collapseAll"
          >
            <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
              <path d="M4.5 18.75l7.5-7.5 7.5 7.5M4.5 12.75l7.5-7.5 7.5 7.5" />
            </svg>
          </button>
        </div>
      </div>
      <div ref="fileList" class="file-list">
        <p v-if="loading" class="muted tiny empty-files">Loading files…</p>
        <p v-else-if="error" class="muted tiny empty-files">{{ error }}</p>
        <p v-else-if="!files.length" class="muted tiny empty-files">No files in this repository.</p>
        <p v-else-if="!visible.length" class="muted tiny empty-files">No files match that search.</p>
        <template v-else>
          <FileTreeNode
            v-for="node in visible"
            :key="node.path"
            :node="node"
            :depth="0"
            :expanded="openFolders"
            :selected-path="selectedPath"
            @toggle="toggleFolder"
            @select="selectFile"
          />
        </template>
      </div>
    </section>
  </div>
</template>
