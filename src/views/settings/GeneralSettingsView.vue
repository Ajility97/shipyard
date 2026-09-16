<script setup lang="ts">
import { useApp } from "../../composables/useApp";
import type { DiffMode } from "../../types";

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
  showToast,
} = useApp();

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
</script>

<template>
  <div class="settings-pane settings-form-page">
    <div class="settings-inner">
      <div class="settings-header">
        <div>
          <div class="brand">General</div>
          <p class="muted tiny">Preferences save as you change them.</p>
        </div>
      </div>

      <section class="settings-card">
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
    </div>
  </div>
</template>
