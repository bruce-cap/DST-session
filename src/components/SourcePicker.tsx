/** Renders the provider source selection cards. */

import claudeLogo from "../assets/providers/claude.svg";
import codexLogo from "../assets/providers/codex.svg";
import deepseekLogo from "../assets/providers/deepseek.svg";
import type { ProviderDescriptor, SessionSource } from "../types";

const providerLogo: Record<ProviderDescriptor["iconKey"], string> = {
  claude: claudeLogo,
  codex: codexLogo,
  deepseek: deepseekLogo
};

export function SourcePicker(props: {
  providers: ProviderDescriptor[];
  source: SessionSource;
  onSourceChange: (source: SessionSource) => void;
}) {
  return (
    <div className="source-picker">
      {props.providers.map((provider, index) => (
        <button
          key={provider.id}
          type="button"
          className={`source-icon-button tooltip-target ${props.source === provider.id ? "active" : ""}`}
          onClick={() => props.onSourceChange(provider.id)}
          aria-label={provider.shortName}
          title={provider.shortName}
          data-tooltip={provider.shortName}
          data-tooltip-align={index === 0 ? "start" : index === props.providers.length - 1 ? "end" : "center"}
        >
          <span className={`source-icon-frame ${provider.badgeKey}`}>
            <img src={providerLogo[provider.iconKey]} alt="" />
          </span>
        </button>
      ))}
    </div>
  );
}
