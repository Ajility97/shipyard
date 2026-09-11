<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import type { RepoActionResult } from "../types";

const props = defineProps<{
  title: string;
  results: RepoActionResult[];
}>();

const emit = defineEmits<{
  close: [];
}>();

const failed = computed(() => props.results.filter((item) => !item.ok).length);

function folderName(path: string) {
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    event.stopImmediatePropagation();
    emit("close");
  }
}

onMounted(() => {
  document.addEventListener("keydown", onKeydown, true);
});

onUnmounted(() => {
  document.removeEventListener("keydown", onKeydown, true);
});
</script>

<template>
  <Teleport to="body">
    <div class="output-layer" @click.self="emit('close')">
      <div class="output-modal" role="dialog" aria-modal="true" :aria-label="title">
        <div class="output-chrome">
          <div class="output-dots" aria-hidden="true">
            <span />
            <span />
            <span />
          </div>
          <div class="output-title">{{ title }}</div>
          <span class="output-count">{{ failed }} failed</span>
        </div>
        <div class="output-body">
          <div
            v-for="result in results"
            :key="result.path"
            class="output-line"
            :class="result.ok ? 'ok' : 'bad'"
          >
            <span class="output-name">{{ folderName(result.path) }}</span>
            <span class="output-sep"> — </span>
            <span class="output-message">{{ result.message }}</span>
          </div>
        </div>
        <div class="output-actions">
          <button class="ghost" type="button" @click="emit('close')">Close</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>
