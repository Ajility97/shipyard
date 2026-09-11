<script setup lang="ts">
import { computed } from "vue";
import { parseDiff, toSplitRows } from "../diff";

const props = defineProps<{
  raw: string;
  mode: "inline" | "split";
}>();

const lines = computed(() => parseDiff(props.raw));
const splitRows = computed(() => toSplitRows(lines.value));
</script>

<template>
  <div v-if="!raw" class="muted" style="padding: 0.85rem">No diff available.</div>
  <div v-else-if="mode === 'inline'">
    <div
      v-for="(line, index) in lines"
      :key="index"
      class="diff-line"
      :class="{
        'diff-add': line.kind === 'add',
        'diff-del': line.kind === 'del',
        'diff-hunk': line.kind === 'hunk' || line.kind === 'meta',
      }"
    >
      <span class="diff-gutter">{{ line.kind === "add" ? "" : line.oldNo ?? "" }}</span>
      <span class="diff-gutter">{{ line.kind === "del" ? "" : line.newNo ?? "" }}</span>
      <span class="diff-code">{{
        line.kind === "add" ? `+${line.text}` : line.kind === "del" ? `-${line.text}` : line.text
      }}</span>
    </div>
  </div>
  <div v-else>
    <div
      v-for="(row, index) in splitRows"
      :key="index"
      class="diff-line side"
      :class="{ 'diff-hunk': row.leftKind === 'hunk' || row.leftKind === 'meta' }"
    >
      <span class="diff-gutter">{{ row.leftNo ?? "" }}</span>
      <span class="diff-code" :class="{ 'diff-del': row.leftKind === 'del' }">{{ row.leftText }}</span>
      <span class="diff-gutter">{{ row.rightNo ?? "" }}</span>
      <span class="diff-code" :class="{ 'diff-add': row.rightKind === 'add' }">{{ row.rightText }}</span>
    </div>
  </div>
</template>
