import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import logoUrl from "./assets/logo.svg";
import { buildResumeCommand, deepseekCommand, normalizeSingleLine } from "./lib/commands";
import { setLocale, useLocale, useT, type Locale, type TFunction } from "./lib/i18n";
import {
  compareUpdatedDesc,
  formatDateTime,
  formatTokenCount,
  groupSessions,
  matchesSession
} from "./lib/session";
import type { AppState, DeepseekLauncher, DeepseekStatus, GroupBy, SessionRecord, SessionSource } from "./types";

const GROUP_KEYS: Record<GroupBy, "group_workspace" | "group_date" | "group_model" | "group_mode" | "group_favorite" | "group_none"> = {
  workspace: "group_workspace",
  date: "group_date",
  model: "group_model",
  mode: "group_mode",
  favorite: "group_favorite",
  none: "group_none"
};

const GROUP_ORDER: GroupBy[] = ["workspace", "date", "model", "mode", "favorite", "none"];

const APP_VERSION = __APP_VERSION__;

const defaultState: AppState = {
  favorites: [],
  launchMode: "new_terminal",
  deepseekLauncher: "cmd"
};

type ThemeMode = "light" | "dark";

export default function App() {
  const t = useT();
  const locale = useLocale();
  const [source, setSource] = useState<SessionSource>("deepseek");
  const [sessions, setSessions] = useState<SessionRecord[]>([]);
  const [appState, setAppState] = useState<AppState>(defaultState);
  const [status, setStatus] = useState<DeepseekStatus | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [groupBy, setGroupBy] = useState<GroupBy>("workspace");
  const [activeGroupKey, setActiveGroupKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [theme, setTheme] = useState<ThemeMode>(() => initialTheme());
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [quickReply, setQuickReply] = useState("");
  const settingsRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    void loadAll();
  }, [source]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    localStorage.setItem("deepseek-session-manager-theme", theme);
  }, [theme]);

  useEffect(() => {
    setActiveGroupKey(null);
  }, [source, groupBy, search]);

  useEffect(() => {
    // Clear quick-reply composer when the user jumps to another session or source.
    setQuickReply("");
  }, [selectedId, source]);

  useEffect(() => {
    if (!settingsOpen) {
      return;
    }
    function onClickAway(event: MouseEvent) {
      if (!settingsRef.current?.contains(event.target as Node)) {
        setSettingsOpen(false);
      }
    }
    window.addEventListener("mousedown", onClickAway);
    return () => window.removeEventListener("mousedown", onClickAway);
  }, [settingsOpen]);

  useEffect(() => {
    if (!notice) {
      return;
    }
    const timer = window.setTimeout(() => setNotice(null), 2400);
    return () => window.clearTimeout(timer);
  }, [notice]);

  const favoriteSet = new Set(appState.favorites);
  const filtered = sessions
    .filter((session) => matchesSession(session, search))
    .sort(compareUpdatedDesc);
  const groups = groupSessions(filtered, groupBy, favoriteSet);
  const visibleGroups = activeGroupKey
    ? groups.filter((group) => group.key === activeGroupKey)
    : groups;
  const visibleSessions = visibleGroups.flatMap((group) => group.sessions);
  const selected =
    sessions.find((session) => session.id === selectedId) ?? visibleSessions[0] ?? null;
  const invalidCount = sessions.filter((session) => session.invalidReason).length;
  const cliMissing = Boolean(status && !status.available);

  async function loadAll() {
    setLoading(true);
    setError(null);
    try {
      const state = await invoke<AppState>("get_app_state");
      const [records, deepseek] = await Promise.all([
        invoke<SessionRecord[]>("list_sessions", { source }),
        invoke<DeepseekStatus>("check_agent", {
          source,
          deepseekLauncher: state.deepseekLauncher
        })
      ]);
      setSessions(records);
      setAppState(state);
      setStatus(deepseek);
      setSelectedId(records[0]?.id ?? null);
    } catch (caught) {
      setSessions([]);
      setError(toMessage(caught));
    } finally {
      setLoading(false);
    }
  }

  async function toggleFavorite(session: SessionRecord) {
    try {
      const nextState = await invoke<AppState>("set_favorite", {
        sessionId: favoriteKey(session),
        favorite: !isFavorite(session, favoriteSet)
      });
      setAppState(nextState);
    } catch (caught) {
      setError(toMessage(caught));
    }
  }

  async function copyCommand(session: SessionRecord, prompt?: string) {
    const command = resumeCommand(session, appState.deepseekLauncher, prompt);
    await navigator.clipboard.writeText(command);
    setNotice(t("copied"));
  }

  async function openFolder(session: SessionRecord) {
    try {
      await invoke("open_session_folder", { path: session.path });
    } catch (caught) {
      setError(toMessage(caught));
    }
  }

  async function resume(session: SessionRecord, prompt?: string) {
    setNotice(null);
    const normalizedPrompt = prompt ? normalizeSingleLine(prompt) : "";
    const effectivePrompt = session.source === "codex" && normalizedPrompt
      ? normalizedPrompt
      : undefined;
    try {
      await invoke("resume_session", {
        source: session.source,
        sessionId: session.id,
        workspace: session.workspace || null,
        launchMode: appState.launchMode,
        deepseekLauncher: appState.deepseekLauncher,
        prompt: effectivePrompt ?? null
      });
      const finalCommand = resumeCommand(session, appState.deepseekLauncher, effectivePrompt);
      setNotice(
        effectivePrompt
          ? t("quick_reply_launched", { command: finalCommand })
          : t("launched", { command: finalCommand })
      );
      if (effectivePrompt) {
        setQuickReply("");
      }
    } catch (caught) {
      setError(toMessage(caught));
    }
  }

  async function changeDeepseekLauncher(next: DeepseekLauncher) {
    try {
      const nextState = await invoke<AppState>("set_deepseek_launcher", { launcher: next });
      setAppState(nextState);
      if (source === "deepseek") {
        const nextStatus = await invoke<DeepseekStatus>("check_agent", {
          source,
          deepseekLauncher: nextState.deepseekLauncher
        });
        setStatus(nextStatus);
      }
    } catch (caught) {
      setError(toMessage(caught));
    }
  }

  return (
    <main className="app-shell">
      <header className="titlebar">
        <div className="brand">
          <div className="brand-mark">
            <img src={logoUrl} alt="" />
          </div>
          <div className="brand-text">
            <strong>Session Manager</strong>
            <span>DeepSeek · Claude Code</span>
          </div>
        </div>

        <div className="search-input titlebar-search">
          <Icon name="search" />
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder={t("search_placeholder")}
          />
          {search ? (
            <button
              type="button"
              className="input-clear"
              aria-label={t("clear_search")}
              onClick={() => setSearch("")}
            >
              <Icon name="close" />
            </button>
          ) : (
            <kbd className="kbd">⌘K</kbd>
          )}
        </div>

        <div className="titlebar-actions">
          <IconButton
            label={t("action_refresh")}
            icon="refresh"
            onClick={() => void loadAll()}
          />
          <div className="settings-wrap" ref={settingsRef}>
            <IconButton
              label={t("action_settings")}
              icon="settings"
              onClick={() => setSettingsOpen((open) => !open)}
              ariaExpanded={settingsOpen}
              active={settingsOpen}
              tooltipAlign="end"
            />
            {settingsOpen && (
              <div className="settings-menu" role="dialog">
                <div className="settings-title">
                  <span>{t("action_settings")}</span>
                  <span className="settings-version">v{APP_VERSION}</span>
                </div>
                <div className="settings-row">
                  <span>{t("settings_language")}</span>
                  <LocaleToggle locale={locale} onChange={setLocale} />
                </div>
                <div className="settings-row">
                  <span>{t("settings_dark")}</span>
                  <button
                    type="button"
                    className={`switch ${theme === "dark" ? "on" : ""}`}
                    onClick={() => setTheme((current) => (current === "dark" ? "light" : "dark"))}
                    aria-pressed={theme === "dark"}
                  >
                    <i />
                  </button>
                </div>
                <div className="settings-note">
                  <span>{t("settings_launch_mode")}</span>
                  <b>{t("settings_launch_mode_value")}</b>
                </div>
                <div className="settings-row">
                  <span>{t("settings_deepseek_launcher")}</span>
                  <LauncherToggle value={appState.deepseekLauncher} onChange={(next) => void changeDeepseekLauncher(next)} />
                </div>
                <div className={`settings-status ${status?.available ? "ok" : "bad"}`}>
                  <span className="status-dot" />
                  <div>
                    <b>{sourceCommand(source, appState.deepseekLauncher)}</b>
                    <small>
                      {status?.available
                        ? status.version || t("settings_cli_available")
                        : status?.message || t("settings_cli_unavailable")}
                    </small>
                  </div>
                </div>
                <p>{t("settings_footer", { source: sourceLabel(source, t) })}</p>
              </div>
            )}
          </div>
        </div>
      </header>

      {(error || notice || invalidCount > 0 || cliMissing) && (
        <section className="message-stack">
          {error && (
            <div className="message error">
              <Icon name="alert" /> <span>{error}</span>
            </div>
          )}
          {cliMissing && !error && (
            <div className="message error">
              <Icon name="alert" />
              <span>{t("cli_missing", { command: sourceCommand(source, appState.deepseekLauncher) })}</span>
            </div>
          )}
          {notice && (
            <div className="message success">
              <Icon name="check" /> <span>{notice}</span>
            </div>
          )}
          {invalidCount > 0 && (
            <div className="message warn">
              <Icon name="alert" /> <span>{t("invalid_count", { count: invalidCount })}</span>
            </div>
          )}
        </section>
      )}

      <section className="workspace">
        <aside className="sidebar">
          <div className="source-picker">
            <button
              type="button"
              className={`source-card ${source === "deepseek" ? "active" : ""}`}
              onClick={() => setSource("deepseek")}
            >
              <span className="source-card-badge deepseek">
                <img src={logoUrl} alt="" />
              </span>
              <span className="source-card-text">
                <b>DeepSeek</b>
                <small>TUI</small>
              </span>
            </button>
            <button
              type="button"
              className={`source-card ${source === "claude" ? "active" : ""}`}
              onClick={() => setSource("claude")}
            >
              <span className="source-card-badge claude">C</span>
              <span className="source-card-text">
                <b>Claude</b>
                <small>Code</small>
              </span>
            </button>
            <button
              type="button"
              className={`source-card ${source === "codex" ? "active" : ""}`}
              onClick={() => setSource("codex")}
            >
              <span className="source-card-badge codex">O</span>
              <span className="source-card-text">
                <b>Codex</b>
                <small>CLI</small>
              </span>
            </button>
          </div>

          <div className="sidebar-section">
            <div className="sidebar-label">
              <span>{t("group_by")}</span>
            </div>
            <div className="group-select">
              {GROUP_ORDER.map((option) => (
                <button
                  key={option}
                  type="button"
                  className={`group-select-item ${groupBy === option ? "active" : ""}`}
                  onClick={() => setGroupBy(option)}
                >
                  {t(GROUP_KEYS[option])}
                </button>
              ))}
            </div>
          </div>

          <div className="sidebar-section">
            <div className="sidebar-label">
              <span>{t("panel_groups")}</span>
              <b>{filtered.length}</b>
            </div>
            <div className="group-list">
              <button
                type="button"
                className={`group-item ${activeGroupKey === null ? "active" : ""}`}
                onClick={() => setActiveGroupKey(null)}
              >
                <span>{t("group_all")}</span>
                <em>{filtered.length}</em>
              </button>
              {groups.map((group) => (
                <button
                  type="button"
                  className={`group-item ${activeGroupKey === group.key ? "active" : ""}`}
                  key={group.key}
                  onClick={() => {
                    setActiveGroupKey((current) => (current === group.key ? null : group.key));
                    setSelectedId(group.sessions[0]?.id ?? null);
                  }}
                  title={group.key}
                >
                  <span>{group.label}</span>
                  <em>{group.sessions.length}</em>
                </button>
              ))}
            </div>
          </div>
        </aside>

        <section className="session-list">
          {loading && (
            <div className="empty">
              <div className="skeleton" />
              <div className="skeleton" />
              <div className="skeleton" />
            </div>
          )}
          {!loading && visibleSessions.length === 0 && (
            <div className="empty empty-state">
              <Icon name="inbox" />
              <p>{t("empty_no_match")}</p>
              <small>{t("empty_no_match_hint")}</small>
            </div>
          )}
          {!loading &&
            visibleGroups.map((group) => (
              <div className="session-group" key={group.key}>
                <h2>
                  <span>{group.label}</span>
                  <em>{group.sessions.length}</em>
                </h2>
                {group.sessions.map((session) => {
                  const favorite = isFavorite(session, favoriteSet);
                  const active = selected?.id === session.id;
                  return (
                    <article
                      key={`${group.key}-${session.id}`}
                      className={`session-card ${active ? "active" : ""} ${session.invalidReason ? "invalid" : ""}`}
                      onClick={() => setSelectedId(session.id)}
                    >
                      <div className="session-card-main">
                        <h3 className="session-title">{session.title || t("untitled")}</h3>
                        <div className="session-meta">
                          <span className="time">{formatRelative(session.updatedAt, locale, t)}</span>
                          <span className="dot" />
                          <span className="session-meta-path">
                            {workspaceLeaf(session.workspace) || session.shortId}
                          </span>
                          {session.model && (
                            <>
                              <span className="dot" />
                              <span className="model-tag">{modelShort(session.model)}</span>
                            </>
                          )}
                        </div>
                      </div>
                      <button
                        type="button"
                        className={`star ${favorite ? "on" : ""}`}
                        aria-label={favorite ? t("unfavorite") : t("favorite")}
                        onClick={(event) => {
                          event.stopPropagation();
                          void toggleFavorite(session);
                        }}
                      >
                        <Icon name={favorite ? "star-fill" : "star"} />
                      </button>
                    </article>
                  );
                })}
              </div>
            ))}
        </section>

        <aside className="detail-panel">
          {selected ? (
            <SessionDetails
              session={selected}
              favorite={isFavorite(selected, favoriteSet)}
              deepseekLauncher={appState.deepseekLauncher}
              locale={locale}
              status={status}
              quickReply={quickReply}
              onQuickReplyChange={setQuickReply}
              t={t}
              onResume={() => void resume(selected, selected.source === "codex" ? quickReply : undefined)}
              onToggleFavorite={() => void toggleFavorite(selected)}
              onCopyCommand={() =>
                void copyCommand(selected, selected.source === "codex" ? quickReply : undefined)
              }
              onOpenFolder={() => void openFolder(selected)}
            />
          ) : (
            <div className="empty empty-state">
              <Icon name="inbox" />
              <p>{t("empty_select")}</p>
            </div>
          )}
        </aside>
      </section>
    </main>
  );
}

