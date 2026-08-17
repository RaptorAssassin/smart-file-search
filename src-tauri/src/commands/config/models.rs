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
    pub ai: AiConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(default)]
pub struct AiConfig {
    pub provider: AiProvider,
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_model_custom: bool,
    pub custom_endpoint: String,
    pub custom_api_key: String,
    pub custom_model: String,
    pub embeddings_enabled: bool,
    pub embed_model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
pub enum AiProvider {
    #[default]
    Ollama,
    Custom,
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
    pub ignore_hidden: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            settings: SettingsConfig {
                theme: Theme::System,
                disable_keyboard_shortcut_hints: Some(false),
                enable_debug_menu: Some(true),
                ai: AiConfig {
                    provider: AiProvider::Ollama,
                    ollama_url: "http://localhost:11434".to_string(),
                    ollama_model: "gemma3:4b".to_string(),
                    ollama_model_custom: false,
                    custom_endpoint: String::new(),
                    custom_api_key: String::new(),
                    custom_model: String::new(),
                    embeddings_enabled: true,
                    embed_model: "nomic-embed-text".to_string(),
                },
            },
            indexing: IndexingConfig {
                excluded_folders: vec![],
                excluded_extensions: vec![],
                excluded_path_patterns: vec![],
                ignore_hidden: true,
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
        assert!(config.indexing.ignore_hidden);
    }

    #[test]
    fn default_ai_config_has_expected_values() {
        let ai = Config::default().settings.ai;
        assert_eq!(ai.provider, AiProvider::Ollama);
        assert_eq!(ai.ollama_url, "http://localhost:11434");
        assert_eq!(ai.ollama_model, "gemma3:4b");
        assert!(!ai.ollama_model_custom);
        assert_eq!(ai.embed_model, "nomic-embed-text");
        assert!(ai.embeddings_enabled);
        assert!(ai.custom_endpoint.is_empty());
        assert!(ai.custom_api_key.is_empty());
        assert!(ai.custom_model.is_empty());
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

    #[test]
    fn ai_provider_serializes_as_plain_string() {
        assert_eq!(
            serde_json::to_string(&AiProvider::Ollama).unwrap(),
            "\"Ollama\""
        );
        assert_eq!(
            serde_json::to_string(&AiProvider::Custom).unwrap(),
            "\"Custom\""
        );
        let provider: AiProvider = serde_json::from_str("\"Custom\"").unwrap();
        assert_eq!(provider, AiProvider::Custom);
    }
}
