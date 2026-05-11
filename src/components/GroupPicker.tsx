/** Renders the group-by selector. */

import type { GroupBy } from "../types";
import type { TFunction } from "../lib/i18n";

const GROUP_KEYS: Record<GroupBy, "group_workspace" | "group_date" | "group_model" | "group_mode" | "group_favorite" | "group_none"> = {
  workspace: "group_workspace",
  date: "group_date",
  model: "group_model",
  mode: "group_mode",
  favorite: "group_favorite",
  none: "group_none"
};

const GROUP_ORDER: GroupBy[] = ["workspace", "date", "model", "mode", "favorite", "none"];

export function GroupPicker({ groupBy, t, onGroupByChange }: { groupBy: GroupBy; t: TFunction; onGroupByChange: (groupBy: GroupBy) => void }) {
  return (
    <div className="sidebar-section">
      <div className="sidebar-label"><span>{t("group_by")}</span></div>
      <div className="group-select">
        {GROUP_ORDER.map((option) => (
          <button key={option} type="button" className={`group-select-item ${groupBy === option ? "active" : ""}`} onClick={() => onGroupByChange(option)}>
            {t(GROUP_KEYS[option])}
          </button>
        ))}
      </div>
    </div>
  );
}
