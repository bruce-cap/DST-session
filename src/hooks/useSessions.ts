/** Loads sessions and derives filtered, grouped, and selected state. */

import { useEffect, useMemo, useState } from "react";
import { checkAgent, listSessions, openSessionFolder, resumeSession } from "../api";
import { normalizeSingleLine } from "../lib/commands";
import { compareUpdatedDesc, groupSessions, matchesSession } from "../lib/session";
import { toMessage } from "../lib/format";
import type { AppState, DeepseekStatus, GroupBy, ProviderDescriptor, SessionRecord, SessionSource } from "../types";

export function useSessions(source: SessionSource, appState: AppState, provider: ProviderDescriptor | null, loadAppState: () => Promise<AppState>) {
  const [sessions, setSessions] = useState<SessionRecord[]>([]);
  const [status, setStatus] = useState<DeepseekStatus | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [groupBy, setGroupBy] = useState<GroupBy>("workspace");
  const [activeGroupKey, setActiveGroupKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (provider) {
      setGroupBy(provider.defaultGroupBy);
    }
  }, [provider]);

  useEffect(() => {
    void loadAll();
  }, [source]);

  useEffect(() => {
    setActiveGroupKey(null);
  }, [source, groupBy, search]);

  const favoriteSet = useMemo(() => new Set(appState.favorites), [appState.favorites]);
  const filtered = useMemo(
    () => sessions.filter((session) => matchesSession(session, search)).sort(compareUpdatedDesc),
    [search, sessions]
  );
  const groups = useMemo(() => groupSessions(filtered, groupBy, favoriteSet), [favoriteSet, filtered, groupBy]);
  const visibleGroups = activeGroupKey ? groups.filter((group) => group.key === activeGroupKey) : groups;
  const visibleSessions = visibleGroups.flatMap((group) => group.sessions);
  const selected = sessions.find((session) => session.id === selectedId) ?? visibleSessions[0] ?? null;
  const invalidCount = sessions.filter((session) => session.invalidReason).length;

  async function loadAll(): Promise<void> {
    setLoading(true);
    setError(null);
    try {
      const state = await loadAppState();
      const [records, nextStatus] = await Promise.all([
        listSessions({ source }),
        checkAgent({ source, deepseekLauncher: state.deepseekLauncher })
      ]);
      setSessions(records);
      setStatus(nextStatus);
      setSelectedId(records[0]?.id ?? null);
    } catch (caught) {
      setSessions([]);
      setError(toMessage(caught));
    } finally {
      setLoading(false);
    }
  }

  async function refreshStatus(nextState = appState): Promise<void> {
    const nextStatus = await checkAgent({ source, deepseekLauncher: nextState.deepseekLauncher });
    setStatus(nextStatus);
  }

  async function openFolder(session: SessionRecord): Promise<void> {
    try {
      await openSessionFolder({ path: session.path });
    } catch (caught) {
      setError(toMessage(caught));
    }
  }

  async function resume(session: SessionRecord, prompt?: string): Promise<{ promptUsed: string | undefined } | null> {
    const normalizedPrompt = prompt ? normalizeSingleLine(prompt) : "";
    const effectivePrompt = provider?.capabilities.quickReply && normalizedPrompt ? normalizedPrompt : undefined;
    try {
      await resumeSession({
        source: session.source,
        sessionId: session.id,
        workspace: session.workspace || null,
        launchMode: appState.launchMode,
        deepseekLauncher: appState.deepseekLauncher,
        prompt: effectivePrompt ?? null
      });
      return { promptUsed: effectivePrompt };
    } catch (caught) {
      setError(toMessage(caught));
      return null;
    }
  }

  return {
    sessions,
    status,
    selected,
    selectedId,
    search,
    groupBy,
    activeGroupKey,
    loading,
    error,
    favoriteSet,
    filtered,
    groups,
    visibleGroups,
    visibleSessions,
    invalidCount,
    setSelectedId,
    setSearch,
    setGroupBy,
    setActiveGroupKey,
    setError,
    setStatus,
    loadAll,
    refreshStatus,
    openFolder,
    resume
  };
}
