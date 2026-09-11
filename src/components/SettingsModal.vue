<script setup lang="ts">
import { onMounted, ref } from "vue";
import * as api from "../api";
import { useApp } from "../composables/useApp";
import type { AppData } from "../types";
import Modal from "./Modal.vue";

const emit = defineEmits<{
  close: [];
}>();

const { replaceSettings } = useApp();
const draft = ref("");
const message = ref("");
const saving = ref(false);
const copied = ref(false);

onMounted(async () => {
  try {
    draft.value = JSON.stringify(await api.getState(), null, 2);
  } catch (err) {
    message.value = String(err);
  }
});

async function copy() {
  try {
    await navigator.clipboard.writeText(draft.value);
    copied.value = true;
    window.setTimeout(() => {
      copied.value = false;
    }, 1600);
  } catch (err) {
    message.value = String(err);
  }
}

async function save() {
  message.value = "";
  let parsed: AppData;
  try {
    parsed = JSON.parse(draft.value) as AppData;
  } catch {
    message.value = "Settings JSON is not valid.";
    return;
  }
  saving.value = true;
  try {
    const next = await replaceSettings(parsed);
    draft.value = JSON.stringify(next, null, 2);
    emit("close");
  } catch (err) {
    message.value = String(err);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <Modal title="Settings" wide @close="emit('close')">
    <p class="muted tiny">
      This is the saved settings file. Edit it here, or paste a copy from another machine.
    </p>
    <div class="settings-editor">
      <button class="ghost tiny settings-copy" type="button" @click="copy">
        {{ copied ? "Copied" : "Copy" }}
      </button>
      <textarea v-model="draft" class="settings-json" spellcheck="false" />
    </div>
    <p v-if="message" class="settings-error">{{ message }}</p>
    <template #actions>
      <button class="ghost" type="button" @click="emit('close')">Cancel</button>
      <button class="primary" type="button" :disabled="saving || !draft.trim()" @click="save">
        {{ saving ? "Saving…" : "Save" }}
      </button>
    </template>
  </Modal>
</template>
