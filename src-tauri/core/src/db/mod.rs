//! SQLite 接続とマイグレーション。bundled SQLite を使うためシステム依存なし。

pub mod master_repository;
pub mod repository;

use crate::error::AppResult;
use rusqlite::Connection;

const MIGRATION_SCHEMA: &str = include_str!("../../../migrations/0001_init_schema.sql");
const MIGRATION_SEED: &str = include_str!("../../../migrations/0002_seed_masters.sql");

/// マイグレーション（スキーマ＋マスタseed）を適用する。冪等（IF NOT EXISTS / INSERT OR IGNORE）。
pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(MIGRATION_SCHEMA)?;
    conn.execute_batch(MIGRATION_SEED)?;
    Ok(())
}

/// ファイルDBを開いてマイグレーション適用。
pub fn open(path: &std::path::Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// インメモリDB（テスト用）。
#[cfg(test)]
pub fn open_in_memory() -> AppResult<Connection> {
    let conn = Connection::open_in_memory()?;
    run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn マイグレーションでマスタが合計22件になる() {
        let conn = open_in_memory().expect("migrate");
        let count: i64 = conn
            .query_row(
                "SELECT (SELECT count(*) FROM reply_policies)
                       + (SELECT count(*) FROM tones)
                       + (SELECT count(*) FROM mail_categories)
                       + (SELECT count(*) FROM refine_presets)
                       + (SELECT count(*) FROM recommended_models)",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 22, "マスタ合計は22件（§1.8）");
    }

    #[test]
    fn マイグレーションは冪等_2回適用しても件数不変() {
        let conn = open_in_memory().expect("migrate");
        run_migrations(&conn).expect("re-run");
        let policies: i64 = conn
            .query_row("SELECT count(*) FROM reply_policies", [], |r| r.get(0))
            .unwrap();
        assert_eq!(policies, 4);
    }
}
