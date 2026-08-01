mod commands;
mod services;

use std::path::PathBuf;
use tauri::Manager;
use tauri_specta::collect_commands;

use crate::commands::config::config::get_config;
use crate::commands::config::config::ConfigManager;

pub struct AppState {
    pub db_path: PathBuf,
    pub config_manager: ConfigManager,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let app_handle = app.handle();

            let (conn, db_path) = services::database::init_database(app_handle)?;

            let config_manager = commands::config::ConfigManager::load_config(app_handle)?;

            app.manage(services::database::DbState {
                conn: std::sync::Mutex::new(conn),
            });

            app.manage(AppState {
                db_path,
                config_manager,
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::debug::get_database_path,
            commands::debug::get_database_size,
            commands::config::config::get_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    let builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(collect_commands![get_config,]);
}
