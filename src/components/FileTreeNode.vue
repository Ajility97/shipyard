<script setup lang="ts">
import type { FileEntry } from "../fileTree";

const props = defineProps<{
  node: FileEntry;
  depth: number;
  expanded: Set<string>;
  selectedPath?: string;
}>();

const emit = defineEmits<{
  toggle: [path: string];
  select: [path: string];
}>();

function isOpen(node: FileEntry) {
  return props.expanded.has(node.path);
}
</script>

<template>
  <div class="file-tree-node">
    <button
      v-if="node.kind === 'folder'"
      class="file-item file-tree-row"
      :class="{ ignored: node.ignored }"
      type="button"
      :style="{ paddingLeft: `${0.55 + depth * 0.85}rem` }"
      :aria-expanded="isOpen(node)"
      :title="node.path"
      @click="emit('toggle', node.path)"
    >
      <svg
        class="file-tree-twist"
        :class="{ open: isOpen(node) }"
        viewBox="0 0 24 24"
        aria-hidden="true"
      >
        <path d="M9 6.75l6.75 5.25L9 17.25" />
      </svg>
      <svg class="file-tree-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path
          v-if="isOpen(node)"
          d="M3.75 9.776c.112-.017.227-.026.344-.026h15.812c.117 0 .232.009.344.026m-16.5 0a2.25 2.25 0 00-1.883 2.542l.857 6a2.25 2.25 0 002.227 1.932H19.05a2.25 2.25 0 002.227-1.932l.857-6a2.25 2.25 0 00-1.883-2.542m-16.5 0V6A2.25 2.25 0 016 3.75h3.879a1.5 1.5 0 011.06.44l2.122 2.12a1.5 1.5 0 001.06.44H18A2.25 2.25 0 0120.25 9v.776"
        />
        <path
          v-else
          d="M2.25 12.75V12A2.25 2.25 0 014.5 9.75h15A2.25 2.25 0 0121.75 12v.75m-8.69-6.44l-2.12-2.12a1.5 1.5 0 00-1.061-.44H4.5A2.25 2.25 0 002.25 6v12a2.25 2.25 0 002.25 2.25h15A2.25 2.25 0 0021.75 18V9a2.25 2.25 0 00-2.25-2.25h-5.379a1.5 1.5 0 01-1.06-.44z"
        />
      </svg>
      <span class="file-tree-name">{{ node.name }}</span>
    </button>
    <button
      v-else
      class="file-item file-tree-row"
      :class="{ ignored: node.ignored, active: selectedPath === node.path }"
      type="button"
      :style="{ paddingLeft: `${0.55 + depth * 0.85}rem` }"
      :title="node.path"
      @click="emit('select', node.path)"
    >
      <span class="file-tree-twist" aria-hidden="true" />
      <svg class="file-tree-icon" viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z"
        />
      </svg>
      <span class="file-tree-name">{{ node.name }}</span>
    </button>
    <template v-if="node.kind === 'folder' && isOpen(node)">
      <FileTreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :depth="depth + 1"
        :expanded="expanded"
        :selected-path="selectedPath"
        @toggle="emit('toggle', $event)"
        @select="emit('select', $event)"
      />
    </template>
  </div>
</template>
