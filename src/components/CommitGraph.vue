<script setup lang="ts">
import { computed } from "vue";
import type { CommitNode } from "../types";
import {
  GRAPH_COLORS,
  GRAPH_ROW_HEIGHT,
  formatCommitDate,
  laneX,
  layoutGraph,
  parseRefs,
  pipePath,
} from "../graphLayout";

const props = defineProps<{ commits: CommitNode[] }>();

const layout = computed(() => layoutGraph(props.commits));
const midY = GRAPH_ROW_HEIGHT / 2;

function color(column: number) {
  return GRAPH_COLORS[column % GRAPH_COLORS.length];
}
</script>

<template>
  <div v-if="!commits.length" class="muted" style="padding: 1rem">No commits yet.</div>
  <div v-else class="commit-graph">
    <div
      v-for="row in layout.rows"
      :key="row.commit.hash"
      class="commit-graph-row"
    >
      <svg
        class="commit-graph-svg"
        :width="layout.width"
        :height="GRAPH_ROW_HEIGHT"
        :viewBox="`0 0 ${layout.width} ${GRAPH_ROW_HEIGHT}`"
      >
        <path
          v-for="(link, index) in row.through"
          :key="`t-${index}`"
          :d="pipePath(link.from, 0, link.to, GRAPH_ROW_HEIGHT)"
          fill="none"
          :stroke="color(link.to)"
          stroke-width="1.6"
        />
        <path
          v-for="(link, index) in row.incoming"
          :key="`i-${index}`"
          :d="pipePath(link.from, 0, link.to, midY)"
          fill="none"
          :stroke="color(link.to)"
          stroke-width="1.6"
        />
        <path
          v-for="(link, index) in row.outgoing"
          :key="`o-${index}`"
          :d="pipePath(link.from, midY, link.to, GRAPH_ROW_HEIGHT)"
          fill="none"
          :stroke="color(link.to)"
          stroke-width="1.6"
        />
        <circle
          :cx="laneX(row.column)"
          :cy="midY"
          r="4.5"
          :fill="color(row.column)"
        />
      </svg>
      <div class="commit-subject">
        <span
          v-for="chip in parseRefs(row.commit.refs)"
          :key="`${chip.kind}-${chip.name}`"
          class="ref-chip"
          :class="chip.kind"
        >{{ chip.name }}</span>
        {{ row.commit.subject }}
      </div>
      <div class="commit-author">{{ row.commit.author }}</div>
      <div class="commit-date">{{ formatCommitDate(row.commit.date) }}</div>
    </div>
  </div>
</template>
