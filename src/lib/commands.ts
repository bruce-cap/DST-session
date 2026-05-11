import type { ProviderLauncher, SessionSource } from "../types";

export const DEEPSEEK_CMD_COMMAND = "deepseek.cmd";
export const DEEPSEEK_PS1_COMMAND = "deepseek.ps1";
export const CLAUDE_CMD_COMMAND = "claude.cmd";
export const CLAUDE_PS1_COMMAND = "claude.ps1";
export const CODEX_CMD_COMMAND = "codex.cmd";
export const CODEX_PS1_COMMAND = "codex.ps1";

export function providerCommand(source: SessionSource, launcher: ProviderLauncher = defaultLauncher(source)): string {
  if (source === "claude") {
    return launcher === "ps1" ? CLAUDE_PS1_COMMAND : CLAUDE_CMD_COMMAND;
  }
  if (source === "codex") {
    return launcher === "cmd" ? CODEX_CMD_COMMAND : CODEX_PS1_COMMAND;
  }
  return launcher === "ps1" ? DEEPSEEK_PS1_COMMAND : DEEPSEEK_CMD_COMMAND;
}

function defaultLauncher(source: SessionSource): ProviderLauncher {
  return source === "codex" ? "ps1" : "cmd";
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
  launcher: ProviderLauncher = defaultLauncher(source),
  prompt?: string
): string {
  const command = providerCommand(source, launcher);
  if (source === "claude") {
    return `${command} --resume ${sessionId}`;
  }

  if (source === "codex") {
    const trimmed = prompt?.trim();
    if (trimmed) {
      return `${command} resume ${sessionId} ${quoteArg(trimmed, launcher)}`;
    }
    return `${command} resume ${sessionId}`;
  }

  return `${command} resume ${sessionId}`;
}

/** Collapse all whitespace (including newlines, tabs) into single spaces. */
export function normalizeSingleLine(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function quoteArg(value: string, launcher: ProviderLauncher): string {
  if (launcher === "ps1") {
    return `'${value.replace(/'/g, "''")}'`;
  }
  return `"${value.replace(/"/g, '""')}"`;
}
