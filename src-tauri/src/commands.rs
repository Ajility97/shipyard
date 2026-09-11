use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;

use crate::git;
use crate::models::{
    AppData, CommitNode, RepoActionResult, RepoEntry, RepoGroup, RepoStatus, WorkingTreeFile,
};
use crate::persist;

pub struct AppState {
    pub data: Mutex<AppData>,
    pub git: Option<PathBuf>,
}

fn require_git(state: &AppState) -> Result<PathBuf, String> {
    state.git.clone().ok_or_else(|| {
        "Git was not found. Install Git and make sure it is available on your PATH.".into()
    })
}

fn find_group_mut<'a>(data: &'a mut AppData, group_id: &str) -> Result<&'a mut RepoGroup, String> {
    data.groups
        .iter_mut()
        .find(|group| group.id == group_id)
        .ok_or_else(|| "Group not found".to_string())
}

fn persist_data(app: &AppHandle, data: &AppData) -> Result<(), String> {
    persist::save(app, data)
}

fn repo_list(state: &AppState, group_id: &str) -> Result<(PathBuf, RepoGroup), String> {
    let git = require_git(state)?;
    let data = state.data.lock().map_err(|err| err.to_string())?;
    let group = data
        .groups
        .iter()
        .find(|group| group.id == group_id)
        .cloned()
        .ok_or_else(|| "Group not found".to_string())?;
    Ok((git, group))
}

#[tauri::command]
pub fn get_state(state: State<AppState>) -> Result<AppData, String> {
    let data = state.data.lock().map_err(|err| err.to_string())?;
    Ok(data.clone())
}

#[tauri::command]
pub fn create_group(
    app: AppHandle,
    state: State<AppState>,
    name: String,
) -> Result<RepoGroup, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Group name is required".into());
    }

    let group = RepoGroup {
        id: uuid::Uuid::new_v4().to_string(),
        name,
        expanded: true,
        pull_from_branch: "develop".into(),
        checkout_fallbacks: vec!["develop".into()],
        header_color: "#16323c".into(),
        repos: Vec::new(),
    };

    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    data.groups.push(group.clone());
    persist_data(&app, &data)?;
    Ok(group)
}

#[tauri::command]
pub fn rename_group(
    app: AppHandle,
    state: State<AppState>,
    group_id: String,
    name: String,
) -> Result<(), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Group name is required".into());
    }
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    find_group_mut(&mut data, &group_id)?.name = name;
    persist_data(&app, &data)
}

#[tauri::command]
pub fn delete_group(app: AppHandle, state: State<AppState>, group_id: String) -> Result<(), String> {
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    let before = data.groups.len();
    data.groups.retain(|group| group.id != group_id);
    if data.groups.len() == before {
        return Err("Group not found".into());
    }
    persist_data(&app, &data)
}

#[tauri::command]
pub fn toggle_group(app: AppHandle, state: State<AppState>, group_id: String) -> Result<bool, String> {
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    let group = find_group_mut(&mut data, &group_id)?;
    group.expanded = !group.expanded;
    let expanded = group.expanded;
    persist_data(&app, &data)?;
    Ok(expanded)
}

#[tauri::command]
pub fn update_group_settings(
    app: AppHandle,
    state: State<AppState>,
    group_id: String,
    pull_from_branch: String,
    checkout_fallbacks: Vec<String>,
    header_color: Option<String>,
) -> Result<(), String> {
    let pull_from_branch = pull_from_branch.trim().to_string();
    if !pull_from_branch.is_empty() {
        git::validate_ref(&pull_from_branch)?;
    }
    let fallbacks: Vec<String> = checkout_fallbacks
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    for fallback in &fallbacks {
        git::validate_ref(fallback)?;
    }

    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    let group = find_group_mut(&mut data, &group_id)?;
    group.pull_from_branch = if pull_from_branch.is_empty() {
        "develop".into()
    } else {
        pull_from_branch
    };
    group.checkout_fallbacks = if fallbacks.is_empty() {
        vec!["develop".into()]
    } else {
        fallbacks
    };
    if let Some(color) = header_color {
        let color = color.trim().to_string();
        if color.starts_with('#') && (color.len() == 7 || color.len() == 4) {
            group.header_color = color;
        }
    }
    persist_data(&app, &data)
}

