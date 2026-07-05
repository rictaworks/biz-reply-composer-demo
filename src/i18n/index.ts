import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import ja from "./locales/ja.json";
import en from "./locales/en.json";
import fr from "./locales/fr.json";
import zh from "./locales/zh.json";
import ru from "./locales/ru.json";
import es from "./locales/es.json";
import ar from "./locales/ar.json";

// 多言語（§CLAUDE.md i18n）: 日本語・英語・フランス語・中国語・ロシア語・スペイン語・アラビア語。
// 返信文の言語はユーザーの入力言語に合わせて Rust コア側で決定する（このUI i18nとは別）。
export const SUPPORTED_LANGUAGES = [
  "ja",
  "en",
  "fr",
  "zh",
  "ru",
  "es",
  "ar",
] as const;

export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

// 右書き（RTL）言語。<html dir> を切り替えるために使う。
export const RTL_LANGUAGES: SupportedLanguage[] = ["ar"];

export const DEFAULT_LANGUAGE: SupportedLanguage = "ja";

export const resources = {
  ja: { translation: ja },
  en: { translation: en },
  fr: { translation: fr },
  zh: { translation: zh },
  ru: { translation: ru },
  es: { translation: es },
  ar: { translation: ar },
} as const;

void i18n.use(initReactI18next).init({
  resources,
  lng: DEFAULT_LANGUAGE,
  fallbackLng: DEFAULT_LANGUAGE,
  interpolation: {
    escapeValue: false, // React が自動エスケープする
  },
});

/** 言語切替に伴い <html lang> / <html dir> を更新する。 */
export function applyDocumentLanguage(lang: SupportedLanguage): void {
  document.documentElement.lang = lang;
  document.documentElement.dir = RTL_LANGUAGES.includes(lang) ? "rtl" : "ltr";
}

export default i18n;
