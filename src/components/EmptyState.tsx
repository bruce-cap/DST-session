/** Renders empty and loading states for workspace panels. */

import { Icon } from "./Icon";

export function EmptyState({ title, hint }: { title: string; hint?: string }) {
  return (
    <div className="empty empty-state">
      <Icon name="inbox" />
      <p>{title}</p>
      {hint && <small>{hint}</small>}
    </div>
  );
}

export function LoadingState() {
  return (
    <div className="empty">
      <div className="skeleton" />
      <div className="skeleton" />
      <div className="skeleton" />
    </div>
  );
}
