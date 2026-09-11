use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::models::AppData;

pub fn data_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|err| format!("Could not resolve the app data directory: {err}"))?;
    fs::create_dir_all(&dir).map_err(|err| format!("Could not create the app data directory: {err}"))?;
    Ok(dir.join("groups.json"))
}

pub fn load(app: &AppHandle) -> Result<AppData, String> {
    let path = data_path(app)?;
    if !path.exists() {
        return Ok(AppData::default());
    }

    let raw = fs::read_to_string(&path)
        .map_err(|err| format!("Could not read groups.json: {err}"))?;
    serde_json::from_str(&raw).map_err(|err| format!("Could not parse groups.json: {err}"))
}

pub fn save(app: &AppHandle, data: &AppData) -> Result<(), String> {
    let path = data_path(app)?;
    let raw = serde_json::to_string_pretty(data)
        .map_err(|err| format!("Could not serialize groups.json: {err}"))?;
    fs::write(&path, raw).map_err(|err| format!("Could not write groups.json: {err}"))
}
