import { useEffect, useMemo, useState } from "react";
import claudeLogo from "../assets/providers/claude.svg";
import codexLogo from "../assets/providers/codex.svg";
import deepseekLogo from "../assets/providers/deepseek.svg";
import { useTokenUsage } from "../hooks/useTokenUsage";
import { formatTokenCount } from "../lib/session";
import { buildHeatmapDays, filterDailyBySourceAndRange, filterModelsBySourceAndRange, heatmapLevel, providerSummaryForSource, usageShare } from "../lib/usage";
import type { Locale, TFunction } from "../lib/i18n";
import type { DailyTokenUsage, ProviderDescriptor, SessionSource, UsageRange, UsageTab } from "../types";
import { Icon } from "./Icon";

const providerLogo: Record<ProviderDescriptor["iconKey"], string> = {
  claude: claudeLogo,
  codex: codexLogo,
  deepseek: deepseekLogo
};

const ranges: UsageRange[] = ["all", "30d", "7d"];
const tabs: UsageTab[] = ["overview", "models"];

export function UsagePage(props: {
  providers: ProviderDescriptor[];
  initialSource: SessionSource;
  locale: Locale;
  t: TFunction;
  onBack: () => void;
}) {
  const { usage, loading, error, reload } = useTokenUsage();
  const [selectedSource, setSelectedSource] = useState<SessionSource>(props.initialSource);
  const [tab, setTab] = useState<UsageTab>("overview");
  const [range, setRange] = useState<UsageRange>("all");

  useEffect(() => {
    setSelectedSource(props.initialSource);
  }, [props.initialSource]);

  const provider = props.providers.find((item) => item.id === selectedSource) ?? props.providers[0] ?? null;
  const source = provider?.id ?? selectedSource;
  const daily = usage ? filterDailyBySourceAndRange(usage.byDay, source, range) : [];
  const heatmapDays = buildHeatmapDays(daily, range);
  const models = usage ? filterModelsBySourceAndRange(usage.byModel, usage.byModelByDay, source, range) : [];
  const providerSummary = usage ? providerSummaryForSource(usage.byProvider, source) : null;
  const rangeTokens = daily.reduce((sum, item) => sum + item.totalTokens, 0);
  const rangeSessions = daily.reduce((sum, item) => sum + item.sessionCount, 0);
  const rangeMessages = daily.reduce((sum, item) => sum + item.messageCount, 0);
  const displayTokens = range === "all" ? providerSummary?.totalTokens ?? 0 : rangeTokens;
  const displaySessions = range === "all" ? providerSummary?.sessionCount ?? 0 : rangeSessions;
  const displayMessages = range === "all" ? providerSummary?.messageCount ?? 0 : rangeMessages;
  const maxDailyTokens = Math.max(...daily.map((item) => item.totalTokens), 0);
  const maxHeatmapTokens = Math.max(...heatmapDays.map((item) => item.totalTokens), 0);
  const modelTotal = models.reduce((sum, item) => sum + item.totalTokens, 0);

  return (
    <section className="usage-page">
      <div className="usage-shell">
        <header className="usage-header">
          <div>
            <button type="button" className="btn-ghost usage-back" onClick={props.onBack}>
              <Icon name="chevron" />
              <span>{props.t("action_back_sessions")}</span>
            </button>
            <h1>{props.t("usage_title")}</h1>
            <p>{props.t("usage_subtitle")}</p>
          </div>
          <button type="button" className="btn-ghost" onClick={() => void reload({ refresh: true, source })} disabled={loading}>
            <Icon name="refresh" />
            <span>{props.t("usage_refresh")}</span>
          </button>
        </header>

        <div className="usage-panel">
          <div className="usage-chart-toolbar">
            <div className="usage-agent-tabs" aria-label={props.t("usage_provider")}>
              {props.providers.map((item) => (
                <button
                  key={item.id}
                  type="button"
                  className={`usage-agent-button ${source === item.id ? "active" : ""}`}
                  onClick={() => setSelectedSource(item.id)}
                  title={item.shortName}
                  aria-label={item.shortName}
                >
                  <span className={`source-icon-frame ${item.badgeKey}`}>
                    <img src={providerLogo[item.iconKey]} alt="" />
                  </span>
                </button>
              ))}
            </div>
            <div className="usage-segmented" aria-label={props.t("usage_title")}>
              {tabs.map((item) => (
                <button key={item} type="button" className={tab === item ? "active" : ""} onClick={() => setTab(item)}>
                  {props.t(item === "overview" ? "usage_overview" : "usage_models")}
                </button>
              ))}
            </div>
            <div className="usage-segmented usage-range" aria-label={props.t("usage_daily")}>
              {ranges.map((item) => (
                <button key={item} type="button" className={range === item ? "active" : ""} onClick={() => setRange(item)}>
                  {props.t(`usage_range_${item}`)}
                </button>
              ))}
            </div>
          </div>

          {loading && <UsageEmpty title={props.t("refreshing")} />}
          {error && !loading && <UsageEmpty title={error} />}
          {!loading && !error && usage && displayTokens === 0 && <UsageEmpty title={props.t("usage_empty")} hint={props.t("usage_empty_hint")} />}

          {!loading && !error && usage && displayTokens > 0 && (
            <>
              <div className="usage-summary-grid">
                <UsageStat label={props.t("usage_total_tokens")} value={formatTokenCount(displayTokens)} />
                <UsageStat label={props.t("usage_total_sessions")} value={number(displaySessions)} />
                <UsageStat label={props.t("usage_total_messages")} value={number(displayMessages)} />
              </div>

              {tab === "overview" ? (
                <OverviewPanel daily={daily} heatmapDays={heatmapDays} maxHeatmapTokens={maxHeatmapTokens} provider={provider} t={props.t} />
              ) : (
                <ModelsPanel daily={daily} maxDailyTokens={maxDailyTokens} models={models} modelTotal={modelTotal} locale={props.locale} t={props.t} />
              )}

              <p className="usage-note">{props.t("usage_data_note")}</p>
            </>
          )}
        </div>
      </div>
    </section>
  );
}

