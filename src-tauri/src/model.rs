//! Defines shared IPC, provider, state, and launch models.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResult {
    pub source: String,
    pub refreshed_at_ms: i64,
    pub previous_count: u64,
    pub current_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceState {
    pub source: String,
    pub last_refresh_at_ms: Option<i64>,
    pub last_success_at_ms: Option<i64>,
    pub last_error: Option<String>,
    pub refresh_watermark: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRecord {
    pub source: String,
    pub id: String,
    pub short_id: String,
    pub title: String,
    pub preview: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub message_count: u64,
    pub total_tokens: u64,
    pub model: String,
    pub workspace: String,
    pub mode: String,
    pub path: String,
    pub invalid_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub favorites: Vec<String>,
    #[serde(default = "default_deepseek_launcher")]
    pub deepseek_launcher: String,
    #[serde(default = "default_provider_launchers")]
    pub provider_launchers: std::collections::BTreeMap<String, String>,
    #[serde(default = "default_auto_refresh_enabled")]
    pub auto_refresh_enabled: bool,
    #[serde(default = "default_auto_refresh_interval_minutes")]
    pub auto_refresh_interval_minutes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepseekStatus {
    pub available: bool,
    pub version: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDescriptor {
    pub id: String,
    pub display_name_key: String,
    pub short_name: String,
    pub icon_key: String,
    pub badge_key: String,
    pub default_group_by: String,
    pub command_label: String,
    pub badge_text: String,
    pub capabilities: ProviderCapabilities,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    pub quick_reply: bool,
    pub launcher_toggle: bool,
    pub favorite: bool,
    pub open_session_folder: bool,
    pub resume: bool,
    pub copy_command: bool,
}

#[derive(Debug, Clone)]
pub struct LaunchPlan {
    pub program: String,
    pub args: Vec<LaunchArg>,
    pub cwd: Option<String>,
    pub shell_wrap: ShellWrap,
    pub prefer_windows_terminal: bool,
    pub error_command_label: String,
    pub use_call_operator: bool,
}

#[derive(Debug, Clone)]
pub struct LaunchArg {
    pub value: String,
    pub single_line: bool,
    pub shell_quote: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellWrap {
    PowerShellScript,
    CmdStart,
}

pub fn default_deepseek_launcher() -> String {
    "cmd".to_string()
}

pub fn default_provider_launchers() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("deepseek".to_string(), "cmd".to_string()),
        ("claude".to_string(), "cmd".to_string()),
        ("codex".to_string(), "ps1".to_string()),
    ])
}

pub fn default_auto_refresh_enabled() -> bool {
    true
}

pub fn default_auto_refresh_interval_minutes() -> u64 {
    5
}
