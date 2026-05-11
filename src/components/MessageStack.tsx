/** Renders transient errors, notices, CLI warnings, and parse warnings. */

import { Icon } from "./Icon";
import type { ProviderDescriptor, SourceState } from "../types";
import type { TFunction } from "../lib/i18n";

export function MessageStack(props: {
  error: string | null;
  notice: string | null;
  invalidCount: number;
  cliMissing: boolean;
  refreshing: boolean;
  sourceState: SourceState | null;
  provider: ProviderDescriptor | null;
  commandLabel?: string;
  t: TFunction;
}) {
  if (!props.error && !props.notice && props.invalidCount === 0 && !props.cliMissing && !props.refreshing && !props.sourceState?.lastError) {
    return null;
  }

  return (
    <section className="message-stack">
      {props.error && <div className="message error"><Icon name="alert" /> <span>{props.error}</span></div>}
      {props.cliMissing && !props.error && <div className="message error"><Icon name="alert" /><span>{props.t("cli_missing", { command: props.commandLabel ?? props.provider?.commandLabel ?? "" })}</span></div>}
      {props.notice && <div className="message success"><Icon name="check" /> <span>{props.notice}</span></div>}
      {props.refreshing && <div className="message info"><Icon name="refresh" /> <span>{props.t("refreshing")}</span></div>}
      {props.sourceState?.lastError && !props.error && <div className="message warn"><Icon name="alert" /> <span>{props.t("refresh_failed", { message: props.sourceState.lastError })}</span></div>}
      {props.invalidCount > 0 && <div className="message warn"><Icon name="alert" /> <span>{props.t("invalid_count", { count: props.invalidCount })}</span></div>}
    </section>
  );
}
