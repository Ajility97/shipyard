<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  path: string;
}>();

const parts = computed(() => {
  const normalized = props.path.replace(/\\/g, "/");
  const slash = normalized.lastIndexOf("/");
  if (slash < 0) {
    return { dir: "", name: props.path };
  }
  return {
    dir: normalized.slice(0, slash),
    name: normalized.slice(slash + 1) || props.path,
  };
});
</script>

<template>
  <span class="path-label" :title="path">
    <span v-if="parts.dir" class="path-dir">{{ parts.dir }}</span>
    <span v-if="parts.dir" class="path-sep">/</span>
    <span class="path-file">{{ parts.name }}</span>
  </span>
</template>
