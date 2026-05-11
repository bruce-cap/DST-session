/** Closes floating UI when pointer events land outside its element. */

import { useEffect, type RefObject } from "react";

export function useClickAway<T extends HTMLElement>(ref: RefObject<T | null>, active: boolean, onAway: () => void): void {
  useEffect(() => {
    if (!active) {
      return;
    }

    function onMouseDown(event: MouseEvent) {
      if (!ref.current?.contains(event.target as Node)) {
        onAway();
      }
    }

    window.addEventListener("mousedown", onMouseDown);
    return () => window.removeEventListener("mousedown", onMouseDown);
  }, [active, onAway, ref]);
}
