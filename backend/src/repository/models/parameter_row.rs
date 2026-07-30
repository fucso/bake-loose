//! ParameterRow DBモデル

use chrono::{DateTime, Utc};
use sqlx::types::Json;
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::models::parameter::{Parameter, ParameterContent, ParameterId};
use crate::domain::models::step::StepId;

/// parameters テーブルの行を表すDBモデル
#[derive(Debug, FromRow)]
pub struct ParameterRow {
    pub id: Uuid,
    pub step_id: Uuid,
    pub content: Json<ParameterContent>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ParameterRow> for Parameter {
    fn from(row: ParameterRow) -> Self {
        Parameter::from_raw(ParameterId(row.id), StepId(row.step_id), row.content.0)
    }
}
