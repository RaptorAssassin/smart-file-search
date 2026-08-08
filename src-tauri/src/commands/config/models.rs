use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Config {
    pub settings: SettingsConfig,
    pub indexing: IndexingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(default)]
pub struct SettingsConfig {
    pub theme: Theme,
    pub disable_keyboard_shortcut_hints: Option<bool>,
    #[serde(default)]
    pub enable_debug_menu: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(default)]
pub struct IndexingConfig {
    pub excluded_folders: Vec<String>,
    pub excluded_extensions: Vec<String>,
    pub excluded_path_patterns: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: SettingsConfig {
                theme: Theme::System,
                disable_keyboard_shortcut_hints: Some(false),
                enable_debug_menu: Some(true),
            },
            indexing: IndexingConfig {
                excluded_folders: vec![],
                excluded_extensions: vec![],
                excluded_path_patterns: vec![],
            },
        }
    }
}
