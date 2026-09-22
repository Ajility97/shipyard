<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import type { CommitNode } from "../types";

const props = defineProps<{
  commit: CommitNode;
  hashes: string[];
  x: number;
  y: number;
  busy: boolean;
  operation: string;
  hasMerge: boolean;
}>();

const emit = defineEmits<{
  checkout: [];
  createBranch: [];
  cherryPick: [];
  revert: [];
  copySha: [];
  copyLink: [];
  createTag: [];
  close: [];
}>();

const menuEl = ref<HTMLElement | null>(null);
const left = ref(props.x);
const top = ref(props.y);

const count = computed(() => props.hashes.length);
const multi = computed(() => count.value > 1);
const blocked = computed(() => props.busy || Boolean(props.operation));
const blockedTitle = computed(() => {
  if (props.busy) {
    return "Wait for the current action to finish.";
  }
  if (props.operation) {
    return "Finish or abort the current operation first.";
  }
  return undefined;
});
const mergeTitle = computed(() =>
  props.hasMerge
    ? count.value > 1
      ? "Cherry-pick and revert skip merge commits."
      : "This is a merge commit."
    : undefined,
);

async function placeMenu() {
  await nextTick();
  const el = menuEl.value;
  if (!el) {
    return;
  }
  const rect = el.getBoundingClientRect();
  const pad = 8;
  let nextLeft = props.x;
  let nextTop = props.y;
  if (nextLeft + rect.width > window.innerWidth - pad) {
    nextLeft = window.innerWidth - rect.width - pad;
  }
  if (nextTop + rect.height > window.innerHeight - pad) {
    nextTop = window.innerHeight - rect.height - pad;
  }
  left.value = Math.max(pad, nextLeft);
  top.value = Math.max(pad, nextTop);
}

function onDocumentPointerDown(event: PointerEvent) {
  const target = event.target;
  if (target instanceof Element && target.closest(".commit-context-menu")) {
    return;
  }
  emit("close");
}

function onDocumentKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    emit("close");
  }
}

function onContextMenu(event: MouseEvent) {
  event.preventDefault();
}

onMounted(() => {
  document.addEventListener("pointerdown", onDocumentPointerDown);
  document.addEventListener("keydown", onDocumentKeydown);
  void placeMenu();
});

onUnmounted(() => {
  document.removeEventListener("pointerdown", onDocumentPointerDown);
  document.removeEventListener("keydown", onDocumentKeydown);
});

watch(
  () => [props.x, props.y, props.commit.hash, props.hashes.join(",")],
  () => {
    left.value = props.x;
    top.value = props.y;
    void placeMenu();
  },
);
</script>

<template>
  <Teleport to="body">
    <div
      ref="menuEl"
      class="file-context-menu commit-context-menu"
      role="menu"
      :style="{ left: `${left}px`, top: `${top}px` }"
      @contextmenu="onContextMenu"
    >
      <button
        class="context-menu-item"
        type="button"
        role="menuitem"
        :disabled="blocked || multi"
        :title="multi ? 'Select one commit to check out.' : blockedTitle"
        @click="emit('checkout')"
      >
        Checkout this commit
      </button>
      <button
        class="context-menu-item"
        type="button"
        role="menuitem"
        :disabled="blocked || multi"
        :title="multi ? 'Select one commit to branch from.' : blockedTitle"
        @click="emit('createBranch')"
      >
        Create branch here
      </button>
      <button
        class="context-menu-item"
        type="button"
        role="menuitem"
        :disabled="blocked || hasMerge"
        :title="mergeTitle ?? blockedTitle"
        @click="emit('cherryPick')"
      >
        {{ multi ? `Cherry-pick ${count} commits` : "Cherry-pick commit" }}
      </button>
      <button
        class="context-menu-item"
        type="button"
        role="menuitem"
        :disabled="blocked || hasMerge"
        :title="mergeTitle ?? blockedTitle"
        @click="emit('revert')"
      >
        {{ multi ? `Revert ${count} commits` : "Revert commit" }}
      </button>
      <div class="context-menu-sep" />
      <button
        class="context-menu-item"
        type="button"
        role="menuitem"
        @click="emit('copySha')"
      >
        {{ multi ? "Copy commit SHAs" : "Copy commit SHA" }}
      </button>
      <button
        class="context-menu-item"
        type="button"
        role="menuitem"
        :disabled="multi"
        :title="multi ? 'Select one commit to copy its remote link.' : undefined"
        @click="emit('copyLink')"
      >
        Copy link to this commit
      </button>
      <div class="context-menu-sep" />
      <button
        class="context-menu-item"
        type="button"
        role="menuitem"
        :disabled="blocked || multi"
        :title="multi ? 'Select one commit to tag.' : blockedTitle"
        @click="emit('createTag')"
      >
        Create tag here
      </button>
    </div>
  </Teleport>
</template>
