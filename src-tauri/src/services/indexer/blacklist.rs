use blake3::hazmat::Mode::Hash;
use serde::Deserialize;
use specta::Type;
use std::collections::HashSet;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::App;
use tauri::{AppHandle, Manager};

#[derive(Debug, Deserialize, Clone, Type)]
pub struct BlacklistConfig {
    pub excluded_folders: Vec<String>,
    pub excluded_extensions: Vec<String>,
    pub excluded_path_patterns: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Type)]
pub struct Blacklist {
    folder_names: HashSet<String>,
    extensions: HashSet<String>,
    path_patterns: HashSet<String>,
}

impl Blacklist {
    pub fn new(
        app: &AppHandle,
        excluded_folders: Vec<String>,
        excluded_extensions: Vec<String>,
        excluded_path_patterns: Vec<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Load hardcoded blacklist from data/blacklist.json
        let resource_path = app
            .path()
            .resolve(Path::new("data/blacklist.json5"), BaseDirectory::Resource)?;

        let json_content = std::fs::read_to_string(&resource_path)?;
        let raw_paths: HashSet<String> = json5::from_str(&json_content)?;

        let mut excluded_folders: HashSet<String> = excluded_folders.into_iter().collect();
        let mut excluded_extensions: HashSet<String> = excluded_extensions.into_iter().collect();
        let mut excluded_path_patterns: HashSet<String> =
            excluded_path_patterns.into_iter().collect();

        // Load user-defined blacklist from config
        excluded_folders.extend(
            raw_paths
                .iter()
                .filter(|path| Path::new(path).is_dir())
                .cloned(),
        );
        excluded_extensions.extend(
            raw_paths
                .iter()
                .filter(|path| Path::new(path).is_file())
                .map(|path| {
                    Path::new(path)
                        .extension()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                })
                .collect::<HashSet<String>>(),
        );
        excluded_path_patterns.extend(
            raw_paths
                .iter()
                .filter(|path| Path::new(path).is_file())
                .cloned(),
        );

        Ok(Blacklist {
            folder_names: excluded_folders,
            extensions: excluded_extensions,
            path_patterns: excluded_path_patterns,
        })
    }
}

pub fn build_blacklist(app: &AppHandle) -> Result<Blacklist, Box<dyn Error>> {
    // Load hardcoded blacklist from data/blacklist.json
    let resource_path = app
        .path()
        .resolve(Path::new("data/blacklist.json5"), BaseDirectory::Resource)?;

    Ok(Blacklist::new(app, vec![], vec![], vec![])?)
}

pub fn should_skip_path(
    path: &PathBuf,
    //app_handle: &AppHandle,
    blacklist: &Blacklist,
) -> bool {
    // let app_state = app_handle.state::<crate::AppState>();
    // let blacklist = &app_state.blacklist;

    if path.is_dir() {
        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if blacklist.folder_names.contains(name) {
                    return true;
                }
            }
        }
    }

    if blacklist.extensions.contains(
        &path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    ) {
        println!("Skipping blacklisted extension: {:?}", path);
        return true;
    }

    for pattern in &blacklist.path_patterns {
        if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
            if glob_pattern.matches_path(path) {
                println!(
                    "Skipping blacklisted path pattern: {:?} matches {:?}",
                    path, pattern
                );
                return true;
            }
        }
    }

    println!("Path is not blacklisted: {:?}", path);
    false
}
