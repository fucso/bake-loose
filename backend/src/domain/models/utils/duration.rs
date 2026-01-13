//! Duration ドメインモデル

use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::time::Duration as StdDuration;

/// 時間の単位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum TimeUnit {
    Second,
    Minute,
    Hour,
}

impl TimeUnit {
    /// DB/JSON 用の文字列表現
    #[cfg(test)]
    fn as_str(&self) -> &'static str {
        match self {
            TimeUnit::Second => "s",
            TimeUnit::Minute => "min",
            TimeUnit::Hour => "h",
        }
    }
}

impl FromStr for TimeUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "s" => Ok(TimeUnit::Second),
            "min" => Ok(TimeUnit::Minute),
            "h" => Ok(TimeUnit::Hour),
            _ => Err(()),
        }
    }
}

/// 期間（値と表示単位を持つ）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Duration {
    std_duration: StdDuration,
    display_unit: TimeUnit,
}

impl Duration {
    /// 秒数を指定して Duration を作成
    pub fn seconds(value: u64) -> Self {
        Self {
            std_duration: StdDuration::from_secs(value),
            display_unit: TimeUnit::Second,
        }
    }

    /// 分を指定して Duration を作成
    pub fn minutes(value: u64) -> Self {
        Self {
            std_duration: StdDuration::from_secs(value * 60),
            display_unit: TimeUnit::Minute,
        }
    }

    /// 時間を指定して Duration を作成
    pub fn hours(value: u64) -> Self {
        Self {
            std_duration: StdDuration::from_secs(value * 3600),
            display_unit: TimeUnit::Hour,
        }
    }

    /// 表示単位を取得
    #[cfg(test)]
    fn unit(&self) -> TimeUnit {
        self.display_unit
    }

    /// 表示単位での値を取得
    pub fn value(&self) -> u64 {
        let secs = self.std_duration.as_secs();
        match self.display_unit {
            TimeUnit::Second => secs,
            TimeUnit::Minute => secs / 60,
            TimeUnit::Hour => secs / 3600,
        }
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
            let recovered_unit: TimeUnit = s.parse().unwrap();
            assert_eq!(*unit, recovered_unit);
        }
    }

    #[test]
    fn test_time_unit_from_str_invalid() {
        let result: Result<TimeUnit, _> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_duration_seconds() {
        let d = Duration::seconds(120);
        assert_eq!(d.std_duration, StdDuration::from_secs(120));
        assert_eq!(d.unit(), TimeUnit::Second);
        assert_eq!(d.value(), 120);
    }

    #[test]
    fn test_duration_minutes() {
        let d = Duration::minutes(2);
        assert_eq!(d.std_duration, StdDuration::from_secs(120));
        assert_eq!(d.unit(), TimeUnit::Minute);
        assert_eq!(d.value(), 2);
    }

    #[test]
    fn test_duration_hours() {
        let d = Duration::hours(1);
        assert_eq!(d.std_duration, StdDuration::from_secs(3600));
        assert_eq!(d.unit(), TimeUnit::Hour);
        assert_eq!(d.value(), 1);
    }

    #[test]
    fn test_duration_value() {
        let d_min = Duration::minutes(90);
        assert_eq!(d_min.value(), 90);

        let d_hr = Duration::hours(2);
        assert_eq!(d_hr.value(), 2);
    }
}
