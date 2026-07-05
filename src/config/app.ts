// アプリ定数（文字列リテラルのハードコード禁止方針に従い、表示文言はi18n・DBへ分離）。
// ここに置くのは「表示されない技術定数」のみ。

/** 入力本文の長さ制約（§1.4 F1 / §1.5 手順1・4）。 */
export const INPUT_LIMITS = {
  minChars: 10,
  maxChars: 8000,
} as const;

/** 生成タイムアウト（§1.5 手順7）。UIの想定表示用。実際の打ち切りはRust側。 */
export const GENERATION_TIMEOUT_MS = 30000;

/** 微調整プリセットのcode順（§1.8）。ラベルはi18n refine.* で解決。 */
export const REFINE_ORDER = [
  "politer",
  "shorter",
  "softer",
  "concrete",
] as const;

/** 画面ルート（Webの概念ではなくアプリ内画面／README「ページ一覧」に対応）。 */
export const ROUTES = {
  main: "/",
  history: "/history",
  health: "/health",
  settings: "/settings",
} as const;
