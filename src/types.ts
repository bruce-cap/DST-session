export type GroupBy = "workspace" | "date" | "model" | "mode" | "favorite" | "none";
export type SessionSource = "deepseek" | "claude";
export type DeepseekLauncher = "cmd" | "ps1";

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

export interface AppState {
  favorites: string[];
  launchMode: "new_terminal" | "embedded";
  deepseekLauncher: DeepseekLauncher;
}

export interface DeepseekStatus {
  available: boolean;
  version: string;
  message: string;
}
