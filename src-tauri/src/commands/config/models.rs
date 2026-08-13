use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct Config {
    pub settings: SettingsConfig,
    pub indexing: IndexingConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(default)]
pub struct SettingsConfig {
    pub theme: Theme,
    pub disable_keyboard_shortcut_hints: Option<bool>,
    #[serde(default)]
    pub enable_debug_menu: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = Config::default();
        assert_eq!(config.settings.theme, Theme::System);
        assert_eq!(config.settings.disable_keyboard_shortcut_hints, Some(false));
        assert_eq!(config.settings.enable_debug_menu, Some(true));
        assert!(config.indexing.excluded_folders.is_empty());
        assert!(config.indexing.excluded_extensions.is_empty());
        assert!(config.indexing.excluded_path_patterns.is_empty());
    }

    #[test]
    fn empty_json_defaults_to_default_config() {
        let config: Config = serde_json::from_str("{}").unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn config_roundtrips_through_json() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let decoded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, config);
    }

    #[test]
    fn theme_serializes_as_plain_string() {
        assert_eq!(serde_json::to_string(&Theme::Dark).unwrap(), "\"Dark\"");
        assert_eq!(serde_json::to_string(&Theme::System).unwrap(), "\"System\"");
        let theme: Theme = serde_json::from_str("\"Light\"").unwrap();
        assert_eq!(theme, Theme::Light);
    }
}
