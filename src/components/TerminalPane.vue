<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import { listen } from "@tauri-apps/api/event";
import { FitAddon } from "@xterm/addon-fit";
import { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";
import * as api from "../api";

const TERMINAL_FONT = "JetBrains Mono, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";

const props = defineProps<{
  cwd: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const host = ref<HTMLDivElement | null>(null);
const message = ref("");

let term: Terminal | null = null;
let fit: FitAddon | null = null;
let sessionId = "";
let stopData: (() => void) | undefined;
let stopExit: (() => void) | undefined;
let resizeObserver: ResizeObserver | null = null;
let starting = false;

function disposeListeners() {
  stopData?.();
  stopExit?.();
  stopData = undefined;
  stopExit = undefined;
}

async function closeSession() {
  const id = sessionId;
  sessionId = "";
  disposeListeners();
  if (id) {
    try {
      await api.closeTerminal(id);
    } catch {
      /* session may already be gone */
    }
  }
}

function writeMessage(text: string) {
  term?.writeln(`\r\n${text}`);
}

async function startSession() {
  if (!term || !fit || starting) {
    return;
  }
  starting = true;
  message.value = "";
  await closeSession();
  try {
    fit.fit();
    const id = await api.openTerminal(props.cwd, term.cols, term.rows);
    sessionId = id;
    stopData = await listen<{ id: string; data: string }>("terminal-data", (event) => {
      if (event.payload.id === sessionId) {
        term?.write(event.payload.data);
      }
    });
    stopExit = await listen<{ id: string }>("terminal-exit", (event) => {
      if (event.payload.id === sessionId) {
        sessionId = "";
        writeMessage("[Process exited]");
      }
    });
    term.focus();
  } catch (err) {
    message.value = String(err);
    writeMessage(String(err));
  } finally {
    starting = false;
  }
}

function syncSize() {
  if (!term || !fit || !sessionId) {
    fit?.fit();
    return;
  }
  fit.fit();
  void api.resizeTerminal(sessionId, term.cols, term.rows).catch(() => {
    /* closed while resizing */
  });
}

onMounted(async () => {
  const node = host.value;
  if (!node) {
    return;
  }
  try {
    await document.fonts.load(`12.5px "JetBrains Mono"`);
    await document.fonts.ready;
  } catch {
    /* use the fallback stack if the face is not available */
  }
  term = new Terminal({
    cursorBlink: true,
    fontFamily: TERMINAL_FONT,
    fontSize: 12.5,
    fontWeight: "400",
    fontWeightBold: "700",
    lineHeight: 1.35,
    scrollback: 5000,
    theme: {
      background: "#0d1016",
      foreground: "#e8edf5",
      cursor: "#118777",
      cursorAccent: "#0d1016",
      selectionBackground: "#2a3344",
      black: "#12151b",
      red: "#f07178",
      green: "#3dd68c",
      yellow: "#e6c07b",
      blue: "#7aa2f7",
      magenta: "#c678dd",
      cyan: "#56b6c2",
      white: "#e8edf5",
      brightBlack: "#8b95a8",
      brightRed: "#f07178",
      brightGreen: "#3dd68c",
      brightYellow: "#e6c07b",
      brightBlue: "#7aa2f7",
      brightMagenta: "#c678dd",
      brightCyan: "#56b6c2",
      brightWhite: "#ffffff",
    },
    macOptionIsMeta: true,
  });
  fit = new FitAddon();
  term.loadAddon(fit);
  term.open(node);
  term.onData((data) => {
    if (!sessionId) {
      return;
    }
    void api.writeTerminal(sessionId, data).catch(() => {
      /* closed while typing */
    });
  });
  resizeObserver = new ResizeObserver(() => {
    syncSize();
  });
  resizeObserver.observe(node);
  await nextTick();
  await new Promise<void>((resolve) => {
    if (node.clientWidth > 0 && node.clientHeight > 0) {
      resolve();
      return;
    }
    const ready = new ResizeObserver(() => {
      if (node.clientWidth > 0 && node.clientHeight > 0) {
        ready.disconnect();
        resolve();
      }
    });
    ready.observe(node);
    window.setTimeout(() => {
      ready.disconnect();
      resolve();
    }, 250);
  });
  await startSession();
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
  void closeSession();
  term?.dispose();
  term = null;
  fit = null;
});

watch(
  () => props.cwd,
  () => {
    if (term) {
      void startSession();
    }
  },
);
</script>

<template>
  <section class="repo-terminal">
    <div class="repo-terminal-bar">
      <span class="repo-terminal-path" :title="cwd">{{ cwd }}</span>
      <button class="ghost tiny" type="button" title="Close terminal" @click="emit('close')">
        Close
      </button>
    </div>
    <div ref="host" class="repo-terminal-host" />
    <p v-if="message" class="repo-terminal-error">{{ message }}</p>
  </section>
</template>
