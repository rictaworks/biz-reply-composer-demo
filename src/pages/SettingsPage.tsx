import { useTranslation } from "react-i18next";
import { useMasters } from "@/hooks/useMasters";
import { LanguageSwitcher } from "@/components/LanguageSwitcher";

export function SettingsPage() {
  const { t } = useTranslation();
  const { masters } = useMasters();
  const defaultModel = masters?.models.find((m) => m.isDefault) ?? masters?.models[0];

  return (
    <section className="page">
      <h1 className="page-title">{t("settings.title")}</h1>

      <div className="field">
        <label htmlFor="model">{t("settings.model")}</label>
        <select id="model" defaultValue={defaultModel?.code} disabled>
          {masters?.models.map((m) => (
            <option key={m.code} value={m.code}>
              {m.name}
            </option>
          ))}
        </select>
        <p className="field-meta">{t("settings.modelNote")}</p>
      </div>

      <div className="field">
        <span className="field-label">{t("settings.language")}</span>
        <LanguageSwitcher />
      </div>
    </section>
  );
}
