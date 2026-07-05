//! 設定（環境変数から読む。ハードコード禁止方針に従い既定値もここへ集約）。
//! グローバル変数禁止のため、生成した Settings は Tauri の管理状態に載せて配る。

/// 実行環境（環境判定: 開発ではセッション自動初期化などで確認ステップを短絡）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    Development,
    Production,
}

impl AppEnv {
    pub fn from_env() -> Self {
        match std::env::var("APP_ENV").as_deref() {
            Ok("production") => AppEnv::Production,
            _ => AppEnv::Development,
        }
    }

    pub fn is_development(self) -> bool {
        matches!(self, AppEnv::Development)
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub env: AppEnv,
    pub ollama_host: String,
    pub default_model: String,
    pub generation_timeout_ms: u64,
    /// 入力本文の長さ制約（§1.4 F1 / §1.5）。
    pub min_chars: usize,
    pub max_chars: usize,
}

impl Settings {
    pub fn from_env() -> Self {
        Settings {
            env: AppEnv::from_env(),
            ollama_host: env_or("OLLAMA_HOST", "http://127.0.0.1:11434"),
            default_model: env_or("OLLAMA_DEFAULT_MODEL", "gemma3:4b"),
            generation_timeout_ms: env_or("GENERATION_TIMEOUT_MS", "30000")
                .parse()
                .unwrap_or(30_000),
            min_chars: 10,
            max_chars: 8_000,
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
