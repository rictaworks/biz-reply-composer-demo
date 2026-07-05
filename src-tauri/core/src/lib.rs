//! biz-reply-composer-demo コアロジック（Tauri非依存）。
//! クラス図§5 の各クラスに対応。Tauriを含まないため Codespaces でも
//! `cargo test -p app_core` を軽量に実行できる（重いTauriビルドはローカル/CIへ委譲）。

pub mod config;
pub mod core;
pub mod db;
pub mod dto;
pub mod error;
pub mod logging;
pub mod time;
