//! Provides filesystem paths and path normalization helpers.

use std::path::{Path, PathBuf};

const APP_DATA_DIR_NAME: &str = ".agent-session-manager";
const LEGACY_APP_DATA_DIR_NAME: &str = ".deepseek-session-manager";

pub fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

pub fn default_sessions_dir() -> PathBuf {
    home_dir().join(".deepseek").join("sessions")
}

pub fn default_claude_projects_dir() -> PathBuf {
    home_dir().join(".claude").join("projects")
}

pub fn default_codex_db_path() -> PathBuf {
    home_dir().join(".codex").join("state_5.sqlite")
}

pub fn app_state_path() -> PathBuf {
    app_data_dir().join("state.json")
}

pub fn app_index_path() -> PathBuf {
    app_data_dir().join("index.sqlite")
}

fn app_data_dir() -> PathBuf {
    let home = home_dir();
    let current = home.join(APP_DATA_DIR_NAME);
    if current.exists() {
        return current;
    }

    let legacy = home.join(LEGACY_APP_DATA_DIR_NAME);
    if legacy.exists() {
        return legacy;
    }

    current
}

pub fn workspace_dir(workspace: Option<String>) -> PathBuf {
    let Some(workspace) = workspace else {
        return home_dir();
    };

    let path = PathBuf::from(workspace);
    if path.is_dir() {
        path
    } else {
        home_dir()
    }
}

pub fn normalize_windows_path(path: &str) -> String {
    let stripped = path.strip_prefix(r"\\?\").unwrap_or(path).trim();
    let mut normalized = if is_windows_drive_path(stripped) {
        stripped.replace('/', "\\")
    } else {
        stripped.to_string()
    };

    if is_windows_drive_path(&normalized) {
        let drive = normalized[0..1].to_ascii_uppercase();
        normalized.replace_range(0..1, &drive);
        normalized = trim_trailing_windows_separators(&normalized);
    }

    normalized
}

fn is_windows_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn trim_trailing_windows_separators(path: &str) -> String {
    if path.len() <= 3 {
        return if path.ends_with('\\') {
            path.to_string()
        } else {
            format!("{path}\\")
        };
    }

    path.trim_end_matches(['\\', '/']).to_string()
}

pub fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("(unknown)")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_windows_path_is_idempotent() {
        let value = r"\\?\C:\Users\Cap";
        let once = normalize_windows_path(value);
        let twice = normalize_windows_path(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn normalize_windows_path_uppercases_drive_and_trims_trailing_slashes() {
        assert_eq!(
            normalize_windows_path(r"c:/Users/Cap/Desktop/dst-session/"),
            r"C:\Users\Cap\Desktop\dst-session"
        );
    }

    #[test]
    fn file_stem_returns_unknown_for_missing_name() {
        assert_eq!(file_stem(Path::new("")), "(unknown)");
    }
}
