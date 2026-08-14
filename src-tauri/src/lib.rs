pub mod commands;
pub mod db;
pub mod error;
pub mod state;

use std::fs;

use db::Database;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_directory = app.path().app_data_dir().map_err(|_| {
                error::AppError::new(
                    "app_data_unavailable",
                    "Unable to locate the application data directory.",
                )
            })?;
            fs::create_dir_all(&app_data_directory).map_err(|_| {
                error::AppError::new(
                    "app_data_unavailable",
                    "Unable to prepare the application data directory.",
                )
            })?;
            let database = Database::open(app_data_directory.join("library.sqlite3"))?;
            app.manage(AppState::new(database));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::backup::prepare_update_backup,
            commands::cover::set_series_cover,
            commands::cover::remove_series_cover_command,
            commands::library::get_dashboard,
            commands::library::list_series,
            commands::library::get_series_detail,
            commands::library::create_series_batch,
            commands::library::update_series_metadata,
            commands::library::add_volume,
            commands::library::update_collection_item,
            commands::library::update_volume_details,
            commands::library::find_volume_by_isbn,
            commands::library::delete_series,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
