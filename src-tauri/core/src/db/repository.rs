//! トランザクション永続化。すべて session_id をオーナーキーにし、
//! 他セッションのデータは参照・操作できない（セッション分離 / 仕様 §8）。

use crate::dto::{HistoryItem, MailContext};
use crate::error::{AppError, AppResult};
use crate::logging::PhaseTimer;
use crate::time::now_jst_iso;
use rusqlite::{params, Connection, OptionalExtension};

/// セッション行を用意（なければ作成）。
pub fn ensure_session(conn: &Connection, session_id: &str) -> AppResult<()> {
    let now = now_jst_iso();
    conn.execute(
        "INSERT OR IGNORE INTO sessions (session_id, created_at, last_reset_at)
         VALUES (?1, ?2, ?2)",
        params![session_id, now],
    )?;
    Ok(())
}

pub fn last_reset_at(conn: &Connection, session_id: &str) -> AppResult<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT last_reset_at FROM sessions WHERE session_id = ?1",
            params![session_id],
            |r| r.get(0),
        )
        .optional()?)
}

pub fn set_last_reset(conn: &Connection, session_id: &str, iso: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE sessions SET last_reset_at = ?2 WHERE session_id = ?1",
        params![session_id, iso],
    )?;
    Ok(())
}

/// F10 自動リセット: 当該セッションのトランザクションデータを全消去する。
/// （アプリ仕様のデータリセット。ファイルシステムの削除ではない。）
pub fn clear_transactions(conn: &Connection, session_id: &str) -> AppResult<()> {
    // generated_replies / generation_logs → received_mails の順で消す（FK整合）。
    conn.execute(
        "DELETE FROM generated_replies WHERE session_id = ?1",
        params![session_id],
    )?;
    conn.execute(
        "DELETE FROM generation_logs WHERE session_id = ?1",
        params![session_id],
    )?;
    conn.execute(
        "DELETE FROM received_mails WHERE session_id = ?1",
        params![session_id],
    )?;
    Ok(())
}

fn id_by_code(conn: &Connection, table: &str, id_col: &str, code: &str) -> AppResult<i64> {
    let sql = format!("SELECT {id_col} FROM {table} WHERE code = ?1");
    conn.query_row(&sql, params![code], |r| r.get(0))
        .map_err(|_| AppError::Generic(format!("未知のcode: {table}.{code}")))
}

pub fn policy_id(conn: &Connection, code: &str) -> AppResult<i64> {
    id_by_code(conn, "reply_policies", "policy_id", code)
}
pub fn tone_id(conn: &Connection, code: &str) -> AppResult<i64> {
    id_by_code(conn, "tones", "tone_id", code)
}
pub fn category_id(conn: &Connection, code: &str) -> AppResult<i64> {
    id_by_code(conn, "mail_categories", "category_id", code)
}
pub fn refine_preset_id(conn: &Connection, code: &str) -> AppResult<i64> {
    id_by_code(conn, "refine_presets", "refine_preset_id", code)
}

/// 受信メールを保存（本文はマスキング済みを渡す）。mail_id を返す。
pub fn save_mail(
    conn: &Connection,
    session_id: &str,
    masked_body: &str,
    context: &MailContext,
    detected_lang: &str,
) -> AppResult<i64> {
    let category = category_id(conn, &context.category).ok();
    let context_json = serde_json::to_string(context)
        .map_err(|e| AppError::Generic(e.to_string()))?;
    conn.execute(
        "INSERT INTO received_mails
            (session_id, category_id, masked_body, extracted_context, detected_lang, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            session_id,
            category,
            masked_body,
            context_json,
            detected_lang,
            now_jst_iso()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

#[allow(clippy::too_many_arguments)]
pub fn save_reply(
    conn: &Connection,
    session_id: &str,
    mail_id: i64,
    policy_code: &str,
    tone_code: &str,
    parent_reply_id: Option<i64>,
    refine_preset_code: Option<&str>,
    masked_reply_body: &str,
    structure_valid: bool,
) -> AppResult<i64> {
    let pid = policy_id(conn, policy_code)?;
    let tid = tone_id(conn, tone_code)?;
    let rpid = match refine_preset_code {
        Some(c) => Some(refine_preset_id(conn, c)?),
        None => None,
    };
    conn.execute(
        "INSERT INTO generated_replies
            (session_id, mail_id, policy_id, tone_id, parent_reply_id,
             refine_preset_id, masked_reply_body, structure_valid, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            session_id,
            mail_id,
            pid,
            tid,
            parent_reply_id,
            rpid,
            masked_reply_body,
            structure_valid as i64,
            now_jst_iso()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 微調整の元返信を取得（session_id スコープ厳守）。(mail_id, body, policy_code, tone_code)。
pub fn get_reply_for_refine(
    conn: &Connection,
    session_id: &str,
    reply_id: i64,
) -> AppResult<(i64, String, String, String)> {
    conn.query_row(
        "SELECT r.mail_id, r.masked_reply_body, p.code, t.code
         FROM generated_replies r
         JOIN reply_policies p ON p.policy_id = r.policy_id
         JOIN tones t ON t.tone_id = r.tone_id
         WHERE r.reply_id = ?1 AND r.session_id = ?2",
        params![reply_id, session_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    )
    .optional()?
    .ok_or_else(|| AppError::Generic("対象の返信が見つかりません".into()))
}

/// セッション内の生成履歴（新しい順）。
pub fn list_history(conn: &Connection, session_id: &str) -> AppResult<Vec<HistoryItem>> {
    let mut stmt = conn.prepare(
        "SELECT r.reply_id, r.masked_reply_body, p.code, t.code, r.structure_valid, r.created_at
         FROM generated_replies r
         JOIN reply_policies p ON p.policy_id = r.policy_id
         JOIN tones t ON t.tone_id = r.tone_id
         WHERE r.session_id = ?1
         ORDER BY r.reply_id DESC",
    )?;
    let rows = stmt.query_map(params![session_id], |r| {
        Ok(HistoryItem {
            reply_id: r.get(0)?,
            body: r.get(1)?,
            policy_code: r.get(2)?,
            tone_code: r.get(3)?,
            structure_valid: r.get::<_, i64>(4)? != 0,
            created_at: r.get(5)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

/// generation_logs へ記録（phase/result/elapsed_ms）。
pub fn insert_log(
    conn: &Connection,
    session_id: &str,
    timer: &PhaseTimer,
    result: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO generation_logs (session_id, phase, result, elapsed_ms, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            session_id,
            timer.phase().as_str(),
            result,
            timer.elapsed_ms() as i64,
            now_jst_iso()
        ],
    )?;
    Ok(())
}
