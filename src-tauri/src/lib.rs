mod commands;
mod services;

use std::path::PathBuf;
use tauri::Manager;

use crate::commands::{
    config::config::{get_config, save_config, ConfigManager},
    debug::{get_database_path, get_database_size},
};
use tauri_specta::{collect_commands, Builder};

#[cfg(debug_assertions)]
use specta_typescript::Typescript;

pub struct AppState {
    pub db_path: PathBuf,
    pub config_manager: ConfigManager,
}

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::new()
        .commands(collect_commands![
            get_database_path,
            get_database_size,
            get_config,
            save_config
        ])
        .dangerously_cast_bigints_to_number()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();
    let bindings_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../src/bindings/bindings.ts");

    #[cfg(debug_assertions)]
    builder
        .export(Typescript::default(), &bindings_path)
        .expect("failed to export types");

    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(builder.invoke_handler())
        .setup(move |app| {
            let app_handle = app.handle();

            let (conn, db_path) = services::database::init_database(app_handle)?;

            let config_manager = ConfigManager::load_config(app_handle)?;

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
