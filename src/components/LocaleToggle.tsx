/** Renders the locale segmented control. */

import type { Locale } from "../lib/i18n";

export function LocaleToggle({ locale, onChange }: { locale: Locale; onChange: (locale: Locale) => void }) {
  return (
    <div className="segmented" role="group" aria-label="Language">
      <button type="button" className={locale === "zh" ? "active" : ""} onClick={() => onChange("zh")} aria-pressed={locale === "zh"}>中</button>
      <button type="button" className={locale === "en" ? "active" : ""} onClick={() => onChange("en")} aria-pressed={locale === "en"}>EN</button>
    </div>
  );
}
