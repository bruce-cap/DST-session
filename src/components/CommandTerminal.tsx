/** Renders the resume command preview and copy control. */

import { useState } from "react";
import type { TFunction } from "../lib/i18n";
import { Icon } from "./Icon";

export function CommandTerminal({ command, t, onCopy }: { command: string; t: TFunction; onCopy: () => void }) {
  const [copied, setCopied] = useState(false);

  function handleCopy() {
    onCopy();
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1400);
  }

  return (
    <section className="terminal">
      <div className="terminal-head">
        <span className="terminal-dots"><i /><i /><i /></span>
        <span className="terminal-title">{t("command_label")}</span>
        <button type="button" className={`terminal-copy ${copied ? "copied" : ""}`} onClick={handleCopy}>
          <Icon name={copied ? "check" : "copy"} />
          <span>{copied ? t("copy_done") : t("copy_now")}</span>
        </button>
      </div>
      <pre className="terminal-body"><span className="prompt">$</span><code>{command}</code><span className="caret" /></pre>
    </section>
  );
}
