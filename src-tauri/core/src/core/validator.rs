//! 入力検証・言語検出・長文判定（クラス図 InputValidator / §1.5 手順1,3,4）。

use crate::config::Settings;
use crate::error::{AppError, AppResult};

/// 本文の入力検証（§1.5 手順1）。空・10文字未満はエラー。長すぎは圧縮対象でありエラーにしない。
pub fn validate(body: &str, settings: &Settings) -> AppResult<()> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Err(AppError::EmptyInput);
    }
    if trimmed.chars().count() < settings.min_chars {
        return Err(AppError::TooShort);
    }
    Ok(())
}

/// 8,000字超なら圧縮が必要（§1.5 手順4）。
pub fn needs_compression(body: &str, settings: &Settings) -> bool {
    body.chars().count() > settings.max_chars
}

/// 言語検出（簡易ヒューリスティック / §1.5 手順3）。返信は同一言語で生成する。
/// 文字種の出現からおおまかに判定する。詳細判定はLLM側の追従に委ねる。
pub fn detect_language(body: &str) -> String {
    let mut ja = 0usize; // ひらがな・カタカナ
    let mut han = 0usize; // 漢字（日中共通）
    let mut cyrillic = 0usize;
    let mut arabic = 0usize;
    let mut latin = 0usize;

    for ch in body.chars() {
        let c = ch as u32;
        match c {
            0x3040..=0x30FF => ja += 1,
            0x4E00..=0x9FFF => han += 1,
            0x0400..=0x04FF => cyrillic += 1,
            0x0600..=0x06FF => arabic += 1,
            0x0041..=0x007A => latin += 1,
            _ => {}
        }
    }

    if ja > 0 {
        "ja".into()
    } else if arabic > 0 {
        "ar".into()
    } else if cyrillic > 0 {
        "ru".into()
    } else if han > 0 {
        // かな無しの漢字主体は中国語とみなす。
        "zh".into()
    } else if latin > 0 {
        "en".into()
    } else {
        "en".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> Settings {
        Settings {
            env: crate::config::AppEnv::Development,
            ollama_host: "http://127.0.0.1:11434".into(),
            default_model: "gemma3:1b".into(),
            generation_timeout_ms: 30_000,
            min_chars: 10,
            max_chars: 8_000,
        }
    }

    #[test]
    fn 空入力はエラー() {
        assert!(matches!(validate("   ", &settings()), Err(AppError::EmptyInput)));
    }

    #[test]
    fn 十文字未満はエラー() {
        assert!(matches!(validate("短い文", &settings()), Err(AppError::TooShort)));
    }

    #[test]
    fn 十文字以上は通過() {
        assert!(validate("これは十文字以上の本文サンプルです", &settings()).is_ok());
    }

    #[test]
    fn 言語検出_日本語と英語() {
        assert_eq!(detect_language("お世話になっております"), "ja");
        assert_eq!(detect_language("Thank you for your email"), "en");
        assert_eq!(detect_language("Здравствуйте"), "ru");
    }

    #[test]
    fn 長文判定() {
        let long = "あ".repeat(8_001);
        assert!(needs_compression(&long, &settings()));
        assert!(!needs_compression("短い", &settings()));
    }
}
