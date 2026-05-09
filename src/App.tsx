import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import logoUrl from "./assets/logo.svg";
import { buildResumeCommand } from "./lib/commands";
import {
  compareUpdatedDesc,
  formatDateTime,
  formatTokenCount,
  groupSessions,
  matchesSession
} from "./lib/session";
import type { AppState, DeepseekStatus, GroupBy, SessionRecord, SessionSource } from "./types";

const GROUP_OPTIONS: Array<{ value: GroupBy; label: string }> = [
  { value: "workspace", label: "按项目" },
  { value: "date", label: "按日期" },
  { value: "model", label: "按模型" },
  { value: "mode", label: "按模式" },
  { value: "favorite", label: "按收藏" },
  { value: "none", label: "全部" }
];

const APP_VERSION = __APP_VERSION__;

const defaultState: AppState = {
  favorites: [],
  launchMode: "new_terminal"
};

type ThemeMode = "light" | "dark";

export default function App() {
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
      const [records, state, deepseek] = await Promise.all([
        invoke<SessionRecord[]>("list_sessions", { source }),
        invoke<AppState>("get_app_state"),
        invoke<DeepseekStatus>("check_agent", { source })
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

  async function copyCommand(session: SessionRecord) {
    const command = resumeCommand(session);
    await navigator.clipboard.writeText(command);
    setNotice("恢复命令已复制");
  }

  async function openFolder(session: SessionRecord) {
    try {
      await invoke("open_session_folder", { path: session.path });
    } catch (caught) {
      setError(toMessage(caught));
    }
  }

  async function resume(session: SessionRecord) {
    setNotice(null);
    try {
      await invoke("resume_session", {
        source: session.source,
        sessionId: session.id,
        workspace: session.workspace || null,
        launchMode: appState.launchMode
      });
      setNotice(`已启动：${resumeCommand(session)}`);
    } catch (caught) {
      setError(toMessage(caught));
    }
  }

  return (
    <main className="app-shell">
      <header className="app-bar">
        <div className="brand">
          <img src={logoUrl} alt="DeepSeek Session Manager logo" />
          <div>
            <strong>DeepSeek Session Manager</strong>
            <span>{sourceLabel(source)} 会话浏览与恢复</span>
          </div>
        </div>

        <div className="top-search">
          <div className="source-switch" aria-label="会话来源">
            <button
              type="button"
              className={source === "deepseek" ? "active tooltip-target" : "tooltip-target"}
              onClick={() => setSource("deepseek")}
              data-tooltip="DeepSeek TUI"
              aria-label="DeepSeek TUI"
            >
              <img src={logoUrl} alt="" />
            </button>
            <button
              type="button"
              className={source === "claude" ? "active tooltip-target" : "tooltip-target"}
              onClick={() => setSource("claude")}
              data-tooltip="Claude Code"
              aria-label="Claude Code"
            >
              <span className="claude-mark">C</span>
            </button>
          </div>
          <div className="search-input">
            <Icon name="search" />
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="搜索标题、消息、workspace、模型或 ID"
            />
            {search && (
              <button
                type="button"
                className="input-clear"
                aria-label="清空搜索"
                onClick={() => setSearch("")}
              >
                <Icon name="close" />
              </button>
            )}
          </div>
          <label className="select-chip tooltip-target" data-tooltip="分组方式">
            <Icon name="layers" />
            <select value={groupBy} onChange={(event) => setGroupBy(event.target.value as GroupBy)}>
              {GROUP_OPTIONS.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
            <Icon name="chevron" />
          </label>
        </div>

        <div className="top-actions">
          <IconButton
            label="启动会话"
            icon="play"
            primary
            onClick={() => selected && void resume(selected)}
            disabled={!selected || !status?.available || Boolean(selected.invalidReason)}
          />
          <IconButton label="复制恢复命令" icon="copy" onClick={() => selected && void copyCommand(selected)} disabled={!selected} />
          <IconButton label="打开会话文件目录" icon="folder" onClick={() => selected && void openFolder(selected)} disabled={!selected} />
          <IconButton label="刷新" icon="refresh" onClick={() => void loadAll()} />
          <div className="settings-wrap" ref={settingsRef}>
            <IconButton
              label="设置"
              icon="settings"
              onClick={() => setSettingsOpen((open) => !open)}
              ariaExpanded={settingsOpen}
              active={settingsOpen}
            />
            {settingsOpen && (
              <div className="settings-menu" role="dialog">
                <div className="settings-title">
                  <span>设置</span>
                  <span className="settings-version">v{APP_VERSION}</span>
                </div>
                <label className="settings-row">
                  <span>暗色主题</span>
                  <button
                    type="button"
                    className={`switch ${theme === "dark" ? "on" : ""}`}
                    onClick={() => setTheme((current) => (current === "dark" ? "light" : "dark"))}
                    aria-pressed={theme === "dark"}
                  >
                    <i />
                  </button>
                </label>
                <div className="settings-note">
                  <span>启动方式</span>
                  <b>新系统终端</b>
                </div>
                <div className={`settings-status ${status?.available ? "ok" : "bad"}`}>
                  <span className="status-dot" />
                  <div>
                    <b>{sourceCommand(source)}</b>
                    <small>
                      {status?.available
                        ? status.version || "可用"
                        : status?.message || "不可用"}
                    </small>
                  </div>
                </div>
                <p>当前只读扫描 {sourceLabel(source)} 原始会话文件，收藏写入本工具状态文件。</p>
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
              <span>
                未检测到 <code>{sourceCommand(source)}</code> 命令。请确认已安装并加入 PATH，或使用「复制恢复命令」手动执行。
              </span>
            </div>
          )}
          {notice && (
            <div className="message success">
              <Icon name="check" /> <span>{notice}</span>
            </div>
          )}
          {invalidCount > 0 && (
            <div className="message warn">
              <Icon name="alert" /> <span>有 {invalidCount} 个 session 文件无法解析，已隔离显示。</span>
            </div>
          )}
        </section>
      )}

      <section className="workspace">
        <aside className="group-panel">
          <div className="panel-title">
            <span>分组</span>
            <strong>{filtered.length}</strong>
          </div>
          <button
            type="button"
            className={`group-item ${activeGroupKey === null ? "active" : ""}`}
            onClick={() => setActiveGroupKey(null)}
          >
            <Icon name="inbox" />
            <span>全部</span>
            <b>{filtered.length}</b>
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
              <Icon name={groupIconFor(groupBy, group.key)} />
              <span>{group.label}</span>
              <b>{group.sessions.length}</b>
            </button>
          ))}
        </aside>

        <section className="session-list">
          {loading && <div className="empty"><div className="skeleton" /><div className="skeleton" /><div className="skeleton" /></div>}
          {!loading && visibleSessions.length === 0 && (
            <div className="empty empty-state">
              <Icon name="inbox" />
              <p>没有匹配的会话</p>
              <small>调整搜索或分组，或点击右上角刷新。</small>
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
                      <header className="session-card-head">
                        <h3 className="session-title">{session.title || "(untitled)"}</h3>
                        <button
                          type="button"
                          className={`star ${favorite ? "on" : ""}`}
                          aria-label={favorite ? "取消收藏" : "收藏"}
                          onClick={(event) => {
                            event.stopPropagation();
                            void toggleFavorite(session);
                          }}
                        >
                          <Icon name={favorite ? "star-fill" : "star"} />
                        </button>
                      </header>
                      <div className="session-path" title={session.workspace || session.path}>
                        <Icon name="folder-small" />
                        <span>{session.workspace || session.path}</span>
                      </div>
                      <footer className="session-foot">
                        <span className="pill mono">{session.shortId}</span>
                        <span className="pill">{formatDateTime(session.updatedAt)}</span>
                        {session.model && <span className="pill subtle">{session.model}</span>}
                        <span className="spacer" />
                        <span className="meta-num">
                          <b>{session.messageCount}</b> 条
                        </span>
                        <span className="meta-num">
                          <b>{formatTokenCount(session.totalTokens)}</b> tokens
                        </span>
                      </footer>
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
              onToggleFavorite={() => void toggleFavorite(selected)}
              onCopyCommand={() => void copyCommand(selected)}
            />
          ) : (
            <div className="empty empty-state">
              <Icon name="inbox" />
              <p>选择一个会话查看详情</p>
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
  onToggleFavorite: () => void;
  onCopyCommand: () => void;
}) {
  const { session } = props;
  const command = resumeCommand(session);
  const workspaceMissing = Boolean(session.workspace) && !session.workspace.match(/^[A-Za-z]:\\/);
  const [copied, setCopied] = useState(false);

  function handleCopy() {
    props.onCopyCommand();
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  }

  return (
    <div className="details">
      <div className="detail-head">
        <p className="eyebrow">Session Detail</p>
        <div className="detail-title-row">
          <h2>{session.title || "(untitled)"}</h2>
          <button
            type="button"
            className={`star lg ${props.favorite ? "on" : ""}`}
            aria-label={props.favorite ? "取消收藏" : "收藏"}
            onClick={props.onToggleFavorite}
          >
            <Icon name={props.favorite ? "star-fill" : "star"} />
          </button>
        </div>
        <div className="detail-chips">
          <span className="pill mono">{session.shortId}</span>
          {session.model && <span className="pill">{session.model}</span>}
          {session.mode && <span className="pill subtle">{session.mode}</span>}
        </div>
      </div>

      {session.invalidReason && (
        <div className="message error">
          <Icon name="alert" /> <span>{session.invalidReason}</span>
        </div>
      )}

      <div className="stat-row">
        <div className="stat">
          <b>{session.messageCount}</b>
          <span>条消息</span>
        </div>
        <div className="stat">
          <b>{formatTokenCount(session.totalTokens)}</b>
          <span>tokens</span>
        </div>
        <div className="stat">
          <b>{formatRelative(session.updatedAt)}</b>
          <span>最近更新</span>
        </div>
      </div>

      <dl>
        <dt>完整 ID</dt>
        <dd className="mono">{session.id}</dd>
        <dt>更新时间</dt>
        <dd>{formatDateTime(session.updatedAt)}</dd>
        <dt>创建时间</dt>
        <dd>{formatDateTime(session.createdAt)}</dd>
        <dt>Workspace</dt>
        <dd>{session.workspace || "未记录"}</dd>
        <dt>文件</dt>
        <dd className="mono small">{session.path}</dd>
      </dl>

      <div className="preview-box">
        <span>首条用户消息</span>
        <p>{session.preview || "无可展示摘要"}</p>
      </div>

      <div className="command-box">
        <div className="command-head">
          <span>恢复命令</span>
          <button
            type="button"
            className={`inline-copy ${copied ? "copied" : ""}`}
            onClick={handleCopy}
          >
            <Icon name={copied ? "check" : "copy"} />
            {copied ? "已复制" : "复制"}
          </button>
        </div>
        <code>{command}</code>
      </div>

      <p className="hint">启动方式：新系统终端。V0.1 不修改原始 session JSON。</p>
      {workspaceMissing && <p className="hint warn">原 workspace 不存在，后端会退回用户主目录启动。</p>}
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
    >
      <Icon name={props.icon} />
    </button>
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

function groupIconFor(groupBy: GroupBy, key: string): IconName {
  if (key === "favorites") return "star-fill";
  if (groupBy === "date") return "calendar";
  if (groupBy === "model") return "cpu";
  if (groupBy === "mode") return "mode";
  if (groupBy === "favorite") return "star";
  return "folder-small";
}

function resumeCommand(session: SessionRecord): string {
  return buildResumeCommand(session.source, session.id);
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

function sourceLabel(source: SessionSource): string {
  return source === "claude" ? "Claude Code" : "DeepSeek TUI";
}

function sourceCommand(source: SessionSource): string {
  return source === "claude" ? "claude" : "deepseek-tui.cmd";
}

function formatRelative(value: string | null): string {
  if (!value) return "-";
  const target = new Date(value).getTime();
  if (Number.isNaN(target)) return formatDateTime(value);
  const delta = Date.now() - target;
  const minute = 60_000;
  const hour = 3600_000;
  const day = 86_400_000;
  if (delta < minute) return "刚刚";
  if (delta < hour) return `${Math.floor(delta / minute)} 分钟前`;
  if (delta < day) return `${Math.floor(delta / hour)} 小时前`;
  if (delta < 7 * day) return `${Math.floor(delta / day)} 天前`;
  return formatDateTime(value);
}
