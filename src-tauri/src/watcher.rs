use std::collections::HashMap;
use std::path::{Component, Path};
use std::sync::Mutex;
use std::time::Duration;

use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

pub const REPO_FILES_CHANGED_EVENT: &str = "repo-files-changed";

const DEBOUNCE: Duration = Duration::from_millis(200);

const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    ".venv",
    "venv",
    "__pycache__",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
];

const IGNORED_GIT_DIRS: &[&str] = &["objects", "logs", "hooks"];

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFilesChanged {
    pub path: String,
    pub git: bool,
}

struct WatchEntry {
    _debouncer: Debouncer<RecommendedWatcher>,
    refs: usize,
}

#[derive(Default)]
pub struct RepoWatcherState {
    watches: Mutex<HashMap<String, WatchEntry>>,
}

fn watch_key(path: &str) -> String {
    path.trim().trim_end_matches('/').to_string()
}

fn path_file_name(path: &Path) -> Option<&str> {
    path.file_name().and_then(|name| name.to_str())
}

fn is_editor_junk(name: &str) -> bool {
    name == ".DS_Store" || name.ends_with('~') || name.ends_with(".swp") || name.ends_with(".swo")
}

fn is_git_lock(name: &str) -> bool {
    name.ends_with(".lock")
}

/// Classify a filesystem path relative to a repo.
/// `Some(true)` is git metadata, `Some(false)` is a worktree change, `None` is noise.
pub fn classify_change(repo: &Path, changed: &Path) -> Option<bool> {
    let relative = if changed == repo {
        return Some(false);
    } else {
        changed.strip_prefix(repo).ok()?
    };

    let mut components = relative.components().peekable();
    let first = match components.next()? {
        Component::Normal(name) => name.to_str()?,
        _ => return Some(false),
    };

    if first == ".git" {
        let Some(Component::Normal(second)) = components.next() else {
            return Some(true);
        };
        let second = second.to_str()?;
        if IGNORED_GIT_DIRS.contains(&second) {
            return None;
        }
        if is_git_lock(second) && components.peek().is_none() {
            return None;
        }
        if let Some(name) = path_file_name(relative) {
            if is_git_lock(name) || is_editor_junk(name) {
                return None;
            }
        }
        return Some(true);
    }

    if IGNORED_DIRS.contains(&first) {
        return None;
    }
    for component in components {
        if let Component::Normal(name) = component {
            if let Some(name) = name.to_str() {
                if IGNORED_DIRS.contains(&name) {
                    return None;
                }
            }
        }
    }
    if let Some(name) = path_file_name(relative) {
        if is_editor_junk(name) {
            return None;
        }
    }
    Some(false)
}

fn emit_changes(app: &AppHandle, repo_path: &str, events: DebounceEventResult) {
    let Ok(events) = events else {
        return;
    };
    let repo = Path::new(repo_path);
    let mut any = false;
    let mut git = false;
    for event in events {
        if let Some(is_git) = classify_change(repo, &event.path) {
            any = true;
            git |= is_git;
        }
    }
    if any {
        let _ = app.emit(
            REPO_FILES_CHANGED_EVENT,
            RepoFilesChanged {
                path: repo_path.to_string(),
                git,
            },
        );
    }
}

#[tauri::command]
pub fn watch_repo(
    app: AppHandle,
    state: State<RepoWatcherState>,
    path: String,
) -> Result<(), String> {
    let key = watch_key(&path);
    if key.is_empty() {
        return Err("Repository path cannot be empty".into());
    }
    if !Path::new(&key).is_dir() {
        return Err("Repository path is not a directory".into());
    }

    let mut watches = state
        .watches
        .lock()
        .map_err(|_| "Could not lock file watchers.".to_string())?;
    if let Some(entry) = watches.get_mut(&key) {
        entry.refs += 1;
        return Ok(());
    }

    let watched = key.clone();
    let handle = app.clone();
    let mut debouncer = new_debouncer(DEBOUNCE, move |result: DebounceEventResult| {
        emit_changes(&handle, &watched, result);
    })
    .map_err(|err| format!("Could not watch repository: {err}"))?;

    debouncer
        .watcher()
        .watch(Path::new(&key), RecursiveMode::Recursive)
        .map_err(|err| format!("Could not watch repository: {err}"))?;

    watches.insert(
        key,
        WatchEntry {
            _debouncer: debouncer,
            refs: 1,
        },
    );
    Ok(())
}

#[tauri::command]
pub fn unwatch_repo(state: State<RepoWatcherState>, path: String) -> Result<(), String> {
    let key = watch_key(&path);
    if key.is_empty() {
        return Ok(());
    }
    let mut watches = state
        .watches
        .lock()
        .map_err(|_| "Could not lock file watchers.".to_string())?;
    let Some(entry) = watches.get_mut(&key) else {
        return Ok(());
    };
    entry.refs = entry.refs.saturating_sub(1);
    if entry.refs == 0 {
        watches.remove(&key);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn classifies_worktree_and_git_paths() {
        let repo = Path::new("/repo");
        assert_eq!(classify_change(repo, Path::new("/repo")), Some(false));
        assert_eq!(
            classify_change(repo, Path::new("/repo/src/App.vue")),
            Some(false)
        );
        assert_eq!(
            classify_change(repo, Path::new("/repo/.git/index")),
            Some(true)
        );
        assert_eq!(
            classify_change(repo, Path::new("/repo/.git/HEAD")),
            Some(true)
        );
        assert_eq!(
            classify_change(repo, Path::new("/repo/.git/refs/heads/main")),
            Some(true)
        );
        assert_eq!(
            classify_change(repo, Path::new("/repo/Cargo.lock")),
            Some(false)
        );
    }

    #[test]
    fn ignores_noisy_paths() {
        let repo = Path::new("/repo");
        assert_eq!(
            classify_change(repo, Path::new("/repo/node_modules/foo/index.js")),
            None
        );
        assert_eq!(
            classify_change(repo, Path::new("/repo/.git/objects/pack/foo.pack")),
            None
        );
        assert_eq!(
            classify_change(repo, Path::new("/repo/.git/logs/HEAD")),
            None
        );
        assert_eq!(
            classify_change(repo, Path::new("/repo/.git/index.lock")),
            None
        );
        assert_eq!(
            classify_change(repo, Path::new("/repo/src/.DS_Store")),
            None
        );
        assert_eq!(
            classify_change(repo, Path::new("/tmp/other/src/App.vue")),
            None
        );
    }
}
