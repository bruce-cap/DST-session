import { useCallback, useEffect, useState } from "react";
import { getTokenUsage, refreshTokenUsage } from "../api/tauri";
import { toMessage } from "../lib/format";
import type { SessionSource, TokenUsageSummary } from "../types";

export function useTokenUsage() {
  const [usage, setUsage] = useState<TokenUsageSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async (options?: { refresh?: boolean; source?: SessionSource }) => {
    setLoading(true);
    setError(null);
    try {
      if (options?.refresh) {
        await refreshTokenUsage(options.source);
      }
      setUsage(await getTokenUsage());
    } catch (err) {
      setError(toMessage(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { usage, loading, error, reload };
}
