/** Renders grouped session navigation. */

import type { SessionGroup } from "../types";
import type { TFunction } from "../lib/i18n";

const GROUP_LABEL_LIMIT = 20;

export function GroupList(props: {
  groups: SessionGroup[];
  activeGroupKey: string | null;
  total: number;
  t: TFunction;
  onActiveGroupChange: (key: string | null) => void;
  onSelectFirstInGroup: (group: SessionGroup) => void;
}) {
  return (
    <div className="sidebar-section">
      <div className="sidebar-label"><span>{props.t("panel_groups")}</span><b>{props.total}</b></div>
      <div className="group-list">
        <button type="button" className={`group-item ${props.activeGroupKey === null ? "active" : ""}`} onClick={() => props.onActiveGroupChange(null)}>
          <span>{props.t("group_all")}</span><em>{props.total}</em>
        </button>
        {props.groups.map((group) => (
          <button
            type="button"
            className={`group-item ${props.activeGroupKey === group.key ? "active" : ""}`}
            key={group.key}
            onClick={() => {
              props.onActiveGroupChange(props.activeGroupKey === group.key ? null : group.key);
              props.onSelectFirstInGroup(group);
            }}
            title={group.label}
          >
            <span>{truncateGroupLabel(group.label)}</span><em>{group.sessions.length}</em>
          </button>
        ))}
      </div>
    </div>
  );
}

function truncateGroupLabel(label: string): string {
  const chars = Array.from(label);
  if (chars.length <= GROUP_LABEL_LIMIT) {
    return label;
  }
  return `${chars.slice(0, GROUP_LABEL_LIMIT).join("")}...`;
}
