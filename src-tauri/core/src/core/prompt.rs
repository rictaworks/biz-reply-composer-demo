//! プロンプト構築（クラス図 PromptBuilder / §1.5 手順5,6）。
//! プロンプトテンプレートはロジックのため本モジュールに集約する（散在ハードコードの回避）。
//! 4部構成の強制・固有名詞捏造の禁止・{{相手名}} {{自分の署名}} トークン固定をシステム指示に含める。

use crate::dto::MailContext;

/// 用件カテゴリの8分類（§1.8）。抽出時にこのcode集合から選ばせる。
pub const CATEGORY_CODES: [&str; 8] = [
    "request",
    "reminder",
    "apology",
    "scheduling",
    "inquiry",
    "gratitude",
    "complaint",
    "other",
];

/// 文脈抽出プロンプト（JSON構造で返させる / §1.5 手順5）。
pub fn build_extraction(body: &str, lang: &str) -> String {
    format!(
        "あなたはビジネスメール解析器です。次の受信メール本文から文脈を抽出し、\
指定のJSONのみを出力してください（前後に説明文を付けない）。\n\
言語: {lang}\n\
カテゴリは次のcodeから1つ選ぶ: {categories}\n\
出力JSONスキーマ:\n\
{{\"category\":\"<code>\",\"requests\":[\"<要求事項>\"],\"deadline\":<\"YYYY-MM-DD\"またはnull>,\"senderSentiment\":\"<相手の感情トーン>\"}}\n\
--- 受信メール本文 ---\n{body}\n--- ここまで ---",
        lang = lang,
        categories = CATEGORY_CODES.join(", "),
        body = body,
    )
}

fn policy_instruction(policy_code: &str) -> &'static str {
    match policy_code {
        "accept" => "相手の依頼・提案を承諾する立場で書く。",
        "decline" => "相手の依頼・提案を丁重に辞退する立場で書く。代替や理由に触れてよい。",
        "hold" => "即答を避け、保留・検討中である旨を伝える立場で書く。",
        "question" => "返信の前に必要な追加質問を礼儀正しく尋ねる立場で書く。",
        _ => "相手に配慮した中立的な立場で書く。",
    }
}

fn tone_instruction(tone_code: &str) -> &'static str {
    match tone_code {
        "formal" => "最も丁寧でフォーマルな敬語表現を用いる。",
        "casual" => "親しみやすくカジュアルだが失礼にならない表現を用いる。",
        _ => "標準的なビジネス敬語（過度に硬くない）を用いる。",
    }
}

/// 返信生成プロンプト（4部構成を強制 / §1.5 手順6）。
pub fn build_generation(
    ctx: &MailContext,
    policy_code: &str,
    tone_code: &str,
    extra: Option<&str>,
    lang: &str,
) -> String {
    let extra_line = match extra {
        Some(e) if !e.trim().is_empty() => format!("追加指示: {e}\n"),
        _ => String::new(),
    };
    format!(
        "あなたは熟練のビジネスメール作成者です。以下の制約を厳守して返信文ドラフトのみを出力してください。\n\
【言語】{lang} で書く（受信メールと同一言語）。\n\
【構成】必ず4部構成にする: (1)宛名 (2)挨拶 (3)本文 (4)結び。各部を改行で分ける。\n\
【宛名・署名】宛名は必ず「{{{{相手名}}}} 様」、署名は必ず「{{{{自分の署名}}}}」というトークンをそのまま使う。実在の氏名・会社名・固有名詞を捏造しない。\n\
【出力形式】前置きの挨拶・確認応答・説明文（「承知しました」「以下がドラフトです」等）を一切付けない。Markdownのコードフェンス（```）や見出しも付けない。返信文ドラフトの本文だけをそのまま出力する。\n\
【方針】{policy}\n\
【トーン】{tone}\n\
{extra}\
【文脈】カテゴリ={category} / 要求事項={requests} / 期日={deadline} / 相手の感情={sentiment}\n\
返信文ドラフト:",
        lang = lang,
        policy = policy_instruction(policy_code),
        tone = tone_instruction(tone_code),
        extra = extra_line,
        category = ctx.category,
        requests = ctx.requests.join(" / "),
        deadline = ctx.deadline.as_deref().unwrap_or("なし"),
        sentiment = ctx.sender_sentiment,
    )
}

fn refine_instruction(preset_code: &str) -> &'static str {
    match preset_code {
        "politer" => "より丁寧で改まった表現に調整する。",
        "shorter" => "意味を保ったまま、より短く簡潔にする。",
        "softer" => "より柔らかく、角の立たない言い回しにする。",
        "concrete" => "曖昧さを減らし、より具体的な記述にする。",
        _ => "自然な範囲で微調整する。",
    }
}

/// 微調整プロンプト（クラス図 build_refine / §1.4 F7）。4部構成・トークンは維持。
pub fn build_refine(parent_body: &str, preset_code: &str, lang: &str) -> String {
    format!(
        "次の返信文ドラフトを、意図と4部構成、および {{{{相手名}}}} / {{{{自分の署名}}}} トークンを保ったまま調整してください。\n\
【言語】{lang}\n\
【調整方針】{instruction}\n\
【出力形式】前置きの説明文やMarkdownのコードフェンス（```）を付けず、調整後の返信文ドラフトの本文だけを出力する。\n\
--- 元のドラフト ---\n{parent}\n--- ここまで ---",
        lang = lang,
        instruction = refine_instruction(preset_code),
        parent = parent_body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> MailContext {
        MailContext {
            category: "scheduling".into(),
            requests: vec!["日程調整".into()],
            deadline: Some("2026-07-10".into()),
            sender_sentiment: "neutral".into(),
        }
    }

    #[test]
    fn 生成プロンプトに4部構成とトークン指示が含まれる() {
        let p = build_generation(&ctx(), "accept", "formal", None, "ja");
        assert!(p.contains("4部構成"));
        assert!(p.contains("{{相手名}}"));
        assert!(p.contains("{{自分の署名}}"));
        assert!(p.contains("捏造しない"));
        assert!(p.contains("コードフェンス"));
    }

    #[test]
    fn 抽出プロンプトに8カテゴリが含まれる() {
        let p = build_extraction("本文", "ja");
        for code in CATEGORY_CODES {
            assert!(p.contains(code));
        }
    }
}
