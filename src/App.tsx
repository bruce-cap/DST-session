import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { buildResumeCommand } from "./lib/commands";
import {
  compareUpdatedDesc,
  formatDateTime,
  formatTokenCount,
  groupSessions,
  matchesSession
} from "./lib/session";
import type { AppState, DeepseekStatus, GroupBy, SessionRecord } from "./types";

const GROUP_OPTIONS: Array<{ value: GroupBy; label: string }> = [
  { value: "workspace", label: "按项目" },
  { value: "date", label: "按日期" },
  { value: "model", label: "按模型" },
  { value: "mode", label: "按模式" },
  { value: "favorite", label: "按收藏" },
  { value: "none", label: "全部" }
];

const defaultState: AppState = {
  favorites: [],
  launchMode: "new_terminal"
};

export default function App() {
  const [sessions, setSessions] = useState<SessionRecord[]>([]);
  const [appState, setAppState] = useState<AppState>(defaultState);
  const [status, setStatus] = useState<DeepseekStatus | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [groupBy, setGroupBy] = useState<GroupBy>("workspace");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    void loadAll();
  }, []);

  const favoriteSet = new Set(appState.favorites);
  const filtered = sessions
    .filter((session) => matchesSession(session, search))
    .sort(compareUpdatedDesc);
  const groups = groupSessions(filtered, groupBy, favoriteSet);
  const selected = sessions.find((session) => session.id === selectedId) ?? filtered[0] ?? null;
  const invalidCount = sessions.filter((session) => session.invalidReason).length;

  async function loadAll() {
    setLoading(true);
    setError(null);
    try {
      const [records, state, deepseek] = await Promise.all([
        invoke<SessionRecord[]>("list_sessions"),
        invoke<AppState>("get_app_state"),
        invoke<DeepseekStatus>("check_deepseek")
      ]);
      setSessions(records);
      setAppState(state);
      setStatus(deepseek);
      setSelectedId((current) => current ?? records[0]?.id ?? null);
    } catch (caught) {
      setError(toMessage(caught));
    } finally {
      setLoading(false);
    }
  }

  async function toggleFavorite(session: SessionRecord) {
    try {
      const nextState = await invoke<AppState>("set_favorite", {
        sessionId: session.id,
        favorite: !favoriteSet.has(session.id)
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
      <section className="hero">
        <div>
          <p className="eyebrow">Local DeepSeek TUI sessions</p>
          <h1>会话管理器</h1>
          <p className="hero-copy">
            只读扫描本机 session JSON，按项目和上下文找回对话，并用完整 ID 快速恢复。
          </p>
        </div>
        <div className={`status-pill ${status?.available ? "ok" : "bad"}`}>
          <span>{status?.available ? "deepseek-tui.cmd 可用" : "deepseek-tui.cmd 不可用"}</span>
          <small>{status?.message ?? "检测中"}</small>
        </div>
      </section>

      <section className="toolbar">
        <input
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          placeholder="搜索标题、消息、workspace、模型或 ID"
        />
        <select value={groupBy} onChange={(event) => setGroupBy(event.target.value as GroupBy)}>
          {GROUP_OPTIONS.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <button type="button" onClick={() => void loadAll()}>
          刷新
        </button>
      </section>

      {(error || notice || invalidCount > 0) && (
        <section className="message-stack">
          {error && <div className="message error">{error}</div>}
          {notice && <div className="message success">{notice}</div>}
          {invalidCount > 0 && (
            <div className="message warn">有 {invalidCount} 个 session 文件无法解析，已隔离显示。</div>
          )}
        </section>
      )}

      <section className="workspace">
        <aside className="group-panel">
          <div className="panel-title">
            <span>分组</span>
            <strong>{filtered.length}</strong>
          </div>
          {groups.map((group) => (
            <button
              type="button"
              className="group-item"
              key={group.key}
              onClick={() => setSelectedId(group.sessions[0]?.id ?? null)}
            >
              <span>{group.label}</span>
              <b>{group.sessions.length}</b>
            </button>
          ))}
        </aside>

        <section className="session-list">
          {loading && <div className="empty">正在读取 sessions...</div>}
          {!loading && filtered.length === 0 && <div className="empty">没有匹配的会话。</div>}
          {!loading &&
            groups.map((group) => (
              <div className="session-group" key={group.key}>
                <h2>{group.label}</h2>
                {group.sessions.map((session) => (
                  <button
                    type="button"
                    key={`${group.key}-${session.id}`}
                    className={`session-card ${selected?.id === session.id ? "active" : ""} ${
                      session.invalidReason ? "invalid" : ""
                    }`}
                    onClick={() => setSelectedId(session.id)}
                  >
                    <span className="session-title">
                      {favoriteSet.has(session.id) ? "★ " : ""}
                      {session.title}
                    </span>
                    <span className="session-meta">
                      {session.shortId} · {formatDateTime(session.updatedAt)} · {session.model || "unknown model"}
                    </span>
                    <span className="session-path">{session.workspace || session.path}</span>
                    <span className="session-foot">
                      {session.messageCount} messages · {formatTokenCount(session.totalTokens)} tokens
                    </span>
                  </button>
                ))}
              </div>
            ))}
        </section>

        <aside className="detail-panel">
          {selected ? (
            <SessionDetails
              session={selected}
              favorite={favoriteSet.has(selected.id)}
              deepseekAvailable={status?.available ?? false}
              launchMode={appState.launchMode}
              onToggleFavorite={() => void toggleFavorite(selected)}
              onCopy={() => void copyCommand(selected)}
              onOpenFolder={() => void openFolder(selected)}
              onResume={() => void resume(selected)}
            />
          ) : (
            <div className="empty">选择一个会话查看详情。</div>
          )}
        </aside>
      </section>
    </main>
  );
}

function SessionDetails(props: {
  session: SessionRecord;
  favorite: boolean;
  deepseekAvailable: boolean;
  launchMode: AppState["launchMode"];
  onToggleFavorite: () => void;
  onCopy: () => void;
  onOpenFolder: () => void;
  onResume: () => void;
}) {
  const { session } = props;
  const command = resumeCommand(session);
  const workspaceMissing = Boolean(session.workspace) && !session.workspace.match(/^[A-Za-z]:\\/);

  return (
    <div className="details">
      <div className="detail-head">
        <p className="eyebrow">Session detail</p>
        <h2>{session.title}</h2>
        <button type="button" className="ghost" onClick={props.onToggleFavorite}>
          {props.favorite ? "取消收藏" : "收藏"}
        </button>
      </div>

      {session.invalidReason && <div className="message error">{session.invalidReason}</div>}

      <dl>
        <dt>完整 ID</dt>
        <dd>{session.id}</dd>
        <dt>更新时间</dt>
        <dd>{formatDateTime(session.updatedAt)}</dd>
        <dt>创建时间</dt>
        <dd>{formatDateTime(session.createdAt)}</dd>
        <dt>Workspace</dt>
        <dd>{session.workspace || "未记录"}</dd>
        <dt>模型</dt>
        <dd>{session.model || "未记录"}</dd>
        <dt>模式</dt>
        <dd>{session.mode || "未记录"}</dd>
        <dt>统计</dt>
        <dd>
          {session.messageCount} messages · {formatTokenCount(session.totalTokens)} tokens
        </dd>
        <dt>文件</dt>
        <dd>{session.path}</dd>
      </dl>

      <div className="preview-box">
        <span>首条用户消息</span>
        <p>{session.preview || "无可展示摘要"}</p>
      </div>

      <div className="command-box">
        <span>恢复命令</span>
        <code>{command}</code>
      </div>

      <div className="actions">
        <button type="button" onClick={props.onResume} disabled={!props.deepseekAvailable || Boolean(session.invalidReason)}>
          继续会话
        </button>
        <button type="button" className="secondary" onClick={props.onCopy}>
          复制命令
        </button>
        <button type="button" className="secondary" onClick={props.onOpenFolder}>
          打开目录
        </button>
      </div>

      <p className="hint">
        启动方式：{props.launchMode === "new_terminal" ? "新系统终端" : "内嵌终端"}。V0.1 不修改 DeepSeek 原始
        session JSON。
      </p>
      {workspaceMissing && <p className="hint">如果原 workspace 不存在，后端会退回用户主目录启动。</p>}
    </div>
  );
}

function resumeCommand(session: SessionRecord): string {
  return buildResumeCommand(session.id);
}

function toMessage(value: unknown): string {
  if (value instanceof Error) {
    return value.message;
  }
  return String(value);
}
