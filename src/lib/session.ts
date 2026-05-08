import type { GroupBy, SessionGroup, SessionRecord } from "../types";

type RawSession = {
  metadata?: {
    id?: unknown;
    title?: unknown;
    created_at?: unknown;
    updated_at?: unknown;
    message_count?: unknown;
    total_tokens?: unknown;
    model?: unknown;
    workspace?: unknown;
    mode?: unknown;
  };
  messages?: Array<{
    role?: unknown;
    content?: unknown;
  }>;
};

export function normalizeSession(raw: RawSession, path: string): SessionRecord {
  const metadata = raw.metadata ?? {};
  const id = asString(metadata.id) || fileStem(path);
  const preview = firstUserText(raw.messages ?? []);
  const title = asString(metadata.title) || preview || "(untitled)";

  return {
    id,
    shortId: id.slice(0, 8),
    title,
    preview,
    createdAt: asString(metadata.created_at) || null,
    updatedAt: asString(metadata.updated_at) || null,
    messageCount: asNumber(metadata.message_count, raw.messages?.length ?? 0),
    totalTokens: asNumber(metadata.total_tokens, 0),
    model: asString(metadata.model),
    workspace: asString(metadata.workspace),
    mode: asString(metadata.mode),
    path,
    invalidReason: null
  };
}

export function matchesSession(session: SessionRecord, query: string): boolean {
  const needle = query.trim().toLowerCase();
  if (!needle) {
    return true;
  }

  return [
    session.id,
    session.shortId,
    session.title,
    session.preview,
    session.workspace,
    session.model,
    session.mode
  ].some((value) => value.toLowerCase().includes(needle));
}

export function groupSessions(
  sessions: SessionRecord[],
  groupBy: GroupBy,
  favorites: Set<string>
): SessionGroup[] {
  const sorted = [...sessions].sort(compareUpdatedDesc);
  const grouped = new Map<string, SessionGroup>();

  for (const session of sorted) {
    const { key, label } = getGroupKey(session, groupBy, favorites);
    if (!grouped.has(key)) {
      grouped.set(key, { key, label, sessions: [] });
    }
    grouped.get(key)?.sessions.push(session);
  }

  return [...grouped.values()].sort((left, right) => {
    const leftTime = latestTimestamp(left.sessions);
    const rightTime = latestTimestamp(right.sessions);
    return rightTime - leftTime;
  });
}

export function getGroupKey(
  session: SessionRecord,
  groupBy: GroupBy,
  favorites: Set<string>
): { key: string; label: string } {
  if (groupBy === "favorite") {
    return favorites.has(session.id)
      ? { key: "favorites", label: "收藏" }
      : { key: "others", label: "未收藏" };
  }

  if (groupBy === "date") {
    const key = session.updatedAt ? new Date(session.updatedAt).toLocaleDateString() : "(no date)";
    return { key, label: key };
  }

  if (groupBy === "model") {
    const key = session.model || "(no model)";
    return { key, label: key };
  }

  if (groupBy === "mode") {
    const key = session.mode || "(no mode)";
    return { key, label: key };
  }

  if (groupBy === "none") {
    return { key: "all", label: "全部会话" };
  }

  const key = session.workspace || "(no workspace)";
  return { key, label: workspaceLeafName(key) };
}

export function formatDateTime(value: string | null): string {
  if (!value) {
    return "-";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString();
}

export function formatTokenCount(tokens: number): string {
  if (tokens >= 1000) {
    return `${Math.round(tokens / 1000)}k`;
  }
  return `${tokens}`;
}

export function compareUpdatedDesc(left: SessionRecord, right: SessionRecord): number {
  return toTimestamp(right.updatedAt) - toTimestamp(left.updatedAt);
}

function latestTimestamp(sessions: SessionRecord[]): number {
  return Math.max(...sessions.map((session) => toTimestamp(session.updatedAt)));
}

function toTimestamp(value: string | null): number {
  if (!value) {
    return 0;
  }
  const time = new Date(value).getTime();
  return Number.isNaN(time) ? 0 : time;
}

function firstUserText(messages: NonNullable<RawSession["messages"]>): string {
  for (const message of messages) {
    if (message.role === "user") {
      const text = contentToText(message.content);
      if (text) {
        return text;
      }
    }
  }
  return "";
}

function contentToText(content: unknown): string {
  if (typeof content === "string") {
    return compact(content);
  }

  if (Array.isArray(content)) {
    return compact(
      content
        .map((item) => {
          if (typeof item === "string") {
            return item;
          }
          if (isRecord(item) && item.type === "text" && typeof item.text === "string") {
            return item.text;
          }
          if (isRecord(item) && typeof item.content === "string") {
            return item.content;
          }
          return "";
        })
        .filter(Boolean)
        .join(" ")
    );
  }

  if (isRecord(content) && typeof content.content === "string") {
    return compact(content.content);
  }

  return "";
}

function asString(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function asNumber(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function compact(value: string): string {
  return value.replace(/\s+/g, " ").trim();
}

function fileStem(path: string): string {
  const normalized = path.replaceAll("\\", "/");
  const fileName = normalized.split("/").pop() ?? path;
  return fileName.replace(/\.json$/i, "");
}

function workspaceLeafName(path: string): string {
  if (path === "(no workspace)") {
    return path;
  }

  const normalized = path.replaceAll("\\", "/").replace(/\/+$/, "");
  const leaf = normalized.split("/").pop();
  return leaf || path;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
