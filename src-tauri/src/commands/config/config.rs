use anyhow::Result;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;
use tauri::AppHandle;
use tauri::Manager;

use crate::commands::config::models::Config;
use crate::services::indexer::blacklist::BlacklistConfig;
use crate::AppState;

pub struct ConfigManager {
    pub config: RwLock<Config>,
    pub blacklist: RwLock<BlacklistConfig>,
    pub config_path: PathBuf,
}

impl ConfigManager {
    pub fn load_config(app_handle: &AppHandle) -> Result<Self> {
        let config_path = Self::get_config_path(app_handle)?;

        Self::ensure_config_exists(&config_path)?;

        let config = Self::read_config(&config_path)?;

        let blacklist = BlacklistConfig {
            excluded_folders: config.indexing.excluded_folders.clone(),
            excluded_extensions: config.indexing.excluded_extensions.clone(),
            excluded_path_patterns: config.indexing.excluded_path_patterns.clone(),
        };

        Ok(Self {
            config: RwLock::new(config),
            blacklist: RwLock::new(blacklist),
            config_path,
        })
    }

    pub fn save_config(&self, config: Config) -> Result<()> {
        *self.config.write().unwrap() = config.clone();

        *self.blacklist.write().unwrap() = BlacklistConfig {
            excluded_folders: config.indexing.excluded_folders.clone(),
            excluded_extensions: config.indexing.excluded_extensions.clone(),
            excluded_path_patterns: config.indexing.excluded_path_patterns.clone(),
        };

        let json = serde_json::to_string_pretty(&config)?;
        fs::write(&self.config_path, json)?;

        Ok(())
    }

    pub fn get_config_path(app_handle: &AppHandle) -> Result<PathBuf> {
        let app_dir = app_handle.path().app_config_dir()?;
        let config_path = app_dir.join("config.json");
        Ok(config_path)
    }

    pub fn ensure_config_exists(config_path: &PathBuf) -> Result<()> {
        if !config_path.exists() {
            let default_config = Config::default();
            let config_json = serde_json::to_string_pretty(&default_config)?;
            fs::create_dir_all(config_path.parent().unwrap())?;
            fs::write(config_path, config_json)?;
        }
        Ok(())
    }

    pub fn read_config(config_path: &Path) -> Result<Config> {
        let config_json = fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&config_json)?;
        Ok(config)
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_config(state: tauri::State<AppState>) -> Config {
    state.config_manager.config.read().unwrap().clone()
}
#[tauri::command]
#[specta::specta]
pub fn save_config(state: tauri::State<AppState>, config: Config) -> Result<(), String> {
    state
        .config_manager
        .save_config(config)
        .map_err(|e| e.to_string())
}
