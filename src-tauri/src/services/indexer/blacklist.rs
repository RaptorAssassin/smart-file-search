use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};

#[derive(Debug, Deserialize, Serialize, Clone, Type)]
pub struct BlacklistConfig {
    pub excluded_folders: Vec<String>,
    pub excluded_extensions: Vec<String>,
    pub excluded_path_patterns: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct BlacklistJson {
    pub folder_names: Vec<String>,
    pub extensions: Vec<String>,
    pub path_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Blacklist {
    folder_names: HashSet<String>,
    extensions: HashSet<String>,
    path_patterns: globset::GlobSet,
}

impl Blacklist {
    /// Builds a new Blacklist by merging the hardcoded blacklist from the JSON file with the custom user configuration.
    pub fn new(
        app: &AppHandle,
        config: BlacklistConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let resource_path = app
            .path()
            .resolve(Path::new("data/blacklist.json5"), BaseDirectory::Resource)?;

        let json_content = std::fs::read_to_string(&resource_path)?;
        let json_config: BlacklistJson = json5::from_str(&json_content)?;

        let folder_names: HashSet<String> = json_config
            .folder_names
            .into_iter()
            .chain(config.excluded_folders)
            .collect();

        let extensions: HashSet<String> = json_config
            .extensions
            .into_iter()
            .map(|ext| ext.trim_start_matches('.').to_lowercase())
            .chain(
                config
                    .excluded_extensions
                    .into_iter()
                    .map(|e| e.trim_start_matches('.').to_lowercase()),
            )
            .collect();

        let path_patterns: HashSet<String> = json_config
            .path_patterns
            .into_iter()
            .chain(config.excluded_path_patterns)
            .collect();

        let mut glob_builder = globset::GlobSetBuilder::new();
        for pattern in &path_patterns {
            let normalized = pattern.replace('\\', "/");
            if let Ok(glob) = globset::Glob::new(&normalized) {
                glob_builder.add(glob);
            }
        }

        let compiled_path_patterns = glob_builder.build()?;

        Ok(Blacklist {
            folder_names,
            extensions,
            path_patterns: compiled_path_patterns,
        })
    }

    pub fn should_skip_path(&self, path: &PathBuf) -> bool {
        // Extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.extensions.contains(&ext.to_lowercase()) {
                return true;
            }
        }

        // Folder Name

        // Glob Pattern

        false
    }
}

pub fn should_skip_path(path: &PathBuf, blacklist: &Blacklist) -> bool {
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

    if blacklist.path_patterns.is_match(path) {
        println!("Skipping blacklisted path pattern: {:?}", path);
        return true;
    }

    println!("Path is not blacklisted: {:?}", path);
    false
}
