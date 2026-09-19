<script setup lang="ts">
import { onMounted } from "vue";
import { useUpdater } from "../../composables/useUpdater";

const { currentVersion, ensureCurrentVersion } = useUpdater();

onMounted(() => {
  void ensureCurrentVersion();
});
</script>

<template>
  <div class="settings-pane settings-form-page">
    <div class="settings-inner">
      <div class="settings-header">
        <div>
          <div class="brand">Updates</div>
          <p class="muted tiny">
            Shipyard checks GitHub on launch and from Check for Updates in the
            menu. Newer builds install in place and restart.
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
      </section>
    </div>
  </div>
</template>
