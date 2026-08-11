//! 現在時刻の取得を抽象化する Clock
//!
//! ドメインアクションの「未指定時は現在時刻を採用する」ロジックをテストで
//! 検証可能にするため、現在時刻の取得を trait を通して抽象化する。
//! 本番コードでは [`SystemClock`] を使用する。

use crate::domain::timezone::JstDateTime;

/// 現在時刻を取得するトレイト
pub trait Clock {
    fn now(&self) -> JstDateTime;
}

/// 実時刻を返す本番用の Clock 実装
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> JstDateTime {
        JstDateTime::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock_now_has_jst_offset() {
        let now = SystemClock.now();
        assert_eq!(now.into_fixed_offset().offset().local_minus_utc(), 9 * 3600);
    }
}