function SessionDetails(props: {
  session: SessionRecord;
  favorite: boolean;
  deepseekLauncher: DeepseekLauncher;
  locale: Locale;
  status: DeepseekStatus | null;
  quickReply: string;
  onQuickReplyChange: (value: string) => void;
  t: TFunction;
  onResume: () => void;
  onToggleFavorite: () => void;
  onCopyCommand: () => void;
  onOpenFolder: () => void;
}) {
  const { session, t, locale, status, quickReply, onQuickReplyChange } = props;
  const isCodex = session.source === "codex";
  const promptPreview = isCodex ? normalizeSingleLine(quickReply) : "";
  const command = resumeCommand(
    session,
    props.deepseekLauncher,
    isCodex && promptPreview ? promptPreview : undefined
  );
  const workspaceMissing = Boolean(session.workspace) && !session.workspace.match(/^[A-Za-z]:\\/);
  const canResume = status?.available && !session.invalidReason;
  const [copied, setCopied] = useState(false);

  function handleCopy() {
    props.onCopyCommand();
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  }

  function handleQuickReplyKeyDown(event: React.KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Enter" && canResume && promptPreview) {
      event.preventDefault();
      props.onResume();
    }
  }

  return (
    <div className="details">
      <header className="hero">
        <div className="hero-eyebrow">
          <span className={`source-dot ${session.source}`} />
          <span>{sourceLabelShort(session.source)}</span>
          <span className="dot" />
          <span>{formatRelative(session.updatedAt, locale, t)}</span>
          <button
            type="button"
            className={`hero-star ${props.favorite ? "on" : ""}`}
            aria-label={props.favorite ? t("unfavorite") : t("favorite")}
            onClick={props.onToggleFavorite}
          >
            <Icon name={props.favorite ? "star-fill" : "star"} />
          </button>
        </div>
        <h1 className="hero-title">{session.title || t("untitled")}</h1>
        {session.preview && <p className="hero-preview">{session.preview}</p>}

        <div className="hero-actions">
          <button
            type="button"
            className="btn-primary"
            onClick={props.onResume}
            disabled={!canResume}
          >
            <Icon name="play" />
            <span>
              {isCodex && promptPreview ? t("quick_reply_send") : t("action_launch")}
            </span>
            <kbd className="kbd inverse">⏎</kbd>
          </button>
          <button type="button" className="btn-ghost" onClick={props.onOpenFolder}>
            <Icon name="folder" />
            <span>{t("action_open_folder")}</span>
          </button>
        </div>
      </header>

      {session.invalidReason && (
        <div className="message error">
          <Icon name="alert" /> <span>{session.invalidReason}</span>
        </div>
      )}

      {isCodex && (
        <section className="quick-reply">
          <div className="section-head">{t("quick_reply_label")}</div>
          <div className="quick-reply-row">
            <input
              type="text"
              value={quickReply}
              onChange={(event) => onQuickReplyChange(event.target.value)}
              onKeyDown={handleQuickReplyKeyDown}
              placeholder={t("quick_reply_placeholder")}
            />
            <button
              type="button"
              className="btn-primary btn-primary-sm"
              onClick={props.onResume}
              disabled={!canResume || !promptPreview}
            >
              <span>{t("quick_reply_send")}</span>
              <kbd className="kbd inverse">⏎</kbd>
            </button>
          </div>
          <p className="hint">{t("quick_reply_hint")}</p>
        </section>
      )}

      <div className="hero-stats">
        <div className="stat">
          <b>{session.messageCount || "—"}</b>
          <span>{t("stat_messages")}</span>
        </div>
        <div className="stat">
          <b>{session.totalTokens ? formatTokenCount(session.totalTokens) : "—"}</b>
          <span>{t("stat_tokens")}</span>
        </div>
        <div className="stat">
          <b>{session.model ? modelShort(session.model) : "—"}</b>
          <span>model</span>
        </div>
      </div>

      <section className="terminal">
        <div className="terminal-head">
          <span className="terminal-dots">
            <i />
            <i />
            <i />
          </span>
          <span className="terminal-title">{t("command_label")}</span>
          <button
            type="button"
            className={`terminal-copy ${copied ? "copied" : ""}`}
            onClick={handleCopy}
          >
            <Icon name={copied ? "check" : "copy"} />
            <span>{copied ? t("copy_done") : t("copy_now")}</span>
          </button>
        </div>
        <pre className="terminal-body">
          <span className="prompt">$</span>
          <code>{command}</code>
          <span className="caret" />
        </pre>
      </section>

      <section className="detail-section">
        <div className="section-head">{t("label_workspace")}</div>
        <dl>
          <dt>{t("label_id")}</dt>
          <dd className="mono">{session.id}</dd>
          <dt>{t("label_updated")}</dt>
          <dd>{formatDateTime(session.updatedAt, locale)}</dd>
          <dt>{t("label_created")}</dt>
          <dd>{formatDateTime(session.createdAt, locale)}</dd>
          <dt>{t("label_workspace")}</dt>
          <dd className="mono">{session.workspace || t("not_recorded")}</dd>
          {session.mode && (
            <>
              <dt>Mode</dt>
              <dd>{session.mode}</dd>
            </>
          )}
          <dt>{t("label_file")}</dt>
          <dd className="mono faint">{session.path || t("not_recorded")}</dd>
        </dl>
      </section>

      {workspaceMissing && <p className="hint warn">{t("hint_workspace_missing")}</p>}
    </div>
  );
}

