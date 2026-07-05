//! フロント（src/types.ts）と一致するデータ契約。JSキーは camelCase。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct MasterItem {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendedModel {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub is_default: bool,
    pub min_ram_gb: Option<i64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Masters {
    pub policies: Vec<MasterItem>,
    pub tones: Vec<MasterItem>,
    pub categories: Vec<MasterItem>,
    pub refine_presets: Vec<MasterItem>,
    pub models: Vec<RecommendedModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MailContext {
    pub category: String,
    pub requests: Vec<String>,
    pub deadline: Option<String>,
    pub sender_sentiment: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateReplyInput {
    pub body: String,
    pub policy_code: String,
    pub tone_code: String,
    #[serde(default)]
    pub extra: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedReply {
    pub reply_id: i64,
    pub mail_id: i64,
    pub body: String,
    pub structure_valid: bool,
    pub context: MailContext,
    pub policy_code: String,
    pub tone_code: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    pub ollama_running: bool,
    pub model_installed: bool,
    pub model: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub reply_id: i64,
    pub body: String,
    pub policy_code: String,
    pub tone_code: String,
    pub structure_valid: bool,
    pub created_at: String,
}
