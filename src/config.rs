//! Config loading.
//!
//! TOML file at `%APPDATA%\stk\config.toml` (`dirs::config_dir()/stk/config.toml`).
//! All keys optional; defaults below.
//!
//! # Store root override
//!
//! The session/stats store root is normally `%LOCALAPPDATA%\stk`
//! (`dirs::data_local_dir()/stk`). If the environment variable `STK_DATA_DIR`
//! is set, it overrides that default entirely (checked BEFORE the dirs-crate
//! default). This exists for test isolation and portable installs.

use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Files at or under this size (bytes) are always allowed. Default 16 KiB.
    pub clamp_threshold: u64,
    /// Hard cap on outline entry lines. Default 80.
    pub outline_max_lines: usize,
    /// Enable the same-session duplicate-read deny layer. Default true.
    pub dedup: bool,
    /// Glob patterns always allowed through (e.g. ["*.lock"]).
    pub exclude: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            clamp_threshold: 16384,
            outline_max_lines: 80,
            dedup: true,
            exclude: Vec::new(),
        }
    }
}

impl Config {
    /// Path of the config file (may not exist).
    pub fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("stk").join("config.toml"))
    }

    /// Load config from disk, falling back to defaults on any problem (fail-open).
    pub fn load() -> Config {
        let Some(path) = Self::path() else {
            return Config::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).unwrap_or_default(),
            Err(_) => Config::default(),
        }
    }

    /// True if `file_path` matches any exclude glob (matched against the full
    /// path and against the file name).
    ///
    /// Patterns are matched unanchored against the full path (a `**/` prefix
    /// is prepended unless already present), so `docs/*.md` matches
    /// `C:\repo\docs\a.md`. Matching is case-insensitive on Windows.
    pub fn is_excluded(&self, file_path: &str) -> bool {
        let name = std::path::Path::new(file_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let opts = glob::MatchOptions {
            case_sensitive: !cfg!(windows),
            require_literal_separator: false,
            require_literal_leading_dot: false,
        };
        self.exclude.iter().any(|pat| {
            let anchored = if pat.starts_with("**/") || pat.starts_with("**\\") {
                pat.clone()
            } else {
                format!("**/{pat}")
            };
            let full = glob::Pattern::new(&anchored)
                .map(|p| p.matches_with(file_path, opts))
                .unwrap_or(false);
            let by_name = glob::Pattern::new(pat)
                .map(|p| p.matches_with(&name, opts))
                .unwrap_or(false);
            full || by_name
        })
    }

    /// Exclude patterns that fail to parse as globs (silently inert during
    /// matching; surfaced by `stk config`).
    pub fn invalid_excludes(&self) -> Vec<String> {
        self.exclude
            .iter()
            .filter(|pat| glob::Pattern::new(pat).is_err())
            .cloned()
            .collect()
    }
}

/// Resolve the store root: `STK_DATA_DIR` env var first, then
/// `dirs::data_local_dir()/stk`.
pub fn store_root() -> PathBuf {
    if let Ok(dir) = std::env::var("STK_DATA_DIR") {
        if !dir.trim().is_empty() {
            return PathBuf::from(dir);
        }
    }
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("stk")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let c = Config::default();
        assert_eq!(c.clamp_threshold, 16384);
        assert_eq!(c.outline_max_lines, 80);
        assert!(c.dedup);
        assert!(c.exclude.is_empty());
    }

    #[test]
    fn partial_toml_fills_defaults() {
        let c: Config = toml::from_str("clamp_threshold = 1024").unwrap();
        assert_eq!(c.clamp_threshold, 1024);
        assert_eq!(c.outline_max_lines, 80);
        assert!(c.dedup);
    }

    #[test]
    fn exclude_glob_matches_file_name() {
        let c: Config = toml::from_str(r#"exclude = ["*.lock"]"#).unwrap();
        assert!(c.is_excluded("C:\\repo\\Cargo.lock"));
        assert!(!c.is_excluded("C:\\repo\\main.rs"));
    }

    #[test]
    fn exclude_glob_directory_scoped_matches_absolute_path() {
        let c: Config = toml::from_str(r#"exclude = ["docs/*.md", "src/generated/*"]"#).unwrap();
        assert!(c.is_excluded("C:\\repo\\docs\\a.md"));
        assert!(c.is_excluded("C:\\repo\\src\\generated\\api.rs"));
        assert!(!c.is_excluded("C:\\repo\\src\\main.rs"));
    }

    #[cfg(windows)]
    #[test]
    fn exclude_glob_case_insensitive_on_windows() {
        let c: Config = toml::from_str(r#"exclude = ["*.lock"]"#).unwrap();
        assert!(c.is_excluded("C:\\repo\\CARGO.LOCK"));
    }

    #[test]
    fn invalid_exclude_patterns_reported() {
        let c: Config = toml::from_str(r#"exclude = ["*.lock", "[bad"]"#).unwrap();
        assert_eq!(c.invalid_excludes(), vec!["[bad".to_string()]);
        assert!(c.is_excluded("C:\\repo\\Cargo.lock"), "valid patterns still work");
    }

    #[test]
    fn store_root_env_override() {
        // Serial-safe: this is the only test touching STK_DATA_DIR.
        std::env::set_var("STK_DATA_DIR", "C:\\tmp\\stk-test-root");
        assert_eq!(store_root(), PathBuf::from("C:\\tmp\\stk-test-root"));
        std::env::remove_var("STK_DATA_DIR");
        let def = store_root();
        assert!(def.ends_with("stk"));
    }
}
