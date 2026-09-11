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

export interface AppData {
  groups: RepoGroup[];
  refreshIntervalSeconds?: number;
}

export interface RepoStatus {
  id: string;
  path: string;
  name: string;
  branch: string;
  ahead: number;
  behind: number;
  dirty: boolean;
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
