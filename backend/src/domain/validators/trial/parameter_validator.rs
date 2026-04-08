//! Parameter の内容検証

use crate::domain::models::parameter::ParameterContent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    EmptyKey,
    EmptyText,
    EmptyNote,
    InvalidDurationValue,
}

/// ParameterContent の検証
pub fn validate_content(content: &ParameterContent) -> Result<(), Error> {
    match content {
        ParameterContent::KeyValue { key, .. } => {
            if key.trim().is_empty() {
                return Err(Error::EmptyKey);
            }
        }
        ParameterContent::Duration { duration, note } => {
            if duration.value < 0.0 {
                return Err(Error::InvalidDurationValue);
            }
            if note.trim().is_empty() {
                return Err(Error::EmptyNote);
            }
        }
        ParameterContent::TimeMarker { at, note } => {
            if at.value < 0.0 {
                return Err(Error::InvalidDurationValue);
            }
            if note.trim().is_empty() {
                return Err(Error::EmptyNote);
            }
        }
        ParameterContent::Text { value } => {
            if value.trim().is_empty() {
                return Err(Error::EmptyText);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::{DurationUnit, DurationValue, ParameterValue};

    mod key_value {
        use super::*;

        #[test]
        fn test_valid_key_value_passes() {
            let content = ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "g".to_string(),
                },
            };
            assert!(validate_content(&content).is_ok());
        }

        #[test]
        fn test_empty_key_returns_error() {
            let content = ParameterContent::KeyValue {
                key: "".to_string(),
                value: ParameterValue::Text {
                    value: "test".to_string(),
                },
            };
            assert_eq!(validate_content(&content), Err(Error::EmptyKey));
        }

        #[test]
        fn test_whitespace_only_key_returns_error() {
            let content = ParameterContent::KeyValue {
                key: "   ".to_string(),
                value: ParameterValue::Text {
                    value: "test".to_string(),
                },
            };
            assert_eq!(validate_content(&content), Err(Error::EmptyKey));
        }
    }

    mod duration {
        use super::*;

        #[test]
        fn test_valid_duration_passes() {
            let content = ParameterContent::Duration {
                duration: DurationValue::new(90.0, DurationUnit::Minute),
                note: "一次発酵".to_string(),
            };
            assert!(validate_content(&content).is_ok());
        }

        #[test]
        fn test_zero_duration_passes() {
            let content = ParameterContent::Duration {
                duration: DurationValue::new(0.0, DurationUnit::Minute),
                note: "開始時".to_string(),
            };
            assert!(validate_content(&content).is_ok());
        }

        #[test]
        fn test_negative_duration_returns_error() {
            let content = ParameterContent::Duration {
                duration: DurationValue::new(-1.0, DurationUnit::Minute),
                note: "一次発酵".to_string(),
            };
            assert_eq!(validate_content(&content), Err(Error::InvalidDurationValue));
        }

        #[test]
        fn test_empty_note_returns_error() {
            let content = ParameterContent::Duration {
                duration: DurationValue::new(90.0, DurationUnit::Minute),
                note: "".to_string(),
            };
            assert_eq!(validate_content(&content), Err(Error::EmptyNote));
        }

        #[test]
        fn test_whitespace_only_note_returns_error() {
            let content = ParameterContent::Duration {
                duration: DurationValue::new(90.0, DurationUnit::Minute),
                note: "   ".to_string(),
            };
            assert_eq!(validate_content(&content), Err(Error::EmptyNote));
        }
    }

    mod time_marker {
        use super::*;

        #[test]
        fn test_valid_time_marker_passes() {
            let content = ParameterContent::TimeMarker {
                at: DurationValue::new(30.0, DurationUnit::Minute),
                note: "焼成開始から".to_string(),
            };
            assert!(validate_content(&content).is_ok());
        }

        #[test]
        fn test_zero_time_marker_passes() {
            let content = ParameterContent::TimeMarker {
                at: DurationValue::new(0.0, DurationUnit::Minute),
                note: "開始時".to_string(),
            };
            assert!(validate_content(&content).is_ok());
        }

        #[test]
        fn test_negative_time_marker_returns_error() {
            let content = ParameterContent::TimeMarker {
                at: DurationValue::new(-1.0, DurationUnit::Minute),
                note: "焼成開始から".to_string(),
            };
            assert_eq!(validate_content(&content), Err(Error::InvalidDurationValue));
        }

        #[test]
        fn test_empty_note_returns_error() {
            let content = ParameterContent::TimeMarker {
                at: DurationValue::new(30.0, DurationUnit::Minute),
                note: "".to_string(),
            };
            assert_eq!(validate_content(&content), Err(Error::EmptyNote));
        }

        #[test]
        fn test_whitespace_only_note_returns_error() {
            let content = ParameterContent::TimeMarker {
                at: DurationValue::new(30.0, DurationUnit::Minute),
                note: "   ".to_string(),
            };
            assert_eq!(validate_content(&content), Err(Error::EmptyNote));
        }
    }

    mod text {
        use super::*;

        #[test]
        fn test_valid_text_passes() {
            let content = ParameterContent::Text {
                value: "生地がべたつく場合は打ち粉を追加".to_string(),
            };
            assert!(validate_content(&content).is_ok());
        }

        #[test]
        fn test_empty_text_returns_error() {
            let content = ParameterContent::Text {
                value: "".to_string(),
            };
            assert_eq!(validate_content(&content), Err(Error::EmptyText));
        }

        #[test]
        fn test_whitespace_only_text_returns_error() {
            let content = ParameterContent::Text {
                value: "   ".to_string(),
            };
            assert_eq!(validate_content(&content), Err(Error::EmptyText));
        }
    }
}
