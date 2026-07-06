// 開発用モックバックエンド（Tauri外＝ブラウザプレビュー / vitest でのみ使用）。
// 本番（デスクトップ）ではRustコアが応答するため、このモックは呼ばれない。
//
// 重要: これは「Tauriが無い開発環境でUIを描画するためのスタブ」であり、
// ollama未起動・モデル未導入・タイムアウト時のルールベース代替生成（禁止されたフォールバック）ではない。
// 生成結果は明示的にデモ用プレースホルダであることを示す。

import type {
  GeneratedReply,
  GenerateReplyInput,
  HealthStatus,
  HistoryItem,
  Masters,
  RefineCode,
} from "@/types";

const MASTERS: Masters = {
  policies: [
    { id: 1, code: "accept", name: "承諾" },
    { id: 2, code: "decline", name: "辞退" },
    { id: 3, code: "hold", name: "保留" },
    { id: 4, code: "question", name: "追加質問" },
  ],
  tones: [
    { id: 1, code: "formal", name: "フォーマル" },
    { id: 2, code: "standard", name: "標準" },
    { id: 3, code: "casual", name: "カジュアル" },
  ],
  categories: [
    { id: 1, code: "request", name: "依頼" },
    { id: 2, code: "reminder", name: "催促" },
    { id: 3, code: "apology", name: "謝罪" },
    { id: 4, code: "scheduling", name: "日程調整" },
    { id: 5, code: "inquiry", name: "問い合わせ" },
    { id: 6, code: "gratitude", name: "御礼" },
    { id: 7, code: "complaint", name: "クレーム" },
    { id: 8, code: "other", name: "その他" },
  ],
  refinePresets: [
    { id: 1, code: "politer", name: "もっと丁寧に" },
    { id: 2, code: "shorter", name: "短く" },
    { id: 3, code: "softer", name: "柔らかく" },
    { id: 4, code: "concrete", name: "具体的に" },
  ],
  models: [
    { id: 1, code: "gemma3:1b", name: "Gemma 3 1B", isDefault: true, minRamGb: 2, note: null },
    { id: 2, code: "llama3-elyza-jp:8b", name: "Llama-3-ELYZA-JP-8B", isDefault: false, minRamGb: 8, note: null },
    { id: 3, code: "phi4-mini", name: "Phi-4 mini", isDefault: false, minRamGb: 4, note: null },
  ],
};

const history: HistoryItem[] = [];
let nextId = 1;

// テストで決定的にするため乱数・現在時刻には依存しない固定タイムスタンプ。
const DEMO_TIMESTAMP = "2026-07-05T12:00:00+09:00";

function demoDraft(input: GenerateReplyInput): GeneratedReply {
  const id = nextId++;
  const reply: GeneratedReply = {
    replyId: id,
    mailId: id,
    // 4部構成（宛名→挨拶→本文→結び）。宛名・署名はトークン固定（固有名詞捏造禁止）。
    body: [
      "{{相手名}} 様",
      "いつもお世話になっております。",
      "（これはTauri外プレビュー用のデモ本文です。実際の返信文はデスクトップ版でollamaが生成します。）",
      "何卒よろしくお願いいたします。",
      "{{自分の署名}}",
    ].join("\n\n"),
    structureValid: true,
    context: {
      category: "request",
      requests: ["デモ要求事項"],
      deadline: null,
      senderSentiment: "neutral",
    },
    policyCode: input.policyCode,
    toneCode: input.toneCode,
    createdAt: DEMO_TIMESTAMP,
  };
  history.unshift({
    replyId: reply.replyId,
    body: reply.body,
    policyCode: reply.policyCode,
    toneCode: reply.toneCode,
    structureValid: reply.structureValid,
    createdAt: reply.createdAt,
  });
  return reply;
}

export const mockBackend = {
  get_masters(): Masters {
    return MASTERS;
  },
  health_check(): HealthStatus {
    // モックでは常に「未確認相当」を返さず、UI確認用に稼働状態を返す。
    return {
      ollamaRunning: true,
      modelInstalled: true,
      modelLoaded: true,
      model: "gemma3:1b",
      checkedAt: DEMO_TIMESTAMP,
    };
  },
  warm_up_model(): void {
    // モックでは即ロード済み扱いのため何もしない。
  },
  generate_reply(input: GenerateReplyInput): GeneratedReply {
    return demoDraft(input);
  },
  refine_reply(_parentReplyId: number, _preset: RefineCode): GeneratedReply {
    return demoDraft({ body: "", policyCode: "accept", toneCode: "standard" });
  },
  list_history(): HistoryItem[] {
    return history;
  },
  check_daily_reset(): boolean {
    return false;
  },
};
