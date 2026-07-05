//! セッション管理（クラス図 SessionManager / §1.4 F10 / §1.7 セッション分離）。
//! 端末ローカル生成のUUIDをオーナーキーにする。JST03:00 跨ぎでトランザクションを全消去。

use crate::db::repository;
use crate::error::AppResult;
use crate::logging::{Phase, PhaseTimer};
use crate::time::{crossed_daily_reset, now_jst_iso};
use rusqlite::Connection;
use uuid::Uuid;

/// 新規セッションIDを発行する（端末ローカル・UUID v4）。
pub fn new_session_id() -> String {
    Uuid::new_v4().to_string()
}

/// セッション行を用意する（§1.7）。
pub fn ensure_session(conn: &Connection, session_id: &str) -> AppResult<()> {
    repository::ensure_session(conn, session_id)
}

/// JST03:00 を跨いだ最初の起動なら当該セッションのトランザクションを全消去する（F10）。
/// 消去した場合 true を返す。
pub fn check_daily_reset(conn: &Connection, session_id: &str) -> AppResult<bool> {
    let timer = PhaseTimer::start(Phase::Reset);
    let last = repository::last_reset_at(conn, session_id)?;
    let should_reset = match last {
        Some(ref iso) => crossed_daily_reset(iso),
        None => false,
    };
    if should_reset {
        repository::clear_transactions(conn, session_id)?;
        repository::set_last_reset(conn, session_id, &now_jst_iso())?;
        repository::insert_log(conn, session_id, &timer, "ok")?;
        timer.log("ok");
    }
    Ok(should_reset)
}
