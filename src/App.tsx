import { useEffect, useRef, useState } from "react";
import { TitleBar } from "./components/TitleBar";
import { SettingsMenu } from "./components/SettingsMenu";
import { MessageStack } from "./components/MessageStack";
import { Sidebar } from "./components/Sidebar";
import { SessionList } from "./components/SessionList";
import { SessionDetails } from "./components/SessionDetails";
import { useAppState } from "./hooks/useAppState";
import { useAutoDismiss } from "./hooks/useAutoDismiss";
import { useClickAway } from "./hooks/useClickAway";
import { setLocale, useLocale } from "./hooks/useLocale";
import { useProviders } from "./hooks/useProviders";
import { useSessions } from "./hooks/useSessions";
import { useTheme } from "./hooks/useTheme";
import { useTranslation } from "./hooks/useTranslation";
import { buildResumeCommand } from "./lib/commands";
import type { SessionSource } from "./types";

const APP_VERSION = __APP_VERSION__;

export default function App() {
  const t = useTranslation();
  const locale = useLocale();
  const [theme, setTheme] = useTheme();
  const [source, setSource] = useState<SessionSource>("deepseek");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [quickReply, setQuickReply] = useState("");
  const settingsRef = useRef<HTMLDivElement | null>(null);
  const providersState = useProviders();
  const provider = providersState.providerFor(source);
  const app = useAppState();
  const sessions = useSessions(source, app.appState, provider, app.loadAppState);
  const error = providersState.error ?? app.error ?? sessions.error;

  useClickAway(settingsRef, settingsOpen, () => setSettingsOpen(false));
  useAutoDismiss(notice, () => setNotice(null));

  useEffect(() => setQuickReply(""), [sessions.selectedId, source]);

  async function changeLauncher(next: typeof app.appState.deepseekLauncher) {
    const nextState = await app.changeDeepseekLauncher(next);
    if (nextState) await sessions.refreshStatus(nextState);
  }

  async function resumeSelected(session = sessions.selected, prompt?: string) {
    if (!session) return;
    setNotice(null);
    const result = await sessions.resume(session, prompt);
    if (!result) return;
    const command = buildResumeCommand(session.source, session.id, app.appState.deepseekLauncher, result.promptUsed);
    setNotice(result.promptUsed ? t("quick_reply_launched", { command }) : t("launched", { command }));
    if (result.promptUsed) setQuickReply("");
  }

  async function copyCommand(session = sessions.selected, prompt?: string) {
    if (!session) return;
    await navigator.clipboard.writeText(buildResumeCommand(session.source, session.id, app.appState.deepseekLauncher, prompt));
    setNotice(t("copied"));
  }

  return (
    <main className="app-shell">
      <TitleBar search={sessions.search} settingsOpen={settingsOpen} settingsRef={settingsRef} settings={
        <SettingsMenu version={APP_VERSION} locale={locale} theme={theme} appState={app.appState} provider={provider} status={sessions.status} t={t} onLocaleChange={setLocale} onThemeChange={setTheme} onLauncherChange={(next) => void changeLauncher(next)} onAutoRefreshChange={(enabled, interval) => void app.changeAutoRefresh(enabled, interval)} />
      } t={t} onSearchChange={sessions.setSearch} onRefresh={() => void sessions.refreshFromSource()} onToggleSettings={() => setSettingsOpen((open) => !open)} />
      <MessageStack error={error} notice={notice} invalidCount={sessions.invalidCount} cliMissing={Boolean(sessions.status && !sessions.status.available)} refreshing={sessions.refreshing} sourceState={sessions.sourceState} provider={provider} t={t} />
      <section className="workspace">
        <Sidebar providers={providersState.providers} source={source} groupBy={sessions.groupBy} groups={sessions.groups} activeGroupKey={sessions.activeGroupKey} total={sessions.filtered.length} t={t} onSourceChange={setSource} onGroupByChange={sessions.setGroupBy} onActiveGroupChange={sessions.setActiveGroupKey} onSelectFirstInGroup={(group) => sessions.setSelectedId(group.sessions[0]?.id ?? null)} />
        <SessionList loading={sessions.loading || providersState.loading} visibleGroups={sessions.visibleGroups} visibleSessions={sessions.visibleSessions} selected={sessions.selected} favoriteSet={sessions.favoriteSet} provider={provider} locale={locale} t={t} onSelect={sessions.setSelectedId} onToggleFavorite={(session) => void app.toggleFavorite(session)} />
        <aside className="detail-panel">
          <SessionDetails session={sessions.selected} provider={provider} favoriteSet={sessions.favoriteSet} appState={app.appState} locale={locale} status={sessions.status} quickReply={quickReply} t={t} onQuickReplyChange={setQuickReply} onResume={(session, prompt) => void resumeSelected(session, prompt)} onToggleFavorite={(session) => void app.toggleFavorite(session)} onCopyCommand={(session, prompt) => void copyCommand(session, prompt)} onOpenFolder={(session) => void sessions.openFolder(session)} />
        </aside>
      </section>
    </main>
  );
}
