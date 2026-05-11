/** Renders the settings popover and provider status information. */

import type { AppState, DeepseekLauncher, ProviderDescriptor, ThemeMode } from "../types";
import type { DeepseekStatus } from "../types";
import type { Locale, TFunction } from "../lib/i18n";
import { LocaleToggle } from "./LocaleToggle";
import { LauncherToggle } from "./LauncherToggle";

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
  onLauncherChange: (launcher: DeepseekLauncher) => void;
  onAutoRefreshChange: (enabled: boolean, intervalMinutes: number) => void;
}) {
  const command = props.provider?.commandLabel ?? "";
  const providerName = props.provider ? props.t(props.provider.displayNameKey) : "";

  return (
    <div className="settings-menu" role="dialog">
      <div className="settings-title"><span>{props.t("action_settings")}</span><span className="settings-version">v{props.version}</span></div>
      <div className="settings-row"><span>{props.t("settings_language")}</span><LocaleToggle locale={props.locale} onChange={props.onLocaleChange} /></div>
      <div className="settings-row">
        <span>{props.t("settings_dark")}</span>
        <button type="button" className={`switch ${props.theme === "dark" ? "on" : ""}`} onClick={() => props.onThemeChange((current) => (current === "dark" ? "light" : "dark"))} aria-pressed={props.theme === "dark"}><i /></button>
      </div>
      <div className="settings-note"><span>{props.t("settings_launch_mode")}</span><b>{props.t("settings_launch_mode_value")}</b></div>
      {props.provider?.capabilities.launcherToggle && (
        <div className="settings-row"><span>{props.t("settings_deepseek_launcher")}</span><LauncherToggle value={props.appState.deepseekLauncher} onChange={props.onLauncherChange} /></div>
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
