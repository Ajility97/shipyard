use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, State};

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

const STANDALONE_GROUP_ID: &str = "standalone";

fn path_already_added(data: &AppData, root: &str) -> bool {
    data.repos.iter().any(|repo| repo.path == root)
        || data
            .groups
            .iter()
            .any(|group| group.repos.iter().any(|repo| repo.path == root))
}

fn sanitize_repos(repos: &mut Vec<RepoEntry>) -> Result<(), String> {
    for repo in repos {
        if repo.id.trim().is_empty() {
            repo.id = uuid::Uuid::new_v4().to_string();
        }
        repo.path = repo.path.trim().to_string();
        if repo.path.is_empty() {
            return Err("Repository path cannot be empty".into());
        }
    }
    Ok(())
}

fn find_repo_entry(data: &AppData, group_id: &str, repo_id: &str) -> Result<RepoEntry, String> {
    if group_id == STANDALONE_GROUP_ID {
        return data
            .repos
            .iter()
            .find(|entry| entry.id == repo_id)
            .cloned()
            .ok_or_else(|| "Repository not found".to_string());
    }
    let group = data
        .groups
        .iter()
        .find(|group| group.id == group_id)
        .ok_or_else(|| "Group not found".to_string())?;
    group
        .repos
        .iter()
        .find(|entry| entry.id == repo_id)
        .cloned()
        .ok_or_else(|| "Repository not found".to_string())
}

