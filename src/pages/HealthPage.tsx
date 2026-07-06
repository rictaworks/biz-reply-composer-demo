import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { api } from "@/lib/tauri";
import type { HealthStatus } from "@/types";

type Row = { labelKey: string; ok: boolean };

export function HealthPage() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<HealthStatus | null>(null);
  const [checking, setChecking] = useState(false);

  const runCheck = useCallback(async () => {
    setChecking(true);
    try {
      setStatus(await api.healthCheck());
    } finally {
      setChecking(false);
    }
  }, []);

  useEffect(() => {
    void runCheck();
  }, [runCheck]);

  const rows: Row[] = status
    ? [
        { labelKey: "health.ollama", ok: status.ollamaRunning },
        { labelKey: "health.model", ok: status.modelInstalled },
        { labelKey: "health.modelLoaded", ok: status.modelLoaded },
      ]
    : [];

  return (
    <section className="page">
      <h1 className="page-title">{t("health.title")}</h1>
      <p className="page-desc">{t("health.description")}</p>

      <button
        type="button"
        className="btn btn-primary"
        disabled={checking}
        onClick={runCheck}
      >
        <FontAwesomeIcon icon="heart-pulse" fixedWidth />
        <span>{checking ? t("health.checking") : t("health.check")}</span>
      </button>

      <ul className="health-list">
        {rows.map((row) => (
          <li key={row.labelKey} className={row.ok ? "health-ok" : "health-ng"}>
            <FontAwesomeIcon
              icon={row.ok ? "circle-check" : "circle-xmark"}
              fixedWidth
            />
            <span>{t(row.labelKey)}</span>
            <span className="health-state">
              {row.ok ? t("health.ok") : t("health.ng")}
            </span>
          </li>
        ))}
      </ul>
    </section>
  );
}
