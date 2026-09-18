<script setup lang="ts">
import { useApp } from "../../composables/useApp";
import { REFRESH_HOURS_PRESETS } from "../../refreshHours";
import type { RefreshHoursPreset } from "../../types";

const { refreshActiveHours, saveRefreshActiveHours, showToast } = useApp();

async function persistHours(next: typeof refreshActiveHours.value) {
  try {
    await saveRefreshActiveHours(next);
  } catch (err) {
    showToast(String(err), "error");
  }
}

function onActiveHoursToggle(enabled: boolean) {
  void persistHours({ ...refreshActiveHours.value, enabled });
}

function onHoursPreset(event: Event) {
  const preset = (event.target as HTMLSelectElement).value as RefreshHoursPreset;
  const times =
    preset === "custom"
      ? { start: refreshActiveHours.value.start, end: refreshActiveHours.value.end }
      : REFRESH_HOURS_PRESETS[preset];
  void persistHours({
    ...refreshActiveHours.value,
    enabled: true,
    preset,
    ...times,
  });
}

function onHoursTime(which: "start" | "end", event: Event) {
  const value = (event.target as HTMLInputElement).value;
  if (!value) {
    return;
  }
  void persistHours({
    ...refreshActiveHours.value,
    enabled: true,
    preset: "custom",
    [which]: value,
  });
}
</script>

<template>
  <div class="settings-pane settings-form-page">
    <div class="settings-inner">
      <div class="settings-header">
        <div>
          <div class="brand">Schedule</div>
          <p class="muted tiny">When auto-fetch is allowed to contact remotes.</p>
        </div>
      </div>

      <section class="settings-card">
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Active hours</h3>
            <p class="muted tiny">
              Pause automatic fetches outside this window. Manual fetch still works.
            </p>
          </div>
          <button
            class="history-switch"
            :class="{ on: refreshActiveHours.enabled }"
            type="button"
            role="switch"
            :aria-checked="refreshActiveHours.enabled"
            @click="onActiveHoursToggle(!refreshActiveHours.enabled)"
          >
            <span class="history-switch-track" aria-hidden="true">
              <span class="history-switch-knob" />
            </span>
            {{ refreshActiveHours.enabled ? "On" : "Off" }}
          </button>
        </div>
        <div v-if="refreshActiveHours.enabled" class="settings-row">
          <div class="settings-row-copy">
            <h3>Preset</h3>
            <p class="muted tiny">Business is 8am–6pm. Personal is 6am–11pm. Local time.</p>
          </div>
          <label class="settings-control">
            <span class="visually-hidden">Active hours preset</span>
            <select :value="refreshActiveHours.preset" @change="onHoursPreset">
              <option value="business">{{ REFRESH_HOURS_PRESETS.business.label }}</option>
              <option value="personal">{{ REFRESH_HOURS_PRESETS.personal.label }}</option>
              <option value="custom">Custom</option>
            </select>
          </label>
        </div>
        <div v-if="refreshActiveHours.enabled" class="settings-row">
          <div class="settings-row-copy">
            <h3>Hours</h3>
            <p class="muted tiny">Changing these times switches the preset to Custom.</p>
          </div>
          <div class="settings-fields">
            <label class="settings-field">
              <span>From</span>
              <input
                type="time"
                step="60"
                :value="refreshActiveHours.start"
                @change="onHoursTime('start', $event)"
              />
            </label>
            <label class="settings-field">
              <span>To</span>
              <input
                type="time"
                step="60"
                :value="refreshActiveHours.end"
                @change="onHoursTime('end', $event)"
              />
            </label>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