fn repo_entry(state: &AppState, group_id: &str, repo_id: &str) -> Result<(PathBuf, RepoEntry), String> {
    let git = require_git(state)?;
    let data = state.data.lock().map_err(|err| err.to_string())?;
    let repo = find_repo_entry(&data, group_id, repo_id)?;
    Ok((git, repo))
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
    data.groups.insert(0, group.clone());
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
pub fn set_all_groups_expanded(
    app: AppHandle,
    state: State<AppState>,
    expanded: bool,
) -> Result<(), String> {
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    for group in &mut data.groups {
        group.expanded = expanded;
    }
    persist_data(&app, &data)
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
pub fn update_files_pane_width(
    app: AppHandle,
    state: State<AppState>,
    width: u32,
) -> Result<u32, String> {
    let width = width.clamp(220, 800);
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    data.files_pane_width = width;
    persist_data(&app, &data)?;
    Ok(width)
}

#[tauri::command]
pub fn update_diff_mode(
    app: AppHandle,
    state: State<AppState>,
    mode: String,
) -> Result<String, String> {
    let mode = sanitize_diff_mode(&mode)?;
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    data.diff_mode = mode.clone();
    persist_data(&app, &data)?;
    Ok(mode)
}

#[tauri::command]
pub fn replace_app_data(
    app: AppHandle,
    state: State<AppState>,
    data: AppData,
) -> Result<AppData, String> {
    let sanitized = sanitize_app_data(data)?;
    let mut lock = state.data.lock().map_err(|err| err.to_string())?;
    *lock = sanitized.clone();
    persist_data(&app, &lock)?;
    Ok(sanitized)
}

fn sanitize_diff_mode(mode: &str) -> Result<String, String> {
    match mode.trim() {
        "inline" | "split" => Ok(mode.trim().to_string()),
        _ => Err("diffMode must be \"inline\" or \"split\"".into()),
    }
}

fn sanitize_app_data(mut data: AppData) -> Result<AppData, String> {
    data.refresh_interval_seconds = if data.refresh_interval_seconds == 0 {
        0
    } else {
        data.refresh_interval_seconds.clamp(30, 86_400)
    };
    data.files_pane_width = data.files_pane_width.clamp(220, 800);
    data.diff_mode = sanitize_diff_mode(&data.diff_mode)?;
    if let Some(window) = &mut data.window {
        window.width = window.width.max(crate::models::MIN_WINDOW_WIDTH);
        window.height = window.height.max(crate::models::MIN_WINDOW_HEIGHT);
    }
    sanitize_repos(&mut data.repos)?;

    for group in &mut data.groups {
        if group.id.trim().is_empty() {
            group.id = uuid::Uuid::new_v4().to_string();
        }
        group.name = group.name.trim().to_string();
        if group.name.is_empty() {
            return Err("Every group needs a name".into());
        }
        let pull = group.pull_from_branch.trim().to_string();
        if !pull.is_empty() {
            git::validate_ref(&pull)?;
        }
        group.pull_from_branch = if pull.is_empty() {
            "develop".into()
        } else {
            pull
        };
        let fallbacks: Vec<String> = group
            .checkout_fallbacks
            .iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect();
        for fallback in &fallbacks {
            git::validate_ref(fallback)?;
        }
        group.checkout_fallbacks = if fallbacks.is_empty() {
            vec!["develop".into()]
        } else {
            fallbacks
        };
        let color = group.header_color.trim().to_string();
        group.header_color = if color.starts_with('#') && (color.len() == 7 || color.len() == 4) {
            color
        } else {
            "#16323c".into()
        };
        sanitize_repos(&mut group.repos)?;
    }
    Ok(data)
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
    if data.repos.iter().any(|repo| repo.path == root) {
        return Err("That repository is already added.".into());
    }
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
pub fn add_standalone_repo(
    app: AppHandle,
    state: State<AppState>,
    path: String,
) -> Result<RepoEntry, String> {
    let git = require_git(&state)?;
    let root = git::repo_root(&git, Path::new(&path))?;

    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    if path_already_added(&data, &root) {
        return Err("That repository is already added.".into());
    }

    let entry = RepoEntry {
        id: uuid::Uuid::new_v4().to_string(),
        path: root,
    };
    data.repos.push(entry.clone());
    persist_data(&app, &data)?;
    Ok(entry)
}

#[tauri::command]
pub fn remove_standalone_repo(
    app: AppHandle,
    state: State<AppState>,
    repo_id: String,
) -> Result<(), String> {
    let mut data = state.data.lock().map_err(|err| err.to_string())?;
    let before = data.repos.len();
    data.repos.retain(|repo| repo.id != repo_id);
    if data.repos.len() == before {
        return Err("Repository not found".into());
    }
    persist_data(&app, &data)
}

#[tauri::command]
pub fn standalone_status(
    state: State<AppState>,
    fetch: Option<bool>,
) -> Result<Vec<RepoStatus>, String> {
    let git = require_git(&state)?;
    let data = state.data.lock().map_err(|err| err.to_string())?;
    let repos = data.repos.clone();
    drop(data);
    let fetch = fetch.unwrap_or(false);
    let mut statuses = Vec::new();
    for repo in repos {
        let path = Path::new(&repo.path);
        if fetch {
            git::fetch_remote(&git, path);
        }
        statuses.push(status_from_live(&repo, git::live_status(&git, path)));
    }
    Ok(statuses)
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
        statuses.push(status_from_live(&repo, git::live_status(&git, path)));
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
    let (git, repo) = repo_entry(&state, &group_id, &repo_id)?;
    let should_fetch = fetch.unwrap_or(true);
    tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&repo.path);
        if should_fetch {
            git::fetch_remote(&git, path);
        }
        Ok(status_from_live(&repo, git::live_status(&git, path)))
    })
    .await
    .map_err(|err| err.to_string())?
}

