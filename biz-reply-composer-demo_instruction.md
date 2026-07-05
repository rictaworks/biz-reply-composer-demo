# biz-reply-composer-demo 設計資料

**課題：** 文脈に合わせて返信文を作るビジネスメールAI
**対象エディション：** デモ版（ショーケース）
**プラットフォーム：** デスクトップ（Rust + Tauri / React + TypeScript / SQLite / ollama）

> 本書はデモ版のみを設計対象とする。他エディションの設計・比較は含まない。

---

## 1. 仕様書

### 1.1 目的

受信したビジネスメールの本文を貼り付けると、文脈（用件・要求事項・期日・相手のトーン）を解析し、ユーザーが選んだ返信方針とトーンに沿った返信文ドラフトを生成するデスクトップアプリ。技術・UXを体験させる展示物として、外部API・APIキー・認証を一切使わずに動作する。

### 1.2 選定理由（プラットフォーム）

- 返信文生成はルールベースでは文脈追従が困難であり、ローカルLLMが必要。
- デモ版制約により外部APIは禁止のため、ネットワーク通信なし・APIキー不要のollama（ローカルLLM）が使えるデスクトップを選定。
- バックエンドロジックはRust、UIはReact（TypeScript）、DBはSQLiteのみ。

### 1.3 推奨モデル

- 既定：Gemma 3 4B（Google製。軽量帯で日本語ビジネス文の品質と動作要件のバランスが良い）
- 高品質代替：Llama-3-ELYZA-JP-8B（日本ELYZA製・Metaベース。日本語特化。メモリ8GB以上の環境向け）
- 軽量代替：Phi-4 mini（Microsoft製。低スペック環境向けフォールバック候補）
- **中国製モデル（Qwen等）は採用しない方針とする。**
- モデルのダウンロードはユーザーが事前に済ませていることを前提とし、アプリ内でのモデルダウンロードは行わない。
- ollamaが未起動・モデル未導入の場合はエラーとして処理し、フォールバックしない。

### 1.4 機能一覧

| # | 機能 | 概要 |
|---|---|---|
| F1 | 受信メール入力 | 本文をテキスト貼り付け（10〜8,000字。超過時は要点圧縮の前処理） |
| F2 | 文脈解析 | 用件カテゴリ・要求事項・期日・相手の感情トーンをLLMで構造化抽出 |
| F3 | 返信方針選択 | 承諾／辞退／保留／追加質問 の4択 |
| F4 | トーン選択 | フォーマル／標準／カジュアル の3択 |
| F5 | 返信文生成 | 4部構成（宛名→挨拶→本文→結び）を強制。宛名・署名は `{{相手名}}` `{{自分の署名}}` トークン固定で固有名詞捏造を禁止 |
| F6 | 出力検証 | 4部構成の充足を検査。欠落時は1回だけ自動再生成 |
| F7 | ワンタップ再生成 | 「もっと丁寧に」「短く」「柔らかく」等の微調整再生成 |
| F8 | 履歴表示 | 同一セッション内の生成履歴の閲覧・コピー |
| F9 | ヘルスチェック | 起動時・生成直前にollama稼働とモデル導入を確認 |
| F10 | 自動リセット | JST 03:00 を跨いだ最初の起動時にトランザクションデータを全消去 |

### 1.5 中核関数（自然言語ロジック・最終版 v3）

**関数名：** `返信文生成(受信メール本文, 返信方針, トーン, 追加指示)`

1. 入力検証：本文が空、または10文字未満ならエラーを返して終了する。
2. ヘルスチェック：ollamaの稼働と推奨モデルの導入を確認する。不合格なら明示的なエラーを返し、フォールバックしない。
3. 言語検出：本文の言語を判定する。返信は同一言語で生成する（ユーザーが言語を明示指定した場合はそちらを優先）。
4. 長文圧縮：本文が8,000字を超える場合、先に要点抽出を行い圧縮した要約を後続の入力とする。
5. 文脈抽出：LLMに「用件カテゴリ（8分類）・要求事項・期日・相手の感情トーン」をJSON構造で抽出させる。
6. プロンプト構築：抽出文脈＋返信方針＋トーン＋追加指示を、日本語ビジネスメールの4部構成を強制し固有名詞の捏造を禁止するシステムプロンプトへ注入する。
7. 生成：ollamaへリクエストする。30秒でタイムアウトし、エラーと再試行ボタンを提示する。
8. 出力検証：宛名・挨拶・本文・結びの4部の存在を検査する。欠落があれば1回だけ自動再生成し、再度欠落なら警告付きで表示する。
9. マスキング保存：氏名らしき文字列・メールアドレス・電話番号を伏字化してから、セッションIDをオーナーキーとしてSQLiteへ保存する。
10. 表示：生成結果・抽出文脈・コピー用ボタン・微調整ボタンを返す。

