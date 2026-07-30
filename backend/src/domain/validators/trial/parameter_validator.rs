//! Parameter の内容検証

use crate::domain::models::parameter::{DurationValue, ParameterContent, ParameterValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NegativeDurationValue,
    EmptyQuantityUnit,
}

/// ParameterContent が不正な値を持たないことを検証する
///
/// - Duration/TimeMarker の DurationValue は非負であること
/// - KeyValue の Quantity は unit が空文字でないこと
pub fn validate(content: &ParameterContent) -> Result<(), Error> {
    match content {
        ParameterContent::KeyValue { value, .. } => validate_value(value),
        ParameterContent::Duration { duration, .. } => validate_duration(duration),
        ParameterContent::TimeMarker { at, .. } => validate_duration(at),
        ParameterContent::Text { .. } => Ok(()),
    }
}

fn validate_value(value: &ParameterValue) -> Result<(), Error> {
    match value {
        ParameterValue::Quantity { unit, .. } => {
            if unit.trim().is_empty() {
                return Err(Error::EmptyQuantityUnit);
            }
            Ok(())
        }
        ParameterValue::Text { .. } => Ok(()),
    }
}

fn validate_duration(duration: &DurationValue) -> Result<(), Error> {
    if duration.value < 0.0 {
        return Err(Error::NegativeDurationValue);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::DurationUnit;

    #[test]
    fn test_validate_ok_for_key_value_text() {
        let content = ParameterContent::KeyValue {
            key: "発酵場所".to_string(),
            value: ParameterValue::Text {
                value: "冷蔵庫".to_string(),
            },
        };
        assert_eq!(validate(&content), Ok(()));
    }

    #[test]
    fn test_validate_ok_for_key_value_quantity_with_unit() {
        let content = ParameterContent::KeyValue {
            key: "強力粉".to_string(),
            value: ParameterValue::Quantity {
                amount: 300.0,
                unit: "g".to_string(),
            },
        };
        assert_eq!(validate(&content), Ok(()));
    }

    #[test]
    fn test_validate_err_for_key_value_quantity_with_empty_unit() {
        let content = ParameterContent::KeyValue {
            key: "強力粉".to_string(),
            value: ParameterValue::Quantity {
                amount: 300.0,
                unit: "   ".to_string(),
            },
        };
        assert_eq!(validate(&content), Err(Error::EmptyQuantityUnit));
    }

    #[test]
    fn test_validate_ok_for_duration_with_non_negative_value() {
        let content = ParameterContent::Duration {
            duration: DurationValue::new(90.0, DurationUnit::Minute),
            note: "一次発酵".to_string(),
        };
        assert_eq!(validate(&content), Ok(()));
    }

    #[test]
    fn test_validate_err_for_duration_with_negative_value() {
        let content = ParameterContent::Duration {
            duration: DurationValue::new(-1.0, DurationUnit::Minute),
            note: "一次発酵".to_string(),
        };
        assert_eq!(validate(&content), Err(Error::NegativeDurationValue));
    }

    #[test]
    fn test_validate_ok_for_time_marker_with_non_negative_value() {
        let content = ParameterContent::TimeMarker {
            at: DurationValue::new(0.0, DurationUnit::Minute),
            note: "焼成開始から".to_string(),
        };
        assert_eq!(validate(&content), Ok(()));
    }

    #[test]
    fn test_validate_err_for_time_marker_with_negative_value() {
        let content = ParameterContent::TimeMarker {
            at: DurationValue::new(-5.0, DurationUnit::Minute),
            note: "焼成開始から".to_string(),
        };
        assert_eq!(validate(&content), Err(Error::NegativeDurationValue));
    }

    #[test]
    fn test_validate_ok_for_text() {
        let content = ParameterContent::Text {
            value: "生地がべたつく場合は打ち粉を追加".to_string(),
        };
        assert_eq!(validate(&content), Ok(()));
    }
}
