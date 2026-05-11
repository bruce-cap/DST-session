/** Renders loading, empty, and grouped session list content. */

import { useEffect, useMemo, useState } from "react";
import type { Locale, TFunction } from "../lib/i18n";
import type { ProviderDescriptor, SessionGroup, SessionRecord } from "../types";
import { EmptyState, LoadingState } from "./EmptyState";
import { SessionCard } from "./SessionCard";

const PAGE_SIZE = 100;

export function SessionList(props: {
  loading: boolean;
  visibleGroups: SessionGroup[];
  visibleSessions: SessionRecord[];
  selected: SessionRecord | null;
  favoriteSet: Set<string>;
  provider: ProviderDescriptor | null;
  locale: Locale;
  t: TFunction;
  onSelect: (sessionId: string) => void;
  onToggleFavorite: (session: SessionRecord) => void;
}) {
  const [visibleCount, setVisibleCount] = useState(PAGE_SIZE);

  useEffect(() => {
    setVisibleCount(PAGE_SIZE);
  }, [props.visibleGroups, props.loading]);

  const pagedGroups = useMemo(() => {
    let remaining = visibleCount;
    return props.visibleGroups
      .map((group) => {
        if (remaining <= 0) {
          return { ...group, sessions: [] };
        }
        const sessions = group.sessions.slice(0, remaining);
        remaining -= sessions.length;
        return { ...group, sessions };
      })
      .filter((group) => group.sessions.length > 0);
  }, [props.visibleGroups, visibleCount]);

  const canLoadMore = visibleCount < props.visibleSessions.length;

  return (
    <section className="session-list">
      {props.loading && <LoadingState />}
      {!props.loading && props.visibleSessions.length === 0 && <EmptyState title={props.t("empty_no_match")} hint={props.t("empty_no_match_hint")} />}
      {!props.loading && pagedGroups.map((group) => (
        <div className="session-group" key={group.key}>
          <h2><span>{group.label}</span><em>{group.sessions.length === props.visibleGroups.find((fullGroup) => fullGroup.key === group.key)?.sessions.length ? group.sessions.length : `${group.sessions.length}/${props.visibleGroups.find((fullGroup) => fullGroup.key === group.key)?.sessions.length ?? group.sessions.length}`}</em></h2>
          {group.sessions.map((session) => (
            <SessionCard
              key={`${group.key}-${session.id}`}
              session={session}
              active={props.selected?.id === session.id}
              favoriteSet={props.favoriteSet}
              provider={props.provider}
              locale={props.locale}
              t={props.t}
              onSelect={() => props.onSelect(session.id)}
              onToggleFavorite={() => props.onToggleFavorite(session)}
            />
          ))}
        </div>
      ))}
      {!props.loading && canLoadMore && (
        <button type="button" className="load-more" onClick={() => setVisibleCount((count) => count + PAGE_SIZE)}>
          {props.t("load_more", { count: Math.min(PAGE_SIZE, props.visibleSessions.length - visibleCount) })}
        </button>
      )}
    </section>
  );
}
