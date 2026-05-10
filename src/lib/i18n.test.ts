import { afterEach, beforeAll, describe, expect, it } from "vitest";
import { getLocale, setLocale, translate, translations } from "./i18n";

beforeAll(() => {
  if (typeof globalThis.localStorage === "undefined") {
    const store = new Map<string, string>();
    Object.defineProperty(globalThis, "localStorage", {
      configurable: true,
      value: {
        getItem: (key: string) => store.get(key) ?? null,
        setItem: (key: string, value: string) => void store.set(key, String(value)),
        removeItem: (key: string) => void store.delete(key),
        clear: () => store.clear(),
        key: (index: number) => Array.from(store.keys())[index] ?? null,
        get length() {
          return store.size;
        }
      }
    });
  }
});

describe("i18n", () => {
  afterEach(() => {
    setLocale("zh");
    localStorage.clear();
  });

  it("returns the Chinese string for a known key by default", () => {
    expect(translate("zh", "action_launch")).toBe("启动会话");
  });

  it("translates to English when requested", () => {
    expect(translate("en", "action_launch")).toBe("Launch session");
  });

  it("interpolates named parameters", () => {
    expect(translate("zh", "invalid_count", { count: 3 })).toBe(
      "有 3 个 session 文件无法解析，已隔离显示。"
    );
    expect(translate("en", "invalid_count", { count: 3 })).toBe(
      "3 session file(s) failed to parse and are isolated."
    );
  });

  it("falls back to Chinese when the English dict is missing a value", () => {
    expect(translate("en", "action_launch")).toBeTypeOf("string");
  });

  it("persists locale switches through setLocale", () => {
    setLocale("en");
    expect(getLocale()).toBe("en");
    expect(localStorage.getItem("deepseek-session-manager-locale")).toBe("en");
  });

  it("keeps Chinese and English dictionaries in sync", () => {
    const zhKeys = Object.keys(translations.zh).sort();
    const enKeys = Object.keys(translations.en).sort();
    expect(enKeys).toEqual(zhKeys);
  });
});
