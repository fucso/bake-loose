//! Parameter の内容検証

use crate::domain::errors::trial_error::Error;
use crate::domain::models::parameter::{DurationValue, ParameterContent, ParameterValue};

/// - Duration/TimeMarker の DurationValue は非負であること
/// - KeyValue の Quantity は unit が空文字でないこと、amount が正の値であること
pub fn validate(content: &ParameterContent) -> Result<(), Error> {
    match content {
        ParameterContent::KeyValue { value, .. } => validate_value(value),
        ParameterContent::Duration { duration, .. } => validate_duration(duration),
        ParameterContent::TimeMarker { at, .. } => validate_duration(at),
        ParameterContent::Text { .. } => Ok(()),
    }
}

/// 値が正（0 より大きい）であることを判定する
///
/// NaN との比較は常に false となるため、この判定は NaN も不正な値として弾く。
/// `!(value <= 0.0)` のように否定形で書くと NaN が正の値として通ってしまうため、
/// 必ず正の側（`value > 0.0`）で比較すること。
fn is_positive(value: f64) -> bool {
    value > 0.0
}

/// 値が非負（0 以上）であることを判定する
///
/// NaN との比較は常に false となるため、この判定は NaN も不正な値として弾く。
/// `!(value < 0.0)` のように否定形で書くと NaN が非負の値として通ってしまうため、
/// 必ず非負の側（`value >= 0.0`）で比較すること。
fn is_non_negative(value: f64) -> bool {
    value >= 0.0
}

fn validate_value(value: &ParameterValue) -> Result<(), Error> {
    match value {
        ParameterValue::Quantity { amount, unit } => {
            if unit.trim().is_empty() {
                return Err(Error::EmptyQuantityUnit);
            }
            if !is_positive(*amount) {
                return Err(Error::NonPositiveQuantityAmount);
            }
            Ok(())
        }
        ParameterValue::Text { .. } => Ok(()),
    }
}

fn validate_duration(duration: &DurationValue) -> Result<(), Error> {
    if !is_non_negative(duration.value) {
        return Err(Error::NegativeDurationValue);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::DurationUnit;

    #[test]
    fn test_is_positive() {
        // (value, expected)
        let cases = [
            (1.0, true),
            (f64::MIN_POSITIVE, true),
            (0.0, false),
            (-0.0, false),
            (-1.0, false),
            (f64::NAN, false),
        ];

        for (value, expected) in cases {
            assert_eq!(is_positive(value), expected, "value = {value}");
        }
    }

    #[test]
    fn test_is_non_negative() {
        // (value, expected)
        let cases = [
            (1.0, true),
            (0.0, true),
            (-0.0, true),
            (-1.0, false),
            (f64::NAN, false),
        ];

        for (value, expected) in cases {
            assert_eq!(is_non_negative(value), expected, "value = {value}");
        }
    }

    #[test]
    fn test_validate_key_value_text() {
        let content = ParameterContent::KeyValue {
            key: "発酵場所".to_string(),
            value: ParameterValue::Text {
                value: "冷蔵庫".to_string(),
            },
        };
        assert_eq!(validate(&content), Ok(()));
    }

    #[test]
    fn test_validate_key_value_quantity_unit() {
        // (unit, expected)
        let cases = [("g", Ok(())), ("   ", Err(Error::EmptyQuantityUnit))];

        for (unit, expected) in cases {
            let content = ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: unit.to_string(),
                },
            };
            assert_eq!(validate(&content), expected);
        }
    }

    #[test]
    fn test_validate_key_value_quantity_amount() {
        // (amount, expected)
        let cases = [
            (300.0, Ok(())),
            (0.0, Err(Error::NonPositiveQuantityAmount)),
            (-1.0, Err(Error::NonPositiveQuantityAmount)),
            (f64::NAN, Err(Error::NonPositiveQuantityAmount)),
        ];

        for (amount, expected) in cases {
            let content = ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount,
                    unit: "g".to_string(),
                },
            };
            assert_eq!(validate(&content), expected);
        }
    }

    #[test]
    fn test_validate_duration() {
        // (value, expected)
        let cases = [
            (90.0, Ok(())),
            (-1.0, Err(Error::NegativeDurationValue)),
            (f64::NAN, Err(Error::NegativeDurationValue)),
        ];

        for (value, expected) in cases {
            let content = ParameterContent::Duration {
                duration: DurationValue::new(value, DurationUnit::Minute),
                note: "一次発酵".to_string(),
            };
            assert_eq!(validate(&content), expected);
        }
    }

    #[test]
    fn test_validate_time_marker() {
        // (value, expected)
        let cases = [
            (0.0, Ok(())),
            (-5.0, Err(Error::NegativeDurationValue)),
            (f64::NAN, Err(Error::NegativeDurationValue)),
        ];

        for (value, expected) in cases {
            let content = ParameterContent::TimeMarker {
                at: DurationValue::new(value, DurationUnit::Minute),
                note: "焼成開始から".to_string(),
            };
            assert_eq!(validate(&content), expected);
        }
    }

    #[test]
    fn test_validate_text() {
        let content = ParameterContent::Text {
            value: "生地がべたつく場合は打ち粉を追加".to_string(),
        };
        assert_eq!(validate(&content), Ok(()));
    }
}
