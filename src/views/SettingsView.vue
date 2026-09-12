<script setup lang="ts">
import { json } from "@codemirror/lang-json";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { EditorView } from "@codemirror/view";
import { tags as t } from "@lezer/highlight";
import { basicSetup } from "codemirror";
import { nextTick, onMounted, onUnmounted, ref } from "vue";
import * as api from "../api";
import { useApp } from "../composables/useApp";
import type { AppData } from "../types";

const { replaceSettings } = useApp();
const editorHost = ref<HTMLDivElement | null>(null);
const draft = ref("");
const message = ref("");
const saving = ref(false);
const saved = ref(false);
const copied = ref(false);
let view: EditorView | null = null;

const editorTheme = EditorView.theme(
  {
    "&": {
      height: "100%",
      color: "var(--text)",
      backgroundColor: "#0d1016",
      fontFamily: "var(--font-code)",
      fontSize: "12px",
    },
    "&.cm-focused": {
      outline: "none",
    },
    ".cm-scroller": {
      fontFamily: "var(--font-code)",
      lineHeight: "1.5",
    },
    ".cm-content": {
      caretColor: "var(--text)",
      padding: "0.45rem 0.8rem",
    },
    ".cm-gutters": {
      backgroundColor: "#0d1016",
      color: "var(--muted)",
      borderRight: "1px solid var(--border)",
    },
    ".cm-activeLine": {
      backgroundColor: "rgba(255, 255, 255, 0.03)",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "transparent",
    },
    ".cm-selectionBackground, &.cm-focused .cm-selectionBackground": {
      backgroundColor: "#2a3344",
    },
    ".cm-cursor": {
      borderLeftColor: "var(--text)",
    },
  },
  { dark: true },
);

const editorHighlight = HighlightStyle.define([
  { tag: t.propertyName, color: "#7ee0c7" },
  { tag: t.string, color: "#3dd68c" },
  { tag: t.number, color: "#e6c07b" },
  { tag: t.bool, color: "#9ec1ff" },
  { tag: t.null, color: "var(--muted)" },
  { tag: t.punctuation, color: "var(--muted)" },
  { tag: t.squareBracket, color: "#d7deea" },
  { tag: t.brace, color: "#d7deea" },
  { tag: t.invalid, color: "var(--bad)" },
]);

function setDraft(value: string) {
  draft.value = value;
  if (!view) {
    return;
  }
  const current = view.state.doc.toString();
  if (current === value) {
    return;
  }
  view.dispatch({
    changes: { from: 0, to: view.state.doc.length, insert: value },
  });
}

onMounted(async () => {
  try {
    draft.value = JSON.stringify(await api.getState(), null, 2);
  } catch (err) {
    message.value = String(err);
  }
  await nextTick();
  if (!editorHost.value) {
    return;
  }
  view = new EditorView({
    doc: draft.value,
    parent: editorHost.value,
    extensions: [
      basicSetup,
      json(),
      editorTheme,
      syntaxHighlighting(editorHighlight),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          draft.value = update.state.doc.toString();
          saved.value = false;
        }
      }),
    ],
  });
});

onUnmounted(() => {
  view?.destroy();
  view = null;
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
  saved.value = false;
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
    setDraft(JSON.stringify(next, null, 2));
    saved.value = true;
  } catch (err) {
    message.value = String(err);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="settings-page">
    <div class="settings-inner">
      <div class="settings-header">
        <div>
          <div class="brand">Settings</div>
          <p class="muted tiny">
            This is the saved settings file. Edit it here, or paste a copy from another machine.
          </p>
        </div>
        <button class="primary" type="button" :disabled="saving || !draft.trim()" @click="save">
          {{ saving ? "Saving…" : saved ? "Saved" : "Save" }}
        </button>
      </div>
      <div class="settings-editor">
        <button class="ghost tiny settings-copy" type="button" @click="copy">
          {{ copied ? "Copied" : "Copy" }}
        </button>
        <div ref="editorHost" class="settings-json" />
      </div>
      <p v-if="message" class="settings-error">{{ message }}</p>
    </div>
  </div>
</template>
