//! JST（日本標準時）関連のユーティリティ
//!
//! このアプリケーションは日本語圏での利用を想定しているため、
//! ドメインで扱う日時（Step の開始・完了日時等）は JST に統一する。

use chrono::{DateTime, FixedOffset, TimeZone, Utc};

/// JST のオフセット（UTC+9固定。日本にサマータイムはない）
pub fn jst_offset() -> FixedOffset {
    FixedOffset::east_opt(9 * 3600).expect("JST offset (+09:00) is always valid")
}

/// 現在時刻をJSTで取得する
pub fn now_jst() -> DateTime<FixedOffset> {
    Utc::now().with_timezone(&jst_offset())
}

/// 任意のタイムゾーンの日時をJSTに変換する
pub fn to_jst<Tz: TimeZone>(dt: DateTime<Tz>) -> DateTime<FixedOffset> {
    dt.with_timezone(&jst_offset())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jst_offset_is_plus_nine_hours() {
        assert_eq!(jst_offset().local_minus_utc(), 9 * 3600);
    }

    #[test]
    fn test_now_jst_has_jst_offset() {
        assert_eq!(now_jst().offset().local_minus_utc(), 9 * 3600);
    }

    #[test]
    fn test_to_jst_preserves_instant() {
        let utc_now = Utc::now();
        let jst_now = to_jst(utc_now);

        assert_eq!(jst_now.offset().local_minus_utc(), 9 * 3600);
        assert_eq!(jst_now.naive_utc(), utc_now.naive_utc());
    }
}
