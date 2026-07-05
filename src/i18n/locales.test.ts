import { describe, it, expect } from "vitest";
import { resources, SUPPORTED_LANGUAGES } from "./index";

/** ネストしたJSONを "a.b.c" 形式のキー集合へ平坦化する。 */
function flattenKeys(obj: Record<string, unknown>, prefix = ""): string[] {
  return Object.entries(obj).flatMap(([key, value]) => {
    const path = prefix ? `${prefix}.${key}` : key;
    if (value !== null && typeof value === "object") {
      return flattenKeys(value as Record<string, unknown>, path);
    }
    return [path];
  });
}

describe("i18n ロケール", () => {
  it("対応言語は7言語（ja/en/fr/zh/ru/es/ar）である", () => {
    expect([...SUPPORTED_LANGUAGES].sort()).toEqual(
      ["ar", "en", "es", "fr", "ja", "ru", "zh"].sort(),
    );
  });

  const jaKeys = flattenKeys(
    resources.ja.translation as Record<string, unknown>,
  ).sort();

  it.each(SUPPORTED_LANGUAGES)(
    "%s は日本語（基準）と同一のキー構造を持つ（欠落・余剰なし）",
    (lang) => {
      const keys = flattenKeys(
        resources[lang].translation as Record<string, unknown>,
      ).sort();
      expect(keys).toEqual(jaKeys);
    },
  );

  it.each(SUPPORTED_LANGUAGES)("%s は全キーが空文字でない", (lang) => {
    const entries = Object.entries(
      flattenValue(resources[lang].translation as Record<string, unknown>),
    );
    for (const [key, value] of entries) {
      expect(value, `${lang}: ${key} が空`).not.toBe("");
    }
  });
});

/** キー→値のフラットなマップを作る（空文字検査用）。 */
function flattenValue(
  obj: Record<string, unknown>,
  prefix = "",
): Record<string, string> {
  return Object.entries(obj).reduce<Record<string, string>>(
    (acc, [key, value]) => {
      const path = prefix ? `${prefix}.${key}` : key;
      if (value !== null && typeof value === "object") {
        Object.assign(acc, flattenValue(value as Record<string, unknown>, path));
      } else {
        acc[path] = String(value);
      }
      return acc;
    },
    {},
  );
}
