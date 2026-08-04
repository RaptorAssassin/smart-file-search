use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::Manager;

pub fn build_blacklist(app: &tauri::App) -> Result<HashSet<PathBuf>, Box<dyn std::error::Error>> {
    let mut blacklist: HashSet<PathBuf> = HashSet::new();

    let resource_path = app
        .path()
        .resolve("/data/blacklist.json", BaseDirectory::Resource)?;

    let json_content = std::fs::read_to_string(&resource_path)?;

    let blacklist: HashSet<PathBuf> = serde_json::from_str(&json_content)?;

    Ok(blacklist)
}

fn should_skip_path(path: &Path, blacklist: &HashSet<PathBuf>) -> bool {
    for blacklisted_path in blacklist {
        if blacklist.contains(blacklisted_path) {
            return true;
        }
    }
    false
}
