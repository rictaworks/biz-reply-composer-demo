# biz-reply-composer-demo

文脈に合わせてビジネスメールの返信文ドラフトを生成する**デスクトップAIデモ**。
受信メール本文を貼り付けると、用件・要求事項・期日・相手のトーンを解析し、選択した返信方針とトーンに沿った返信文を生成する。

- **技術:** Rust + Tauri / React + TypeScript / SQLite / ローカルLLM `ollama`
- **デモ制約:** 外部API・APIキー・認証なし。ネットワーク越し呼び出しなし。端末ローカル完結。
- **時刻:** JST（自動リセットは JST 03:00 基準） / **エンコード:** UTF-8
- **詳細仕様:** [`biz-reply-composer-demo_instruction.md`](biz-reply-composer-demo_instruction.md)

> 本アプリは**デスクトップ版**のため、Webの「ログイン画面」「ページURL」「REST API」は存在しない。
> 下記はそれぞれ **起動・セッション** / **アプリ画面** / **Tauriコマンド** に読み替えたもの。

---

## 自動ログイン（＝起動・セッション）

- **認証は存在しない**（デモ制約）。ユーザー登録・パスワード・Googleログインなし。
- 起動時に端末ローカルで **セッションID（UUID）** を自動生成し、以降の全データのオーナーキーとする。
- 開発環境では環境判定によりセッションを自動初期化する（テスト容易化）。
- 起動手順（開発 / ローカルPC・実機ウィンドウ確認）:

  ```bash
  # 前提: ollama を起動し、推奨モデル（既定 Gemma 3 4B）を導入済みであること
  ollama serve
  ollama pull gemma3:4b
  # アプリ（開発モード）
  npm install
  npm run tauri dev
  ```

- 検証だけを行う（Codespaces / GUIなしでも可）:

  ```bash
  npm run typecheck && npm test && npm run build      # フロント: 型・vitest・ビルド
  cd src-tauri && cargo test -p app_core              # Rustコア（Tauri非依存）: ユニットテスト
  ```

  > ※ Tauri本体のフルビルド（`cargo build` 全体 / `npm run tauri build`）はCodespacesでは行わない（重い）。
  > ロジックは `app_core` クレートで検証し、TauriビルドはローカルPC・GitHub Actionsで実施する。

  > 環境分担: **Codespaces** = コード記述・フロント確認・Rustチェック・ビルド／
  > **ローカルPC** = Tauri ウィンドウの実機確認／
  > **GitHub Actions** = Windows/macOS/Linux の配布用ビルド（`.github/workflows/release.yml`）。

---

## ページ一覧（＝アプリ画面一覧）

Webページ/URLは持たない。以下はアプリ内画面（ハッシュルート）。**骨組み実装済み**（`src/pages/`）。

| 画面名 | ルート（想定） | 概要 |
|---|---|---|
| メイン（作成） | `/` | 受信メール貼付・返信方針/トーン選択・返信文生成・結果表示（F1〜F7） |
| 履歴 | `/history` | 同一セッション内の生成履歴の閲覧・コピー（F8） |
| ヘルスチェック | `/health` | ollama 稼働・モデル導入状況の確認（F9） |
| 設定 | `/settings` | 推奨モデル選択・言語（i18n）切替 |

---

## API一覧（＝Tauriコマンド一覧）

REST エンドポイントは持たない。Rustコア（Tauri）が公開する **`invoke` コマンド**。**骨組み実装済み**（`src-tauri/src/commands.rs`）。

| タイトル | コマンド（エンドポイント相当） | 概要 |
|---|---|---|
| 返信文生成 | `generate_reply(body, policy, tone, extra)` | 中核関数。入力検証→ヘルスチェック→言語検出/圧縮→文脈抽出→生成→4部構成検証→マスキング保存（§1.5） |
| 微調整再生成 | `refine_reply(parent_reply_id, preset)` | 「もっと丁寧に／短く／柔らかく／具体的に」で再生成（F7） |
| ヘルスチェック | `health_check()` | ollama 稼働・推奨モデル導入の確認（F9・フォールバックなし） |
| 履歴取得 | `list_history(session_id)` | セッション内の生成履歴一覧（F8） |
| マスタ取得 | `get_masters()` | 返信方針4/トーン3/カテゴリ8/微調整4/推奨モデル3（計22件） |
| 自動リセット | `check_daily_reset()` | JST 03:00 跨ぎ検知時にトランザクションデータを全消去（F10） |

- **SPEC:** [`biz-reply-composer-demo_instruction.md`](biz-reply-composer-demo_instruction.md)（中核関数 §1.5、クラス図 §5、シーケンス図 §4）

---

## 開発ルール

開発規約・ブランチ/PR運用・TDD・セキュリティは [`CLAUDE.md`](CLAUDE.md) を参照。
