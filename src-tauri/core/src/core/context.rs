//! 文脈抽出（クラス図 ContextExtractor / §1.5 手順5）。
//! LLMにJSONで返させ、頑健に解析する。失敗はフォールバックせずエラー化。

use crate::core::ollama::OllamaClient;
use crate::core::prompt;
use crate::dto::MailContext;
use crate::error::{AppError, AppResult};

pub fn extract(client: &OllamaClient, body: &str, lang: &str) -> AppResult<MailContext> {
    let prompt = prompt::build_extraction(body, lang);
    let raw = client.complete(&prompt)?;
    parse_context(&raw)
}

/// 応答テキストから最初のJSONオブジェクトを取り出して解析する。
fn parse_context(raw: &str) -> AppResult<MailContext> {
    let json_slice = extract_json_object(raw)
        .ok_or_else(|| AppError::Generic("文脈JSONが見つかりません".into()))?;
    let mut ctx: MailContext = serde_json::from_str(json_slice)
        .map_err(|e| AppError::Generic(format!("文脈JSON解析失敗: {e}")))?;

    // カテゴリが8分類外なら other に丸める（捏造カテゴリを持ち込まない）。
    if !prompt::CATEGORY_CODES.contains(&ctx.category.as_str()) {
        ctx.category = "other".to_string();
    }
    Ok(ctx)
}

/// 波括弧の対応を数えて最初の完全なJSONオブジェクト範囲を返す。
fn extract_json_object(text: &str) -> Option<&str> {
    let start = text.find('{')?;
    let mut depth = 0usize;
    for (i, ch) in text[start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[start..start + i + ch.len_utf8()]);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(non_snake_case)]
    fn 前後に説明が付いたJSONでも抽出できる() {
        let raw = "以下です:\n{\"category\":\"scheduling\",\"requests\":[\"日程調整\"],\"deadline\":null,\"senderSentiment\":\"neutral\"} 以上";
        let ctx = parse_context(raw).unwrap();
        assert_eq!(ctx.category, "scheduling");
        assert_eq!(ctx.requests, vec!["日程調整".to_string()]);
        assert!(ctx.deadline.is_none());
    }

    #[test]
    fn 未知カテゴリはotherに丸める() {
        let raw = "{\"category\":\"unknown_x\",\"requests\":[],\"deadline\":null,\"senderSentiment\":\"neutral\"}";
        assert_eq!(parse_context(raw).unwrap().category, "other");
    }
}
