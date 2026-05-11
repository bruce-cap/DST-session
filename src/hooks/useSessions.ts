/** Loads sessions and derives filtered, grouped, and selected state. */

import { useEffect, useMemo, useRef, useState } from "react";
import { checkAgent, getSourceState, listSessions, openSessionFolder, refreshSessions, resumeSession } from "../api";
import { normalizeSingleLine } from "../lib/commands";
import { compareUpdatedDesc, groupSessions, matchesSession } from "../lib/session";
import { toMessage } from "../lib/format";
import type { AppState, DeepseekStatus, GroupBy, ProviderDescriptor, SessionRecord, SessionSource, SourceState } from "../types";

export function useSessions(source: SessionSource, appState: AppState, provider: ProviderDescriptor | null, loadAppState: () => Promise<AppState>) {
  const [sessions, setSessions] = useState<SessionRecord[]>([]);
  const [status, setStatus] = useState<DeepseekStatus | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [groupBy, setGroupBy] = useState<GroupBy>("workspace");
  const [activeGroupKey, setActiveGroupKey] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [sourceState, setSourceState] = useState<SourceState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const loadRequestId = useRef(0);

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

  useEffect(() => {
    if (!appState.autoRefreshEnabled) {
      return;
    }
    const intervalMs = Math.max(1, appState.autoRefreshIntervalMinutes) * 60_000;
    const timer = window.setInterval(() => {
      void refreshFromSource(false);
    }, intervalMs);
    return () => window.clearInterval(timer);
  }, [appState.autoRefreshEnabled, appState.autoRefreshIntervalMinutes, source]);

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
    const requestId = loadRequestId.current + 1;
    loadRequestId.current = requestId;
    setLoading(true);
    setStatus(null);
    setError(null);
    try {
      const state = await loadAppState();
      const [records, nextSourceState] = await Promise.all([
        listSessions({ source }),
        getSourceState({ source })
      ]);
      if (requestId !== loadRequestId.current) return;

      setSessions(records);
      setSourceState(nextSourceState);
      setSelectedId((current) => current && records.some((record) => record.id === current) ? current : records[0]?.id ?? null);
      if (records.length === 0 && !nextSourceState?.lastSuccessAtMs) {
        void refreshFromSource(false);
      }
      void checkAgent({ source, deepseekLauncher: state.deepseekLauncher })
        .then((nextStatus) => {
          if (requestId === loadRequestId.current) {
            setStatus(nextStatus);
          }
        })
        .catch((caught) => {
          if (requestId === loadRequestId.current) {
            setError(toMessage(caught));
          }
        });
    } catch (caught) {
      if (requestId !== loadRequestId.current) return;
      setSessions([]);
      setError(toMessage(caught));
    } finally {
      if (requestId === loadRequestId.current) {
        setLoading(false);
      }
    }
  }

  async function refreshFromSource(showLoading = true): Promise<void> {
    const requestId = loadRequestId.current;
    if (showLoading) {
      setRefreshing(true);
    }
    setError(null);
    try {
      await refreshSessions({ source });
      const [records, nextSourceState] = await Promise.all([
        listSessions({ source }),
        getSourceState({ source })
      ]);
      if (requestId !== loadRequestId.current) return;

      setSessions(records);
      setSourceState(nextSourceState);
      setSelectedId((current) => current && records.some((record) => record.id === current) ? current : records[0]?.id ?? null);
    } catch (caught) {
      if (requestId !== loadRequestId.current) return;
      setError(toMessage(caught));
      const nextSourceState = await getSourceState({ source }).catch(() => null);
      if (requestId !== loadRequestId.current) return;
      setSourceState(nextSourceState);
    } finally {
      if (showLoading && requestId === loadRequestId.current) {
        setRefreshing(false);
      }
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
    refreshing,
    sourceState,
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
    refreshFromSource,
    refreshStatus,
    openFolder,
    resume
  };
}
