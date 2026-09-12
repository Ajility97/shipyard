mod commands;
mod git;
mod menu;
mod models;
mod persist;
mod window_state;

use commands::AppState;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .menu(|handle| menu::build(handle))
        .on_menu_event(|app, event| {
            menu::handle_event(app, event.id().as_ref());
        })
        .setup(|app| {
            let handle = app.handle().clone();
            let data = persist::load(&handle).unwrap_or_default();
            let bounds = data.window.clone();
            app.manage(AppState {
                data: Mutex::new(data),
                git: git::resolve_git_binary(),
            });
            if let (Some(window), Some(bounds)) = (app.get_webview_window("main"), bounds) {
                window_state::apply(&window, &bounds);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            window_state::handle_event(window, event);
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::create_group,
            commands::rename_group,
            commands::delete_group,
            commands::toggle_group,
            commands::set_all_groups_expanded,
            commands::update_group_settings,
            commands::update_app_settings,
            commands::update_files_pane_width,
            commands::update_diff_mode,
            commands::replace_app_data,
            commands::add_repo,
            commands::add_standalone_repo,
            commands::remove_standalone_repo,
            commands::standalone_status,
            commands::remove_repo,
            commands::group_status,
            commands::refresh_repo,
            commands::pull_repo,
            commands::pull_current,
            commands::pull_from_branch,
            commands::checkout_repo,
            commands::checkout_all,
            commands::log_graph,
            commands::working_tree,
            commands::discard_all_changes,
            commands::file_diff,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
