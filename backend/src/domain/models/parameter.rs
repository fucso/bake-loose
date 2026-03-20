//! Parameter ドメインモデル

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::models::step::StepId;

/// パラメーターID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ParameterId(pub Uuid);

impl ParameterId {
    /// 新しいパラメーターIDを生成する
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ParameterId {
    fn default() -> Self {
        Self::new()
    }
}

/// 時間の単位
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DurationUnit {
    Day,
    Hour,
    Minute,
    Second,
}

/// 数値と単位を持つ時間量
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DurationValue {
    pub value: f64,
    pub unit: DurationUnit,
}

impl DurationValue {
    pub fn new(value: f64, unit: DurationUnit) -> Self {
        Self { value, unit }
    }
}

/// パラメーターの値（key-value の value 部分）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParameterValue {
    Text { value: String },
    Quantity { amount: f64, unit: String },
}

/// パラメーターの内容（型付きバリアント）
///
/// # JSON 構造例
///
/// ## KeyValue (Text)
/// ```json
/// { "type": "key_value", "key": "発酵場所", "value": { "type": "text", "value": "冷蔵庫" } }
/// ```
///
/// ## KeyValue (Quantity)
/// ```json
/// { "type": "key_value", "key": "強力粉", "value": { "type": "quantity", "amount": 300, "unit": "g" } }
/// ```
///
/// ## Duration
/// ```json
/// { "type": "duration", "duration": { "value": 90, "unit": "minute" }, "note": "一次発酵" }
/// ```
///
/// ## TimeMarker
/// ```json
/// { "type": "time_marker", "at": { "value": 30, "unit": "minute" }, "note": "焼成開始から" }
/// ```
///
/// ## Text
/// ```json
/// { "type": "text", "value": "生地がべたつく場合は打ち粉を追加" }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParameterContent {
    /// キーと値のペア（例: 強力粉: 300g）
    KeyValue { key: String, value: ParameterValue },
    /// 経過時間（例: 発酵時間 90分）
    Duration {
        duration: DurationValue,
        note: String,
    },
    /// 時間マーカー（例: 焼成開始から30分後）
    TimeMarker { at: DurationValue, note: String },
    /// 自由記述テキスト
    Text { value: String },
}

/// パラメーター（ステップに紐づく記録要素）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    id: ParameterId,
    step_id: StepId,
    content: ParameterContent,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Parameter {
    /// 新しいパラメーターを作成する（ID は自動生成）
    pub fn new(step_id: StepId, content: ParameterContent) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: ParameterId::new(),
            step_id,
            content,
            created_at: now,
            updated_at: now,
        }
    }

    /// 生データからパラメーターを構築する
    pub fn from_raw(
        id: ParameterId,
        step_id: StepId,
        content: ParameterContent,
        created_at: chrono::DateTime<chrono::Utc>,
        updated_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id,
            step_id,
            content,
            created_at,
            updated_at,
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

    pub fn created_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_content_key_value_with_quantity() {
        let content = ParameterContent::KeyValue {
            key: "強力粉".to_string(),
            value: ParameterValue::Quantity {
                amount: 300.0,
                unit: "g".to_string(),
            },
        };
        match content {
            ParameterContent::KeyValue { key, value } => {
                assert_eq!(key, "強力粉");
                match value {
                    ParameterValue::Quantity { amount, unit } => {
                        assert_eq!(amount, 300.0);
                        assert_eq!(unit, "g");
                    }
                    _ => panic!("expected Quantity"),
                }
            }
            _ => panic!("expected KeyValue"),
        }
    }

    #[test]
    fn test_parameter_content_duration_with_note() {
        let content = ParameterContent::Duration {
            duration: DurationValue::new(90.0, DurationUnit::Minute),
            note: "一次発酵".to_string(),
        };
        match content {
            ParameterContent::Duration { duration, note } => {
                assert_eq!(duration.value, 90.0);
                assert_eq!(duration.unit, DurationUnit::Minute);
                assert_eq!(note, "一次発酵");
            }
            _ => panic!("expected Duration"),
        }
    }

    #[test]
    fn test_duration_value_creation() {
        let duration = DurationValue::new(45.0, DurationUnit::Minute);
        assert_eq!(duration.value, 45.0);
        assert_eq!(duration.unit, DurationUnit::Minute);
    }
}
