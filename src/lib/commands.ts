import type { SessionSource } from "../types";

export const DEEPSEEK_TUI_COMMAND = "deepseek-tui.cmd";
export const CLAUDE_CODE_COMMAND = "claude";

export function buildResumeCommand(source: SessionSource, sessionId: string): string {
  if (source === "claude") {
    return `${CLAUDE_CODE_COMMAND} --resume ${sessionId}`;
  }
  return `${DEEPSEEK_TUI_COMMAND} resume ${sessionId}`;
}
