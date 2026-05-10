import { describe, expect, it } from "vitest";
import { buildResumeCommand } from "./commands";

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
});
