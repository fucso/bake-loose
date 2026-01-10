//! Unit ドメインモデル

use serde::{Deserialize, Serialize};

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

    /// 文字列からの変換
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "g" => Some(Unit::Gram),
            "kg" => Some(Unit::Kilogram),
            "°C" => Some(Unit::Celsius),
            "ml" => Some(Unit::Milliliter),
            "l" => Some(Unit::Liter),
            "%" => Some(Unit::Percent),
            _ => None,
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
            let recovered_unit = Unit::from_str(s);
            assert_eq!(Some(*unit), recovered_unit);
        }
    }
}
