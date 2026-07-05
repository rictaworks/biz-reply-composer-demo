import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { useTranslation } from "react-i18next";
import type { AppErrorCode } from "@/types";

type NoticeKind = "error" | "warning";

interface NoticeProps {
  kind?: NoticeKind;
  /** i18n error.* のキー。 */
  errorCode: AppErrorCode;
  onRetry?: () => void;
}

// ネイティブ alert/confirm/prompt は全面禁止のため、画面内インライン通知で表現する。
export function Notice({ kind = "error", errorCode, onRetry }: NoticeProps) {
  const { t } = useTranslation();
  return (
    <div className={`notice notice-${kind}`} role="alert">
      <FontAwesomeIcon icon="triangle-exclamation" fixedWidth />
      <span className="notice-message">{t(`error.${errorCode}`)}</span>
      {onRetry ? (
        <button type="button" className="btn btn-ghost" onClick={onRetry}>
          <FontAwesomeIcon icon="rotate" fixedWidth />
          <span>{t("error.retry")}</span>
        </button>
      ) : null}
    </div>
  );
}
