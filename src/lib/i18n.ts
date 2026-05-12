import { useSyncExternalStore } from "react";

export type Locale = "zh" | "en";

const zh = {
  brand_subtitle: "{source} 会话浏览与恢复",
  search_placeholder: "搜索标题、消息、workspace、模型或 ID",
  clear_search: "清空搜索",
  group_by: "分组方式",
  group_workspace: "按项目",
  group_date: "按日期",
  group_model: "按模型",
  group_mode: "按模式",
  group_favorite: "按收藏",
  group_none: "全部",
  action_launch: "启动会话",
  action_copy: "复制恢复命令",
  action_open_folder: "打开会话文件目录",
  action_refresh: "刷新",
  action_settings: "设置",
  action_usage: "Token 用量",
  action_back_sessions: "返回会话",
  settings_dark: "暗色主题",
  settings_language: "语言",
  settings_deepseek_launcher: "DeepSeek 启动脚本",
  settings_agent_launcher: "启动脚本",
  settings_auto_refresh: "自动刷新",
  settings_auto_refresh_interval: "刷新间隔",
  settings_minutes: "{n} 分钟",
  settings_cli_available: "可用",
  settings_cli_unavailable: "不可用",
  settings_footer: "当前只读扫描 {source} 原始会话文件，收藏写入本工具状态文件。",
  cli_missing:
    "未检测到 {command} 命令。请确认已安装并加入 PATH，或使用「复制恢复命令」手动执行。",
  copied: "恢复命令已复制",
  launched: "已启动：{command}",
  invalid_count: "有 {count} 个 session 文件无法解析，已隔离显示。",
  refreshing: "正在刷新会话索引…",
  refresh_failed: "上次刷新失败：{message}",
  panel_groups: "分组",
  group_all: "全部",
  empty_no_match: "没有匹配的会话",
  empty_no_match_hint: "调整搜索或分组，或点击右上角刷新。",
  empty_select: "选择一个会话查看详情",
  load_more: "再加载 {count} 条",
  untitled: "(未命名)",
  card_messages_unit: "条",
  session_detail: "Session Detail",
  unfavorite: "取消收藏",
  favorite: "收藏",
  stat_messages: "条消息",
  stat_tokens: "tokens",
  stat_updated: "最近更新",
  label_id: "完整 ID",
  label_updated: "更新时间",
  label_created: "创建时间",
  label_workspace: "Workspace",
  label_file: "文件",
  not_recorded: "未记录",
  preview_label: "首条用户消息",
  preview_empty: "无可展示摘要",
  command_label: "恢复命令",
  copy_now: "复制",
  copy_done: "已复制",
  hint_workspace_missing: "原 workspace 不存在，后端会退回用户主目录启动。",
  source_deepseek: "DeepSeek TUI",
  source_claude: "Claude Code",
  source_codex: "Codex",
  quick_reply_label: "快速回复",
  quick_reply_placeholder: "输入一句话继续会话",
  quick_reply_send: "发送",
  quick_reply_hint: "仅支持单行内容。多行会被自动压成一行。",
  quick_reply_launched: "已启动快速回复：{command}",
  usage_title: "Token 用量",
  usage_subtitle: "按 agent 查看已索引会话的 token 用量、每日趋势和模型分布。",
  usage_overview: "Overview",
  usage_models: "Models",
  usage_range_all: "All",
  usage_range_30d: "30d",
  usage_range_7d: "7d",
  usage_total_tokens: "总 Token",
  usage_total_sessions: "Sessions",
  usage_total_messages: "Messages",
  usage_active_provider: "活跃 agent",
  usage_daily: "每日用量",
  usage_by_model: "模型用量",
  usage_provider: "Agent",
  usage_tokens: "tokens",
  usage_sessions: "sessions",
  usage_messages: "messages",
  usage_share: "占比",
  usage_latest_activity: "最近活动",
  usage_refresh: "刷新用量",
  usage_empty: "暂无 token 用量数据",
  usage_empty_hint: "请先回到会话页刷新该 agent 的索引。",
  usage_data_note: "统计基于已索引 session，daily usage 按 session 最后更新时间的本地日期归属，属于 session-level 近似统计。",
  rel_just_now: "刚刚",
  rel_minutes: "{n} 分钟前",
  rel_hours: "{n} 小时前",
  rel_days: "{n} 天前"
};

type Dict = typeof zh;
export type TranslationKey = keyof Dict;

