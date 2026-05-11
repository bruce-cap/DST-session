/** Renders the DeepSeek launcher segmented control. */

import type { DeepseekLauncher } from "../types";

export function LauncherToggle({ value, onChange }: { value: DeepseekLauncher; onChange: (value: DeepseekLauncher) => void }) {
  return (
    <div className="segmented" role="group" aria-label="DeepSeek launcher">
      <button type="button" className={value === "cmd" ? "active" : ""} onClick={() => onChange("cmd")} aria-pressed={value === "cmd"}>cmd</button>
      <button type="button" className={value === "ps1" ? "active" : ""} onClick={() => onChange("ps1")} aria-pressed={value === "ps1"}>ps1</button>
    </div>
  );
}
