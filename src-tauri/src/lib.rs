mod commands;
mod git;
mod models;
mod persist;

use commands::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let data = persist::load(&handle).unwrap_or_default();
            app.manage(AppState {
                data: Mutex::new(data),
                git: git::resolve_git_binary(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::create_group,
            commands::rename_group,
            commands::delete_group,
            commands::toggle_group,
            commands::update_group_settings,
            commands::update_app_settings,
            commands::add_repo,
            commands::remove_repo,
            commands::group_status,
            commands::refresh_repo,
            commands::pull_current,
            commands::pull_from_branch,
            commands::checkout_all,
            commands::log_graph,
            commands::working_tree,
            commands::file_diff,
            commands::request_notification_permission,
            commands::notify_user,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
