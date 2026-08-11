use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;
use std::path::Path;
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
    case_insensitive: bool,
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

        let case_insensitive = cfg!(windows) || cfg!(target_os = "macos");

        let folder_names: HashSet<String> = json_config
            .folder_names
            .into_iter()
            .map(|name| name.to_lowercase())
            .chain(
                config
                    .excluded_folders
                    .into_iter()
                    .map(|name| name.to_lowercase()),
            )
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
            let mut normalized = pattern.replace('\\', "/");
            if normalized.len() >= 2
                && normalized.as_bytes()[0].is_ascii_alphabetic()
                && normalized.as_bytes()[1] == b':'
            {
                normalized = format!("?:{}", &normalized[2..]);
            }
            if let Ok(glob) = globset::GlobBuilder::new(&normalized)
                .case_insensitive(case_insensitive)
                .build()
            {
                glob_builder.add(glob);
            }
        }

        let compiled_path_patterns = glob_builder.build()?;

        Ok(Blacklist {
            folder_names,
            extensions,
            path_patterns: compiled_path_patterns,
            case_insensitive,
        })
    }

    pub fn should_skip_path(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.extensions.contains(&ext.to_lowercase()) {
                return true;
            }
        }

        let is_dir = path.is_dir();
        let components: Vec<_> = path.components().collect();
        for (i, component) in components.iter().enumerate() {
            if !is_dir && i + 1 == components.len() {
                break;
            }
            if let Some(name) = component.as_os_str().to_str() {
                let name = if self.case_insensitive {
                    name.to_lowercase()
                } else {
                    name.to_owned()
                };
                if self.folder_names.contains(&name) {
                    return true;
                }
            }
        }

        self.path_patterns.is_match(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blacklist(folders: &[&str], extensions: &[&str], patterns: &[&str]) -> Blacklist {
        let mut glob_builder = globset::GlobSetBuilder::new();
        for pattern in patterns {
            let mut normalized = pattern.replace('\\', "/");
            if normalized.len() >= 2
                && normalized.as_bytes()[0].is_ascii_alphabetic()
                && normalized.as_bytes()[1] == b':'
            {
                normalized = format!("?:{}", &normalized[2..]);
            }
            glob_builder
                .add(globset::GlobBuilder::new(&normalized).build().unwrap());
        }

        Blacklist {
            folder_names: folders.iter().map(|s| s.to_lowercase()).collect(),
            extensions: extensions
                .iter()
                .map(|s| s.trim_start_matches('.').to_lowercase())
                .collect(),
            path_patterns: glob_builder.build().unwrap(),
            case_insensitive: cfg!(windows) || cfg!(target_os = "macos"),
        }
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("sfs_bl_test_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn skips_matching_extension() {
        let bl = blacklist(&[], &["tmp"], &[]);
        assert!(bl.should_skip_path(Path::new("C:\\Users\\me\\downloads\\file.tmp")));
    }

    #[test]
    fn extension_match_is_case_insensitive() {
        let bl = blacklist(&[], &["tmp"], &[]);
        assert!(bl.should_skip_path(Path::new("C:\\Users\\me\\file.TMP")));
    }

    #[test]
    fn extension_config_with_leading_dot_matches() {
        let bl = blacklist(&[], &[".tmp"], &[]);
        assert!(bl.should_skip_path(Path::new("C:\\Users\\me\\file.tmp")));
    }

    #[test]
    fn does_not_skip_unknown_extension() {
        let bl = blacklist(&[], &["tmp"], &[]);
        assert!(!bl.should_skip_path(Path::new("C:\\Users\\me\\file.txt")));
    }

    #[test]
    fn skips_path_with_blacklisted_folder() {
        let bl = blacklist(&["node_modules"], &[], &[]);
        assert!(bl.should_skip_path(Path::new("C:\\proj\\node_modules\\lib\\main.rs")));
    }

    #[test]
    fn skips_blacklisted_folder_itself() {
        let root = temp_dir("folder_itself");
        let target = root.join("node_modules");
        std::fs::create_dir_all(&target).unwrap();
        let bl = blacklist(&["node_modules"], &[], &[]);
        assert!(bl.should_skip_path(&target));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn does_not_skip_file_named_like_blacklisted_folder() {
        let root = temp_dir("file_named");
        let file = root.join("build");
        std::fs::write(&file, "x").unwrap();
        let bl = blacklist(&["build"], &[], &[]);
        assert!(!bl.should_skip_path(&file));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn folder_match_is_case_insensitive_on_windows_macos() {
        let root = temp_dir("case");
        let target = root.join("NODE_MODULES").join("lib");
        std::fs::create_dir_all(&target).unwrap();
        let bl = blacklist(&["node_modules"], &[], &[]);
        let expected = cfg!(windows) || cfg!(target_os = "macos");
        assert_eq!(bl.should_skip_path(&target), expected);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn drive_pattern_matches_any_drive_letter() {
        let bl = blacklist(&[], &[], &["C:/Windows/**"]);
        assert!(bl.should_skip_path(Path::new("D:/Windows/System32/foo.dll")));
    }

    #[test]
    fn does_not_skip_path_without_blacklisted_folder() {
        let bl = blacklist(&["node_modules"], &[], &[]);
        assert!(!bl.should_skip_path(Path::new("C:\\proj\\src\\main.rs")));
    }

    #[test]
    fn skips_path_matching_glob_pattern() {
        let bl = blacklist(&[], &[], &["**/build/**"]);
        assert!(bl.should_skip_path(Path::new("build/x/y.txt")));
    }

    #[test]
    fn does_not_skip_path_not_matching_glob_pattern() {
        let bl = blacklist(&[], &[], &["**/build/**"]);
        assert!(!bl.should_skip_path(Path::new("src/main.rs")));
    }

    #[test]
    fn empty_blacklist_skips_nothing() {
        let bl = blacklist(&[], &[], &[]);
        assert!(!bl.should_skip_path(Path::new("C:\\Users\\me\\file.txt")));
    }
}
