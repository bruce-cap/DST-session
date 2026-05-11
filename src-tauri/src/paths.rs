//! Provides filesystem paths and path normalization helpers.

use std::path::{Path, PathBuf};

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
    home_dir().join(".deepseek-session-manager")
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
    path.strip_prefix(r"\\?\").unwrap_or(path).to_string()
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
    fn file_stem_returns_unknown_for_missing_name() {
        assert_eq!(file_stem(Path::new("")), "(unknown)");
    }
}
