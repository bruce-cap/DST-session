//! Exposes Tauri IPC commands backed by providers and state modules.

use crate::index;
use crate::launcher;
use crate::model::{
    AppState, DeepseekStatus, ProviderDescriptor, RefreshResult, SessionRecord, SourceState,
    TokenUsageSummary,
};
use crate::providers::{AgentCheckContext, ProviderRegistry, ResumeRequest};
use crate::state::{
    normalize_auto_refresh_interval, normalize_provider_launcher, read_app_state, write_app_state,
};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::PathBuf;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeSessionRequest {
    pub source: Option<String>,
    pub session_id: String,
    pub deepseek_launcher: Option<String>,
    pub launcher: Option<String>,
    pub prompt: Option<String>,
}

#[tauri::command]
pub fn list_providers(
    registry: State<'_, ProviderRegistry>,
) -> Result<Vec<ProviderDescriptor>, String> {
    Ok(registry.descriptors())
}

#[tauri::command]
pub fn list_sessions(source: Option<String>) -> Result<Vec<SessionRecord>, String> {
    let source = source.unwrap_or_else(|| crate::providers::DEFAULT_SOURCE.to_string());
    index::read_sessions(&source)
}

#[tauri::command]
pub fn refresh_sessions(
    registry: State<'_, ProviderRegistry>,
    source: Option<String>,
    sessions_dir: Option<String>,
) -> Result<RefreshResult, String> {
    let source = source.unwrap_or_else(|| crate::providers::DEFAULT_SOURCE.to_string());
    let records = match registry
        .resolve_or_default(Some(&source))
        .list_sessions(sessions_dir.map(PathBuf::from))
    {
        Ok(records) => records,
        Err(error) => {
            let _ = index::record_refresh_error(&source, &error);
            return Err(error);
        }
    };
    index::refresh_source(&source, records)
}

#[tauri::command]
pub fn get_source_state(source: Option<String>) -> Result<Option<SourceState>, String> {
    let source = source.unwrap_or_else(|| crate::providers::DEFAULT_SOURCE.to_string());
    index::read_source_state(&source)
}

#[tauri::command]
pub fn get_token_usage() -> Result<TokenUsageSummary, String> {
    index::read_token_usage()
}

#[tauri::command]
pub fn refresh_token_usage(source: Option<String>) -> Result<RefreshResult, String> {
    let source = source.unwrap_or_else(|| crate::providers::DEFAULT_SOURCE.to_string());
    crate::usage::refresh_token_usage_for_source(&source)
}

#[tauri::command]
pub fn get_app_state() -> Result<AppState, String> {
    read_app_state()
}

#[tauri::command]
pub fn set_favorite(session_id: String, favorite: bool) -> Result<AppState, String> {
    let mut state = read_app_state()?;
    let mut favorites: BTreeSet<String> = state.favorites.into_iter().collect();
    if favorite {
        favorites.insert(session_id);
    } else {
        favorites.remove(&session_id);
    }
    state.favorites = favorites.into_iter().collect();
    write_app_state(&state)?;
    Ok(state)
}

#[tauri::command]
pub fn set_deepseek_launcher(launcher: String) -> Result<AppState, String> {
    set_provider_launcher("deepseek".to_string(), launcher)
}

#[tauri::command]
pub fn set_provider_launcher(source: String, launcher: String) -> Result<AppState, String> {
    let mut state = read_app_state()?;
    let launcher = normalize_provider_launcher(&source, Some(launcher));
    state
        .provider_launchers
        .insert(source.clone(), launcher.clone());
    if source == "deepseek" {
        state.deepseek_launcher = launcher;
    }
    write_app_state(&state)?;
    read_app_state()
}

#[tauri::command]
pub fn set_auto_refresh(enabled: bool, interval_minutes: u64) -> Result<AppState, String> {
    let mut state = read_app_state()?;
    state.auto_refresh_enabled = enabled;
    state.auto_refresh_interval_minutes = normalize_auto_refresh_interval(interval_minutes);
    write_app_state(&state)?;
    Ok(state)
}

#[tauri::command]
pub fn check_agent(
    registry: State<'_, ProviderRegistry>,
    source: Option<String>,
    deepseek_launcher: Option<String>,
    launcher: Option<String>,
) -> DeepseekStatus {
    let source = source.unwrap_or_else(|| crate::providers::DEFAULT_SOURCE.to_string());
    let launcher = launcher
        .or(deepseek_launcher)
        .map(|value| normalize_provider_launcher(&source, Some(value)));
    registry
        .resolve_or_default(Some(&source))
        .check_agent(AgentCheckContext { launcher })
}

#[tauri::command]
pub fn open_session_folder(path: String) -> Result<(), String> {
    launcher::open_folder(PathBuf::from(path))
}

#[tauri::command]
pub fn resume_session(
    registry: State<'_, ProviderRegistry>,
    request: ResumeSessionRequest,
) -> Result<(), String> {
    let source = request
        .source
        .unwrap_or_else(|| crate::providers::DEFAULT_SOURCE.to_string());
    let launcher = request
        .launcher
        .or(request.deepseek_launcher)
        .map(|value| normalize_provider_launcher(&source, Some(value)));
    let session = index::read_session(&source, &request.session_id)?;
    if let Some(reason) = session.invalid_reason.as_deref() {
        return Err(reason.to_string());
    }
    let plan = registry
        .resolve_or_default(Some(&source))
        .plan_resume(ResumeRequest {
            session_id: request.session_id,
            workspace: Some(session.workspace),
            launcher,
            prompt: request.prompt,
        })?;
    launcher::execute_plan(plan)
}
