/** Renders SVG icons from a registry keyed by icon name. */

import type { ReactElement } from "react";

export type IconName =
  | "play"
  | "copy"
  | "folder"
  | "folder-small"
  | "refresh"
  | "settings"
  | "layers"
  | "search"
  | "close"
  | "chevron"
  | "star"
  | "star-fill"
  | "check"
  | "alert"
  | "inbox"
  | "calendar"
  | "cpu"
  | "mode";

const icons: Record<IconName, ReactElement> = {
  play: <path d="M8 5.8v12.4c0 .9 1 1.4 1.7.9l9-6.2c.6-.4.6-1.3 0-1.8l-9-6.2C9 4.4 8 4.9 8 5.8Z" />,
  copy: <><rect x="8" y="6" width="12" height="13" rx="2.5" /><path d="M5 13.5V7a2 2 0 0 1 2-2h6.5" /></>,
  folder: <path d="M3.5 7.5A2.5 2.5 0 0 1 6 5h4l2 2h6A2.5 2.5 0 0 1 20.5 9.5v6A3.5 3.5 0 0 1 17 19H7a3.5 3.5 0 0 1-3.5-3.5v-8Z" />,
  "folder-small": <path d="M3.5 7.5A2 2 0 0 1 5.5 5.5H9l2 2h7A2 2 0 0 1 20 9.5v6A2.5 2.5 0 0 1 17.5 18h-12A2.5 2.5 0 0 1 3 15.5v-8Z" />,
  refresh: <><path d="M18.5 9A7 7 0 0 0 6 6.8L4.5 8.5" /><path d="M4.5 4.5v4h4" /><path d="M5.5 15A7 7 0 0 0 18 17.2l1.5-1.7" /><path d="M19.5 19.5v-4h-4" /></>,
  settings: <><circle cx="12" cy="12" r="3.2" /><path d="M19.4 13.2c.1-.4.1-.8.1-1.2s0-.8-.1-1.2l2-1.5-2-3.4-2.4 1a8 8 0 0 0-2.1-1.2L14.4 3h-4.8l-.4 2.7c-.8.3-1.5.7-2.1 1.2l-2.4-1-2 3.4 2 1.5a8.3 8.3 0 0 0 0 2.4l-2 1.5 2 3.4 2.4-1c.6.5 1.3.9 2.1 1.2l.4 2.7h4.8l.4-2.7c.8-.3 1.5-.7 2.1-1.2l2.4 1 2-3.4-2.1-1.5Z" /></>,
  layers: <><path d="m12 4 9 4.8-9 4.8-9-4.8Z" /><path d="m3 14.4 9 4.8 9-4.8" /></>,
  search: <><circle cx="11" cy="11" r="6.5" /><path d="m20 20-3.5-3.5" /></>,
  close: <path d="m6 6 12 12M18 6 6 18" />,
  chevron: <path d="m7 10 5 5 5-5" />,
  star: <path d="m12 3.6 2.7 5.5 6 .9-4.35 4.2 1.03 6-5.38-2.82-5.38 2.82 1.03-6L3.3 10l6-.9Z" />,
  "star-fill": <path d="m12 3.6 2.7 5.5 6 .9-4.35 4.2 1.03 6-5.38-2.82-5.38 2.82 1.03-6L3.3 10l6-.9Z" />,
  check: <path d="m5 12 5 5 9-10" />,
  alert: <><path d="M12 3.5 21 20H3Z" /><path d="M12 10v4.2M12 17.2v.2" /></>,
  inbox: <><path d="M4 4.5h16V14h-5l-1.5 2h-3L9 14H4Z" /><path d="M4 14v5a1 1 0 0 0 1 1h14a1 1 0 0 0 1-1v-5" /></>,
  calendar: <><rect x="3.5" y="5" width="17" height="15" rx="2.5" /><path d="M3.5 10h17M8 3.5v4M16 3.5v4" /></>,
  cpu: <><rect x="6" y="6" width="12" height="12" rx="2" /><rect x="9.5" y="9.5" width="5" height="5" rx="1" /><path d="M10 3.5v2.5M14 3.5v2.5M10 18v2.5M14 18v2.5M3.5 10H6M3.5 14H6M18 10h2.5M18 14h2.5" /></>,
  mode: <><circle cx="12" cy="12" r="8" /><path d="M12 4v16" /></>
};

export function Icon({ name }: { name: IconName }) {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" className={name === "star-fill" ? "filled" : undefined}>
      {icons[name]}
    </svg>
  );
}
