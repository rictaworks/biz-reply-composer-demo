// Rustコア（Tauriコマンド）とフロントで共有するデータ契約。
// 実際のロジックはRust側（src-tauri）に置き、ここは型のみ。

export type PolicyCode = "accept" | "decline" | "hold" | "question";
export type ToneCode = "formal" | "standard" | "casual";
export type RefineCode = "politer" | "shorter" | "softer" | "concrete";
export type CategoryCode =
  | "request"
  | "reminder"
  | "apology"
  | "scheduling"
  | "inquiry"
  | "gratitude"
  | "complaint"
  | "other";

/** マスタ1件（表示名はi18nでcodeから解決するため name は開発者用日本語）。 */
export interface MasterItem {
  id: number;
  code: string;
  name: string;
}

export interface RecommendedModel extends MasterItem {
  isDefault: boolean;
  minRamGb: number | null;
  note: string | null;
}

/** get_masters の戻り（計22件）。 */
export interface Masters {
  policies: MasterItem[];
  tones: MasterItem[];
  categories: MasterItem[];
  refinePresets: MasterItem[];
  models: RecommendedModel[];
}

/** 文脈抽出結果（§1.5 手順5 / クラス図 MailContext）。 */
export interface MailContext {
  category: CategoryCode;
  requests: string[];
  deadline: string | null;
  senderSentiment: string;
}

export interface GenerateReplyInput {
  body: string;
  policyCode: PolicyCode;
  toneCode: ToneCode;
  extra?: string;
}

/** 生成された返信（マスキング済み本文のみ保持）。 */
export interface GeneratedReply {
  replyId: number;
  mailId: number;
  body: string;
  structureValid: boolean;
  context: MailContext;
  policyCode: PolicyCode;
  toneCode: ToneCode;
  createdAt: string;
}

export interface HealthStatus {
  ollamaRunning: boolean;
  modelInstalled: boolean;
  model: string;
  checkedAt: string;
}

export interface HistoryItem {
  replyId: number;
  body: string;
  policyCode: PolicyCode;
  toneCode: ToneCode;
  structureValid: boolean;
  createdAt: string;
}

/** フォールバックしないエラーの分類（§1.5 / 仕様 §8）。i18n の error.* キーと対応。 */
export type AppErrorCode =
  | "empty_input"
  | "too_short"
  | "too_long"
  | "ollama_down"
  | "model_missing"
  | "timeout"
  | "structure_incomplete"
  | "generic";

export interface AppError {
  code: AppErrorCode;
  detail?: string;
}
