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
}

impl Blacklist {
    /// Builds a Blacklist from the bundled JSON blacklist and the user's config.
    pub fn new(
        app: &AppHandle,
        config: BlacklistConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let resource_path = app
            .path()
            .resolve(Path::new("data/blacklist.json5"), BaseDirectory::Resource)?;

        let json_content = std::fs::read_to_string(&resource_path)?;
        let json_config: BlacklistJson = json5::from_str(&json_content)?;

        let folder_names: Vec<String> = json_config
            .folder_names
            .into_iter()
            .chain(config.excluded_folders)
            .collect();

        let extensions: Vec<String> = json_config
            .extensions
            .into_iter()
            .chain(config.excluded_extensions)
            .collect();

        let path_patterns: Vec<String> = json_config
            .path_patterns
            .into_iter()
            .chain(config.excluded_path_patterns)
            .collect();

        Self::from_lists(folder_names, extensions, path_patterns)
    }

    /// Compiles raw lists into a matchable Blacklist, normalizing case for real on-disk paths.
    pub fn from_lists(
        folder_names: Vec<String>,
        extensions: Vec<String>,
        path_patterns: Vec<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let folder_names: HashSet<String> = folder_names
            .into_iter()
            .map(|name| name.to_lowercase())
            .collect();

        let extensions: HashSet<String> = extensions
            .into_iter()
            .map(|ext| ext.trim_start_matches('.').to_lowercase())
            .collect();

        let mut glob_builder = globset::GlobSetBuilder::new();
        for pattern in &path_patterns {
            let normalized = pattern.replace('\\', "/");
            if let Ok(glob) = globset::GlobBuilder::new(&normalized)
                .case_insensitive(true)
                .build()
            {
                glob_builder.add(glob);
            }
        }

        Ok(Blacklist {
            folder_names,
            extensions,
            path_patterns: glob_builder.build()?,
        })
    }

    /// Checks if a path should be skipped based on the blacklist rules.
    pub fn should_skip_path(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.extensions.contains(&ext.to_lowercase()) {
                return true;
            }
        }

        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if self.folder_names.contains(&name.to_lowercase()) {
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
        Blacklist::from_lists(
            folders.iter().map(|s| s.to_string()).collect(),
            extensions.iter().map(|s| s.to_string()).collect(),
            patterns.iter().map(|s| s.to_string()).collect(),
        )
        .unwrap()
    }

    fn bundled_blacklist() -> Blacklist {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/blacklist.json5");
        let content = std::fs::read_to_string(&path).unwrap();
        let cfg: BlacklistJson = json5::from_str(&content).unwrap();
        Blacklist::from_lists(cfg.folder_names, cfg.extensions, cfg.path_patterns).unwrap()
    }

    #[test]
    fn bundled_blacklist_json_is_valid() {
        let _ = bundled_blacklist();
    }

    #[test]
    fn bundled_blacklist_skips_system_and_trash_paths() {
        let bl = bundled_blacklist();
        for p in [
            "C:\\Windows\\System32\\ntdll.dll",
            "C:\\Windows\\WinSxS\\amd64_microsoft_1.dll",
            "C:\\Program Files\\Common Files\\a.dll",
            "C:\\Program Files (x86)\\x\\y.exe",
            "C:\\ProgramData\\foo\\bar.dat",
            "C:\\Users\\Karl\\AppData\\Local\\Temp\\a.tmp",
            "C:\\Users\\Karl\\AppData\\Roaming\\Mozilla\\prefs.js",
            "C:\\$Recycle.Bin\\S-1-5-21-1\\$R123",
            "C:\\Users\\Karl\\Documents\\node_modules\\lodash\\index.js",
            "C:\\Users\\Karl\\proj\\__pycache__\\x.pyc",
            "C:\\Users\\Karl\\proj\\.git\\config",
            "C:\\Users\\Karl\\proj\\dist\\bundle.js",
            "C:\\Users\\Karl\\Pictures\\thumbs.db",
            "C:\\Users\\Karl\\Desktop\\desktop.ini",
            "C:\\Users\\Karl\\Downloads\\installer.exe",
            "C:\\Users\\Karl\\Downloads\\patch.dmp",
            "C:\\Users\\Karl\\proj\\file.tmp",
        ] {
            assert!(bl.should_skip_path(Path::new(p)), "expected skip: {p}");
        }
    }

