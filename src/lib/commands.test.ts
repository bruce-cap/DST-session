import { describe, expect, it } from "vitest";
import { buildResumeCommand } from "./commands";

describe("command helpers", () => {
  it("builds the DeepSeek TUI resume command with the cmd launcher", () => {
    expect(buildResumeCommand("a22e6c3d-86bf-4f20-a806-749bd57fed1d")).toBe(
      "deepseek-tui.cmd resume a22e6c3d-86bf-4f20-a806-749bd57fed1d"
    );
  });
});
