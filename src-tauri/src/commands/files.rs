#[tauri::command]
#[specta::specta]
pub fn open_file(path: String) -> Result<(), String> {
    tauri_plugin_opener::open_path(path, None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub fn reveal_in_folder(path: String) -> Result<(), String> {
    tauri_plugin_opener::reveal_item_in_dir(path).map_err(|e| e.to_string())
}