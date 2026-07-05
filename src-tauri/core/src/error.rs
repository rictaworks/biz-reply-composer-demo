//! アプリのエラー型。フォールバック禁止方針（§1.5 / 仕様 §8）に基づき、
//! ollama未起動・モデル未導入・タイムアウトはすべて明示的なエラーとして表面化する。
//! フロント（types.ts の AppErrorCode）と code 文字列を一致させる。

use serde::ser::{Serialize, SerializeStruct, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("本文が空です")]
    EmptyInput,
    #[error("本文が10文字未満です")]
    TooShort,
    #[error("本文が長すぎます")]
    TooLong,
    #[error("ollamaが起動していません")]
    OllamaDown,
    #[error("推奨モデルが未導入です: {0}")]
    ModelMissing(String),
    #[error("生成がタイムアウトしました")]
    Timeout,
    #[error("返信文の4部構成に欠落があります")]
    StructureIncomplete,
    #[error("データベースエラー: {0}")]
    Database(String),
    #[error("エラー: {0}")]
    Generic(String),
}

impl AppError {
    /// フロントの i18n error.* キーと一致する安定コード。
    pub fn code(&self) -> &'static str {
        match self {
            AppError::EmptyInput => "empty_input",
            AppError::TooShort => "too_short",
            AppError::TooLong => "too_long",
            AppError::OllamaDown => "ollama_down",
            AppError::ModelMissing(_) => "model_missing",
            AppError::Timeout => "timeout",
            AppError::StructureIncomplete => "structure_incomplete",
            AppError::Database(_) | AppError::Generic(_) => "generic",
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

// フロントへは { code, detail } で送る（握り潰さず詳細を保持）。
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("AppError", 2)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("detail", &self.to_string())?;
        st.end()
    }
}

pub type AppResult<T> = Result<T, AppError>;
