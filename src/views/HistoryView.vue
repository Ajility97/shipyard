<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { confirm } from "@tauri-apps/plugin-dialog";
import * as api from "../api";
import type { CommandLogEntry } from "../types";

const entries = ref<CommandLogEntry[]>([]);
const message = ref("");
const clearing = ref(false);
const showDetail = ref(false);
const terminal = ref<HTMLDivElement | null>(null);
let stopLog: (() => void) | undefined;
let stopCleared: (() => void) | undefined;
let stickToBottom = true;

function folderName(path: string) {
  const parts = path.split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function formatStamp(at: number) {
  return new Date(at).toLocaleString(undefined, { hour12: false });
}

function onScroll() {
  const node = terminal.value;
  if (!node) {
    return;
  }
  stickToBottom = node.scrollHeight - node.scrollTop - node.clientHeight < 48;
}

async function scrollIfNeeded() {
  if (!stickToBottom) {
    return;
  }
  await nextTick();
  const node = terminal.value;
  if (node) {
    node.scrollTop = node.scrollHeight;
  }
}

onMounted(async () => {
  try {
    entries.value = await api.commandHistory();
    await scrollIfNeeded();
  } catch (err) {
    message.value = String(err);
  }
  void listen<CommandLogEntry>("command-log", (event) => {
    if (entries.value.some((entry) => entry.id === event.payload.id)) {
      return;
    }
    entries.value = [...entries.value, event.payload];
    void scrollIfNeeded();
  }).then((unlisten) => {
    stopLog = unlisten;
  });
  void listen("command-log-cleared", () => {
    entries.value = [];
  }).then((unlisten) => {
    stopCleared = unlisten;
  });
});

onUnmounted(() => {
  stopLog?.();
  stopCleared?.();
});

async function clearLogs() {
  if (!entries.value.length || clearing.value) {
    return;
  }
  const ok = await confirm("Clear the local git command history? This cannot be undone.", {
    title: "Clear history",
    kind: "warning",
    okLabel: "Clear",
    cancelLabel: "Cancel",
  });
  if (!ok) {
    return;
  }
  clearing.value = true;
  message.value = "";
  try {
    await api.clearCommandHistory();
    entries.value = [];
  } catch (err) {
    message.value = String(err);
  } finally {
    clearing.value = false;
  }
}
</script>

<template>
  <div class="settings-page history-page">
    <div class="settings-inner">
      <div class="settings-header">
        <div>
          <div class="brand">History</div>
          <p class="muted tiny">
            Every git command Krakdown runs is recorded here, including refresh, pull, push, and
            commit.
          </p>
        </div>
        <div class="history-header-actions">
          <div class="segmented" role="group" aria-label="History detail">
            <button
              type="button"
              :class="{ active: !showDetail }"
              :aria-pressed="!showDetail"
              @click="showDetail = false"
            >
              Commands
            </button>
            <button
              type="button"
              :class="{ active: showDetail }"
              :aria-pressed="showDetail"
              @click="showDetail = true"
            >
              Full detail
            </button>
          </div>
          <button
            class="ghost danger"
            type="button"
            :disabled="clearing || !entries.length"
            @click="clearLogs"
          >
            {{ clearing ? "Clearing…" : "Clear logs" }}
          </button>
        </div>
      </div>
      <div ref="terminal" class="history-terminal" @scroll="onScroll">
        <p v-if="!entries.length" class="muted tiny history-empty">
          No commands yet. Refresh a group or open a repo to see each git invocation.
        </p>
        <article
          v-for="entry in entries"
          :key="entry.id"
          class="history-entry"
          :class="{ bad: !entry.success, compact: !showDetail }"
        >
          <header class="history-meta">
            <span class="history-time">{{ formatStamp(entry.at) }}</span>
            <span class="history-cwd" :title="entry.cwd">{{ folderName(entry.cwd) }}</span>
            <span class="history-status">
              {{ entry.success ? "exit 0" : "failed" }} · {{ entry.durationMs }}ms
            </span>
          </header>
          <pre class="history-command" :title="entry.program">$ {{ entry.command }}</pre>
          <template v-if="showDetail">
            <pre v-if="entry.stdout.trim()" class="history-output">{{ entry.stdout }}</pre>
            <pre v-if="entry.stderr.trim()" class="history-output err">{{ entry.stderr }}</pre>
          </template>
        </article>
      </div>
      <p v-if="message" class="settings-error">{{ message }}</p>
    </div>
  </div>
</template>
