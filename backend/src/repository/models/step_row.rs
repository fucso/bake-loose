//! StepRow DBモデル

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::models::parameter::Parameter;
use crate::domain::models::step::{Step, StepId};
use crate::domain::models::trial::TrialId;
use crate::domain::timezone::JstDateTime;

/// steps テーブルの行を表すDBモデル
#[derive(Debug, FromRow)]
pub struct StepRow {
    pub id: Uuid,
    pub trial_id: Uuid,
    pub name: String,
    pub position: i16,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl StepRow {
    /// Parameter一覧と合わせてドメインモデルに変換する
    pub fn into_domain(self, parameters: Vec<Parameter>) -> Step {
        Step::from_raw(
            StepId(self.id),
            TrialId(self.trial_id),
            self.name,
            self.position,
            self.started_at.map(JstDateTime::from_utc),
            self.completed_at.map(JstDateTime::from_utc),
            parameters,
        )
    }
}
