-- biz-reply-composer-demo スキーマ（ER図 §2 準拠）
-- デモ制約: 認証なし・端末ローカルのセッションIDのみ。全トランザクションに session_id をオーナーキー付与。
-- 文字コード UTF-8 / 時刻 JST（Rust側で ISO8601+09:00 を格納）。
-- 個人情報は保存しない。入力に混入した場合は保存前に伏字化（PiiMasker）。

PRAGMA foreign_keys = ON;

-- ============================================================
-- マスタ（§1.8: 方針4 / トーン3 / カテゴリ8 / 微調整4 / 推奨モデル3 = 22件）
-- name は開発者管理用の日本語（管理画面は日本語のみ）。
-- code はUI i18n の解決キー（安定・機械可読）。表示名はフロントの locale で解決する。
-- ============================================================

CREATE TABLE IF NOT EXISTS reply_policies (
    policy_id   INTEGER PRIMARY KEY,
    code        TEXT    NOT NULL UNIQUE,
    name        TEXT    NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS tones (
    tone_id     INTEGER PRIMARY KEY,
    code        TEXT    NOT NULL UNIQUE,
    name        TEXT    NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS mail_categories (
    category_id INTEGER PRIMARY KEY,
    code        TEXT    NOT NULL UNIQUE,
    name        TEXT    NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS refine_presets (
    refine_preset_id INTEGER PRIMARY KEY,
    code             TEXT    NOT NULL UNIQUE,
    name             TEXT    NOT NULL,
    sort_order       INTEGER NOT NULL DEFAULT 0
);

-- 推奨モデルマスタ（§1.3 / §1.8）。中国製モデルは採用しない方針。
CREATE TABLE IF NOT EXISTS recommended_models (
    model_id     INTEGER PRIMARY KEY,
    code         TEXT    NOT NULL UNIQUE,   -- ollama タグ
    name         TEXT    NOT NULL,
    is_default   INTEGER NOT NULL DEFAULT 0, -- 0/1
    min_ram_gb   INTEGER,
    note         TEXT,
    sort_order   INTEGER NOT NULL DEFAULT 0
);

-- ============================================================
-- トランザクション（すべて session_id をオーナーキーに持つ）
-- ============================================================

CREATE TABLE IF NOT EXISTS sessions (
    session_id     TEXT PRIMARY KEY,          -- 端末ローカル生成UUID
    created_at     TEXT NOT NULL,             -- ISO8601 (JST)
    last_reset_at  TEXT NOT NULL              -- JST03:00 リセット判定用
);

CREATE TABLE IF NOT EXISTS received_mails (
    mail_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id       TEXT NOT NULL,
    category_id      INTEGER,
    masked_body      TEXT NOT NULL,           -- 伏字化済み本文
    extracted_context TEXT,                   -- 抽出文脈JSON
    detected_lang    TEXT,
    created_at       TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE,
    FOREIGN KEY (category_id) REFERENCES mail_categories(category_id)
);

CREATE TABLE IF NOT EXISTS generated_replies (
    reply_id         INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id       TEXT NOT NULL,
    mail_id          INTEGER NOT NULL,
    policy_id        INTEGER NOT NULL,
    tone_id          INTEGER NOT NULL,
    parent_reply_id  INTEGER,                 -- 微調整元（NULL可）
    refine_preset_id INTEGER,                 -- NULL可
    masked_reply_body TEXT NOT NULL,
    structure_valid  INTEGER NOT NULL DEFAULT 0, -- 4部構成検証結果 0/1
    created_at       TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE,
    FOREIGN KEY (mail_id) REFERENCES received_mails(mail_id) ON DELETE CASCADE,
    FOREIGN KEY (policy_id) REFERENCES reply_policies(policy_id),
    FOREIGN KEY (tone_id) REFERENCES tones(tone_id),
    FOREIGN KEY (parent_reply_id) REFERENCES generated_replies(reply_id) ON DELETE SET NULL,
    FOREIGN KEY (refine_preset_id) REFERENCES refine_presets(refine_preset_id)
);

CREATE TABLE IF NOT EXISTS generation_logs (
    log_id       INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id   TEXT NOT NULL,
    phase        TEXT NOT NULL,   -- extract | generate | regenerate | health | reset | validate
    result       TEXT NOT NULL,   -- ok | timeout | ollama_down | model_missing | invalid_input | ...
    elapsed_ms   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);

-- セッション分離のためのインデックス（他セッションの参照・操作を防ぐ検索最適化）
CREATE INDEX IF NOT EXISTS idx_received_mails_session ON received_mails(session_id);
CREATE INDEX IF NOT EXISTS idx_generated_replies_session ON generated_replies(session_id);
CREATE INDEX IF NOT EXISTS idx_generation_logs_session ON generation_logs(session_id);
