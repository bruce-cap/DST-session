/** Loads provider descriptors used to drive source-specific UI. */

import { useEffect, useMemo, useState } from "react";
import { listProviders } from "../api";
import type { ProviderDescriptor, SessionSource } from "../types";
import { toMessage } from "../lib/format";

export function useProviders() {
  const [providers, setProviders] = useState<ProviderDescriptor[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    listProviders()
      .then((records) => {
        if (!cancelled) {
          setProviders(records);
          setError(null);
        }
      })
      .catch((caught: unknown) => {
        if (!cancelled) {
          setError(toMessage(caught));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const providerById = useMemo(() => new Map(providers.map((provider) => [provider.id, provider])), [providers]);
  const defaultProvider = providers[0] ?? null;

  function providerFor(source: SessionSource): ProviderDescriptor | null {
    return providerById.get(source) ?? defaultProvider;
  }

  return { providers, providerById, providerFor, defaultProvider, loading, error };
}
