<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{
  status: string;
}>();

const kind = computed(() => {
  const status = props.status.trim().toLowerCase();
  if (status === "modified" || status === "m") {
    return "modified";
  }
  if (status === "untracked" || status === "?") {
    return "untracked";
  }
  if (status === "added" || status === "a") {
    return "added";
  }
  if (status === "deleted" || status === "d") {
    return "deleted";
  }
  if (status === "renamed" || status === "r") {
    return "renamed";
  }
  if (status === "copied" || status === "c") {
    return "copied";
  }
  if (status === "conflicted" || status === "u") {
    return "conflicted";
  }
  return "unknown";
});
</script>

<template>
  <span class="file-status-icon" :class="kind" :title="status">
    <svg v-if="kind === 'modified'" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125"
      />
    </svg>
    <svg v-else-if="kind === 'untracked'" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 9v6m3-3H9m12 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
    </svg>
    <svg v-else-if="kind === 'added'" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 4.5v15m7.5-7.5h-15" />
    </svg>
    <svg v-else-if="kind === 'deleted'" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M5 12h14" />
    </svg>
    <svg v-else-if="kind === 'renamed'" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M7.5 21 3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5"
      />
    </svg>
    <svg v-else-if="kind === 'copied'" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M8.25 7.5V6.108c0-1.135.845-2.098 1.976-2.192a48.424 48.424 0 0 1 3.548 0c1.13.094 1.976 1.057 1.976 2.192V7.5M6.75 7.5h10.5a.75.75 0 0 1 .75.75v9.75a.75.75 0 0 1-.75.75H6.75a.75.75 0 0 1-.75-.75V8.25a.75.75 0 0 1 .75-.75Z"
      />
    </svg>
    <svg v-else-if="kind === 'conflicted'" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"
      />
    </svg>
    <svg v-else viewBox="0 0 24 24" aria-hidden="true">
      <path d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
    </svg>
  </span>
</template>
