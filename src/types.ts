export type GroupBy = "workspace" | "date" | "model" | "mode" | "favorite" | "none";
export type SessionSource = "deepseek" | "claude" | "codex";
export type ProviderLauncher = "cmd" | "ps1";
export type DeepseekLauncher = ProviderLauncher;
export type ThemeMode = "light" | "dark";

export interface SessionRecord {
  source: SessionSource;
  id: string;
  shortId: string;
  title: string;
  preview: string;
  createdAt: string | null;
  updatedAt: string | null;
  messageCount: number;
  totalTokens: number;
  model: string;
  workspace: string;
  mode: string;
  path: string;
  invalidReason?: string | null;
}

export interface SessionGroup {
  key: string;
  label: string;
  sessions: SessionRecord[];
}

export interface RefreshResult {
  source: SessionSource;
  refreshedAtMs: number;
  previousCount: number;
  currentCount: number;
}

export interface SourceState {
  source: SessionSource;
  lastRefreshAtMs: number | null;
  lastSuccessAtMs: number | null;
  lastError: string | null;
  refreshWatermark: string | null;
}

export interface AppState {
  favorites: string[];
  deepseekLauncher: DeepseekLauncher;
  providerLaunchers: Record<SessionSource, ProviderLauncher>;
  autoRefreshEnabled: boolean;
  autoRefreshIntervalMinutes: number;
}

export interface DeepseekStatus {
  available: boolean;
  version: string;
  message: string;
}

export interface ProviderCapabilities {
  quickReply: boolean;
  launcherToggle: boolean;
  favorite: boolean;
  openSessionFolder: boolean;
  resume: boolean;
  copyCommand: boolean;
}

export interface ProviderDescriptor {
  id: SessionSource;
  displayNameKey: "source_deepseek" | "source_claude" | "source_codex";
  shortName: string;
  iconKey: "deepseek" | "claude" | "codex";
  badgeKey: "deepseek" | "claude" | "codex";
  defaultGroupBy: GroupBy;
  commandLabel: string;
  badgeText: string;
  capabilities: ProviderCapabilities;
}
