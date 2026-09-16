export interface RepoEntry {
  id: string;
  path: string;
  label?: string;
  headerColor?: string;
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

export const MIN_WINDOW_WIDTH = 960;
export const MIN_WINDOW_HEIGHT = 640;
export const DEFAULT_WINDOW_WIDTH = 1280;
export const DEFAULT_WINDOW_HEIGHT = 800;

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
  staged: boolean;
}

export interface CommitFile {
  path: string;
  oldPath: string | null;
  status: string;
}

export interface LocalBranch {
  name: string;
  current: boolean;
  merged: boolean;
  partial: boolean;
  protected: boolean;
  pending: boolean;
}

export interface DeleteMergedResult {
  deleted: string[];
  refused: string[];
  errors: string[];
  message: string;
}

export interface BranchOverview {
  mergeTarget: string | null;
  branches: LocalBranch[];
}

export interface StashEntry {
  index: number;
  message: string;
  date: string;
}

export interface CommandLogEntry {
  id: string;
  at: number;
  cwd: string;
  program: string;
  args: string[];
  command: string;
  success: boolean;
  durationMs: number;
  stdout: string;
  stderr: string;
}
