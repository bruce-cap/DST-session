import type { DeepseekLauncher, SessionSource } from "../types";

export const DEEPSEEK_CMD_COMMAND = "deepseek.cmd";
export const DEEPSEEK_PS1_COMMAND = "deepseek.ps1";
export const CLAUDE_CODE_COMMAND = "claude";
export const CODEX_COMMAND = "codex.ps1";

export function deepseekCommand(launcher: DeepseekLauncher = "cmd"): string {
  return launcher === "ps1" ? DEEPSEEK_PS1_COMMAND : DEEPSEEK_CMD_COMMAND;
}

/**
 * Produce the shell command used to resume a session.
 *
 * For Codex, an optional `prompt` can be appended so the new terminal
 * immediately runs `codex resume <id> "<prompt>"` (quick reply). The
 * prompt is single-line only; callers should normalize newlines before
 * passing it in.
 */
export function buildResumeCommand(
  source: SessionSource,
  sessionId: string,
  deepseekLauncher: DeepseekLauncher = "cmd",
  prompt?: string
): string {
  if (source === "claude") {
    return `${CLAUDE_CODE_COMMAND} --resume ${sessionId}`;
  }

  if (source === "codex") {
    const trimmed = prompt?.trim();
    if (trimmed) {
      return `${CODEX_COMMAND} resume ${sessionId} ${quoteArg(trimmed)}`;
    }
    return `${CODEX_COMMAND} resume ${sessionId}`;
  }

  return `${deepseekCommand(deepseekLauncher)} resume ${sessionId}`;
}

/** Collapse all whitespace (including newlines, tabs) into single spaces. */
export function normalizeSingleLine(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

/** Shell-quote an argument for display or copy. Doubles embedded quotes. */
function quoteArg(value: string): string {
  return `"${value.replace(/"/g, '\\"')}"`;
}