    #[test]
    fn bundled_blacklist_skips_drive_less_walker_paths() {
        let bl = bundled_blacklist();
        for p in [
            "/Windows\\System32\\ntdll.dll",
            "/XboxGames\\Minecraft for Windows\\Content\\data\\behavior_packs\\vanilla_1.19.10\\manifest.json",
            "/Drivers\\Realtek\\rtk_net\\setup.exe",
            "/Users\\Karl\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\share\\doc\\rust\\html\\core\\index.html",
            "/Users\\Karl\\.lunarclient\\textures\\assets\\lunar\\cosmetics\\cloaks\\snowarches.webp",
            "/Users\\Karl\\Documents\\Unity Projects\\test project\\Library\\PackageCache\\com.unity.visualscripting@b4d700247d4b\\Runtime\\EventUnit.cs",
            "/VC_RED.cab",
            "/.GamingRoot",
        ] {
            assert!(bl.should_skip_path(Path::new(p)), "expected skip: {p}");
        }
    }

    #[test]
    fn bundled_blacklist_keeps_user_files() {
        let bl = bundled_blacklist();
        for p in [
            "C:\\Users\\Karl\\Documents\\report.pdf",
            "C:\\Users\\Karl\\Code\\smart-file-search\\src\\main.ts",
            "C:\\Users\\Karl\\Pictures\\vacation\\IMG_2024.jpg",
            "C:\\Users\\Karl\\Documents\\node_modules-notes.txt",
            "/Users\\Karl\\Documents\\report.pdf",
            "/Users\\Karl\\Downloads\\vacation\\IMG_2024.jpg",
            "/Users\\Karl\\Code\\smart-file-search\\src\\main.ts",
            "/Games\\WarThunder\\content\\base\\res\\ships\\uk_frigate.grp",
        ] {
            assert!(!bl.should_skip_path(Path::new(p)), "expected keep: {p}");
        }
    }

    #[test]
    fn walker_filters_blacklisted_entries() {
        let root = std::env::temp_dir().join("sfs-blacklist-test");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        std::fs::create_dir_all(root.join("__pycache__")).unwrap();
        std::fs::write(root.join("src/main.ts"), "export const x = 1;").unwrap();
        std::fs::write(root.join("node_modules/pkg/index.js"), "require('x');").unwrap();
        std::fs::write(root.join("__pycache__/x.pyc"), b"\x00\x01").unwrap();
        std::fs::write(root.join("README.md"), "# test").unwrap();

        let bl = blacklist(&["node_modules", "__pycache__"], &["pyc"], &[]);
        let mut seen: Vec<String> = Vec::new();
        let walker = ignore::WalkBuilder::new(&root)
            .threads(1)
            .hidden(false)
            .git_ignore(false)
            .same_file_system(true)
            .filter_entry(move |e| !bl.should_skip_path(e.path()))
            .build();
        for entry in walker {
            let entry = entry.unwrap();
            if entry.path().is_file() {
                seen.push(entry.path().to_string_lossy().replace('\\', "/"));
            }
        }
        let _ = std::fs::remove_dir_all(&root);

        let seen_all = seen.join("\n");
        assert!(
            seen.iter().any(|p| p.ends_with("src/main.ts")),
            "missing src/main.ts in:\n{seen_all}"
        );
        assert!(
            seen.iter().any(|p| p.ends_with("README.md")),
            "missing README.md in:\n{seen_all}"
        );
        assert!(
            seen.iter().all(|p| !p.contains("node_modules")
                && !p.contains("__pycache__")
                && !p.ends_with(".pyc")),
            "blacklisted entries leaked:\n{seen_all}"
        );
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
        let bl = blacklist(&["node_modules"], &[], &[]);
        assert!(bl.should_skip_path(Path::new("C:\\proj\\node_modules")));
    }

    #[test]
    fn does_not_skip_path_without_blacklisted_folder() {
        let bl = blacklist(&["node_modules"], &[], &[]);
        assert!(!bl.should_skip_path(Path::new("C:\\proj\\src\\main.rs")));
    }

    #[test]
    fn folder_match_is_case_insensitive() {
        let bl = blacklist(&["node_modules"], &[], &[]);
        assert!(bl.should_skip_path(Path::new("C:\\proj\\NODE_MODULES\\lib\\main.rs")));
    }

    #[test]
    fn skips_path_matching_glob_pattern() {
        let bl = blacklist(&[], &[], &["**/build/**"]);
        assert!(bl.should_skip_path(Path::new("build/x/y.txt")));
    }

    #[test]
    fn glob_pattern_is_case_insensitive() {
        let bl = blacklist(&[], &[], &["**/Build/**"]);
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
