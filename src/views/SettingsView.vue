<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import * as api from "../api";
import { useApp } from "../composables/useApp";
import { useTabs } from "../composables/useTabs";
import type { DiffMode, WindowState } from "../types";
import {
  DEFAULT_WINDOW_HEIGHT,
  DEFAULT_WINDOW_WIDTH,
  MIN_WINDOW_HEIGHT,
  MIN_WINDOW_WIDTH,
} from "../types";

const FILES_PANE_MIN = 220;
const FILES_PANE_MAX = 800;

const {
  refreshIntervalSeconds,
  filesPaneWidth,
  setFilesPaneWidth,
  saveFilesPaneWidth,
  diffMode,
  saveDiffMode,
  saveRefreshInterval,
  windowState,
  saveWindowState,
  showToast,
} = useApp();
const { openSettingsJson } = useTabs();

const windowDraft = ref<WindowState>({
  x: 0,
  y: 0,
  width: DEFAULT_WINDOW_WIDTH,
  height: DEFAULT_WINDOW_HEIGHT,
  maximized: false,
});

function assignWindow(next: WindowState) {
  windowDraft.value = {
    x: next.x,
    y: next.y,
    width: next.width,
    height: next.height,
    maximized: Boolean(next.maximized),
  };
}

async function loadWindow() {
  try {
    assignWindow(windowState.value ?? (await api.getWindowState()));
  } catch (err) {
    showToast(String(err), "error");
  }
}

async function persistWindow(next: WindowState) {
  try {
    assignWindow(await saveWindowState(next));
  } catch (err) {
    showToast(String(err), "error");
  }
}

function numberFromInput(event: Event, fallback: number) {
  const value = Number((event.target as HTMLInputElement).value);
  return Number.isFinite(value) ? value : fallback;
}

let stopLiveWindow: (() => void) | undefined;
let liveWindowTimer: ReturnType<typeof setTimeout> | null = null;

async function syncLiveWindow() {
  try {
    const win = getCurrentWindow();
    const maximized = await win.isMaximized();
    if (maximized) {
      if (!windowDraft.value.maximized) {
        assignWindow({ ...windowDraft.value, maximized: true });
      }
      return;
    }
    const scale = await win.scaleFactor();
    const size = (await win.innerSize()).toLogical(scale);
    const pos = (await win.outerPosition()).toLogical(scale);
    assignWindow({
      x: Math.round(pos.x),
      y: Math.round(pos.y),
      width: Math.max(MIN_WINDOW_WIDTH, Math.round(size.width)),
      height: Math.max(MIN_WINDOW_HEIGHT, Math.round(size.height)),
      maximized: false,
    });
  } catch {
    /* the window can close while a move is in flight */
  }
}

function queueLiveWindow() {
  if (liveWindowTimer !== null) {
    return;
  }
  liveWindowTimer = setTimeout(() => {
    liveWindowTimer = null;
    void syncLiveWindow();
  }, 50);
}

onMounted(() => {
  void loadWindow();
  const win = getCurrentWindow();
  void Promise.all([win.onMoved(queueLiveWindow), win.onResized(queueLiveWindow)]).then(
    (stoppers) => {
      stopLiveWindow = () => {
        for (const stop of stoppers) {
          stop();
        }
      };
    },
  );
});

onUnmounted(() => {
  stopLiveWindow?.();
  if (liveWindowTimer !== null) {
    clearTimeout(liveWindowTimer);
    liveWindowTimer = null;
  }
});

watch(windowState, (next) => {
  if (next) {
    assignWindow(next);
  }
});

async function onRefreshInterval(event: Event) {
  const value = Number((event.target as HTMLSelectElement).value);
  try {
    await saveRefreshInterval(value);
  } catch (err) {
    showToast(String(err), "error");
  }
}

async function onDiffMode(mode: DiffMode) {
  try {
    await saveDiffMode(mode);
  } catch (err) {
    showToast(String(err), "error");
  }
}

function onWidthInput(event: Event) {
  setFilesPaneWidth(Number((event.target as HTMLInputElement).value));
}

async function onWidthChange(event: Event) {
  try {
    await saveFilesPaneWidth(Number((event.target as HTMLInputElement).value));
  } catch (err) {
    showToast(String(err), "error");
  }
}

async function onWindowWidth(event: Event) {
  const width = Math.max(MIN_WINDOW_WIDTH, Math.round(numberFromInput(event, windowDraft.value.width)));
  await persistWindow({ ...windowDraft.value, width, maximized: false });
}

async function onWindowHeight(event: Event) {
  const height = Math.max(
    MIN_WINDOW_HEIGHT,
    Math.round(numberFromInput(event, windowDraft.value.height)),
  );
  await persistWindow({ ...windowDraft.value, height, maximized: false });
}

async function onWindowX(event: Event) {
  const x = Math.round(numberFromInput(event, windowDraft.value.x));
  await persistWindow({ ...windowDraft.value, x, maximized: false });
}

async function onWindowY(event: Event) {
  const y = Math.round(numberFromInput(event, windowDraft.value.y));
  await persistWindow({ ...windowDraft.value, y, maximized: false });
}

