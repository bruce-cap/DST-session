/** Automatically clears transient values after a delay. */

import { useEffect } from "react";

export function useAutoDismiss(value: string | null, onDismiss: () => void, delay = 2400): void {
  useEffect(() => {
    if (!value) {
      return;
    }
    const timer = window.setTimeout(onDismiss, delay);
    return () => window.clearTimeout(timer);
  }, [delay, onDismiss, value]);
}
