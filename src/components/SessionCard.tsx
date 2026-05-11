/** Renders a selectable session summary row. */

import { isFavorite } from "../lib/favorites";
import { formatRelative, modelShort, workspaceLeaf } from "../lib/format";
import { formatTokenCount } from "../lib/session";
import type { Locale, TFunction } from "../lib/i18n";
import type { ProviderDescriptor, SessionRecord } from "../types";
import { Icon } from "./Icon";

export function SessionCard(props: {
  session: SessionRecord;
  active: boolean;
  favoriteSet: Set<string>;
  provider: ProviderDescriptor | null;
  locale: Locale;
  t: TFunction;
  onSelect: () => void;
  onToggleFavorite: () => void;
}) {
  const favorite = isFavorite(props.session, props.favoriteSet);
  return (
    <article className={`session-card ${props.active ? "active" : ""} ${props.session.invalidReason ? "invalid" : ""}`} onClick={props.onSelect}>
      <div className="session-card-main">
        <h3 className="session-title">{props.session.title || props.t("untitled")}</h3>
        <div className="session-meta">
          <span className="time">{formatRelative(props.session.updatedAt, props.locale, props.t)}</span>
          <span className="dot" />
          <span className="session-meta-path">{workspaceLeaf(props.session.workspace) || props.session.shortId}</span>
          {props.session.model && <><span className="dot" /><span className="model-tag">{modelShort(props.session.model)}</span></>}
          {props.session.totalTokens > 0 && <><span className="dot" /><span className="token-tag">{formatTokenCount(props.session.totalTokens)} tokens</span></>}
        </div>
      </div>
      {props.provider?.capabilities.favorite && (
        <button type="button" className={`star ${favorite ? "on" : ""}`} aria-label={favorite ? props.t("unfavorite") : props.t("favorite")} onClick={(event) => { event.stopPropagation(); props.onToggleFavorite(); }}>
          <Icon name={favorite ? "star-fill" : "star"} />
        </button>
      )}
    </article>
  );
}
