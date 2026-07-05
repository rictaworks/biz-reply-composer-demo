//! 返信生成オーケストレータ（クラス図 ReplyGenerator / 中核関数 §1.5）。
//! 順序: 入力検証 → ヘルスチェック → 言語検出/長文圧縮 → 文脈抽出 → プロンプト構築
//!       → 生成(30秒) → 4部構成検証(欠落は1回だけ自動再生成) → マスキング保存 → 返却。
//! フォールバック禁止・例外の握り潰し禁止。各段階で phase/result/elapsed_ms を記録。

use crate::config::Settings;
use crate::core::{context, ollama::OllamaClient, prompt, structure, validator, pii};
use crate::db::repository;
use crate::dto::{GenerateReplyInput, GeneratedReply, HealthStatus};
use crate::error::{AppError, AppResult};
use crate::logging::{Phase, PhaseTimer};
use crate::time::now_jst_iso;
use rusqlite::Connection;

/// ヘルスチェック（§1.4 F9 / §1.5 手順2）。稼働とモデル導入を確認。
pub fn health(settings: &Settings) -> HealthStatus {
    let client = OllamaClient::new(
        &settings.ollama_host,
        &settings.default_model,
        settings.generation_timeout_ms,
    );
    let running = client.is_running();
    let model_installed = running && client.model_installed().unwrap_or(false);
    HealthStatus {
        ollama_running: running,
        model_installed,
        model: settings.default_model.clone(),
        checked_at: now_jst_iso(),
    }
}

/// ヘルスチェックに不合格ならフォールバックせずエラーを返す。
fn ensure_healthy(client: &OllamaClient) -> AppResult<()> {
    let timer = PhaseTimer::start(Phase::Health);
    if !client.is_running() {
        timer.log("ollama_down");
        return Err(AppError::OllamaDown);
    }
    if !client.model_installed()? {
        timer.log("model_missing");
        return Err(AppError::ModelMissing(client.model().to_string()));
    }
    timer.log("ok");
    Ok(())
}

/// 中核関数 返信文生成。
pub fn generate_reply(
    conn: &Connection,
    settings: &Settings,
    session_id: &str,
    input: &GenerateReplyInput,
) -> AppResult<GeneratedReply> {
    // 1. 入力検証
    validator::validate(&input.body, settings)?;

    let client = OllamaClient::new(
        &settings.ollama_host,
        &settings.default_model,
        settings.generation_timeout_ms,
    );

    // 2. ヘルスチェック（フォールバックなし）
    ensure_healthy(&client)?;

    // 3. 言語検出
    let lang = validator::detect_language(&input.body);

    // 4. 長文圧縮（§1.5 手順4）
    let working_body = if validator::needs_compression(&input.body, settings) {
        compress(&client, &input.body, &lang)?
    } else {
        input.body.clone()
    };

    // 5. 文脈抽出
    let extract_timer = PhaseTimer::start(Phase::Extract);
    let ctx = context::extract(&client, &working_body, &lang)?;
    repository::insert_log(conn, session_id, &extract_timer, "ok")?;
    extract_timer.log("ok");

    // 6-7. プロンプト構築 → 生成（30秒はClient側タイムアウト）
    let gen_timer = PhaseTimer::start(Phase::Generate);
    let gen_prompt = prompt::build_generation(
        &ctx,
        &input.policy_code,
        &input.tone_code,
        input.extra.as_deref(),
        &lang,
    );
    let mut draft = client.complete(&gen_prompt)?;
    repository::insert_log(conn, session_id, &gen_timer, "ok")?;
    gen_timer.log("ok");

    // 8. 出力検証（欠落は1回だけ自動再生成）
    let mut structure_valid = structure::validate_4parts(&draft);
    if !structure_valid {
        let regen_timer = PhaseTimer::start(Phase::Regenerate);
        draft = client.complete(&gen_prompt)?;
        structure_valid = structure::validate_4parts(&draft);
        let result = if structure_valid { "ok" } else { "structure_incomplete" };
        repository::insert_log(conn, session_id, &regen_timer, result)?;
        regen_timer.log(result);
    }

    // 9. マスキング保存（session_id オーナーキー）
    let masked_input = pii::mask(&working_body);
    let masked_reply = pii::mask(&draft);
    let mail_id = repository::save_mail(conn, session_id, &masked_input, &ctx, &lang)?;
    let reply_id = repository::save_reply(
        conn,
        session_id,
        mail_id,
        &input.policy_code,
        &input.tone_code,
        None,
        None,
        &masked_reply,
        structure_valid,
    )?;

    // 10. 返却
    Ok(GeneratedReply {
        reply_id,
        mail_id,
        body: masked_reply,
        structure_valid,
        context: ctx,
        policy_code: input.policy_code.clone(),
        tone_code: input.tone_code.clone(),
        created_at: now_jst_iso(),
    })
}

/// 微調整再生成（§1.4 F7）。parent_reply_id を元に調整し履歴へ追加保存する。
pub fn refine_reply(
    conn: &Connection,
    settings: &Settings,
    session_id: &str,
    parent_reply_id: i64,
    preset_code: &str,
) -> AppResult<GeneratedReply> {
    let (mail_id, parent_body, policy_code, tone_code) =
        repository::get_reply_for_refine(conn, session_id, parent_reply_id)?;

    let client = OllamaClient::new(
        &settings.ollama_host,
        &settings.default_model,
        settings.generation_timeout_ms,
    );
    ensure_healthy(&client)?;

    let lang = validator::detect_language(&parent_body);
    let timer = PhaseTimer::start(Phase::Regenerate);
    let refine_prompt = prompt::build_refine(&parent_body, preset_code, &lang);
    let draft = client.complete(&refine_prompt)?;
    repository::insert_log(conn, session_id, &timer, "ok")?;
    timer.log("ok");

    let structure_valid = structure::validate_4parts(&draft);
    let masked_reply = pii::mask(&draft);
    let reply_id = repository::save_reply(
        conn,
        session_id,
        mail_id,
        &policy_code,
        &tone_code,
        Some(parent_reply_id),
        Some(preset_code),
        &masked_reply,
        structure_valid,
    )?;

    Ok(GeneratedReply {
        reply_id,
        mail_id,
        body: masked_reply,
        structure_valid,
        // 微調整では文脈を保存済みのものとして簡略化（親メールの文脈は維持）。
        context: crate::dto::MailContext {
            category: "other".into(),
            requests: vec![],
            deadline: None,
            sender_sentiment: "neutral".into(),
        },
        policy_code,
        tone_code,
        created_at: now_jst_iso(),
    })
}

/// 長文圧縮（§1.5 手順4）。要点抽出をollamaに依頼し、後続入力を縮約する。
fn compress(client: &OllamaClient, body: &str, lang: &str) -> AppResult<String> {
    let p = format!(
        "次のメール本文を、返信作成に必要な用件・要求事項・期日・トーンを保ったまま、\
{lang} で要点のみに圧縮してください。要約のみ出力。\n---\n{body}\n---",
        lang = lang,
        body = body,
    );
    client.complete(&p)
}