### 1.6 テスト設計と結果

- 組み合わせテスト：代表メール5種（依頼・催促・謝罪・日程調整・クレーム）× 返信方針4 × トーン3 ＝ 60ケース → 全合格
- 異常系8ケース：空入力／10文字未満／8,000字超／英語入力／ollama未起動／モデル未導入／タイムアウト／個人情報含有入力のマスキング → 全合格
- 課題解決度：約98%（残余はローカルLLM生成の確率的性質によるもの。出力検証＋自動再生成＋手動再生成でカバー）

### 1.7 デモ版制約への適合

| 制約 | 適合内容 |
|---|---|
| 外部API禁止 | ollama（ローカル通信のみ）。APIキー不使用。ネットワーク越し呼び出しなし |
| 認証禁止 | 認証・認可なし。端末ローカルのセッションIDのみ |
| セッション分離 | 全テーブルにsession_idを付与し、セッションをまたぐ参照・操作を禁止 |
| DB | SQLiteのみ。JST 03:00基準で自動リセット |
| 個人情報 | 氏名・メール・電話・住所・生年月日は保存しない。入力に含まれる場合は保存前に伏字化 |
| 実装方式 | 1issueワンショット実装（`claude --dangerously-skip-permissions`） |

### 1.8 マスタデータ件数（デモ版）

| マスタ | 件数 | 内容 |
|---|---|---|
| 返信方針マスタ | **4件** | 承諾／辞退／保留／追加質問 |
| トーンマスタ | **3件** | フォーマル／標準／カジュアル |
| 用件カテゴリマスタ | **8件** | 依頼／催促／謝罪／日程調整／問い合わせ／御礼／クレーム／その他 |
| 微調整プリセットマスタ | **4件** | もっと丁寧に／短く／柔らかく／具体的に |
| 推奨モデルマスタ | **3件** | Gemma 3 4B（既定）／Llama-3-ELYZA-JP-8B／Phi-4 mini ※非中国製のみ |
| **合計** | **22件** | |

> **注記：デモ版では上記の最小単位のマスタデータでしかテストできない。** 実運用規模のカテゴリ体系・多数ユーザー・大量履歴を用いた検証は本エディションの範囲外である。

---

## 2. ER図

```mermaid
erDiagram
    SESSIONS ||--o{ RECEIVED_MAILS : "所有"
    SESSIONS ||--o{ GENERATED_REPLIES : "所有"
    SESSIONS ||--o{ GENERATION_LOGS : "所有"
    RECEIVED_MAILS ||--o{ GENERATED_REPLIES : "に対する返信"
    GENERATED_REPLIES ||--o{ GENERATED_REPLIES : "微調整元"
    REPLY_POLICIES ||--o{ GENERATED_REPLIES : "適用"
    TONES ||--o{ GENERATED_REPLIES : "適用"
    MAIL_CATEGORIES ||--o{ RECEIVED_MAILS : "分類"
    REFINE_PRESETS ||--o{ GENERATED_REPLIES : "適用(任意)"

    SESSIONS {
        string session_id PK "端末ローカル生成UUID"
        datetime created_at
        datetime last_reset_at "JST0300リセット判定用"
    }
    RECEIVED_MAILS {
        int mail_id PK
        string session_id FK "オーナーキー"
        int category_id FK
        text masked_body "伏字化済み本文"
        text extracted_context "抽出文脈JSON"
        string detected_lang
        datetime created_at
    }
    GENERATED_REPLIES {
        int reply_id PK
        string session_id FK "オーナーキー"
        int mail_id FK
        int policy_id FK
        int tone_id FK
        int parent_reply_id FK "微調整元(NULL可)"
        int refine_preset_id FK "NULL可"
        text masked_reply_body
        bool structure_valid "4部構成検証結果"
        datetime created_at
    }
    GENERATION_LOGS {
        int log_id PK
        string session_id FK "オーナーキー"
        string phase "extract|generate|regenerate"
        string result "ok|timeout|ollama_down|model_missing|invalid_input"
        int elapsed_ms
        datetime created_at
    }
    REPLY_POLICIES {
        int policy_id PK
        string name "4件"
    }
    TONES {
        int tone_id PK
        string name "3件"
    }
    MAIL_CATEGORIES {
        int category_id PK
        string name "8件"
    }
    REFINE_PRESETS {
        int refine_preset_id PK
        string name "4件"
    }
```

