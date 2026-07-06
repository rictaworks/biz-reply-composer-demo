//! biz-reply-composer-demo Rustコア（Tauri）。
//! デスクトップデモ: 外部API・認証なし。ローカルollama + SQLite で完結。
//! ロジックは Tauri非依存の `app_core` クレートに置き、本クレートは配線に徹する。

pub mod commands;
pub mod state;

use app_core::config::Settings;
use app_core::core::session;
use app_core::db;
use crate::state::AppState;
use std::sync::Mutex;
use tauri::Manager;

/// アプリのDBファイルパスを解決する（app_data_dir 配下）。
fn resolve_db_path(app: &tauri::App) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join("biz-reply.db"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let settings = Settings::from_env();

            let db_path = resolve_db_path(app)?;
            let conn = db::open(&db_path)?;

            // 開発環境ではセッションを自動初期化（環境判定 / テスト容易化）。
            let session_id = session::new_session_id();
            session::ensure_session(&conn, &session_id)?;
            // 起動時にJST03:00跨ぎの自動リセット判定（F10）。
            let _ = session::check_daily_reset(&conn, &session_id);

            app.manage(AppState {
                db: Mutex::new(conn),
                settings,
                session_id,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_masters,
            commands::health_check,
            commands::warm_up_model,
            commands::generate_reply,
            commands::refine_reply,
            commands::list_history,
            commands::check_daily_reset,
        ])
        .run(tauri::generate_context!())
        .expect("Tauriアプリの起動に失敗しました");
}
