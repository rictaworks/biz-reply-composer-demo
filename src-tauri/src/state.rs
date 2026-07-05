//! アプリ状態（グローバル変数禁止のため Tauri の管理状態に載せて配布する）。

use app_core::config::Settings;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct AppState {
    /// rusqlite::Connection は Sync ではないため Mutex で保護する。
    pub db: Mutex<Connection>,
    pub settings: Settings,
    /// 端末ローカルのセッションID（全トランザクションのオーナーキー）。
    pub session_id: String,
}