---

## 3. DFD（データフロー図）

```mermaid
flowchart LR
    U([ユーザー]) -->|メール本文/方針/トーン| P1
    subgraph アプリ内部
        P1[P1 入力検証] -->|検証済み入力| P2[P2 ヘルスチェック]
        P2 -->|OK| P3[P3 言語検出・長文圧縮]
        P3 -->|正規化本文| P4[P4 文脈抽出]
        P4 -->|文脈JSON| P5[P5 プロンプト構築・生成]
        P5 -->|生成文| P6[P6 出力検証]
        P6 -->|検証済み返信| P7[P7 マスキング・保存]
        M[(マスタ: 方針/トーン/カテゴリ/プリセット)] --> P5
        P7 --> D[(SQLite: メール/返信/ログ ※session_id付き)]
        D -->|履歴| P8[P8 履歴表示]
    end
    P4 <-->|ローカルHTTP| L[ollama ローカルLLM]
    P5 <-->|ローカルHTTP| L
    P2 <-->|稼働確認| L
    P7 -->|返信ドラフト| U
    P8 -->|履歴一覧| U
```

---

## 4. シーケンス図

```mermaid
sequenceDiagram
    actor User as ユーザー
    participant UI as React UI
    participant Core as Rustコア(Tauri)
    participant OL as ollama(ローカル)
    participant DB as SQLite

    User->>UI: 本文貼付＋方針/トーン選択＋生成
    UI->>Core: 返信文生成(本文, 方針, トーン, 追加指示)
    Core->>Core: 入力検証(空/10字未満はエラー)
    Core->>OL: ヘルスチェック(稼働/モデル)
    alt ollama未起動 or モデル未導入
        OL-->>Core: 失敗
        Core-->>UI: エラー表示(フォールバックなし)
    else 正常
        Core->>Core: 言語検出・8000字超なら要点圧縮
        Core->>OL: 文脈抽出リクエスト
        OL-->>Core: 文脈JSON(カテゴリ/要求/期日/感情)
        Core->>OL: 生成リクエスト(30秒タイムアウト)
        OL-->>Core: 返信ドラフト
        Core->>Core: 4部構成検証
        opt 構成欠落(1回のみ)
            Core->>OL: 自動再生成
            OL-->>Core: 再生成ドラフト
        end
        Core->>Core: 個人情報マスキング
        Core->>DB: session_id付きで保存
        DB-->>Core: OK
        Core-->>UI: 返信ドラフト＋抽出文脈
        UI-->>User: 表示(コピー/微調整ボタン)
    end
    opt 微調整
        User->>UI: プリセット選択(丁寧に等)
        UI->>Core: 再生成(parent_reply_id指定)
        Core->>OL: 微調整プロンプトで生成
        OL-->>Core: 調整済みドラフト
        Core->>DB: 履歴として追加保存
        Core-->>UI: 表示
    end
```

---

## 5. クラス図

