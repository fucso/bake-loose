//! TimeUnit and Duration ドメインモデル

use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;

/// 時間の単位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
}

impl TimeUnit {
    /// DB/JSON 用の文字列表現
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeUnit::Second => "s",
            TimeUnit::Minute => "min",
            TimeUnit::Hour => "h",
        }
    }

    /// 文字列からの変換
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "s" => Some(TimeUnit::Second),
            "min" => Some(TimeUnit::Minute),
            "h" => Some(TimeUnit::Hour),
            _ => None,
        }
    }
}

/// 期間（値と表示単位を持つ）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Duration {
    value: StdDuration,
    display_unit: TimeUnit,
}

impl Duration {
    /// 秒数を指定して Duration を作成
    pub fn seconds(value: u64) -> Self {
        Self {
            value: StdDuration::from_secs(value),
            display_unit: TimeUnit::Second,
        }
    }

    /// 分を指定して Duration を作成
    pub fn minutes(value: u64) -> Self {
        Self {
            value: StdDuration::from_secs(value * 60),
            display_unit: TimeUnit::Minute,
        }
    }

    /// 時間を指定して Duration を作成
    pub fn hours(value: u64) -> Self {
        Self {
            value: StdDuration::from_secs(value * 3600),
            display_unit: TimeUnit::Hour,
        }
    }

    /// 内部の StdDuration を取得
    pub fn as_std(&self) -> &StdDuration {
        &self.value
    }

    /// 表示単位を取得
    pub fn display_unit(&self) -> TimeUnit {
        self.display_unit
    }

    /// 表示単位での値を取得
    pub fn display_value(&self) -> u64 {
        let secs = self.value.as_secs();
        match self.display_unit {
            TimeUnit::Second => secs,
            TimeUnit::Minute => secs / 60,
            TimeUnit::Hour => secs / 3600,
        }
    }

    /// 秒数での値を取得（比較・計算用）
    pub fn as_secs(&self) -> u64 {
        self.value.as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_unit_as_str_roundtrip() {
        let all_units = [TimeUnit::Second, TimeUnit::Minute, TimeUnit::Hour];
        for unit in all_units.iter() {
            let s = unit.as_str();
            let recovered_unit = TimeUnit::from_str(s);
            assert_eq!(Some(*unit), recovered_unit);
        }
    }

    #[test]
    fn test_duration_seconds() {
        let d = Duration::seconds(120);
        assert_eq!(d.as_secs(), 120);
        assert_eq!(d.display_unit(), TimeUnit::Second);
        assert_eq!(d.display_value(), 120);
    }

    #[test]
    fn test_duration_minutes() {
        let d = Duration::minutes(2);
        assert_eq!(d.as_secs(), 120);
        assert_eq!(d.display_unit(), TimeUnit::Minute);
        assert_eq!(d.display_value(), 2);
    }

    #[test]
    fn test_duration_hours() {
        let d = Duration::hours(1);
        assert_eq!(d.as_secs(), 3600);
        assert_eq!(d.display_unit(), TimeUnit::Hour);
        assert_eq!(d.display_value(), 1);
    }

    #[test]
    fn test_duration_display_value() {
        let d_min = Duration::minutes(90); // 5400s
        assert_eq!(d_min.display_value(), 90);

        let d_hr = Duration::hours(2); // 7200s
        assert_eq!(d_hr.display_value(), 2);
    }

    #[test]
    fn test_duration_comparison() {
        let d1 = Duration::minutes(1);
        let d2 = Duration::seconds(60);
        let d3 = Duration::seconds(61);

        assert_eq!(d1.as_secs(), d2.as_secs());
        assert!(d2.as_secs() < d3.as_secs());
    }
}
