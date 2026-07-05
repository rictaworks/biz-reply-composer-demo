//! 時刻ユーティリティ。時刻は JST 固定（自動リセットは JST 03:00 基準 / §1.4 F10）。

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

/// JST（UTC+9）。
pub fn jst_offset() -> FixedOffset {
    FixedOffset::east_opt(9 * 3600).expect("JST offset")
}

/// 現在時刻（JST）。
pub fn now_jst() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&jst_offset())
}

/// ISO8601（+09:00）文字列。DB格納・表示に使う。
pub fn now_jst_iso() -> String {
    now_jst().to_rfc3339()
}

/// 「JST03:00 を跨いだか」を判定する。
/// 直近リセット時刻から見て、次に来る 03:00 境界を現在時刻が越えていれば true。
pub fn crossed_daily_reset(last_reset_iso: &str) -> bool {
    let Ok(last) = DateTime::parse_from_rfc3339(last_reset_iso) else {
        // 解析不能なら安全側でリセット対象とする。
        return true;
    };
    reset_day(now_jst()) > reset_day(last.with_timezone(&jst_offset()))
}

/// リセット判定用の「日付」。03:00 を1日の境界とみなすため 3時間だけ後ろへ平行移動する。
/// （NaiveDate は Ord 実装があるため、日付の大小比較で跨ぎを判定できる。）
fn reset_day(t: DateTime<FixedOffset>) -> NaiveDate {
    (t - chrono::Duration::hours(3)).date_naive()
}
