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
    pub excluded_paths: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: SettingsConfig { theme: Theme::Dark },
            indexing: IndexingConfig {
                excluded_paths: vec!["C:\\Windows".into(), "C:\\Program Files".into()],
            },
        }
    }
}