#[tauri::command]
pub fn update_app_settings(
    app: AppHandle,
    state: State<AppState>,
    refresh_interval_seconds: u64,
) -> Result<u64, String> {
    let seconds = if refresh_interval_seconds == 0 {
        0
    } else {
        refresh_interval_seconds.clamp(30, 86_400)
    };
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    data.refresh_interval_seconds = seconds;
    persist_data(&app, &data)?;
    Ok(seconds)
}

#[tauri::command]
pub fn add_repo(
    app: AppHandle,
    state: State<AppState>,
    group_id: String,
    path: String,
) -> Result<RepoEntry, String> {
    let git = require_git(&state)?;
    let root = git::repo_root(&git, Path::new(&path))?;

    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    let group = find_group_mut(&mut data, &group_id)?;
    if group.repos.iter().any(|repo| repo.path == root) {
        return Err("That repository is already in this group.".into());
    }

    let entry = RepoEntry {
        id: uuid::Uuid::new_v4().to_string(),
        path: root,
    };
    group.repos.push(entry.clone());
    persist_data(&app, &data)?;
    Ok(entry)
}

#[tauri::command]
pub fn remove_repo(
    app: AppHandle,
    state: State<AppState>,
    group_id: String,
    repo_id: String,
) -> Result<(), String> {
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    let group = find_group_mut(&mut data, &group_id)?;
    let before = group.repos.len();
    group.repos.retain(|repo| repo.id != repo_id);
    if group.repos.len() == before {
        return Err("Repository not found".into());
    }
    persist_data(&app, &data)
}

#[tauri::command]
pub fn group_status(
    state: State<AppState>,
    group_id: String,
    fetch: Option<bool>,
) -> Result<Vec<RepoStatus>, String> {
    let (git, group) = repo_list(&state, &group_id)?;
    let fetch = fetch.unwrap_or(false);
    let mut statuses = Vec::new();
    for repo in group.repos {
        let path = Path::new(&repo.path);
        if fetch {
            git::fetch_remote(&git, path);
        }
        let live = git::live_status(&git, path);
        let (branch, ahead, behind, dirty) = match live {
            Ok(status) => (status.branch, status.ahead, status.behind, status.dirty),
            Err(err) => (format!("error: {err}"), 0, 0, false),
        };
        statuses.push(RepoStatus {
            id: repo.id,
            name: git::folder_name(&repo.path),
            path: repo.path,
            branch,
            ahead,
            behind,
            dirty,
        });
    }
    Ok(statuses)
}

#[tauri::command]
pub async fn refresh_repo(
    state: State<'_, AppState>,
    group_id: String,
    repo_id: String,
    fetch: Option<bool>,
) -> Result<RepoStatus, String> {
    let (git, group) = repo_list(&state, &group_id)?;
    let repo = group
        .repos
        .into_iter()
        .find(|entry| entry.id == repo_id)
        .ok_or_else(|| "Repository not found".to_string())?;
    let should_fetch = fetch.unwrap_or(true);
    tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&repo.path);
        if should_fetch {
            git::fetch_remote(&git, path);
        }
        let live = git::live_status(&git, path);
        let (branch, ahead, behind, dirty) = match live {
            Ok(status) => (status.branch, status.ahead, status.behind, status.dirty),
            Err(err) => (format!("error: {err}"), 0, 0, false),
        };
        Ok(RepoStatus {
            id: repo.id,
            name: git::folder_name(&repo.path),
            path: repo.path,
            branch,
            ahead,
            behind,
            dirty,
        })
    })
    .await
    .map_err(|err| err.to_string())?
}

