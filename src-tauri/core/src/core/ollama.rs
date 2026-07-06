//! ローカルollamaクライアント（クラス図 OllamaClient / §1.5 手順2,7）。
//! デモ制約: 127.0.0.1 のローカルHTTPのみ。外部API・APIキー・ネット越し呼び出しなし。
//! フォールバック禁止: 未起動・モデル未導入・タイムアウトはすべて明示的なエラーにする。

use crate::error::{AppError, AppResult};
use std::time::Duration;

pub struct OllamaClient {
    host: String,
    model: String,
    agent: ureq::Agent,
}

impl OllamaClient {
    pub fn new(host: &str, model: &str, timeout_ms: u64) -> Self {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(30))
            .timeout(Duration::from_millis(timeout_ms))
            .build();
        OllamaClient {
            host: host.trim_end_matches('/').to_string(),
            model: model.to_string(),
            agent,
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// ollama が稼働しているか（/api/version）。
    pub fn is_running(&self) -> bool {
        self.agent
            .get(&format!("{}/api/version", self.host))
            .call()
            .is_ok()
    }

    /// 指定モデルが導入済みか（/api/tags のnameを前方一致で確認）。
    pub fn model_installed(&self) -> AppResult<bool> {
        let resp = self
            .agent
            .get(&format!("{}/api/tags", self.host))
            .call()
            .map_err(map_transport)?;
        let json: serde_json::Value = resp.into_json().map_err(|e| AppError::Generic(e.to_string()))?;
        let installed = json
            .get("models")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter().any(|m| {
                    m.get("name")
                        .and_then(|n| n.as_str())
                        .map(|name| name == self.model || name.starts_with(&format!("{}:", self.model)) || name.split(':').next() == Some(&self.model))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        Ok(installed)
    }

    /// プロンプトを送って生成テキストを得る（stream=false / タイムアウトは Agent 設定）。
    pub fn complete(&self, prompt: &str) -> AppResult<String> {
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
        });
        let resp = self
            .agent
            .post(&format!("{}/api/generate", self.host))
            .send_json(body)
            .map_err(map_transport)?;
        let json: serde_json::Value = resp.into_json().map_err(|e| AppError::Generic(e.to_string()))?;
        json.get("response")
            .and_then(|r| r.as_str())
            .map(|s| s.trim().to_string())
            .ok_or_else(|| AppError::Generic("ollama応答にresponseがありません".into()))
    }
}

/// ureq の転送エラーを AppError へ写像する（フォールバックしない）。
fn map_transport(err: ureq::Error) -> AppError {
    match err {
        ureq::Error::Status(code, _) => AppError::Generic(format!("ollama HTTP {code}")),
        ureq::Error::Transport(t) => {
            if is_timeout_error(&t) {
                AppError::Timeout
            } else {
                AppError::OllamaDown
            }
        }
    }
}

/// タイムアウト判定（ロケール非依存）。
/// 注意: Windowsの日本語ロケール環境では、OSレベルのソケットタイムアウトエラー
/// （os error 10060 / WSAETIMEDOUT）のメッセージが日本語で表示されるため、
/// 英語文字列（"timed out" 等）の部分一致では検出できない。
/// std::io::ErrorKind::TimedOut という言語非依存の列挙型比較で判定する。
fn is_timeout_error(err: &(dyn std::error::Error + 'static)) -> bool {
    let mut cur: Option<&(dyn std::error::Error + 'static)> = Some(err);
    while let Some(e) = cur {
        if let Some(io_err) = e.downcast_ref::<std::io::Error>() {
            if io_err.kind() == std::io::ErrorKind::TimedOut {
                return true;
            }
        }
        cur = e.source();
    }
    false
}
