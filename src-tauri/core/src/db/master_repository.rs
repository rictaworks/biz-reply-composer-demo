//! マスタ取得（返信方針4/トーン3/カテゴリ8/微調整4/推奨モデル3＝22件）。

use crate::dto::{Masters, MasterItem, RecommendedModel};
use crate::error::AppResult;
use rusqlite::Connection;

fn load_items(conn: &Connection, table: &str, id_col: &str) -> AppResult<Vec<MasterItem>> {
    // table/id_col は内部固定値のみ（外部入力を混ぜない＝SQLインジェクション回避）。
    let sql = format!(
        "SELECT {id_col}, code, name FROM {table} ORDER BY sort_order, {id_col}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| {
        Ok(MasterItem {
            id: r.get(0)?,
            code: r.get(1)?,
            name: r.get(2)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

pub fn load_masters(conn: &Connection) -> AppResult<Masters> {
    let policies = load_items(conn, "reply_policies", "policy_id")?;
    let tones = load_items(conn, "tones", "tone_id")?;
    let categories = load_items(conn, "mail_categories", "category_id")?;
    let refine_presets = load_items(conn, "refine_presets", "refine_preset_id")?;

    let mut stmt = conn.prepare(
        "SELECT model_id, code, name, is_default, min_ram_gb, note
         FROM recommended_models ORDER BY sort_order, model_id",
    )?;
    let models = stmt
        .query_map([], |r| {
            Ok(RecommendedModel {
                id: r.get(0)?,
                code: r.get(1)?,
                name: r.get(2)?,
                is_default: r.get::<_, i64>(3)? != 0,
                min_ram_gb: r.get(4)?,
                note: r.get(5)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(Masters {
        policies,
        tones,
        categories,
        refine_presets,
        models,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[test]
    fn 各マスタの件数が仕様どおり() {
        let conn = open_in_memory().unwrap();
        let m = load_masters(&conn).unwrap();
        assert_eq!(m.policies.len(), 4);
        assert_eq!(m.tones.len(), 3);
        assert_eq!(m.categories.len(), 8);
        assert_eq!(m.refine_presets.len(), 4);
        assert_eq!(m.models.len(), 3);
        // 既定モデルは Gemma 3 4B（§1.3）。
        let default = m.models.iter().find(|x| x.is_default).unwrap();
        assert_eq!(default.code, "gemma3:4b");
    }
}
