/** Renders the selected session detail panel and actions. */

import { buildResumeCommand, normalizeSingleLine } from "../lib/commands";
import { isFavorite } from "../lib/favorites";
import { formatRelative, modelShort } from "../lib/format";
import { formatDateTime, formatTokenCount } from "../lib/session";
import type { Locale, TFunction } from "../lib/i18n";
import type { AppState, DeepseekStatus, ProviderDescriptor, SessionRecord } from "../types";
import { CommandTerminal } from "./CommandTerminal";
import { EmptyState } from "./EmptyState";
import { Icon } from "./Icon";
import { QuickReply } from "./QuickReply";

export function SessionDetails(props: {
  session: SessionRecord | null;
  provider: ProviderDescriptor | null;
  favoriteSet: Set<string>;
  appState: AppState;
  locale: Locale;
  status: DeepseekStatus | null;
  quickReply: string;
  t: TFunction;
  onQuickReplyChange: (value: string) => void;
  onResume: (session: SessionRecord, prompt?: string) => void;
  onToggleFavorite: (session: SessionRecord) => void;
  onCopyCommand: (session: SessionRecord, prompt?: string) => void;
  onOpenFolder: (session: SessionRecord) => void;
}) {
  if (!props.session) {
    return <EmptyState title={props.t("empty_select")} />;
  }

  const session = props.session;
  const capabilities = props.provider?.capabilities;
  const promptPreview = capabilities?.quickReply ? normalizeSingleLine(props.quickReply) : "";
  const command = buildResumeCommand(session.source, session.id, props.appState.deepseekLauncher, promptPreview || undefined);
  const workspaceMissing = Boolean(session.workspace) && !session.workspace.match(/^[A-Za-z]:\\/);
  const canResume = Boolean(props.status?.available && !session.invalidReason && capabilities?.resume);
  const favorite = isFavorite(session, props.favoriteSet);
  const summary = session.preview || session.title || props.t("untitled");

  return (
    <div className="details">
      <header className="hero">
        <div className="hero-eyebrow">
          <span className={`source-dot ${props.provider?.badgeKey ?? ""}`} />
          <span>{props.provider?.shortName ?? ""}</span>
          <span className="dot" />
          <span>{formatRelative(session.updatedAt, props.locale, props.t)}</span>
          {capabilities?.favorite && (
            <button type="button" className={`hero-star ${favorite ? "on" : ""}`} aria-label={favorite ? props.t("unfavorite") : props.t("favorite")} onClick={() => props.onToggleFavorite(session)}>
              <Icon name={favorite ? "star-fill" : "star"} />
            </button>
          )}
        </div>
        <div className="hero-summary">
          <span>{props.t("preview_label")}</span>
          <p>{summary}</p>
        </div>
        <div className="hero-actions">
          {capabilities?.resume && (
            <button type="button" className="btn-primary" onClick={() => props.onResume(session, promptPreview || undefined)} disabled={!canResume}>
              <Icon name="play" />
              <span>{promptPreview ? props.t("quick_reply_send") : props.t("action_launch")}</span>
              <kbd className="kbd inverse">⏎</kbd>
            </button>
          )}
          {capabilities?.openSessionFolder && (
            <button type="button" className="btn-ghost" onClick={() => props.onOpenFolder(session)}>
              <Icon name="folder" />
              <span>{props.t("action_open_folder")}</span>
            </button>
          )}
        </div>
      </header>

      {session.invalidReason && <div className="message error"><Icon name="alert" /> <span>{session.invalidReason}</span></div>}
      {capabilities?.quickReply && <QuickReply value={props.quickReply} canResume={canResume} t={props.t} onChange={props.onQuickReplyChange} onSubmit={() => props.onResume(session, promptPreview || undefined)} />}

      <div className="hero-stats">
        <div className="stat"><b>{session.messageCount || "—"}</b><span>{props.t("stat_messages")}</span></div>
        <div className="stat"><b>{session.totalTokens ? formatTokenCount(session.totalTokens) : "—"}</b><span>{props.t("stat_tokens")}</span></div>
        <div className="stat"><b>{session.model ? modelShort(session.model) : "—"}</b><span>model</span></div>
      </div>

      {capabilities?.copyCommand && <CommandTerminal command={command} t={props.t} onCopy={() => props.onCopyCommand(session, promptPreview || undefined)} />}

      <section className="detail-section">
        <div className="section-head">{props.t("label_workspace")}</div>
        <dl>
          <dt>{props.t("label_id")}</dt><dd className="mono">{session.id}</dd>
          <dt>{props.t("label_updated")}</dt><dd>{formatDateTime(session.updatedAt, props.locale)}</dd>
          <dt>{props.t("label_created")}</dt><dd>{formatDateTime(session.createdAt, props.locale)}</dd>
          <dt>{props.t("label_workspace")}</dt><dd className="mono">{session.workspace || props.t("not_recorded")}</dd>
          {session.mode && <><dt>Mode</dt><dd>{session.mode}</dd></>}
          <dt>{props.t("label_file")}</dt><dd className="mono faint">{session.path || props.t("not_recorded")}</dd>
        </dl>
      </section>
      {workspaceMissing && <p className="hint warn">{props.t("hint_workspace_missing")}</p>}
    </div>
  );
}
