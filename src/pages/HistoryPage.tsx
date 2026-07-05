import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { api } from "@/lib/tauri";
import { CopyButton } from "@/components/CopyButton";
import type { HistoryItem } from "@/types";

export function HistoryPage() {
  const { t } = useTranslation();
  const [items, setItems] = useState<HistoryItem[]>([]);

  useEffect(() => {
    let cancelled = false;
    api.listHistory().then((rows) => {
      if (!cancelled) setItems(rows);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="page">
      <h1 className="page-title">{t("history.title")}</h1>
      {items.length === 0 ? (
        <p className="empty">{t("history.empty")}</p>
      ) : (
        <ul className="history-list">
          {items.map((item) => (
            <li key={item.replyId} className="history-item">
              <div className="history-meta">
                <span className="badge">{t(`policy.${item.policyCode}`)}</span>
                <span className="badge">{t(`tone.${item.toneCode}`)}</span>
                <time>{item.createdAt}</time>
              </div>
              <pre className="reply-body">{item.body}</pre>
              <CopyButton text={item.body} />
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
