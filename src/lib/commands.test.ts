import { describe, expect, it } from "vitest";
import { buildResumeCommand, normalizeSingleLine } from "./commands";

describe("command helpers", () => {
  it("builds the DeepSeek resume command with the cmd launcher by default", () => {
    expect(buildResumeCommand("deepseek", "a22e6c3d-86bf-4f20-a806-749bd57fed1d")).toBe(
      "deepseek.cmd resume a22e6c3d-86bf-4f20-a806-749bd57fed1d"
    );
  });

  it("builds the DeepSeek resume command with the ps1 launcher", () => {
    expect(buildResumeCommand("deepseek", "a22e6c3d-86bf-4f20-a806-749bd57fed1d", "ps1")).toBe(
      "deepseek.ps1 resume a22e6c3d-86bf-4f20-a806-749bd57fed1d"
    );
  });

  it("builds the Claude Code resume command", () => {
    expect(buildResumeCommand("claude", "3fff3ed8-303a-424f-85f2-4a243b3d5ffc")).toBe(
      "claude --resume 3fff3ed8-303a-424f-85f2-4a243b3d5ffc"
    );
  });

  it("builds a plain Codex resume command without a prompt", () => {
    expect(buildResumeCommand("codex", "thread-abc123")).toBe("codex.ps1 resume thread-abc123");
  });

  it("builds a Codex quick-reply command with a quoted prompt", () => {
    expect(buildResumeCommand("codex", "thread-abc123", "cmd", "继续上一轮")).toBe(
      'codex.ps1 resume thread-abc123 "继续上一轮"'
    );
  });

  it("ignores empty or whitespace-only prompts for Codex", () => {
    expect(buildResumeCommand("codex", "thread-abc123", "cmd", "   ")).toBe(
      "codex.ps1 resume thread-abc123"
    );
  });

  it("escapes double quotes in a Codex prompt", () => {
    expect(buildResumeCommand("codex", "tid", "cmd", 'say "hi"')).toBe(
      'codex.ps1 resume tid "say \\"hi\\""'
    );
  });

  it("normalizes multi-line input to a single line", () => {
    expect(normalizeSingleLine("line one\nline two\r\n\ttrailing")).toBe(
      "line one line two trailing"
    );
  });
});
