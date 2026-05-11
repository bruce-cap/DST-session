/** Renders the quick-reply composer for providers that support prompts. */

import type { KeyboardEvent } from "react";
import { normalizeSingleLine } from "../lib/commands";
import type { TFunction } from "../lib/i18n";

export function QuickReply(props: {
  value: string;
  canResume: boolean;
  t: TFunction;
  onChange: (value: string) => void;
  onSubmit: () => void;
}) {
  const promptPreview = normalizeSingleLine(props.value);

  function handleKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Enter" && props.canResume && promptPreview) {
      event.preventDefault();
      props.onSubmit();
    }
  }

  return (
    <section className="quick-reply">
      <div className="section-head">{props.t("quick_reply_label")}</div>
      <div className="quick-reply-row">
        <input type="text" value={props.value} onChange={(event) => props.onChange(event.target.value)} onKeyDown={handleKeyDown} placeholder={props.t("quick_reply_placeholder")} />
        <button type="button" className="btn-primary btn-primary-sm" onClick={props.onSubmit} disabled={!props.canResume || !promptPreview}>
          <span>{props.t("quick_reply_send")}</span><kbd className="kbd inverse">⏎</kbd>
        </button>
      </div>
      <p className="hint">{props.t("quick_reply_hint")}</p>
    </section>
  );
}
