//! JST（日本標準時）関連のドメインモデル
//!
//! このアプリケーションは日本語圏での利用を想定しているため、
//! ドメインで扱う日時（Step の開始・完了日時等）は JST に統一する。
//!
//! `chrono::DateTime<FixedOffset>` を直接扱うのはこのモジュールに限定し、
//! 他のモジュールは `JstDateTime` を経由して日時を扱う。

use chrono::{DateTime, FixedOffset, Utc};
use serde::{Deserialize, Serialize};

/// JST のオフセット（UTC+9固定。日本にサマータイムはない）
fn jst_offset() -> FixedOffset {
    FixedOffset::east_opt(9 * 3600).expect("JST offset (+09:00) is always valid")
}

/// JST（日本標準時）で表現された日時
///
/// アプリケーション内の日時はすべてこの型を通して扱う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct JstDateTime(DateTime<FixedOffset>);

impl JstDateTime {
    /// 現在時刻を取得する
    pub fn now() -> Self {
        Self::from_utc(Utc::now())
    }

    /// UTCの日時からJSTへ変換する
    pub fn from_utc(dt: DateTime<Utc>) -> Self {
        Self(dt.with_timezone(&jst_offset()))
    }

    /// 任意のオフセット付き日時をJSTへ正規化して変換する
    ///
    /// GraphQL入力など、外部から受け取ったオフセット付き日時をJSTへ正規化する境界で使用する。
    pub fn from_fixed_offset(dt: DateTime<FixedOffset>) -> Self {
        Self(dt.with_timezone(&jst_offset()))
    }

    /// `chrono::DateTime<FixedOffset>` として取り出す
    ///
    /// GraphQLスカラーへの変換・DB永続化のバインドなど、外部境界専用。
    pub fn into_fixed_offset(self) -> DateTime<FixedOffset> {
        self.0
    }
}

impl std::ops::Sub<chrono::Duration> for JstDateTime {
    type Output = JstDateTime;

    fn sub(self, rhs: chrono::Duration) -> JstDateTime {
        JstDateTime(self.0 - rhs)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_now_has_jst_offset() {
        assert_eq!(JstDateTime::now().0.offset().local_minus_utc(), 9 * 3600);
    }

    #[test]
    fn test_from_utc_preserves_instant_and_has_jst_offset() {
        let utc_now = Utc::now();
        let jst = JstDateTime::from_utc(utc_now);

        assert_eq!(jst.0.offset().local_minus_utc(), 9 * 3600);
        assert_eq!(jst.0.naive_utc(), utc_now.naive_utc());
    }

    #[test]
    fn test_from_fixed_offset_normalizes_to_jst() {
        let other_offset = FixedOffset::east_opt(0).unwrap();
        let dt = other_offset.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();

        let jst = JstDateTime::from_fixed_offset(dt);

        assert_eq!(jst.0.offset().local_minus_utc(), 9 * 3600);
        assert_eq!(jst.0.naive_utc(), dt.naive_utc());
    }

    #[test]
    fn test_into_fixed_offset_roundtrips() {
        let jst = JstDateTime::now();
        let fixed = jst.into_fixed_offset();
        assert_eq!(JstDateTime(fixed), jst);
    }

    #[test]
    fn test_sub_duration() {
        let now = JstDateTime::now();
        let one_hour_ago = now - chrono::Duration::hours(1);
        assert!(one_hour_ago.into_fixed_offset() < now.into_fixed_offset());
    }
}
