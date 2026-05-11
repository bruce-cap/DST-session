/** Wraps Tauri IPC commands with typed frontend functions. */

import { invoke } from "@tauri-apps/api/core";
import type { AppState, DeepseekLauncher, DeepseekStatus, ProviderDescriptor, RefreshResult, SessionRecord, SessionSource, SourceState } from "../types";

export function listProviders(): Promise<ProviderDescriptor[]> {
  return invoke<ProviderDescriptor[]>("list_providers");
}

export function listSessions(params: { source?: SessionSource }): Promise<SessionRecord[]> {
  return invoke<SessionRecord[]>("list_sessions", {
    source: params.source
  });
}

export function refreshSessions(params: { source?: SessionSource; sessionsDir?: string | null }): Promise<RefreshResult> {
  return invoke<RefreshResult>("refresh_sessions", {
    source: params.source,
    sessionsDir: params.sessionsDir ?? null
  });
}

export function getSourceState(params: { source?: SessionSource }): Promise<SourceState | null> {
  return invoke<SourceState | null>("get_source_state", {
    source: params.source
  });
}

export function getAppState(): Promise<AppState> {
  return invoke<AppState>("get_app_state");
}

export function setFavorite(params: { sessionId: string; favorite: boolean }): Promise<AppState> {
  return invoke<AppState>("set_favorite", params);
}

export function setDeepseekLauncher(params: { launcher: DeepseekLauncher }): Promise<AppState> {
  return invoke<AppState>("set_deepseek_launcher", params);
}

export function setAutoRefresh(params: { enabled: boolean; intervalMinutes: number }): Promise<AppState> {
  return invoke<AppState>("set_auto_refresh", params);
}

export function checkAgent(params: { source?: SessionSource; deepseekLauncher?: DeepseekLauncher }): Promise<DeepseekStatus> {
  return invoke<DeepseekStatus>("check_agent", params);
}

export function openSessionFolder(params: { path: string }): Promise<void> {
  return invoke<void>("open_session_folder", params);
}

export function resumeSession(params: {
  source?: SessionSource;
  sessionId: string;
  workspace?: string | null;
  launchMode?: AppState["launchMode"];
  deepseekLauncher?: DeepseekLauncher;
  prompt?: string | null;
}): Promise<void> {
  return invoke<void>("resume_session", params);
}
