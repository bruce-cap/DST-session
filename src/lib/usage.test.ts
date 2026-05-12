import { describe, expect, it } from "vitest";
import { buildHeatmapDays, filterDailyBySourceAndRange, filterModelsBySourceAndRange, heatmapLevel, rangeStartDate } from "./usage";
import type { DailyTokenUsage, ModelDailyTokenUsage, ModelTokenUsage } from "../types";

const daily: DailyTokenUsage[] = [
  { date: "2026-05-01", source: "codex", totalTokens: 100, sessionCount: 1, messageCount: 2 },
  { date: "2026-05-05", source: "claude", totalTokens: 500, sessionCount: 1, messageCount: 2 },
  { date: "2026-05-08", source: "codex", totalTokens: 200, sessionCount: 2, messageCount: 4 },
  { date: "2026-05-10", source: "codex", totalTokens: 300, sessionCount: 3, messageCount: 6 }
];

const allTimeModels: ModelTokenUsage[] = [
  { source: "codex", model: "gpt-5", totalTokens: 600, sessionCount: 6, messageCount: 12 },
  { source: "claude", model: "claude-opus", totalTokens: 500, sessionCount: 1, messageCount: 2 }
];

const dailyModels: ModelDailyTokenUsage[] = [
  { date: "2026-05-01", source: "codex", model: "gpt-5", totalTokens: 100, sessionCount: 1, messageCount: 2 },
  { date: "2026-05-08", source: "codex", model: "gpt-5", totalTokens: 200, sessionCount: 2, messageCount: 4 },
  { date: "2026-05-10", source: "codex", model: "gpt-4", totalTokens: 300, sessionCount: 3, messageCount: 6 }
];

describe("usage helpers", () => {
  it("filters daily usage by source and range", () => {
    expect(filterDailyBySourceAndRange(daily, "codex", "7d").map((item) => item.date)).toEqual([
      "2026-05-08",
      "2026-05-10"
    ]);
  });

  it("uses all-time model totals for all range", () => {
    expect(filterModelsBySourceAndRange(allTimeModels, dailyModels, "codex", "all")).toEqual([
      allTimeModels[0]
    ]);
  });

  it("recomputes model totals for finite ranges", () => {
    expect(filterModelsBySourceAndRange(allTimeModels, dailyModels, "codex", "7d")).toEqual([
      { source: "codex", model: "gpt-4", totalTokens: 300, sessionCount: 3, messageCount: 6 },
      { source: "codex", model: "gpt-5", totalTokens: 200, sessionCount: 2, messageCount: 4 }
    ]);
  });

  it("fills missing heatmap dates", () => {
    const heatmap = buildHeatmapDays(filterDailyBySourceAndRange(daily, "codex", "all"), "all");
    expect(heatmap.map((item) => item.date)).toContain("2026-05-02");
    expect(heatmap.find((item) => item.date === "2026-05-02")?.totalTokens).toBe(0);
  });

  it("calculates range start dates and heatmap levels", () => {
    expect(rangeStartDate(["2026-05-10"], "7d")).toBe("2026-05-04");
    expect(heatmapLevel(0, 100)).toBe(0);
    expect(heatmapLevel(10, 100)).toBe(1);
    expect(heatmapLevel(60, 100)).toBe(3);
    expect(heatmapLevel(100, 100)).toBe(4);
  });
});