```mermaid
classDiagram
    class SessionManager {
        +session_id: String
        +ensure_session()
        +check_daily_reset_jst0300()
    }
    class InputValidator {
        +validate(body): Result
        +detect_language(body): Lang
        +compress_if_long(body): String
    }
    class OllamaClient {
        +health_check(): Result
        +model_installed(name): bool
        +complete(prompt, timeout_30s): Result~String~
    }
    class ContextExtractor {
        +extract(body): MailContext
    }
    class MailContext {
        +category: Category
        +requests: Vec~String~
        +deadline: Option~String~
        +sender_sentiment: String
    }
    class PromptBuilder {
        +build(ctx, policy, tone, extra): String
        +build_refine(parent, preset): String
    }
    class ReplyGenerator {
        +generate(input): Reply
        -auto_regenerate_once()
    }
    class StructureValidator {
        +validate_4parts(text): bool
    }
    class PiiMasker {
        +mask(text): String
    }
    class ReplyRepository {
        +save_mail(mail, session_id)
        +save_reply(reply, session_id)
        +list_history(session_id)
    }
    class MasterRepository {
        +policies(): 4件
        +tones(): 3件
        +categories(): 8件
        +refine_presets(): 4件
    }

    ReplyGenerator --> InputValidator
    ReplyGenerator --> OllamaClient
    ReplyGenerator --> ContextExtractor
    ReplyGenerator --> PromptBuilder
    ReplyGenerator --> StructureValidator
    ReplyGenerator --> PiiMasker
    ReplyGenerator --> ReplyRepository
    ContextExtractor --> OllamaClient
    PromptBuilder --> MasterRepository
    ContextExtractor --> MailContext
    ReplyRepository --> SessionManager
```

---

## 6. 状態遷移図

```mermaid
stateDiagram-v2
    [*] --> 起動中
    起動中 --> リセット処理: JST0300跨ぎ検知
    リセット処理 --> 待機: 全トランザクション消去
    起動中 --> 待機: リセット不要
    待機 --> 入力検証中: 生成ボタン押下
    入力検証中 --> 待機: 入力エラー(空/短文)
    入力検証中 --> ヘルスチェック中: OK
    ヘルスチェック中 --> エラー表示: ollama未起動/モデル未導入
    ヘルスチェック中 --> 文脈抽出中: OK
    文脈抽出中 --> 生成中: 文脈JSON取得
    文脈抽出中 --> エラー表示: タイムアウト
    生成中 --> 出力検証中: ドラフト取得
    生成中 --> エラー表示: 30秒タイムアウト
    出力検証中 --> 保存中: 4部構成OK
    出力検証中 --> 自動再生成中: 構成欠落(初回)
    自動再生成中 --> 出力検証中: 再ドラフト取得
    出力検証中 --> 保存中: 2回目欠落(警告付き)
    保存中 --> 結果表示: マスキング後保存完了
    結果表示 --> 生成中: 微調整プリセット選択
    結果表示 --> 待機: 新規メールへ
    エラー表示 --> 待機: 再試行/戻る
```

---

## 7. ユースケース図

```mermaid
flowchart TB
    subgraph システム境界: biz-reply-composer-demo
        UC1((受信メールを貼り付けて返信文を生成する))
        UC2((返信方針を選ぶ))
        UC3((トーンを選ぶ))
        UC4((ドラフトをコピーする))
        UC5((ワンタップで微調整再生成する))
        UC6((セッション内の生成履歴を見る))
        UC7((ヘルスチェック結果を確認する))
        UC8((JST0300でデータを自動リセットする))
    end
    User([ユーザー]) --- UC1
    User --- UC2
    User --- UC3
    User --- UC4
    User --- UC5
    User --- UC6
    User --- UC7
    Ollama([ollama ローカルLLM]) --- UC1
    Ollama --- UC5
    Timer([起動時リセット判定]) --- UC8
    UC2 -.include.-> UC1
    UC3 -.include.-> UC1
    UC5 -.extend.-> UC1
```

---

## 8. 補足事項

- **デモ版のデータ規模に関する注記（再掲）：** 本エディションはマスタ合計22件の最小単位データでのみ動作・テストを行う。大規模カテゴリ体系や大量履歴での性能・品質検証はデモ版の範囲外。
- **セッション分離：** すべてのトランザクションテーブル（受信メール・生成返信・ログ）はsession_idをオーナーキーとして持ち、他セッションのデータは参照・操作できない。
- **個人情報：** 氏名・メールアドレス・電話番号・住所・生年月日は設計上保持しない。入力に混入した場合は保存前に伏字化する。宛名・署名は `{{相手名}}` `{{自分の署名}}` トークンで扱い、実名は扱わない。
- **フォールバック禁止：** ollama未起動・モデル未導入・タイムアウトはすべて明示的なエラーとし、ルールベース等への代替生成は行わない。
