/** Provides favorite key helpers for session records. */

import type { SessionRecord } from "../types";

export function favoriteKey(session: SessionRecord): string {
  return `${session.source}:${session.id}`;
}

export function isFavorite(session: SessionRecord, favorites: Set<string>): boolean {
  return favorites.has(favoriteKey(session)) || favorites.has(session.id);
}
