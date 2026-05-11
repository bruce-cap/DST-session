/** Composes source selection and grouped navigation. */

import type { GroupBy, ProviderDescriptor, SessionGroup, SessionSource } from "../types";
import type { TFunction } from "../lib/i18n";
import { SourcePicker } from "./SourcePicker";
import { GroupPicker } from "./GroupPicker";
import { GroupList } from "./GroupList";

export function Sidebar(props: {
  providers: ProviderDescriptor[];
  source: SessionSource;
  groupBy: GroupBy;
  groups: SessionGroup[];
  activeGroupKey: string | null;
  total: number;
  t: TFunction;
  onSourceChange: (source: SessionSource) => void;
  onGroupByChange: (groupBy: GroupBy) => void;
  onActiveGroupChange: (key: string | null) => void;
  onSelectFirstInGroup: (group: SessionGroup) => void;
}) {
  return (
    <aside className="sidebar">
      <SourcePicker providers={props.providers} source={props.source} onSourceChange={props.onSourceChange} />
      <GroupPicker groupBy={props.groupBy} t={props.t} onGroupByChange={props.onGroupByChange} />
      <GroupList groups={props.groups} activeGroupKey={props.activeGroupKey} total={props.total} t={props.t} onActiveGroupChange={props.onActiveGroupChange} onSelectFirstInGroup={props.onSelectFirstInGroup} />
    </aside>
  );
}
