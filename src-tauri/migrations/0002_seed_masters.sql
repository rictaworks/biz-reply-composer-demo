-- マスタ seed（§1.8 合計22件）。name は日本語（開発者管理用）。code はUI i18n解決キー。
-- 冪等: INSERT OR IGNORE（code UNIQUE）。

-- 返信方針マスタ 4件（承諾／辞退／保留／追加質問）
INSERT OR IGNORE INTO reply_policies (policy_id, code, name, sort_order) VALUES
    (1, 'accept',   '承諾',     1),
    (2, 'decline',  '辞退',     2),
    (3, 'hold',     '保留',     3),
    (4, 'question', '追加質問', 4);

-- トーンマスタ 3件（フォーマル／標準／カジュアル）
INSERT OR IGNORE INTO tones (tone_id, code, name, sort_order) VALUES
    (1, 'formal',   'フォーマル', 1),
    (2, 'standard', '標準',       2),
    (3, 'casual',   'カジュアル', 3);

-- 用件カテゴリマスタ 8件（依頼／催促／謝罪／日程調整／問い合わせ／御礼／クレーム／その他）
INSERT OR IGNORE INTO mail_categories (category_id, code, name, sort_order) VALUES
    (1, 'request',    '依頼',       1),
    (2, 'reminder',   '催促',       2),
    (3, 'apology',    '謝罪',       3),
    (4, 'scheduling', '日程調整',   4),
    (5, 'inquiry',    '問い合わせ', 5),
    (6, 'gratitude',  '御礼',       6),
    (7, 'complaint',  'クレーム',   7),
    (8, 'other',      'その他',     8);

-- 微調整プリセットマスタ 4件（もっと丁寧に／短く／柔らかく／具体的に）
INSERT OR IGNORE INTO refine_presets (refine_preset_id, code, name, sort_order) VALUES
    (1, 'politer',  'もっと丁寧に', 1),
    (2, 'shorter',  '短く',         2),
    (3, 'softer',   '柔らかく',     3),
    (4, 'concrete', '具体的に',     4);

-- 推奨モデルマスタ 3件（§1.3 非中国製のみ / 既定=Gemma 3 1B）
INSERT OR IGNORE INTO recommended_models (model_id, code, name, is_default, min_ram_gb, note, sort_order) VALUES
    (1, 'gemma3:1b',           'Gemma 3 1B',            1, 2, 'Google製。実効空きメモリ800MB〜1.7GB程度のCPU推論機でも現実的に完走する（既定）。', 1),
    (2, 'llama3-elyza-jp:8b',  'Llama-3-ELYZA-JP-8B',   0, 8, 'ELYZA製（Metaベース）。日本語特化。メモリ8GB以上向け。',                    2),
    (3, 'phi4-mini',           'Phi-4 mini',            0, 4, 'Microsoft製。低スペック環境向けフォールバック候補。',                        3);
