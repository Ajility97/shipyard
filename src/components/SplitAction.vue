<script setup lang="ts">
defineProps<{
  disabled?: boolean;
  primaryTitle?: string;
  moreTitle?: string;
}>();

const emit = defineEmits<{
  primary: [];
  more: [];
}>();
</script>

<template>
  <div class="split-action">
    <button
      class="ghost tiny split-action-main"
      type="button"
      :disabled="disabled"
      :title="primaryTitle"
      @click.stop="emit('primary')"
    >
      <slot />
    </button>
    <button
      class="ghost tiny split-action-caret"
      type="button"
      :disabled="disabled"
      :title="moreTitle"
      aria-label="More options"
      @click.stop="emit('more')"
    >
      <svg viewBox="0 0 16 16" aria-hidden="true">
        <path d="M4.2 6.2 8 10l3.8-3.8" />
      </svg>
    </button>
  </div>
</template>
