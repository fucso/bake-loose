//! Unit ドメインモデル

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// 計測単位（時間系を除く）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    // 質量
    Gram,
    Kilogram,

    // 温度
    Celsius,

    // 体積
    Milliliter,
    Liter,

    // 割合
    Percent,
}

impl Unit {
    /// DB/JSON 用の文字列表現
    pub fn as_str(&self) -> &'static str {
        match self {
            Unit::Gram => "g",
            Unit::Kilogram => "kg",
            Unit::Celsius => "°C",
            Unit::Milliliter => "ml",
            Unit::Liter => "l",
            Unit::Percent => "%",
        }
    }
}

impl FromStr for Unit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "g" => Ok(Unit::Gram),
            "kg" => Ok(Unit::Kilogram),
            "°C" => Ok(Unit::Celsius),
            "ml" => Ok(Unit::Milliliter),
            "l" => Ok(Unit::Liter),
            "%" => Ok(Unit::Percent),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_as_str_roundtrip() {
        let all_units = [
            Unit::Gram,
            Unit::Kilogram,
            Unit::Celsius,
            Unit::Milliliter,
            Unit::Liter,
            Unit::Percent,
        ];

        for unit in all_units.iter() {
            let s = unit.as_str();
            let recovered_unit: Unit = s.parse().unwrap();
            assert_eq!(*unit, recovered_unit);
        }
    }

    #[test]
    fn test_unit_from_str_invalid() {
        let result: Result<Unit, _> = "invalid".parse();
        assert!(result.is_err());
    }
}
