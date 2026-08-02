use crate::AppState;
use std::fs;

#[tauri::command]
#[specta::specta]
pub fn get_database_path(state: tauri::State<AppState>) -> String {
    state.db_path.display().to_string()
}

#[tauri::command]
#[specta::specta]
pub fn get_database_size(state: tauri::State<AppState>) -> Result<u64, String> {
    let db_path = state.db_path.clone();

    let metadata = fs::metadata(db_path).map_err(|e| e.to_string())?;

    Ok(metadata.len())
}
