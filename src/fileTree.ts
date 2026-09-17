import type { RepoFile } from "./types";

export interface FileEntry {
  name: string;
  path: string;
  kind: "file" | "folder";
  ignored: boolean;
  children: FileEntry[];
}

export function buildFileTree(files: RepoFile[]): FileEntry[] {
  const root: FileEntry[] = [];
  const folders = new Map<string, FileEntry>();

  const ensureFolder = (dir: string): FileEntry[] => {
    if (!dir) {
      return root;
    }
    const existing = folders.get(dir);
    if (existing) {
      return existing.children;
    }
    const parts = dir.split("/");
    const name = parts[parts.length - 1] ?? dir;
    const parent = parts.slice(0, -1).join("/");
    const node: FileEntry = { name, path: dir, kind: "folder", ignored: false, children: [] };
    folders.set(dir, node);
    ensureFolder(parent).push(node);
    return node.children;
  };

  for (const file of files) {
    const parts = file.path.split("/").filter(Boolean);
    if (!parts.length) {
      continue;
    }
    if (file.directory) {
      ensureFolder(file.path);
      const folder = folders.get(file.path);
      if (folder) {
        folder.ignored = file.ignored;
      }
      continue;
    }
    const name = parts[parts.length - 1] ?? file.path;
    const parent = parts.slice(0, -1).join("/");
    ensureFolder(parent).push({
      name,
      path: file.path,
      kind: "file",
      ignored: file.ignored,
      children: [],
    });
  }

  const markIgnoredFolders = (nodes: FileEntry[]) => {
    for (const node of nodes) {
      if (node.kind !== "folder") {
        continue;
      }
      markIgnoredFolders(node.children);
      if (!node.ignored && node.children.length && node.children.every((child) => child.ignored)) {
        node.ignored = true;
      }
    }
  };

  const sortNodes = (nodes: FileEntry[]) => {
    nodes.sort((left, right) => {
      if (left.kind !== right.kind) {
        return left.kind === "folder" ? -1 : 1;
      }
      return left.name.localeCompare(right.name, undefined, { sensitivity: "base" });
    });
    for (const node of nodes) {
      if (node.children.length) {
        sortNodes(node.children);
      }
    }
  };

  markIgnoredFolders(root);
  sortNodes(root);
  return root;
}

function queryTokens(query: string) {
  return query
    .trim()
    .toLowerCase()
    .split(/\s+/)
    .filter(Boolean);
}

function matchesTokens(text: string, tokens: string[]) {
  if (!tokens.length) {
    return true;
  }
  const haystack = text.toLowerCase();
  let from = 0;
  for (const token of tokens) {
    const index = haystack.indexOf(token, from);
    if (index === -1) {
      return false;
    }
    from = index + token.length;
  }
  return true;
}

function matchesQuery(node: FileEntry, tokens: string[]) {
  return matchesTokens(node.name, tokens) || matchesTokens(node.path, tokens);
}

export function filterFileTree(nodes: FileEntry[], query: string): FileEntry[] {
  const tokens = queryTokens(query);
  if (!tokens.length) {
    return nodes;
  }

  const visit = (node: FileEntry): FileEntry | null => {
    if (node.kind === "folder") {
      if (matchesQuery(node, tokens)) {
        return node;
      }
      const children = node.children
        .map(visit)
        .filter((child): child is FileEntry => child !== null);
      return children.length ? { ...node, children } : null;
    }
    return matchesQuery(node, tokens) ? node : null;
  };

  return nodes.map(visit).filter((node): node is FileEntry => node !== null);
}

export function searchExpandedPaths(nodes: FileEntry[], query: string): Set<string> {
  const tokens = queryTokens(query);
  const open = new Set<string>();
  if (!tokens.length) {
    return open;
  }

  const walk = (node: FileEntry) => {
    if (node.kind !== "folder" || !node.children.length) {
      return;
    }
    const selfMatch = matchesQuery(node, tokens);
    open.add(node.path);
    if (selfMatch) {
      return;
    }
    for (const child of node.children) {
      walk(child);
    }
  };

  for (const node of nodes) {
    walk(node);
  }
  return open;
}

export function treeHasFolders(nodes: FileEntry[]): boolean {
  return nodes.some((node) => node.kind === "folder" || treeHasFolders(node.children));
}
