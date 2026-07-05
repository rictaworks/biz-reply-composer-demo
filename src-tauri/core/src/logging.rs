//! 構造化ログ（例外の握り潰し禁止・デバッグトレース用）。
//! 各処理段階で phase / result / elapsed_ms を残す（CLAUDE.md 開発ワークフロー準拠）。
//! DBの generation_logs へも記録できるよう、値はプレーンな型で扱う。

use std::time::Instant;

#[derive(Debug, Clone, Copy)]
pub enum Phase {
    Health,
    Extract,
    Generate,
    Regenerate,
    Validate,
    Reset,
}

impl Phase {
    pub fn as_str(self) -> &'static str {
        match self {
            Phase::Health => "health",
            Phase::Extract => "extract",
            Phase::Generate => "generate",
            Phase::Regenerate => "regenerate",
            Phase::Validate => "validate",
            Phase::Reset => "reset",
        }
    }
}

/// 経過時間つきのフェーズ計測。drop 時ではなく明示的に finish して結果を確定する。
pub struct PhaseTimer {
    phase: Phase,
    start: Instant,
}

impl PhaseTimer {
    pub fn start(phase: Phase) -> Self {
        PhaseTimer {
            phase,
            start: Instant::now(),
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }

    /// 標準エラーへ1行の構造化ログを出す。result は "ok" | "timeout" | ... 。
    pub fn log(&self, result: &str) {
        eprintln!(
            "{{\"phase\":\"{}\",\"result\":\"{}\",\"elapsed_ms\":{}}}",
            self.phase.as_str(),
            result,
            self.elapsed_ms()
        );
    }
}
