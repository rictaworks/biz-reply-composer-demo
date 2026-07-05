import { useTranslation } from "react-i18next";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import {
  SUPPORTED_LANGUAGES,
  applyDocumentLanguage,
  type SupportedLanguage,
} from "@/i18n";

const LANGUAGE_LABELS: Record<SupportedLanguage, string> = {
  ja: "日本語",
  en: "English",
  fr: "Français",
  zh: "中文",
  ru: "Русский",
  es: "Español",
  ar: "العربية",
};

export function LanguageSwitcher() {
  const { i18n, t } = useTranslation();

  function handleChange(event: React.ChangeEvent<HTMLSelectElement>) {
    const lang = event.target.value as SupportedLanguage;
    void i18n.changeLanguage(lang);
    applyDocumentLanguage(lang);
  }

  return (
    <label className="language-switcher">
      <FontAwesomeIcon icon="language" fixedWidth />
      <span className="sr-only">{t("settings.language")}</span>
      <select
        value={i18n.resolvedLanguage}
        onChange={handleChange}
        aria-label={t("settings.language")}
      >
        {SUPPORTED_LANGUAGES.map((lang) => (
          <option key={lang} value={lang}>
            {LANGUAGE_LABELS[lang]}
          </option>
        ))}
      </select>
    </label>
  );
}
