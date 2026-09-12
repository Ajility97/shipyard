export interface RepoEntry {
  id: string;
  path: string;
}

export interface RepoGroup {
  id: string;
  name: string;
  expanded: boolean;
  pullFromBranch: string;
  checkoutFallbacks: string[];
  headerColor?: string;
  repos: RepoEntry[];
}

export type DiffMode = "inline" | "split";

export const STANDALONE_GROUP_ID = "standalone";

export interface WindowState {
  x: number;
  y: number;
  width: number;
  height: number;
  maximized?: boolean;
}

export interface AppData {
  groups: RepoGroup[];
  repos?: RepoEntry[];
  refreshIntervalSeconds?: number;
  filesPaneWidth?: number;
  diffMode?: DiffMode;
  window?: WindowState;
}

export interface RepoStatus {
  id: string;
  path: string;
  name: string;
  branch: string;
  ahead: number;
  behind: number;
  dirty: boolean;
  insertions: number;
  deletions: number;
  changedFiles: number;
}

export interface RepoActionResult {
  path: string;
  ok: boolean;
  message: string;
}

export interface CommitNode {
  hash: string;
  parents: string[];
  subject: string;
  author: string;
  date: string;
  refs: string;
}

export interface WorkingTreeFile {
  path: string;
  status: string;
  untracked: boolean;
}
