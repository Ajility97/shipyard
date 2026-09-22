import { EDITOR_OPTIONS, type WorkingTreeFile } from "./types";

export function isConflicted(file: Pick<WorkingTreeFile, "status">) {
  return file.status.trim().toLowerCase() === "conflicted";
}

export function conflictCountLabel(count: number) {
  return count === 1 ? "1 conflict" : `${count} conflicts`;
}

export function operationNoun(operation: string) {
  switch (operation) {
    case "rebase":
      return "rebase";
    case "cherry-pick":
      return "cherry-pick";
    case "revert":
      return "revert";
    default:
      return "merge";
  }
}

export function operationTitle(operation: string) {
  switch (operation) {
    case "rebase":
      return "Rebase in progress";
    case "cherry-pick":
      return "Cherry-pick in progress";
    case "revert":
      return "Revert in progress";
    case "merge":
      return "Merge in progress";
    default:
      return "Conflicts need resolving";
  }
}

export function abortLabel(operation: string) {
  return `Abort ${operationNoun(operation)}`;
}

export function continueLabel(operation: string) {
  return `Continue ${operationNoun(operation)}`;
}

export function openInEditorLabel(editor: string) {
  const id = editor.trim();
  if (!id || id === "system") {
    return "Open in editor";
  }
  const known = EDITOR_OPTIONS.find((option) => option.id === id);
  return `Open in ${known?.short ?? known?.label ?? id}`;
}

export type IgnoreKind = "file" | "extension" | "folder";

export function fileBasename(path: string) {
  const normalized = path.replace(/\\/g, "/");
  return normalized.slice(normalized.lastIndexOf("/") + 1) || path;
}

export function fileExtension(path: string) {
  const name = fileBasename(path);
  const rest = name.startsWith(".") ? name.slice(1) : name;
  const dot = rest.lastIndexOf(".");
  if (dot <= 0 || dot === rest.length - 1) {
    return "";
  }
  return `.${rest.slice(dot + 1)}`;
}

export function fileParentFolder(path: string) {
  const normalized = path.replace(/\\/g, "/");
  const slash = normalized.lastIndexOf("/");
  if (slash <= 0) {
    return "";
  }
  const parent = normalized.slice(0, slash);
  const name = parent.slice(parent.lastIndexOf("/") + 1);
  return name ? `${name}/` : "";
}

export function absoluteFilePath(repoPath: string, file: string) {
  return `${repoPath.replace(/\/+$/, "")}/${file.replace(/^\/+/, "")}`;
}