function OverviewPanel(props: {
  daily: DailyTokenUsage[];
  heatmapDays: DailyTokenUsage[];
  maxHeatmapTokens: number;
  provider: ProviderDescriptor | null;
  t: TFunction;
}) {
  const totalTokens = props.daily.reduce((sum, item) => sum + item.totalTokens, 0);
  const totalSessions = props.daily.reduce((sum, item) => sum + item.sessionCount, 0);

  return (
    <div className="usage-overview">
      <div className="usage-heatmap" aria-label={props.t("usage_daily")}>
        {props.heatmapDays.map((item) => (
          <span
            key={item.date}
            className={`usage-heatmap-cell level-${heatmapLevel(item.totalTokens, props.maxHeatmapTokens)}`}
            title={`${item.date} · ${number(item.totalTokens)} tokens · ${number(item.sessionCount)} sessions`}
          />
        ))}
      </div>
      <div className="usage-current-agent">
        <span className={`source-dot ${props.provider?.badgeKey ?? ""}`} />
        <b>{props.provider?.shortName ?? "—"}</b>
        <span>{formatTokenCount(totalTokens)} tokens</span>
        <span>{number(totalSessions)} {props.t("usage_sessions")}</span>
      </div>
    </div>
  );
}

function ModelsPanel(props: {
  daily: DailyTokenUsage[];
  maxDailyTokens: number;
  models: Array<{ model: string; totalTokens: number; sessionCount: number; messageCount: number }>;
  modelTotal: number;
  locale: Locale;
  t: TFunction;
}) {
  return (
    <div className="usage-models-panel">
      <div className="usage-bars" aria-label={props.t("usage_daily")}>
        {props.daily.map((item) => {
          const height = props.maxDailyTokens === 0 ? 4 : Math.max(4, Math.round((item.totalTokens / props.maxDailyTokens) * 154));
          return (
            <div key={item.date} className="usage-bar-wrap" title={`${item.date} · ${number(item.totalTokens)} tokens`}>
              <div className="usage-bar-value">{formatTokenCount(item.totalTokens)}</div>
              <div className="usage-bar" style={{ height }} />
              <div className="usage-bar-label">{shortDate(item.date, props.locale)}</div>
            </div>
          );
        })}
      </div>
      <div className="usage-model-list">
        {props.models.slice(0, 8).map((item, index) => {
          const share = usageShare(item.totalTokens, props.modelTotal);
          return (
            <div key={item.model} className="usage-model-row">
              <span className={`usage-model-swatch shade-${index % 5}`} />
              <span className="usage-model-name">{item.model}</span>
              <span className="usage-model-meta">{formatTokenCount(item.totalTokens)} · {number(item.sessionCount)} {props.t("usage_sessions")}</span>
              <b>{percent(share)}</b>
            </div>
          );
        })}
      </div>
    </div>
  );
}

function UsageStat(props: { label: string; value: string }) {
  return (
    <div className="usage-stat-card">
      <b>{props.value}</b>
      <span>{props.label}</span>
    </div>
  );
}

function UsageEmpty(props: { title: string; hint?: string }) {
  return (
    <div className="usage-empty">
      <Icon name="cpu" />
      <p>{props.title}</p>
      {props.hint && <small>{props.hint}</small>}
    </div>
  );
}

function number(value: number): string {
  return new Intl.NumberFormat("en-US").format(value);
}

function percent(value: number): string {
  return new Intl.NumberFormat("en-US", { style: "percent", minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(value);
}

function shortDate(date: string, locale: Locale): string {
  const parsed = new Date(`${date}T00:00:00`);
  if (Number.isNaN(parsed.getTime())) return date;
  return parsed.toLocaleDateString(locale === "zh" ? "zh-CN" : "en-US", { month: "short", day: "numeric" });
}
