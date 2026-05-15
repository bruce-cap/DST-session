import type { DailyTokenUsage, ModelDailyTokenUsage, ModelTokenUsage, ProviderTokenUsage, SessionSource, UsageRange } from "../types";

const DAY_MS = 86_400_000;

export function filterDailyBySourceAndRange(items: DailyTokenUsage[], source: SessionSource, range: UsageRange): DailyTokenUsage[] {
  const sourceItems = items.filter((item) => item.source === source);
  const start = rangeStartDate(sourceItems.map((item) => item.date), range);
  return sourceItems.filter((item) => !start || item.date >= start);
}

export function filterModelsBySourceAndRange(
  allTimeItems: ModelTokenUsage[],
  dailyItems: ModelDailyTokenUsage[],
  source: SessionSource,
  range: UsageRange
): ModelTokenUsage[] {
  if (range === "all") {
    return allTimeItems.filter((item) => item.source === source);
  }

  const start = rangeStartDate(
    dailyItems.filter((item) => item.source === source).map((item) => item.date),
    range
  );
  const buckets = new Map<string, ModelTokenUsage>();

  for (const item of dailyItems) {
    if (item.source !== source || (start && item.date < start)) {
      continue;
    }

    const bucket = buckets.get(item.model) ?? {
      source,
      model: item.model,
      inputTokens: 0,
      outputTokens: 0,
      totalTokens: 0,
      sessionCount: 0,
      messageCount: 0
    };
    bucket.inputTokens += item.inputTokens;
    bucket.outputTokens += item.outputTokens;
    bucket.totalTokens += item.totalTokens;
    bucket.sessionCount += item.sessionCount;
    bucket.messageCount += item.messageCount;
    buckets.set(item.model, bucket);
  }

  return [...buckets.values()].sort((left, right) => right.totalTokens - left.totalTokens || left.model.localeCompare(right.model));
}

export function providerSummaryForSource(items: ProviderTokenUsage[], source: SessionSource): ProviderTokenUsage | null {
  return items.find((item) => item.source === source) ?? null;
}

export function usageShare(value: number, total: number): number {
  return total <= 0 ? 0 : value / total;
}

export function rangeStartDate(dates: string[], range: UsageRange): string | null {
  if (range === "all" || dates.length === 0) {
    return null;
  }

  const latest = dates.reduce((max, date) => (date > max ? date : max), dates[0]);
  const latestTime = Date.parse(`${latest}T00:00:00`);
  if (Number.isNaN(latestTime)) {
    return null;
  }

  const days = range === "30d" ? 30 : 7;
  const start = new Date(latestTime - (days - 1) * DAY_MS);
  return toDateKey(start);
}

export function buildHeatmapDays(items: DailyTokenUsage[], range: UsageRange): DailyTokenUsage[] {
  if (items.length === 0) {
    return [];
  }

  const firstDate = items[0].date;
  const lastDate = items[items.length - 1].date;
  const startDate = range === "all" ? firstDate : rangeStartDate(items.map((item) => item.date), range) ?? firstDate;
  const buckets = new Map(items.map((item) => [item.date, item]));
  const days: DailyTokenUsage[] = [];

  for (let time = Date.parse(`${startDate}T00:00:00`), end = Date.parse(`${lastDate}T00:00:00`); time <= end; time += DAY_MS) {
    const date = toDateKey(new Date(time));
    days.push(buckets.get(date) ?? {
      date,
      source: items[0].source,
      inputTokens: 0,
      outputTokens: 0,
      totalTokens: 0,
      sessionCount: 0,
      messageCount: 0
    });
  }

  return days;
}

export function heatmapLevel(value: number, max: number): number {
  if (value <= 0 || max <= 0) return 0;
  const ratio = value / max;
  if (ratio <= 0.25) return 1;
  if (ratio <= 0.5) return 2;
  if (ratio <= 0.75) return 3;
  return 4;
}

function toDateKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}