#[tauri::command]
pub async fn pull_repo(
    state: State<'_, AppState>,
    group_id: String,
    repo_id: String,
    branch: Option<String>,
) -> Result<RepoActionResult, String> {
    let (git, group) = repo_list(&state, &group_id)?;
    let repo = group
        .repos
        .into_iter()
        .find(|entry| entry.id == repo_id)
        .ok_or_else(|| "Repository not found".to_string())?;
    let branch = branch
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    if let Some(ref name) = branch {
        git::validate_ref(name)?;
    }
    tauri::async_runtime::spawn_blocking(move || {
        let args: Vec<String> = match &branch {
            Some(name) => vec!["pull".into(), "origin".into(), name.clone()],
            None => vec!["pull".into()],
        };
        let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let output = match git::run_git(&git, Path::new(&repo.path), &arg_refs) {
            Ok(output) => output,
            Err(err) => {
                return Ok(RepoActionResult {
                    path: repo.path,
                    ok: false,
                    message: err,
                });
            }
        };
        let fallback = match &branch {
            Some(name) => format!("Pulled origin/{name} into the current branch"),
            None => "Pulled current branch".into(),
        };
        Ok(RepoActionResult {
            path: repo.path,
            ok: output.success,
            message: fallback_message(&git::combined_message(&output), output.success, &fallback),
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
pub async fn checkout_repo(
    state: State<'_, AppState>,
    group_id: String,
    repo_id: String,
    target: String,
    fallbacks: Vec<String>,
) -> Result<RepoActionResult, String> {
    let (git, group) = repo_list(&state, &group_id)?;
    let repo = group
        .repos
        .into_iter()
        .find(|entry| entry.id == repo_id)
        .ok_or_else(|| "Repository not found".to_string())?;
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
    tauri::async_runtime::spawn_blocking(move || {
        match git::checkout_with_fallbacks(&git, Path::new(&repo.path), &branches) {
            Ok(message) => Ok(RepoActionResult {
                path: repo.path,
                ok: true,
                message,
            }),
            Err(message) => Ok(RepoActionResult {
                path: repo.path,
                ok: false,
                message,
            }),
        }
    })
    .await
    .map_err(|err| err.to_string())?
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
pub fn discard_all_changes(state: State<AppState>, path: String) -> Result<(), String> {
    let git = require_git(&state)?;
    git::discard_all_changes(&git, Path::new(&path))
}

#[tauri::command]
pub fn stage_file(state: State<AppState>, path: String, file: String) -> Result<(), String> {
    let git = require_git(&state)?;
    git::stage_file(&git, Path::new(&path), &file)
}

#[tauri::command]
pub fn stage_all(state: State<AppState>, path: String) -> Result<(), String> {
    let git = require_git(&state)?;
    git::stage_all(&git, Path::new(&path))
}

#[tauri::command]
pub fn unstage_file(state: State<AppState>, path: String, file: String) -> Result<(), String> {
    let git = require_git(&state)?;
    git::unstage_file(&git, Path::new(&path), &file)
}

#[tauri::command]
pub fn unstage_all(state: State<AppState>, path: String) -> Result<(), String> {
    let git = require_git(&state)?;
    git::unstage_all(&git, Path::new(&path))
}

#[tauri::command]
pub fn file_diff(
    state: State<AppState>,
    path: String,
    file: String,
    staged: bool,
) -> Result<String, String> {
    let git = require_git(&state)?;
    git::file_diff(&git, Path::new(&path), &file, staged)
}

fn status_from_live(repo: &RepoEntry, live: Result<git::LiveStatus, String>) -> RepoStatus {
    match live {
        Ok(status) => RepoStatus {
            id: repo.id.clone(),
            name: git::folder_name(&repo.path),
            path: repo.path.clone(),
            branch: status.branch,
            ahead: status.ahead,
            behind: status.behind,
            dirty: status.dirty,
            insertions: status.insertions,
            deletions: status.deletions,
            changed_files: status.changed_files,
        },
        Err(err) => RepoStatus {
            id: repo.id.clone(),
            name: git::folder_name(&repo.path),
            path: repo.path.clone(),
            branch: format!("error: {err}"),
            ahead: 0,
            behind: 0,
            dirty: false,
            insertions: 0,
            deletions: 0,
            changed_files: 0,
        },
    }
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
