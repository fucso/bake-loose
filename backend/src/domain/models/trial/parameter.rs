//! Parameter ドメインモデル

use super::step::StepId;
use crate::domain::models::utils::{Duration, Unit};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Parameter の一意識別子
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParameterId(pub Uuid);

impl ParameterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ParameterId {
    fn default() -> Self {
        Self::new()
    }
}

/// パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    id: ParameterId,
    step_id: StepId,
    content: ParameterContent,
}

impl Parameter {
    pub fn new(step_id: StepId, content: ParameterContent) -> Self {
        Self {
            id: ParameterId::new(),
            step_id,
            content,
        }
    }

    pub fn from_raw(id: ParameterId, step_id: StepId, content: ParameterContent) -> Self {
        Self {
            id,
            step_id,
            content,
        }
    }

    pub fn id(&self) -> &ParameterId {
        &self.id
    }

    pub fn step_id(&self) -> &StepId {
        &self.step_id
    }

    pub fn content(&self) -> &ParameterContent {
        &self.content
    }
}

/// パラメーターの内容
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterContent {
    KeyValue(KeyValueParameter),
    Text(TextParameter),
    DurationRange(DurationRangeParameter),
    TimePoint(TimePointParameter),
}

/// Key-Value パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyValueParameter {
    pub key: String,
    pub value: ParameterValue,
}

/// パラメーター値
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    Text(String),
    Quantity { amount: i32, unit: Unit },
}

/// テキストパラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextParameter {
    pub value: String,
}

/// 期間パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DurationRangeParameter {
    pub duration: Duration,
    pub note: String,
}

/// 時点パラメーター
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimePointParameter {
    pub elapsed: Duration,
    pub note: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_new() {
        let step_id = StepId::new();
        let content = ParameterContent::Text(TextParameter {
            value: "Test".to_string(),
        });
        let param = Parameter::new(step_id.clone(), content.clone());

        assert_eq!(*param.step_id(), step_id);
        assert_eq!(param.content(), &content);
    }

    #[test]
    fn test_parameter_from_raw() {
        let id = ParameterId::new();
        let step_id = StepId::new();
        let content = ParameterContent::Text(TextParameter {
            value: "Test".to_string(),
        });
        let param = Parameter::from_raw(id.clone(), step_id.clone(), content.clone());

        assert_eq!(*param.id(), id);
        assert_eq!(*param.step_id(), step_id);
        assert_eq!(param.content(), &content);
    }

    #[test]
    fn test_key_value_with_quantity() {
        let param = KeyValueParameter {
            key: "Flour".to_string(),
            value: ParameterValue::Quantity {
                amount: 500,
                unit: Unit::Gram,
            },
        };
        assert_eq!(param.key, "Flour");
        match param.value {
            ParameterValue::Quantity { amount, unit } => {
                assert_eq!(amount, 500);
                assert_eq!(unit, Unit::Gram);
            }
            _ => panic!("Incorrect ParameterValue type"),
        }
    }

    #[test]
    fn test_key_value_with_text() {
        let param = KeyValueParameter {
            key: "Note".to_string(),
            value: ParameterValue::Text("Use warm water".to_string()),
        };
        assert_eq!(param.key, "Note");
        match param.value {
            ParameterValue::Text(text) => assert_eq!(text, "Use warm water"),
            _ => panic!("Incorrect ParameterValue type"),
        }
    }

    #[test]
    fn test_text_parameter() {
        let param = TextParameter {
            value: "Let it rise for 1 hour".to_string(),
        };
        assert_eq!(param.value, "Let it rise for 1 hour");
    }

    #[test]
    fn test_duration_range_parameter() {
        let duration = Duration::minutes(30);
        let param = DurationRangeParameter {
            duration: duration.clone(),
            note: "Kneading time".to_string(),
        };
        assert_eq!(param.duration, duration);
        assert_eq!(param.note, "Kneading time".to_string());
    }

    #[test]
    fn test_time_point_parameter() {
        let elapsed = Duration::hours(1);
        let param = TimePointParameter {
            elapsed: elapsed.clone(),
            note: "First proofing complete".to_string(),
        };
        assert_eq!(param.elapsed, elapsed);
        assert_eq!(param.note, "First proofing complete");
    }
}
