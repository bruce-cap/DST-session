import type { ProviderLauncher } from "../types";
import type { TFunction } from "../lib/i18n";

export function ProviderLaunchSettings(props: {
  launcher: ProviderLauncher;
  t: TFunction;
  onLauncherChange: (value: ProviderLauncher) => void;
}) {
  return (
    <div className="settings-group">
      <div className="settings-row">
        <span>{props.t("settings_agent_launcher")}</span>
        <div className="segmented" role="group" aria-label={props.t("settings_agent_launcher")}>
          <button type="button" className={props.launcher === "cmd" ? "active" : ""} onClick={() => props.onLauncherChange("cmd")} aria-pressed={props.launcher === "cmd"}>cmd</button>
          <button type="button" className={props.launcher === "ps1" ? "active" : ""} onClick={() => props.onLauncherChange("ps1")} aria-pressed={props.launcher === "ps1"}>ps1</button>
        </div>
      </div>
    </div>
  );
}