async function onMaximized(maximized: boolean) {
  await persistWindow({ ...windowDraft.value, maximized });
}

async function resetWindow() {
  await persistWindow({
    ...windowDraft.value,
    width: DEFAULT_WINDOW_WIDTH,
    height: DEFAULT_WINDOW_HEIGHT,
    maximized: false,
  });
}
</script>

<template>
  <div class="settings-page settings-form-page">
    <div class="settings-inner">
      <div class="settings-header">
        <div>
          <div class="brand">Settings</div>
          <p class="muted tiny">
            Preferences save as you change them. Open the JSON file to edit or back up everything,
            including repositories and groups.
          </p>
        </div>
        <button
          class="settings-json-open"
          type="button"
          title="Open settings.json"
          aria-label="Open settings.json"
          @click="openSettingsJson"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" />
            <path d="M14 2v4a2 2 0 0 0 2 2h4" />
            <path d="M10 12a1 1 0 0 0-1 1v1a1 1 0 0 1-1 1 1 1 0 0 1 1 1v1a1 1 0 0 0 1 1" />
            <path d="M14 18a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1 1 1 0 0 1-1-1v-1a1 1 0 0 0-1-1" />
          </svg>
        </button>
      </div>

      <section class="settings-card">
        <h2 class="settings-card-title">Preferences</h2>
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Auto-refresh</h3>
            <p class="muted tiny">How often Shipyard fetches remote status for every repository.</p>
          </div>
          <label class="settings-control">
            <span class="visually-hidden">Auto-refresh interval</span>
            <select :value="refreshIntervalSeconds" @change="onRefreshInterval">
              <option :value="0">Off</option>
              <option :value="60">1 minute</option>
              <option :value="300">5 minutes</option>
              <option :value="900">15 minutes</option>
              <option :value="1800">30 minutes</option>
            </select>
          </label>
        </div>
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Diff layout</h3>
            <p class="muted tiny">Default view when you open a file diff.</p>
          </div>
          <div class="segmented" role="group" aria-label="Diff layout">
            <button
              type="button"
              :class="{ active: diffMode === 'inline' }"
              :aria-pressed="diffMode === 'inline'"
              @click="onDiffMode('inline')"
            >
              Inline
            </button>
            <button
              type="button"
              :class="{ active: diffMode === 'split' }"
              :aria-pressed="diffMode === 'split'"
              @click="onDiffMode('split')"
            >
              Side by side
            </button>
          </div>
        </div>
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Files pane width</h3>
            <p class="muted tiny">Width of the working-tree list in a repository tab.</p>
          </div>
          <label class="settings-control settings-slider">
            <span class="visually-hidden">Files pane width</span>
            <input
              type="range"
              :min="FILES_PANE_MIN"
              :max="FILES_PANE_MAX"
              :value="filesPaneWidth"
              @input="onWidthInput"
              @change="onWidthChange"
            />
            <span class="settings-slider-value">{{ filesPaneWidth }}px</span>
          </label>
        </div>
      </section>

      <section class="settings-card">
        <h2 class="settings-card-title">Window</h2>
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Size</h3>
            <p class="muted tiny">
              Minimum {{ MIN_WINDOW_WIDTH }}×{{ MIN_WINDOW_HEIGHT }}. These numbers follow the window
              as you move or resize it.
            </p>
          </div>
          <div class="settings-fields">
            <label class="settings-field">
              <span>Width</span>
              <input
                type="number"
                :min="MIN_WINDOW_WIDTH"
                step="1"
                :value="windowDraft.width"
                @change="onWindowWidth"
              />
            </label>
            <label class="settings-field">
              <span>Height</span>
              <input
                type="number"
                :min="MIN_WINDOW_HEIGHT"
                step="1"
                :value="windowDraft.height"
                @change="onWindowHeight"
              />
            </label>
          </div>
        </div>
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Position</h3>
            <p class="muted tiny">Top-left of the window, in pixels.</p>
          </div>
          <div class="settings-fields">
            <label class="settings-field">
              <span>X</span>
              <input type="number" step="1" :value="windowDraft.x" @change="onWindowX" />
            </label>
            <label class="settings-field">
              <span>Y</span>
              <input type="number" step="1" :value="windowDraft.y" @change="onWindowY" />
            </label>
          </div>
        </div>
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Maximized</h3>
            <p class="muted tiny">Fill the current display. Size and position are kept for restore.</p>
          </div>
          <button
            class="history-switch"
            :class="{ on: windowDraft.maximized }"
            type="button"
            role="switch"
            :aria-checked="windowDraft.maximized"
            @click="onMaximized(!windowDraft.maximized)"
          >
            <span class="history-switch-track" aria-hidden="true">
              <span class="history-switch-knob" />
            </span>
            {{ windowDraft.maximized ? "On" : "Off" }}
          </button>
        </div>
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Reset size</h3>
            <p class="muted tiny">
              Restore the default {{ DEFAULT_WINDOW_WIDTH }}×{{ DEFAULT_WINDOW_HEIGHT }} window.
              Position is kept.
            </p>
          </div>
          <button class="ghost" type="button" @click="resetWindow">Reset</button>
        </div>
      </section>
    </div>
  </div>
</template>
