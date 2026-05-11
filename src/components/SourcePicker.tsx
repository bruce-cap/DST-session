/** Renders the provider source selection cards. */

import logoUrl from "../assets/logo.svg";
import type { ProviderDescriptor, SessionSource } from "../types";

export function SourcePicker(props: {
  providers: ProviderDescriptor[];
  source: SessionSource;
  onSourceChange: (source: SessionSource) => void;
}) {
  return (
    <div className="source-picker">
      {props.providers.map((provider) => (
        <button
          key={provider.id}
          type="button"
          className={`source-card ${props.source === provider.id ? "active" : ""}`}
          onClick={() => props.onSourceChange(provider.id)}
        >
          <span className={`source-card-badge ${provider.badgeKey}`}>
            {provider.badgeText ? provider.badgeText : <img src={logoUrl} alt="" />}
          </span>
          <span className="source-card-text">
            <b>{provider.shortName.split(" ")[0]}</b>
            <small>{provider.shortName.split(" ").slice(1).join(" ") || "CLI"}</small>
          </span>
        </button>
      ))}
    </div>
  );
}
