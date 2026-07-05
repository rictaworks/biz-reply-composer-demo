//! Tauriコマンド（README「API一覧＝Tauriコマンド一覧」に対応）。
//! JSからは camelCase 引数で呼ぶ（Tauriが snake_case へ変換）。
//!
//! NOTE: 生成系は現状同期コマンド。生成中のUIブロックを避けるため、
//! 将来 tauri::async_runtime::spawn_blocking への移行を検討（TODO）。

use app_core::core::{generator, session};
use app_core::db::{master_repository, repository};
use app_core::dto::{GenerateReplyInput, GeneratedReply, HealthStatus, HistoryItem, Masters};
use app_core::error::{AppError, AppResult};
use crate::state::AppState;
use rusqlite::Connection;
use std::sync::MutexGuard;
use tauri::State;

fn lock<'a>(state: &'a State<AppState>) -> AppResult<MutexGuard<'a, Connection>> {
    state
        .db
        .lock()
        .map_err(|e| AppError::Generic(format!("DBロック失敗: {e}")))
}

#[tauri::command]
pub fn get_masters(state: State<AppState>) -> AppResult<Masters> {
    let conn = lock(&state)?;
    master_repository::load_masters(&conn)
}

#[tauri::command]
pub fn health_check(state: State<AppState>) -> AppResult<HealthStatus> {
    Ok(generator::health(&state.settings))
}

#[tauri::command]
pub fn generate_reply(
    state: State<AppState>,
    input: GenerateReplyInput,
) -> AppResult<GeneratedReply> {
    let conn = lock(&state)?;
    generator::generate_reply(&conn, &state.settings, &state.session_id, &input)
}

#[tauri::command]
pub fn refine_reply(
    state: State<AppState>,
    parent_reply_id: i64,
    preset: String,
) -> AppResult<GeneratedReply> {
    let conn = lock(&state)?;
    generator::refine_reply(
        &conn,
        &state.settings,
        &state.session_id,
        parent_reply_id,
        &preset,
    )
}

#[tauri::command]
pub fn list_history(state: State<AppState>) -> AppResult<Vec<HistoryItem>> {
    let conn = lock(&state)?;
    repository::list_history(&conn, &state.session_id)
}

#[tauri::command]
pub fn check_daily_reset(state: State<AppState>) -> AppResult<bool> {
    let conn = lock(&state)?;
    session::check_daily_reset(&conn, &state.session_id)
}
