// Tauriコマンド呼び出しの型付きラッパ。
// 環境判定（§CLAUDE.md「環境判定を必ず実装し分岐可能にする」）:
//   - Tauri内（デスクトップ本番）: Rustコアの invoke を呼ぶ。
//   - Tauri外（ブラウザプレビュー / vitest）: 開発用モックへ切替（UI確認・テスト容易化）。
// フォールバック禁止の対象はollamaエラー時のルールベース代替生成であり、この開発モックは別物。

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import type {
  GeneratedReply,
  GenerateReplyInput,
  HealthStatus,
  HistoryItem,
  Masters,
  RefineCode,
} from "@/types";
import { mockBackend } from "./mockBackend";

/** 実行環境がTauri（デスクトップ）かどうか。 */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    return tauriInvoke<T>(command, args);
  }
  // 開発モック（Tauri外）。同期関数を Promise でラップ。
  const fn = (mockBackend as Record<string, (...a: never[]) => unknown>)[command];
  if (!fn) {
    throw new Error(`mock backend にコマンド未定義: ${command}`);
  }
  return Promise.resolve(fn(...(Object.values(args ?? {}) as never[])) as T);
}

/** Tauriコマンド一覧（README「API一覧＝Tauriコマンド一覧」に対応）。 */
export const api = {
  getMasters: () => call<Masters>("get_masters"),
  healthCheck: () => call<HealthStatus>("health_check"),
  generateReply: (input: GenerateReplyInput) =>
    call<GeneratedReply>("generate_reply", { input }),
  refineReply: (parentReplyId: number, preset: RefineCode) =>
    call<GeneratedReply>("refine_reply", { parentReplyId, preset }),
  listHistory: () => call<HistoryItem[]>("list_history"),
  checkDailyReset: () => call<boolean>("check_daily_reset"),
};
