<script setup lang="ts">
import { GROUPS_TAB_ID, useTabs } from "../composables/useTabs";

const { tabs, activeId, activate, closeRepo } = useTabs();
</script>

<template>
  <div class="tab-bar" role="tablist" aria-label="Open views">
    <div
      v-for="tab in tabs"
      :key="tab.id"
      class="tab"
      :class="{ active: activeId === tab.id }"
      role="tab"
      :aria-selected="activeId === tab.id"
      @click="activate(tab.id)"
    >
      <span class="tab-title">{{ tab.title }}</span>
      <button
        v-if="tab.closable"
        class="tab-close"
        type="button"
        :aria-label="`Close ${tab.title}`"
        @click.stop="closeRepo(tab.id)"
      >
        ×
      </button>
      <span v-else-if="tab.id === GROUPS_TAB_ID" class="tab-lock" aria-hidden="true" />
    </div>
  </div>
</template>
