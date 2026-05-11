//! Reads and writes the application-owned state file.

use crate::model::{
    default_auto_refresh_enabled, default_auto_refresh_interval_minutes, default_deepseek_launcher,
    default_provider_launchers, AppState,
};
use crate::paths::app_state_path;
use std::fs;

pub fn read_app_state() -> Result<AppState, String> {
    let path = app_state_path();
    if !path.exists() {
        return Ok(default_app_state());
    }

    let content = fs::read_to_string(&path)
        .map_err(|error| format!("读取状态文件失败 {}: {error}", path.display()))?;
    let mut state: AppState = serde_json::from_str(&content)
        .map_err(|error| format!("状态文件格式错误 {}: {error}", path.display()))?;
    normalize_app_state(&mut state);
    Ok(state)
}

pub fn write_app_state(state: &AppState) -> Result<(), String> {
    let path = app_state_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建状态目录失败 {}: {error}", parent.display()))?;
    }

    let mut state = state.clone();
    normalize_app_state(&mut state);
    let content =
        serde_json::to_string_pretty(&state).map_err(|error| format!("序列化状态失败: {error}"))?;
    fs::write(&path, content)
        .map_err(|error| format!("写入状态文件失败 {}: {error}", path.display()))
}

pub fn default_app_state() -> AppState {
    AppState {
        favorites: Vec::new(),
        deepseek_launcher: default_deepseek_launcher(),
        provider_launchers: default_provider_launchers(),
        auto_refresh_enabled: default_auto_refresh_enabled(),
        auto_refresh_interval_minutes: default_auto_refresh_interval_minutes(),
    }
}

pub fn normalize_deepseek_launcher(value: Option<String>) -> String {
    normalize_provider_launcher("deepseek", value)
}

pub fn normalize_provider_launcher(source: &str, value: Option<String>) -> String {
    match value.as_deref() {
        Some("ps1") => "ps1".to_string(),
        Some("cmd") => "cmd".to_string(),
        _ if source == "codex" => "ps1".to_string(),
        _ => "cmd".to_string(),
    }
}

pub fn normalize_auto_refresh_interval(value: u64) -> u64 {
    value.clamp(1, 60)
}

fn normalize_app_state(state: &mut AppState) {
    state.deepseek_launcher = normalize_deepseek_launcher(Some(state.deepseek_launcher.clone()));
    for source in ["deepseek", "claude", "codex"] {
        let launcher = state
            .provider_launchers
            .get(source)
            .cloned()
            .or_else(|| (source == "deepseek").then(|| state.deepseek_launcher.clone()));
        state.provider_launchers.insert(
            source.to_string(),
            normalize_provider_launcher(source, launcher),
        );
    }
    state.deepseek_launcher = state
        .provider_launchers
        .get("deepseek")
        .cloned()
        .unwrap_or_else(default_deepseek_launcher);
    state.auto_refresh_interval_minutes =
        normalize_auto_refresh_interval(state.auto_refresh_interval_minutes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_app_state_uses_new_terminal() {
        let state = default_app_state();
        assert_eq!(state.deepseek_launcher, "cmd");
        assert_eq!(
            state.provider_launchers.get("deepseek"),
            Some(&"cmd".to_string())
        );
        assert_eq!(
            state.provider_launchers.get("claude"),
            Some(&"cmd".to_string())
        );
        assert_eq!(
            state.provider_launchers.get("codex"),
            Some(&"ps1".to_string())
        );
        assert!(state.auto_refresh_enabled);
        assert_eq!(state.auto_refresh_interval_minutes, 5);
    }

    #[test]
    fn normalize_deepseek_launcher_allows_only_ps1_or_cmd() {
        assert_eq!(normalize_deepseek_launcher(Some("ps1".to_string())), "ps1");
        assert_eq!(
            normalize_deepseek_launcher(Some("other".to_string())),
            "cmd"
        );
        assert_eq!(normalize_deepseek_launcher(None), "cmd");
    }

    #[test]
    fn normalize_provider_launcher_defaults_codex_to_ps1() {
        assert_eq!(
            normalize_provider_launcher("codex", Some("cmd".to_string())),
            "cmd"
        );
        assert_eq!(
            normalize_provider_launcher("codex", Some("other".to_string())),
            "ps1"
        );
        assert_eq!(normalize_provider_launcher("claude", None), "cmd");
    }
}
