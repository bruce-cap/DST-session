/** Formats UI-only session labels and relative timestamps. */

import { formatDateTime } from "./session";
import type { Locale, TFunction } from "./i18n";

export function formatRelative(value: string | null, locale: Locale, t: TFunction): string {
  if (!value) return "-";
  const target = new Date(value).getTime();
  if (Number.isNaN(target)) return formatDateTime(value, locale);
  const delta = Date.now() - target;
  const minute = 60_000;
  const hour = 3600_000;
  const day = 86_400_000;
  if (delta < minute) return t("rel_just_now");
  if (delta < hour) return t("rel_minutes", { n: Math.floor(delta / minute) });
  if (delta < day) return t("rel_hours", { n: Math.floor(delta / hour) });
  if (delta < 7 * day) return t("rel_days", { n: Math.floor(delta / day) });
  return formatDateTime(value, locale);
}

export function workspaceLeaf(path: string): string {
  if (!path) return "";
  const normalized = path.replaceAll("\\", "/").replace(/\/+$/, "");
  return normalized.split("/").pop() || path;
}

export function modelShort(model: string): string {
  return model.replace(/^deepseek-/i, "").replace(/^claude-/i, "");
}

export function toMessage(value: unknown): string {
  if (value instanceof Error) {
    return value.message;
  }
  return String(value);
}
