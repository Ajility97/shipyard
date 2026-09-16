<script setup lang="ts">
import { onMounted } from "vue";
import { useUpdater } from "../../composables/useUpdater";

const {
  status,
  statusText,
  currentVersion,
  availableVersion,
  busy,
  ensureCurrentVersion,
  checkForUpdates,
  installUpdate,
} = useUpdater();

onMounted(() => {
  void ensureCurrentVersion();
});

async function onCheckForUpdates() {
  await checkForUpdates({ prompt: false });
}
</script>

<template>
  <div class="settings-pane settings-form-page">
    <div class="settings-inner">
      <div class="settings-header">
        <div>
          <div class="brand">Updates</div>
          <p class="muted tiny">
            Shipyard checks GitHub on launch. Newer builds install in place and restart.
          </p>
        </div>
      </div>

      <section class="settings-card">
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Current version</h3>
            <p class="muted tiny">The build running on this Mac.</p>
          </div>
          <span class="muted tiny">{{ currentVersion ? `v${currentVersion}` : "…" }}</span>
        </div>
        <div class="settings-row">
          <div class="settings-row-copy">
            <h3>Check for updates</h3>
            <p class="muted tiny">Compare this build to the latest GitHub release.</p>
          </div>
          <div class="settings-update-actions">
            <button
              v-if="status === 'available'"
              class="ghost"
              type="button"
              :disabled="busy"
              @click="installUpdate"
            >
              Install v{{ availableVersion }} and restart
            </button>
            <button class="ghost" type="button" :disabled="busy" @click="onCheckForUpdates">
              {{ busy && status === "checking" ? "Checking…" : "Check for updates" }}
            </button>
            <p v-if="statusText" class="muted tiny" :class="{ 'settings-error': status === 'error' }">
              {{ statusText }}
            </p>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