#[tauri::command]
pub fn pull_current(state: State<AppState>, group_id: String) -> Result<Vec<RepoActionResult>, String> {
    let (git, group) = repo_list(&state, &group_id)?;
    Ok(group
        .repos
        .into_iter()
        .map(|repo| {
            let output = match git::run_git(&git, Path::new(&repo.path), &["pull"]) {
                Ok(output) => output,
                Err(err) => {
                    return RepoActionResult {
                        path: repo.path,
                        ok: false,
                        message: err,
                    };
                }
            };
            RepoActionResult {
                path: repo.path,
                ok: output.success,
                message: fallback_message(&git::combined_message(&output), output.success, "Pulled current branch"),
            }
        })
        .collect())
}

#[tauri::command]
pub fn pull_from_branch(
    state: State<AppState>,
    group_id: String,
) -> Result<Vec<RepoActionResult>, String> {
    let (git, group) = repo_list(&state, &group_id)?;
    let branch = group.pull_from_branch.trim();
    if branch.is_empty() {
        return Err("Pull-from branch is not set".into());
    }
    git::validate_ref(branch)?;
    Ok(group
        .repos
        .into_iter()
        .map(|repo| {
            let output = match git::run_git(&git, Path::new(&repo.path), &["pull", "origin", branch]) {
                Ok(output) => output,
                Err(err) => {
                    return RepoActionResult {
                        path: repo.path,
                        ok: false,
                        message: err,
                    };
                }
            };
            RepoActionResult {
                path: repo.path,
                ok: output.success,
                message: fallback_message(
                    &git::combined_message(&output),
                    output.success,
                    &format!("Pulled origin/{branch} into the current branch"),
                ),
            }
        })
        .collect())
}

#[tauri::command]
pub fn checkout_all(
    state: State<AppState>,
    group_id: String,
    target: String,
    fallbacks: Vec<String>,
) -> Result<Vec<RepoActionResult>, String> {
    let (git, group) = repo_list(&state, &group_id)?;
    let mut branches = Vec::new();
    let target = target.trim().to_string();
    if !target.is_empty() {
        git::validate_ref(&target)?;
        branches.push(target);
    }
    for fallback in fallbacks {
        let fallback = fallback.trim().to_string();
        if !fallback.is_empty() && !branches.contains(&fallback) {
            git::validate_ref(&fallback)?;
            branches.push(fallback);
        }
    }
    if branches.is_empty() {
        return Err("Provide a branch to check out".into());
    }

    Ok(group
        .repos
        .into_iter()
        .map(|repo| match git::checkout_with_fallbacks(&git, Path::new(&repo.path), &branches) {
            Ok(message) => RepoActionResult {
                path: repo.path,
                ok: true,
                message,
            },
            Err(message) => RepoActionResult {
                path: repo.path,
                ok: false,
                message,
            },
        })
        .collect())
}

#[tauri::command]
pub fn log_graph(state: State<AppState>, path: String) -> Result<Vec<CommitNode>, String> {
    let git = require_git(&state)?;
    git::log_graph(&git, Path::new(&path))
}

#[tauri::command]
pub fn working_tree(state: State<AppState>, path: String) -> Result<Vec<WorkingTreeFile>, String> {
    let git = require_git(&state)?;
    git::working_tree(&git, Path::new(&path))
}

#[tauri::command]
pub fn request_notification_permission(app: AppHandle) -> Result<(), String> {
    let _ = app.notification().request_permission();
    Ok(())
}

#[tauri::command]
pub fn notify_user(app: AppHandle, title: String, body: String) -> Result<(), String> {
    app.notification()
        .builder()
        .title(title)
        .body(body)
        .show()
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn file_diff(state: State<AppState>, path: String, file: String) -> Result<String, String> {
    let git = require_git(&state)?;
    git::file_diff(&git, Path::new(&path), &file)
}

fn fallback_message(message: &str, ok: bool, success_fallback: &str) -> String {
    if !message.trim().is_empty() {
        message.trim().to_string()
    } else if ok {
        success_fallback.to_string()
    } else {
        "Git command failed".into()
    }
}