const en: Dict = {
  brand_subtitle: "{source} session browser & resume",
  search_placeholder: "Search title, message, workspace, model, or ID",
  clear_search: "Clear search",
  group_by: "Group by",
  group_workspace: "Project",
  group_date: "Date",
  group_model: "Model",
  group_mode: "Mode",
  group_favorite: "Favorites",
  group_none: "All",
  action_launch: "Launch session",
  action_copy: "Copy resume command",
  action_open_folder: "Open session folder",
  action_refresh: "Refresh",
  action_settings: "Settings",
  action_usage: "Token usage",
  action_back_sessions: "Back to sessions",
  settings_dark: "Dark theme",
  settings_language: "Language",
  settings_deepseek_launcher: "DeepSeek launcher",
  settings_agent_launcher: "Launch script",
  settings_auto_refresh: "Auto refresh",
  settings_auto_refresh_interval: "Refresh interval",
  settings_minutes: "{n} min",
  settings_cli_available: "Available",
  settings_cli_unavailable: "Unavailable",
  settings_footer:
    "Read-only scan of {source} session files. Favorites are written to this app's own state file.",
  cli_missing:
    'Command "{command}" not found. Install it and add it to PATH, or use "Copy resume command" to run manually.',
  copied: "Resume command copied",
  launched: "Launched: {command}",
  invalid_count: "{count} session file(s) failed to parse and are isolated.",
  refreshing: "Refreshing session index…",
  refresh_failed: "Last refresh failed: {message}",
  panel_groups: "Groups",
  group_all: "All",
  empty_no_match: "No matching sessions",
  empty_no_match_hint: "Adjust search or group, or click refresh in the top right.",
  empty_select: "Select a session to view details",
  load_more: "Load {count} more",
  untitled: "(untitled)",
  card_messages_unit: "msgs",
  session_detail: "Session Detail",
  unfavorite: "Unfavorite",
  favorite: "Favorite",
  stat_messages: "messages",
  stat_tokens: "tokens",
  stat_updated: "Last updated",
  label_id: "Full ID",
  label_updated: "Updated",
  label_created: "Created",
  label_workspace: "Workspace",
  label_file: "File",
  not_recorded: "Not recorded",
  preview_label: "First user message",
  preview_empty: "No preview available",
  command_label: "Resume command",
  copy_now: "Copy",
  copy_done: "Copied",
  hint_workspace_missing: "Original workspace does not exist. Backend falls back to home directory.",
  source_deepseek: "DeepSeek TUI",
  source_claude: "Claude Code",
  source_codex: "Codex",
  quick_reply_label: "Quick reply",
  quick_reply_placeholder: "Type one line to continue the session",
  quick_reply_send: "Send",
  quick_reply_hint: "Single line only. Line breaks are collapsed to spaces.",
  quick_reply_launched: "Quick reply launched: {command}",
  usage_title: "Token usage",
  usage_subtitle: "View indexed token usage, daily trends, and model distribution by agent.",
  usage_overview: "Overview",
  usage_models: "Models",
  usage_range_all: "All",
  usage_range_30d: "30d",
  usage_range_7d: "7d",
  usage_total_tokens: "Total tokens",
  usage_total_sessions: "Sessions",
  usage_total_messages: "Messages",
  usage_active_provider: "Active agents",
  usage_daily: "Daily usage",
  usage_by_model: "Model usage",
  usage_provider: "Agent",
  usage_tokens: "tokens",
  usage_sessions: "sessions",
  usage_messages: "messages",
  usage_share: "Share",
  usage_latest_activity: "Latest activity",
  usage_refresh: "Refresh usage",
  usage_empty: "No token usage data",
  usage_empty_hint: "Refresh this agent's index from the sessions page first.",
  usage_data_note: "Usage is based on indexed sessions. Daily usage is attributed to each session's latest local activity date, so it is a session-level approximation.",
  rel_just_now: "just now",
  rel_minutes: "{n} min ago",
  rel_hours: "{n} h ago",
  rel_days: "{n} d ago"
};

export const translations: Record<Locale, Dict> = { zh, en };

export type TParams = Record<string, string | number>;
export type TFunction = (key: TranslationKey, params?: TParams) => string;

const STORAGE_KEY = "agent-session-manager-locale";
const LEGACY_STORAGE_KEY = "deepseek-session-manager-locale";
const listeners = new Set<() => void>();
let currentLocale: Locale = detectInitialLocale();

function detectInitialLocale(): Locale {
  if (typeof localStorage !== "undefined") {
    const saved = localStorage.getItem(STORAGE_KEY) ?? localStorage.getItem(LEGACY_STORAGE_KEY);
    if (saved === "zh" || saved === "en") {
      localStorage.setItem(STORAGE_KEY, saved);
      return saved;
    }
  }
  if (typeof navigator !== "undefined" && navigator.language) {
    return navigator.language.toLowerCase().startsWith("zh") ? "zh" : "en";
  }
  return "zh";
}

export function getLocale(): Locale {
  return currentLocale;
}

export function setLocale(next: Locale): void {
  if (next === currentLocale) {
    return;
  }
  currentLocale = next;
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(STORAGE_KEY, next);
    localStorage.removeItem(LEGACY_STORAGE_KEY);
  }
  if (typeof document !== "undefined") {
    document.documentElement.lang = localeToBcp47(next);
  }
  listeners.forEach((fn) => fn());
}

function subscribe(fn: () => void): () => void {
  listeners.add(fn);
  return () => {
    listeners.delete(fn);
  };
}

export function useLocale(): Locale {
  return useSyncExternalStore(subscribe, getLocale, getLocale);
}

export function translate(locale: Locale, key: TranslationKey, params?: TParams): string {
  const template = translations[locale]?.[key] ?? translations.zh[key] ?? key;
  if (!params) {
    return template;
  }
  return template.replace(/\{(\w+)\}/g, (_, name: string) => {
    const value = params[name];
    return value === undefined ? `{${name}}` : String(value);
  });
}

export function useT(): TFunction {
  const locale = useLocale();
  return (key, params) => translate(locale, key, params);
}

export function localeToBcp47(locale: Locale): string {
  return locale === "zh" ? "zh-CN" : "en-US";
}
