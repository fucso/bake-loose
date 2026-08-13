//! ParameterContent のバリアント一致検証

use crate::domain::models::parameter::{ParameterContent, ParameterValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    VariantMismatch,
}

/// 同じ ParameterValue バリアント（Text/Quantity）かどうかを判定する（内部の値は問わない）
fn same_value_variant(a: &ParameterValue, b: &ParameterValue) -> bool {
    matches!(
        (a, b),
        (ParameterValue::Text { .. }, ParameterValue::Text { .. })
            | (
                ParameterValue::Quantity { .. },
                ParameterValue::Quantity { .. }
            )
    )
}

/// 同じ ParameterContent バリアントかどうかを判定する（内部の値は問わない）
///
/// KeyValue については、内部の ParameterValue（Text/Quantity）も同じバリアントであることを要求する。
fn same_variant(a: &ParameterContent, b: &ParameterContent) -> bool {
    match (a, b) {
        (
            ParameterContent::KeyValue { value: a_value, .. },
            ParameterContent::KeyValue { value: b_value, .. },
        ) => same_value_variant(a_value, b_value),
        (ParameterContent::Duration { .. }, ParameterContent::Duration { .. }) => true,
        (ParameterContent::TimeMarker { .. }, ParameterContent::TimeMarker { .. }) => true,
        (ParameterContent::Text { .. }, ParameterContent::Text { .. }) => true,
        _ => false,
    }
}

/// 既存の ParameterContent と新しい ParameterContent が同じバリアントであることを検証する
pub fn require_same_variant(
    existing: &ParameterContent,
    new: &ParameterContent,
) -> Result<(), Error> {
    if same_variant(existing, new) {
        Ok(())
    } else {
        Err(Error::VariantMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{DurationUnit, DurationValue, ParameterValue};

    #[test]
    fn test_require_same_variant_ok_for_matching_key_value_quantity() {
        let a = ParameterContent::KeyValue {
            key: "強力粉".to_string(),
            value: ParameterValue::Quantity {
                amount: 300.0,
                unit: "g".to_string(),
            },
        };
        let b = ParameterContent::KeyValue {
            key: "薄力粉".to_string(),
            value: ParameterValue::Quantity {
                amount: 50.0,
                unit: "g".to_string(),
            },
        };

        assert_eq!(require_same_variant(&a, &b), Ok(()));
    }

    #[test]
    fn test_require_same_variant_ok_for_matching_key_value_text() {
        let a = ParameterContent::KeyValue {
            key: "発酵場所".to_string(),
            value: ParameterValue::Text {
                value: "冷蔵庫".to_string(),
            },
        };
        let b = ParameterContent::KeyValue {
            key: "発酵場所".to_string(),
            value: ParameterValue::Text {
                value: "常温".to_string(),
            },
        };

        assert_eq!(require_same_variant(&a, &b), Ok(()));
    }

    #[test]
    fn test_require_same_variant_err_for_key_value_quantity_and_text() {
        let a = ParameterContent::KeyValue {
            key: "強力粉".to_string(),
            value: ParameterValue::Quantity {
                amount: 300.0,
                unit: "g".to_string(),
            },
        };
        let b = ParameterContent::KeyValue {
            key: "薄力粉".to_string(),
            value: ParameterValue::Text {
                value: "適量".to_string(),
            },
        };

        assert_eq!(require_same_variant(&a, &b), Err(Error::VariantMismatch));
    }

    #[test]
    fn test_require_same_variant_ok_for_matching_text() {
        let a = ParameterContent::Text {
            value: "打ち粉を追加".to_string(),
        };
        let b = ParameterContent::Text {
            value: "打ち粉を多めに".to_string(),
        };

        assert_eq!(require_same_variant(&a, &b), Ok(()));
    }

    #[test]
    fn test_require_same_variant_err_for_text_and_duration() {
        let a = ParameterContent::Text {
            value: "打ち粉を追加".to_string(),
        };
        let b = ParameterContent::Duration {
            duration: DurationValue::new(90.0, DurationUnit::Minute),
            note: "一次発酵".to_string(),
        };

        assert_eq!(require_same_variant(&a, &b), Err(Error::VariantMismatch));
    }

    #[test]
    fn test_require_same_variant_err_for_duration_and_time_marker() {
        let a = ParameterContent::Duration {
            duration: DurationValue::new(90.0, DurationUnit::Minute),
            note: "一次発酵".to_string(),
        };
        let b = ParameterContent::TimeMarker {
            at: DurationValue::new(30.0, DurationUnit::Minute),
            note: "焼成開始から".to_string(),
        };

        assert_eq!(require_same_variant(&a, &b), Err(Error::VariantMismatch));
    }
}
