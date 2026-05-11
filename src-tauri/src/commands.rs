//! Exposes Tauri IPC commands backed by providers and state modules.

use crate::launcher;
use crate::model::{AppState, DeepseekStatus, ProviderDescriptor, SessionRecord};
use crate::providers::{AgentCheckContext, ProviderRegistry, ResumeRequest};
use crate::state::{normalize_deepseek_launcher, read_app_state, write_app_state};
use std::collections::BTreeSet;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub fn list_providers(registry: State<'_, ProviderRegistry>) -> Result<Vec<ProviderDescriptor>, String> {
    Ok(registry.descriptors())
}

#[tauri::command]
pub fn list_sessions(
    registry: State<'_, ProviderRegistry>,
    source: Option<String>,
    sessions_dir: Option<String>,
) -> Result<Vec<SessionRecord>, String> {
    registry
        .resolve_or_default(source.as_deref())
        .list_sessions(sessions_dir.map(PathBuf::from))
}

#[tauri::command]
pub fn get_app_state() -> Result<AppState, String> {
    read_app_state()
}

#[tauri::command]
pub fn set_favorite(session_id: String, favorite: bool) -> Result<AppState, String> {
    let mut state = read_app_state()?;
    let mut favorites: BTreeSet<String> = state.favorites.into_iter().collect();
    if favorite { favorites.insert(session_id); } else { favorites.remove(&session_id); }
    state.favorites = favorites.into_iter().collect();
    write_app_state(&state)?;
    Ok(state)
}

#[tauri::command]
pub fn set_deepseek_launcher(launcher: String) -> Result<AppState, String> {
    let mut state = read_app_state()?;
    state.deepseek_launcher = normalize_deepseek_launcher(Some(launcher));
    write_app_state(&state)?;
    Ok(state)
}

#[tauri::command]
pub fn check_agent(
    registry: State<'_, ProviderRegistry>,
    source: Option<String>,
    deepseek_launcher: Option<String>,
) -> DeepseekStatus {
    registry
        .resolve_or_default(source.as_deref())
        .check_agent(AgentCheckContext { deepseek_launcher })
}

#[tauri::command]
pub fn open_session_folder(path: String) -> Result<(), String> {
    launcher::open_folder(PathBuf::from(path))
}

#[tauri::command]
pub fn resume_session(
    registry: State<'_, ProviderRegistry>,
    source: Option<String>,
    session_id: String,
    workspace: Option<String>,
    launch_mode: Option<String>,
    deepseek_launcher: Option<String>,
    prompt: Option<String>,
) -> Result<(), String> {
    if launch_mode.unwrap_or_else(|| "new_terminal".to_string()) != "new_terminal" {
        return Err("V0.1 暂只支持打开新的系统终端。".to_string());
    }
    let plan = registry.resolve_or_default(source.as_deref()).plan_resume(ResumeRequest {
        session_id,
        workspace,
        deepseek_launcher,
        prompt,
    })?;
    launcher::execute_plan(plan)
}
