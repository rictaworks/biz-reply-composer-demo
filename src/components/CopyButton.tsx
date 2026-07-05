import { useState } from "react";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { useTranslation } from "react-i18next";

export function CopyButton({ text }: { text: string }) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      // クリップボード不可環境でもクラッシュさせない（通知は握り潰さずボタン状態のみ）。
      setCopied(false);
    }
  }

  return (
    <button type="button" className="btn btn-secondary" onClick={handleCopy}>
      <FontAwesomeIcon icon={copied ? "check" : "copy"} fixedWidth />
      <span>{copied ? t("main.copied") : t("main.copy")}</span>
    </button>
  );
}
