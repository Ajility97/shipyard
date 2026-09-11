import { invoke } from "@tauri-apps/api/core";
import type {
  AppData,
  CommitNode,
  RepoActionResult,
  RepoEntry,
  RepoGroup,
  RepoStatus,
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

export function removeRepo(groupId: string, repoId: string) {
  return invoke<void>("remove_repo", { groupId, repoId });
}

export function updateAppSettings(refreshIntervalSeconds: number) {
  return invoke<number>("update_app_settings", { refreshIntervalSeconds });
}

export function groupStatus(groupId: string, fetch = false) {
  return invoke<RepoStatus[]>("group_status", { groupId, fetch });
}

export function refreshRepo(groupId: string, repoId: string, fetch = true) {
  return invoke<RepoStatus>("refresh_repo", { groupId, repoId, fetch });
}

export function pullCurrent(groupId: string) {
  return invoke<RepoActionResult[]>("pull_current", { groupId });
}

export function pullFromBranch(groupId: string) {
  return invoke<RepoActionResult[]>("pull_from_branch", { groupId });
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

export function fileDiff(path: string, file: string) {
  return invoke<string>("file_diff", { path, file });
}
