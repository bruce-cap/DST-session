/** Renders loading, empty, and grouped session list content. */

import type { Locale, TFunction } from "../lib/i18n";
import type { ProviderDescriptor, SessionGroup, SessionRecord } from "../types";
import { EmptyState, LoadingState } from "./EmptyState";
import { SessionCard } from "./SessionCard";

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
  return (
    <section className="session-list">
      {props.loading && <LoadingState />}
      {!props.loading && props.visibleSessions.length === 0 && <EmptyState title={props.t("empty_no_match")} hint={props.t("empty_no_match_hint")} />}
      {!props.loading && props.visibleGroups.map((group) => (
        <div className="session-group" key={group.key}>
          <h2><span>{group.label}</span><em>{group.sessions.length}</em></h2>
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
    </section>
  );
}
