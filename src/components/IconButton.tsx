/** Renders a tooltip-enabled icon button. */

import { Icon, type IconName } from "./Icon";

export function IconButton(props: {
  label: string;
  icon: IconName;
  onClick: () => void;
  disabled?: boolean;
  primary?: boolean;
  active?: boolean;
  ariaExpanded?: boolean;
  tooltipAlign?: "start" | "center" | "end";
}) {
  return (
    <button
      type="button"
      className={`icon-button tooltip-target ${props.primary ? "primary" : ""} ${props.active ? "active" : ""}`}
      onClick={props.onClick}
      disabled={props.disabled}
      aria-label={props.label}
      aria-expanded={props.ariaExpanded}
      data-tooltip={props.label}
      data-tooltip-align={props.tooltipAlign ?? "center"}
    >
      <Icon name={props.icon} />
    </button>
  );
}
