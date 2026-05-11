/** Loads and mutates persisted app state through typed IPC. */

import { useState } from "react";
import { getAppState, setDeepseekLauncher, setFavorite } from "../api";
import { favoriteKey, isFavorite } from "../lib/favorites";
import { toMessage } from "../lib/format";
import type { AppState, DeepseekLauncher, SessionRecord } from "../types";

const defaultState: AppState = {
  favorites: [],
  launchMode: "new_terminal",
  deepseekLauncher: "cmd"
};

export function useAppState() {
  const [appState, setAppState] = useState<AppState>(defaultState);
  const [error, setError] = useState<string | null>(null);

  async function loadAppState(): Promise<AppState> {
    const state = await getAppState();
    setAppState(state);
    return state;
  }

  async function toggleFavorite(session: SessionRecord): Promise<void> {
    try {
      const favoriteSet = new Set(appState.favorites);
      const nextState = await setFavorite({
        sessionId: favoriteKey(session),
        favorite: !isFavorite(session, favoriteSet)
      });
      setAppState(nextState);
      setError(null);
    } catch (caught) {
      setError(toMessage(caught));
    }
  }

  async function changeDeepseekLauncher(next: DeepseekLauncher): Promise<AppState | null> {
    try {
      const nextState = await setDeepseekLauncher({ launcher: next });
      setAppState(nextState);
      setError(null);
      return nextState;
    } catch (caught) {
      setError(toMessage(caught));
      return null;
    }
  }

  return { appState, setAppState, loadAppState, toggleFavorite, changeDeepseekLauncher, error, setError };
}
