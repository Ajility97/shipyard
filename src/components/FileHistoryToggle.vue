<script setup lang="ts">
defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  click: [];
}>();
</script>

<template>
  <button
    class="ghost tiny"
    :class="{ active: open }"
    type="button"
    :aria-pressed="open"
    :title="open ? 'Hide file history' : 'Show file history'"
    @click="emit('click')"
  >
    <svg class="button-icon" viewBox="0 0 24 24" aria-hidden="true">
      <path
        d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
      />
    </svg>
    File History
  </button>
</template>
