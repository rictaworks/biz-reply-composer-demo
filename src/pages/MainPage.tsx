import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { api } from "@/lib/tauri";
import { useMasters } from "@/hooks/useMasters";
import { INPUT_LIMITS } from "@/config/app";
import { CopyButton } from "@/components/CopyButton";
import { Notice } from "@/components/Notice";
import type {
  AppErrorCode,
  GeneratedReply,
  HealthStatus,
  PolicyCode,
  RefineCode,
  ToneCode,
} from "@/types";

/** モデルウォームアップ状態のポーリング間隔（ms）。 */
const MODEL_STATUS_POLL_MS = 1500;

/** Rust側のエラー文字列を i18n コードへ写像する（未知は generic）。 */
function toErrorCode(error: unknown): AppErrorCode {
  const known: AppErrorCode[] = [
    "empty_input",
    "too_short",
    "too_long",
    "ollama_down",
    "model_missing",
    "timeout",
    "structure_incomplete",
    "generic",
  ];
  const raw = typeof error === "string" ? error : (error as { code?: string })?.code;
  return (known.includes(raw as AppErrorCode) ? raw : "generic") as AppErrorCode;
}

export function MainPage() {
  const { t } = useTranslation();
  const { masters } = useMasters();

  const [body, setBody] = useState("");
  const [policy, setPolicy] = useState<PolicyCode>("accept");
  const [tone, setTone] = useState<ToneCode>("standard");
  const [extra, setExtra] = useState("");

  const [reply, setReply] = useState<GeneratedReply | null>(null);
  const [errorCode, setErrorCode] = useState<AppErrorCode | null>(null);
  const [busy, setBusy] = useState(false);
  const [health, setHealth] = useState<HealthStatus | null>(null);
  const warmUpTriggered = useRef(false);

  const charCount = useMemo(() => [...body].length, [body]);
  const belowMin = charCount > 0 && charCount < INPUT_LIMITS.minChars;

  // モデルが読み込み（ウォーム）済みになるまで生成ボタンを無効化する。
  // 未読み込みのままだと実際の生成でコールドロード分の遅延・タイムアウトを招くため。
  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout> | undefined;

    async function poll() {
      try {
        const status = await api.healthCheck();
        if (cancelled) return;
        setHealth(status);

        if (
          status.ollamaRunning &&
          status.modelInstalled &&
          !status.modelLoaded &&
          !warmUpTriggered.current
        ) {
          warmUpTriggered.current = true;
          void api.warmUpModel().catch(() => {
            // 失敗してもポーリングで状態を再確認し続ける。
            warmUpTriggered.current = false;
          });
        }

        if (!status.modelLoaded && !cancelled) {
          timer = setTimeout(() => void poll(), MODEL_STATUS_POLL_MS);
        }
      } catch {
        if (!cancelled) {
          timer = setTimeout(() => void poll(), MODEL_STATUS_POLL_MS);
        }
      }
    }

    void poll();

    return () => {
      cancelled = true;
      if (timer) clearTimeout(timer);
    };
  }, []);

  const modelReady = health?.modelLoaded ?? false;

  async function handleGenerate() {
    setErrorCode(null);
    setBusy(true);
    try {
      const result = await api.generateReply({
        body,
        policyCode: policy,
        toneCode: tone,
        extra: extra || undefined,
      });
      setReply(result);
    } catch (e) {
      setErrorCode(toErrorCode(e));
    } finally {
      setBusy(false);
    }
  }

  async function handleRefine(preset: RefineCode) {
    if (!reply) return;
    setErrorCode(null);
    setBusy(true);
    try {
      const result = await api.refineReply(reply.replyId, preset);
      setReply(result);
    } catch (e) {
      setErrorCode(toErrorCode(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="page">
      <h1 className="page-title">{t("main.title")}</h1>

      <div className="field">
        <label htmlFor="mail-body">{t("main.inputLabel")}</label>
        <textarea
          id="mail-body"
          rows={10}
          value={body}
          maxLength={INPUT_LIMITS.maxChars}
          placeholder={t("main.inputPlaceholder")}
          onChange={(e) => setBody(e.target.value)}
        />
        <div className="field-meta">{t("main.charCount", { count: charCount })}</div>
        {belowMin ? <Notice kind="warning" errorCode="too_short" /> : null}
      </div>

      <div className="field-row">
        <div className="field">
          <label htmlFor="policy">{t("main.policyLabel")}</label>
          <select
            id="policy"
            value={policy}
            onChange={(e) => setPolicy(e.target.value as PolicyCode)}
          >
            {masters?.policies.map((p) => (
              <option key={p.code} value={p.code}>
                {t(`policy.${p.code}`)}
              </option>
            ))}
          </select>
        </div>
        <div className="field">
          <label htmlFor="tone">{t("main.toneLabel")}</label>
          <select
            id="tone"
            value={tone}
            onChange={(e) => setTone(e.target.value as ToneCode)}
          >
            {masters?.tones.map((tn) => (
              <option key={tn.code} value={tn.code}>
                {t(`tone.${tn.code}`)}
              </option>
            ))}
          </select>
        </div>
      </div>

      <div className="field">
        <label htmlFor="extra">{t("main.extraLabel")}</label>
        <input
          id="extra"
          type="text"
          value={extra}
          placeholder={t("main.extraPlaceholder")}
          onChange={(e) => setExtra(e.target.value)}
        />
      </div>

      <button
        type="button"
        className="btn btn-primary"
        disabled={busy || charCount < INPUT_LIMITS.minChars || !modelReady}
        onClick={handleGenerate}
      >
        <FontAwesomeIcon icon="pen-to-square" fixedWidth />
        <span>{busy ? t("main.generating") : t("main.generate")}</span>
      </button>

      {health && !health.ollamaRunning ? (
        <Notice kind="warning" errorCode="ollama_down" />
      ) : health && health.ollamaRunning && !health.modelInstalled ? (
        <Notice kind="warning" errorCode="model_missing" />
      ) : health && !health.modelLoaded ? (
        <div className="notice notice-warning" role="status">
          <FontAwesomeIcon icon="spinner" fixedWidth spin />
          <span className="notice-message">{t("main.modelLoading")}</span>
        </div>
      ) : null}

      {errorCode ? (
        <Notice errorCode={errorCode} onRetry={handleGenerate} />
      ) : null}

      {reply ? (
        <article className="result">
          <header className="result-header">
            <h2>{t("main.resultTitle")}</h2>
            <CopyButton text={reply.body} />
          </header>
          {!reply.structureValid ? (
            <Notice kind="warning" errorCode="structure_incomplete" />
          ) : null}
          <pre className="reply-body">{reply.body}</pre>

          <section className="context">
            <h3>{t("main.contextTitle")}</h3>
            <dl>
              <dt>{t("context.category")}</dt>
              <dd>{t(`category.${reply.context.category}`)}</dd>
              <dt>{t("context.requests")}</dt>
              <dd>{reply.context.requests.join(" / ") || t("context.none")}</dd>
              <dt>{t("context.deadline")}</dt>
              <dd>{reply.context.deadline ?? t("context.none")}</dd>
              <dt>{t("context.sentiment")}</dt>
              <dd>{reply.context.senderSentiment}</dd>
            </dl>
          </section>

          <section className="refine">
            <h3>{t("main.refineTitle")}</h3>
            <div className="refine-buttons">
              {masters?.refinePresets.map((preset) => (
                <button
                  key={preset.code}
                  type="button"
                  className="btn btn-secondary"
                  disabled={busy}
                  onClick={() => handleRefine(preset.code as RefineCode)}
                >
                  <FontAwesomeIcon icon="rotate" fixedWidth />
                  <span>{t(`refine.${preset.code}`)}</span>
                </button>
              ))}
            </div>
          </section>
        </article>
      ) : null}
    </section>
  );
}