function IconButton(props: {
  label: string;
  icon: IconName;
  onClick: () => void;
  disabled?: boolean;
  primary?: boolean;
  active?: boolean;
  ariaExpanded?: boolean;
  tooltipAlign?: "start" | "center" | "end";
}) {
  return (
    <button
      type="button"
      className={`icon-button tooltip-target ${props.primary ? "primary" : ""} ${props.active ? "active" : ""}`}
      onClick={props.onClick}
      disabled={props.disabled}
      aria-label={props.label}
      aria-expanded={props.ariaExpanded}
      data-tooltip={props.label}
      data-tooltip-align={props.tooltipAlign ?? "center"}
    >
      <Icon name={props.icon} />
    </button>
  );
}

function LocaleToggle({ locale, onChange }: { locale: Locale; onChange: (locale: Locale) => void }) {
  return (
    <div className="segmented" role="group" aria-label="Language">
      <button
        type="button"
        className={locale === "zh" ? "active" : ""}
        onClick={() => onChange("zh")}
        aria-pressed={locale === "zh"}
      >
        中
      </button>
      <button
        type="button"
        className={locale === "en" ? "active" : ""}
        onClick={() => onChange("en")}
        aria-pressed={locale === "en"}
      >
        EN
      </button>
    </div>
  );
}

