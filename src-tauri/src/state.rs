//! Reads and writes the application-owned state file.

use crate::model::{default_auto_refresh_enabled, default_auto_refresh_interval_minutes, default_deepseek_launcher, AppState};
use crate::paths::app_state_path;
use std::fs;

pub fn read_app_state() -> Result<AppState, String> {
    let path = app_state_path();
    if !path.exists() {
        return Ok(default_app_state());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("读取状态文件失败 {}: {error}", path.display()))?;
    serde_json::from_str(&content)
        .map_err(|error| format!("状态文件格式错误 {}: {error}", path.display()))
}

pub fn write_app_state(state: &AppState) -> Result<(), String> {
    let path = app_state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建状态目录失败 {}: {error}", parent.display()))?;
    }

    let content =
        serde_json::to_string_pretty(state).map_err(|error| format!("序列化状态失败: {error}"))?;
    fs::write(&path, content)
        .map_err(|error| format!("写入状态文件失败 {}: {error}", path.display()))
}

pub fn default_app_state() -> AppState {
    AppState {
        favorites: Vec::new(),
        launch_mode: "new_terminal".to_string(),
        deepseek_launcher: default_deepseek_launcher(),
        auto_refresh_enabled: default_auto_refresh_enabled(),
        auto_refresh_interval_minutes: default_auto_refresh_interval_minutes(),
    }
}

pub fn normalize_deepseek_launcher(value: Option<String>) -> String {
    match value.as_deref() {
        Some("ps1") => "ps1".to_string(),
        _ => default_deepseek_launcher(),
    }
}

pub fn normalize_auto_refresh_interval(value: u64) -> u64 {
    value.clamp(1, 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_app_state_uses_new_terminal() {
        let state = default_app_state();
        assert_eq!(state.launch_mode, "new_terminal");
        assert_eq!(state.deepseek_launcher, "cmd");
        assert!(state.auto_refresh_enabled);
        assert_eq!(state.auto_refresh_interval_minutes, 5);
    }

    #[test]
    fn normalize_deepseek_launcher_allows_only_ps1_or_cmd() {
        assert_eq!(normalize_deepseek_launcher(Some("ps1".to_string())), "ps1");
        assert_eq!(normalize_deepseek_launcher(Some("other".to_string())), "cmd");
        assert_eq!(normalize_deepseek_launcher(None), "cmd");
    }
}
