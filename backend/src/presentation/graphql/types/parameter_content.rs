//! ParameterContent GraphQL 型
//!
//! ドメインモデルの ParameterContent をラップした GraphQL Union 型を提供する。

use async_graphql::{Object, Union};

use crate::domain::models::parameter::{
    DurationValue as DomainDurationValue, ParameterContent as DomainParameterContent,
    ParameterValue as DomainParameterValue,
};

/// GraphQL 用の ParameterContent union 型
#[derive(Union)]
pub enum ParameterContent {
    KeyValue(KeyValueParameter),
    Duration(DurationParameter),
    TimeMarker(TimeMarkerParameter),
    Text(TextParameter),
}

impl From<DomainParameterContent> for ParameterContent {
    fn from(content: DomainParameterContent) -> Self {
        match content {
            DomainParameterContent::KeyValue { key, value } => {
                ParameterContent::KeyValue(KeyValueParameter {
                    key,
                    value: value.into(),
                })
            }
            DomainParameterContent::Duration { duration, note } => {
                ParameterContent::Duration(DurationParameter {
                    duration: duration.into(),
                    note,
                })
            }
            DomainParameterContent::TimeMarker { at, note } => {
                ParameterContent::TimeMarker(TimeMarkerParameter {
                    at: at.into(),
                    note,
                })
            }
            DomainParameterContent::Text { value } => {
                ParameterContent::Text(TextParameter { value })
            }
        }
    }
}

/// KeyValue パラメーター
pub struct KeyValueParameter {
    key: String,
    value: ParameterValue,
}

#[Object]
impl KeyValueParameter {
    async fn key(&self) -> &str {
        &self.key
    }

    async fn value(&self) -> &ParameterValue {
        &self.value
    }
}

/// ParameterValue union
#[derive(Union)]
pub enum ParameterValue {
    Text(TextValue),
    Quantity(QuantityValue),
}

impl From<DomainParameterValue> for ParameterValue {
    fn from(value: DomainParameterValue) -> Self {
        match value {
            DomainParameterValue::Text { value } => ParameterValue::Text(TextValue { value }),
            DomainParameterValue::Quantity { amount, unit } => {
                ParameterValue::Quantity(QuantityValue { amount, unit })
            }
        }
    }
}

/// テキスト値
pub struct TextValue {
    value: String,
}

#[Object]
impl TextValue {
    async fn value(&self) -> &str {
        &self.value
    }
}

/// 数量値
pub struct QuantityValue {
    amount: f64,
    unit: String,
}

#[Object]
impl QuantityValue {
    async fn amount(&self) -> f64 {
        self.amount
    }

    async fn unit(&self) -> &str {
        &self.unit
    }
}

/// Duration パラメーター
pub struct DurationParameter {
    duration: DurationValue,
    note: Option<String>,
}

#[Object]
impl DurationParameter {
    async fn duration(&self) -> &DurationValue {
        &self.duration
    }

    async fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }
}

/// TimeMarker パラメーター
pub struct TimeMarkerParameter {
    at: DurationValue,
    note: String,
}

#[Object]
impl TimeMarkerParameter {
    async fn at(&self) -> &DurationValue {
        &self.at
    }

    async fn note(&self) -> &str {
        &self.note
    }
}

/// Text パラメーター
pub struct TextParameter {
    value: String,
}

#[Object]
impl TextParameter {
    async fn value(&self) -> &str {
        &self.value
    }
}

/// DurationValue 型
pub struct DurationValue {
    value: f64,
    unit: String,
}

#[Object]
impl DurationValue {
    async fn value(&self) -> f64 {
        self.value
    }

    async fn unit(&self) -> &str {
        &self.unit
    }
}

impl From<DomainDurationValue> for DurationValue {
    fn from(dv: DomainDurationValue) -> Self {
        Self {
            value: dv.value,
            unit: dv.unit,
        }
    }
}