function LauncherToggle({ value, onChange }: { value: DeepseekLauncher; onChange: (value: DeepseekLauncher) => void }) {
  return (
    <div className="segmented" role="group" aria-label="DeepSeek launcher">
      <button
        type="button"
        className={value === "cmd" ? "active" : ""}
        onClick={() => onChange("cmd")}
        aria-pressed={value === "cmd"}
      >
        cmd
      </button>
      <button
        type="button"
        className={value === "ps1" ? "active" : ""}
        onClick={() => onChange("ps1")}
        aria-pressed={value === "ps1"}
      >
        ps1
      </button>
    </div>
  );
}

type IconName =
  | "play"
  | "copy"
  | "folder"
  | "folder-small"
  | "refresh"
  | "settings"
  | "layers"
  | "search"
  | "close"
  | "chevron"
  | "star"
  | "star-fill"
  | "check"
  | "alert"
  | "inbox"
  | "calendar"
  | "cpu"
  | "mode";

function Icon({ name }: { name: IconName }) {
  switch (name) {
    case "play":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M8 5.8v12.4c0 .9 1 1.4 1.7.9l9-6.2c.6-.4.6-1.3 0-1.8l-9-6.2C9 4.4 8 4.9 8 5.8Z" />
        </svg>
      );
    case "copy":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <rect x="8" y="6" width="12" height="13" rx="2.5" />
          <path d="M5 13.5V7a2 2 0 0 1 2-2h6.5" />
        </svg>
      );
    case "folder":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M3.5 7.5A2.5 2.5 0 0 1 6 5h4l2 2h6A2.5 2.5 0 0 1 20.5 9.5v6A3.5 3.5 0 0 1 17 19H7a3.5 3.5 0 0 1-3.5-3.5v-8Z" />
        </svg>
      );
    case "folder-small":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M3.5 7.5A2 2 0 0 1 5.5 5.5H9l2 2h7A2 2 0 0 1 20 9.5v6A2.5 2.5 0 0 1 17.5 18h-12A2.5 2.5 0 0 1 3 15.5v-8Z" />
        </svg>
      );
    case "refresh":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M18.5 9A7 7 0 0 0 6 6.8L4.5 8.5" />
          <path d="M4.5 4.5v4h4" />
          <path d="M5.5 15A7 7 0 0 0 18 17.2l1.5-1.7" />
          <path d="M19.5 19.5v-4h-4" />
        </svg>
      );
    case "settings":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="3.2" />
          <path d="M19.4 13.2c.1-.4.1-.8.1-1.2s0-.8-.1-1.2l2-1.5-2-3.4-2.4 1a8 8 0 0 0-2.1-1.2L14.4 3h-4.8l-.4 2.7c-.8.3-1.5.7-2.1 1.2l-2.4-1-2 3.4 2 1.5a8.3 8.3 0 0 0 0 2.4l-2 1.5 2 3.4 2.4-1c.6.5 1.3.9 2.1 1.2l.4 2.7h4.8l.4-2.7c.8-.3 1.5-.7 2.1-1.2l2.4 1 2-3.4-2.1-1.5Z" />
        </svg>
      );
    case "layers":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="m12 4 9 4.8-9 4.8-9-4.8Z" />
          <path d="m3 14.4 9 4.8 9-4.8" />
        </svg>
      );
    case "search":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="11" cy="11" r="6.5" />
          <path d="m20 20-3.5-3.5" />
        </svg>
      );
    case "close":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="m6 6 12 12M18 6 6 18" />
        </svg>
      );
    case "chevron":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="m7 10 5 5 5-5" />
        </svg>
      );
    case "star":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="m12 3.6 2.7 5.5 6 .9-4.35 4.2 1.03 6-5.38-2.82-5.38 2.82 1.03-6L3.3 10l6-.9Z" />
        </svg>
      );
    case "star-fill":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true" className="filled">
          <path d="m12 3.6 2.7 5.5 6 .9-4.35 4.2 1.03 6-5.38-2.82-5.38 2.82 1.03-6L3.3 10l6-.9Z" />
        </svg>
      );
    case "check":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="m5 12 5 5 9-10" />
        </svg>
      );
    case "alert":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M12 3.5 21 20H3Z" />
          <path d="M12 10v4.2M12 17.2v.2" />
        </svg>
      );
    case "inbox":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M4 4.5h16V14h-5l-1.5 2h-3L9 14H4Z" />
          <path d="M4 14v5a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-5" />
        </svg>
      );
    case "calendar":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <rect x="3.5" y="5" width="17" height="15" rx="2.5" />
          <path d="M3.5 10h17M8 3.5v4M16 3.5v4" />
        </svg>
      );
    case "cpu":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <rect x="6" y="6" width="12" height="12" rx="2" />
          <rect x="9.5" y="9.5" width="5" height="5" rx="1" />
          <path d="M10 3.5v2.5M14 3.5v2.5M10 18v2.5M14 18v2.5M3.5 10H6M3.5 14H6M18 10h2.5M18 14h2.5" />
        </svg>
      );
    case "mode":
      return (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <circle cx="12" cy="12" r="8" />
          <path d="M12 4v16" />
        </svg>
      );
  }
}

