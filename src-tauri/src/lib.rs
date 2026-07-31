use std::path::PathBuf;
use tauri::Manager;

mod commands;
mod services;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub struct AppState {
    pub db_path: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            let (conn, db_path) = services::database::init_database(app.handle())?;

            app.manage(services::database::DbState {
                conn: std::sync::Mutex::new(conn),
            });

            app.manage(AppState { db_path });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![commands::debug::get_database_path, commands::debug::get_database_size])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
