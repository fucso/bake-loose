//! ParameterContent のバリアント一致検証

use crate::domain::models::parameter::ParameterContent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    VariantMismatch,
}

/// 同じ ParameterContent バリアントかどうかを判定する（内部の値は問わない）
fn same_variant(a: &ParameterContent, b: &ParameterContent) -> bool {
    matches!(
        (a, b),
        (
            ParameterContent::KeyValue { .. },
            ParameterContent::KeyValue { .. }
        ) | (
            ParameterContent::Duration { .. },
            ParameterContent::Duration { .. }
        ) | (
            ParameterContent::TimeMarker { .. },
            ParameterContent::TimeMarker { .. }
        ) | (ParameterContent::Text { .. }, ParameterContent::Text { .. })
    )
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
    fn test_require_same_variant_ok_for_matching_key_value() {
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

        assert_eq!(require_same_variant(&a, &b), Ok(()));
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
