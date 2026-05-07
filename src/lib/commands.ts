export const DEEPSEEK_TUI_COMMAND = "deepseek-tui.cmd";

export function buildResumeCommand(sessionId: string): string {
  return `${DEEPSEEK_TUI_COMMAND} resume ${sessionId}`;
}
