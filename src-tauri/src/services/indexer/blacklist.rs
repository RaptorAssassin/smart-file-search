use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

pub fn build_blacklist(
    app: &AppHandle,
    excluded_paths: Vec<String>,
) -> Result<HashSet<PathBuf>, Box<dyn std::error::Error>> {
    // Load hardcoded blacklist from data/blacklist.json
    let resource_path = app
        .path()
        .resolve(Path::new("data/blacklist.json5"), BaseDirectory::Resource)?;

    let json_content = std::fs::read_to_string(&resource_path)?;
    let raw_paths: HashSet<String> = json5::from_str(&json_content)?;
    let mut blacklist: HashSet<PathBuf> = raw_paths.into_iter().map(PathBuf::from).collect();

    // Load user-defined blacklist from config
    blacklist.extend(excluded_paths.iter().map(PathBuf::from));

    Ok(blacklist)
}

pub fn should_skip_path(
    path: &PathBuf,
    //app_handle: &AppHandle,
    blacklist: &HashSet<PathBuf>,
) -> bool {
    // let app_state = app_handle.state::<crate::AppState>();
    // let blacklist = &app_state.blacklist;

    if blacklist.contains(path) {
        println!("Skipping blacklisted path: {:?}", path);
        return true;
    }

    blacklist.iter().any(|b| {
        path.components()
            .any(|comp| comp.as_os_str() == b.as_os_str())
    });

    println!("Path is not blacklisted: {:?}", path);
    false
}