function resumeCommand(
  session: SessionRecord,
  deepseekLauncher: DeepseekLauncher,
  prompt?: string
): string {
  return buildResumeCommand(session.source, session.id, deepseekLauncher, prompt);
}

function toMessage(value: unknown): string {
  if (value instanceof Error) {
    return value.message;
  }
  return String(value);
}

function initialTheme(): ThemeMode {
  const saved = localStorage.getItem("deepseek-session-manager-theme");
  if (saved === "light" || saved === "dark") {
    return saved;
  }
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function favoriteKey(session: SessionRecord): string {
  return `${session.source}:${session.id}`;
}

function isFavorite(session: SessionRecord, favorites: Set<string>): boolean {
  return favorites.has(favoriteKey(session)) || favorites.has(session.id);
}

function sourceLabel(source: SessionSource, t: TFunction): string {
  if (source === "claude") return t("source_claude");
  if (source === "codex") return t("source_codex");
  return t("source_deepseek");
}

function sourceLabelShort(source: SessionSource): string {
  if (source === "claude") return "Claude Code";
  if (source === "codex") return "Codex";
  return "DeepSeek TUI";
}

function sourceCommand(source: SessionSource, deepseekLauncher: DeepseekLauncher = "cmd"): string {
  if (source === "claude") return "claude";
  if (source === "codex") return "codex.ps1";
  return deepseekCommand(deepseekLauncher);
}

function formatRelative(value: string | null, locale: Locale, t: TFunction): string {
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

function workspaceLeaf(path: string): string {
  if (!path) return "";
  const normalized = path.replaceAll("\\", "/").replace(/\/+$/, "");
  return normalized.split("/").pop() || path;
}

function modelShort(model: string): string {
  return model.replace(/^deepseek-/i, "").replace(/^claude-/i, "");
}
