/** Renders the settings popover and provider status information. */

import type { AppState, ProviderDescriptor, ProviderLauncher, ThemeMode } from "../types";
import type { DeepseekStatus } from "../types";
import type { Locale, TFunction } from "../lib/i18n";
import { LocaleToggle } from "./LocaleToggle";
import { ProviderLaunchSettings } from "./ProviderLaunchSettings";

export function SettingsMenu(props: {
  version: string;
  locale: Locale;
  theme: ThemeMode;
  appState: AppState;
  provider: ProviderDescriptor | null;
  status: DeepseekStatus | null;
  t: TFunction;
  onLocaleChange: (locale: Locale) => void;
  onThemeChange: (theme: ThemeMode | ((current: ThemeMode) => ThemeMode)) => void;
  onProviderLauncherChange: (launcher: ProviderLauncher) => void;
  onAutoRefreshChange: (enabled: boolean, intervalMinutes: number) => void;
}) {
  const source = props.provider?.id ?? "deepseek";
  const launcher = props.appState.providerLaunchers[source] ?? (source === "codex" ? "ps1" : "cmd");
  const command = props.provider ? commandLabel(props.provider.commandLabel, launcher) : "";
  const providerName = props.provider ? props.t(props.provider.displayNameKey) : "";

  return (
    <div className="settings-menu" role="dialog">
      <div className="settings-title"><span>{props.t("action_settings")}</span><span className="settings-version">v{props.version}</span></div>
      <div className="settings-row"><span>{props.t("settings_language")}</span><LocaleToggle locale={props.locale} onChange={props.onLocaleChange} /></div>
      <div className="settings-row">
        <span>{props.t("settings_dark")}</span>
        <button type="button" className={`switch ${props.theme === "dark" ? "on" : ""}`} onClick={() => props.onThemeChange((current) => (current === "dark" ? "light" : "dark"))} aria-pressed={props.theme === "dark"}><i /></button>
      </div>
      {props.provider?.capabilities.launcherToggle && (
        <ProviderLaunchSettings launcher={launcher} t={props.t} onLauncherChange={props.onProviderLauncherChange} />
      )}
      <div className="settings-row">
        <span>{props.t("settings_auto_refresh")}</span>
        <button type="button" className={`switch ${props.appState.autoRefreshEnabled ? "on" : ""}`} onClick={() => props.onAutoRefreshChange(!props.appState.autoRefreshEnabled, props.appState.autoRefreshIntervalMinutes)} aria-pressed={props.appState.autoRefreshEnabled}><i /></button>
      </div>
      <div className="settings-row">
        <span>{props.t("settings_auto_refresh_interval")}</span>
        <select className="settings-select" value={props.appState.autoRefreshIntervalMinutes} onChange={(event) => props.onAutoRefreshChange(props.appState.autoRefreshEnabled, Number(event.target.value))}>
          {[1, 5, 10, 15, 30, 60].map((minutes) => <option key={minutes} value={minutes}>{props.t("settings_minutes", { n: minutes })}</option>)}
        </select>
      </div>
      <div className={`settings-status ${props.status?.available ? "ok" : "bad"}`}>
        <span className="status-dot" />
        <div>
          <b>{command}</b>
          <small>{props.status?.available ? props.status.version || props.t("settings_cli_available") : props.status?.message || props.t("settings_cli_unavailable")}</small>
        </div>
      </div>
      <p>{props.t("settings_footer", { source: providerName })}</p>
    </div>
  );
}

function commandLabel(command: string, launcher: ProviderLauncher): string {
  if (command.endsWith(".cmd") || command.endsWith(".ps1")) {
    return command.replace(/\.(cmd|ps1)$/, `.${launcher}`);
  }
  return `${command}.${launcher}`;
}
