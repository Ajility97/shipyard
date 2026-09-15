import { invoke } from "@tauri-apps/api/core";
import type {
  AppData,
  BranchOverview,
  CommandLogEntry,
  CommitFile,
  CommitNode,
  DiffMode,
  RepoActionResult,
  RepoEntry,
  RepoGroup,
  RepoStatus,
  StashEntry,
  WindowState,
  WorkingTreeFile,
} from "./types";

export function getState() {
  return invoke<AppData>("get_state");
}

export function createGroup(name: string) {
  return invoke<RepoGroup>("create_group", { name });
}

export function renameGroup(groupId: string, name: string) {
  return invoke<void>("rename_group", { groupId, name });
}

export function deleteGroup(groupId: string) {
  return invoke<void>("delete_group", { groupId });
}

export function toggleGroup(groupId: string) {
  return invoke<boolean>("toggle_group", { groupId });
}

export function setAllGroupsExpanded(expanded: boolean) {
  return invoke<void>("set_all_groups_expanded", { expanded });
}

export function updateGroupSettings(
  groupId: string,
  pullFromBranch: string,
  checkoutFallbacks: string[],
  headerColor?: string,
) {
  return invoke<void>("update_group_settings", {
    groupId,
    pullFromBranch,
    checkoutFallbacks,
    headerColor,
  });
}

export function addRepo(groupId: string, path: string) {
  return invoke<RepoEntry>("add_repo", { groupId, path });
}

export function addStandaloneRepo(path: string) {
  return invoke<RepoEntry>("add_standalone_repo", { path });
}

export function removeStandaloneRepo(repoId: string) {
  return invoke<void>("remove_standalone_repo", { repoId });
}

export function standaloneStatus(fetch = false) {
  return invoke<RepoStatus[]>("standalone_status", { fetch });
}

export function removeRepo(groupId: string, repoId: string) {
  return invoke<void>("remove_repo", { groupId, repoId });
}

export function updateAppSettings(refreshIntervalSeconds: number) {
  return invoke<number>("update_app_settings", { refreshIntervalSeconds });
}

export function updateFilesPaneWidth(width: number) {
  return invoke<number>("update_files_pane_width", { width });
}

export function updateDiffMode(mode: DiffMode) {
  return invoke<DiffMode>("update_diff_mode", { mode });
}

export function getWindowState() {
  return invoke<WindowState>("get_window_state");
}

export function updateWindowState(window: WindowState) {
  return invoke<WindowState>("update_window_state", { window });
}

export function replaceAppData(data: AppData) {
  return invoke<AppData>("replace_app_data", { data });
}

export function groupStatus(groupId: string, fetch = false) {
  return invoke<RepoStatus[]>("group_status", { groupId, fetch });
}

export function refreshRepo(groupId: string, repoId: string, fetch = true) {
  return invoke<RepoStatus>("refresh_repo", { groupId, repoId, fetch });
}

export function pullRepo(groupId: string, repoId: string, branch?: string) {
  return invoke<RepoActionResult>("pull_repo", {
    groupId,
    repoId,
    branch: branch?.trim() || null,
  });
}

export function pullCurrent(groupId: string) {
  return invoke<RepoActionResult[]>("pull_current", { groupId });
}

export function pullFromBranch(groupId: string) {
  return invoke<RepoActionResult[]>("pull_from_branch", { groupId });
}

export function checkoutRepo(
  groupId: string,
  repoId: string,
  target: string,
  fallbacks: string[],
) {
  return invoke<RepoActionResult>("checkout_repo", { groupId, repoId, target, fallbacks });
}

export function checkoutAll(groupId: string, target: string, fallbacks: string[]) {
  return invoke<RepoActionResult[]>("checkout_all", { groupId, target, fallbacks });
}

export function logGraph(path: string) {
  return invoke<CommitNode[]>("log_graph", { path });
}

export function workingTree(path: string) {
  return invoke<WorkingTreeFile[]>("working_tree", { path });
}

export function discardAllChanges(path: string) {
  return invoke<void>("discard_all_changes", { path });
}

export function commit(path: string, title: string, description: string) {
  return invoke<string>("commit", { path, title, description });
}

export function stageFile(path: string, file: string) {
  return invoke<void>("stage_file", { path, file });
}

export function stageAll(path: string) {
  return invoke<void>("stage_all", { path });
}

export function unstageFile(path: string, file: string) {
  return invoke<void>("unstage_file", { path, file });
}

export function unstageAll(path: string) {
  return invoke<void>("unstage_all", { path });
}

export function listLocalBranches(path: string) {
  return invoke<string[]>("list_local_branches", { path });
}

export function branchOverview(path: string, preferred?: string) {
  return invoke<BranchOverview>("branch_overview", {
    path,
    preferred: preferred?.trim() || null,
  });
}

export function deleteLocalBranch(path: string, branch: string, force = false) {
  return invoke<string>("delete_local_branch", { path, branch, force });
}

export function deleteMergedBranches(path: string, preferred?: string) {
  return invoke<string>("delete_merged_branches", {
    path,
    preferred: preferred?.trim() || null,
  });
}

export function checkoutLocalBranch(path: string, branch: string) {
  return invoke<string>("checkout_local_branch", { path, branch });
}

export function createAndCheckoutBranch(path: string, branch: string) {
  return invoke<string>("create_and_checkout_branch", { path, branch });
}

export function repoPull(path: string) {
  return invoke<string>("repo_pull", { path });
}

export function repoPush(path: string) {
  return invoke<string>("repo_push", { path });
}

export function fileDiff(path: string, file: string, staged = false) {
  return invoke<string>("file_diff", { path, file, staged });
}

export function commitFiles(path: string, hash: string) {
  return invoke<CommitFile[]>("commit_files", { path, hash });
}

export function commitFileDiff(path: string, hash: string, file: string) {
  return invoke<string>("commit_file_diff", { path, hash, file });
}

export function stashList(path: string) {
  return invoke<StashEntry[]>("stash_list", { path });
}

export function stashPush(path: string, message: string) {
  return invoke<string>("stash_push", { path, message });
}

export function stashApply(path: string, index: number) {
  return invoke<string>("stash_apply", { path, index });
}

export function stashPop(path: string, index: number) {
  return invoke<string>("stash_pop", { path, index });
}

export function stashDrop(path: string, index: number) {
  return invoke<string>("stash_drop", { path, index });
}

export function writeTextFile(path: string, contents: string) {
  return invoke<void>("write_text_file", { path, contents });
}

export function readTextFile(path: string) {
  return invoke<string>("read_text_file", { path });
}

export function settingsFilePath() {
  return invoke<string>("settings_file_path");
}

export function revealSettingsFile() {
  return invoke<void>("reveal_settings_file");
}

export function commandHistory() {
  return invoke<CommandLogEntry[]>("command_history");
}

export function clearCommandHistory() {
  return invoke<void>("clear_command_history");
}
