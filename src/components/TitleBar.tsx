/** Renders the application title bar, search, refresh, and settings entry. */

import type { RefObject, ReactNode } from "react";
import logoUrl from "../assets/logo.svg";
import { Icon } from "./Icon";
import { IconButton } from "./IconButton";
import type { TFunction } from "../lib/i18n";

export function TitleBar(props: {
  search: string;
  settingsOpen: boolean;
  settingsRef: RefObject<HTMLDivElement | null>;
  settings: ReactNode;
  t: TFunction;
  onSearchChange: (value: string) => void;
  onRefresh: () => void;
  onToggleSettings: () => void;
}) {
  return (
    <header className="titlebar">
      <div className="brand">
        <div className="brand-mark"><img src={logoUrl} alt="" /></div>
        <div className="brand-text"><strong>Agent Session Manager</strong><span>DeepSeek · Claude Code · Codex</span></div>
      </div>
      <div className="search-input titlebar-search">
        <Icon name="search" />
        <input value={props.search} onChange={(event) => props.onSearchChange(event.target.value)} placeholder={props.t("search_placeholder")} />
        {props.search ? (
          <button type="button" className="input-clear" aria-label={props.t("clear_search")} onClick={() => props.onSearchChange("")}><Icon name="close" /></button>
        ) : (
          <kbd className="kbd">⌘K</kbd>
        )}
      </div>
      <div className="titlebar-actions">
        <IconButton label={props.t("action_refresh")} icon="refresh" onClick={props.onRefresh} />
        <div className="settings-wrap" ref={props.settingsRef}>
          <IconButton label={props.t("action_settings")} icon="settings" onClick={props.onToggleSettings} ariaExpanded={props.settingsOpen} active={props.settingsOpen} tooltipAlign="end" />
          {props.settingsOpen && props.settings}
        </div>
      </div>
    </header>
  );
}
