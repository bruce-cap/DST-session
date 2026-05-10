import type { DeepseekLauncher, SessionSource } from "../types";

export const DEEPSEEK_CMD_COMMAND = "deepseek.cmd";
export const DEEPSEEK_PS1_COMMAND = "deepseek.ps1";
export const CLAUDE_CODE_COMMAND = "claude";

export function deepseekCommand(launcher: DeepseekLauncher = "cmd"): string {
  return launcher === "ps1" ? DEEPSEEK_PS1_COMMAND : DEEPSEEK_CMD_COMMAND;
}

export function buildResumeCommand(
  source: SessionSource,
  sessionId: string,
  deepseekLauncher: DeepseekLauncher = "cmd"
): string {
  if (source === "claude") {
    return `${CLAUDE_CODE_COMMAND} --resume ${sessionId}`;
  }
  return `${deepseekCommand(deepseekLauncher)} resume ${sessionId}`;
}
