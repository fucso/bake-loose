//! Trial, Step, Parameter の DB モデル

use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::models::parameter::{Parameter, ParameterContent, ParameterId};
use crate::domain::models::project::ProjectId;
use crate::domain::models::step::{Step, StepId};
use crate::domain::models::trial::{Trial, TrialId, TrialStatus};

/// trials テーブルの行を表す DB モデル
#[derive(Debug, FromRow)]
pub struct TrialRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// steps テーブルの行を表す DB モデル
#[derive(Debug, FromRow)]
pub struct StepRow {
    pub id: Uuid,
    pub trial_id: Uuid,
    pub name: String,
    pub position: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// parameters テーブルの行を表す DB モデル
#[derive(Debug, FromRow)]
pub struct ParameterRow {
    pub id: Uuid,
    pub step_id: Uuid,
    pub content: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// DB の status 文字列からドメインの TrialStatus へ変換する
fn parse_trial_status(s: &str) -> TrialStatus {
    match s {
        "completed" => TrialStatus::Completed,
        _ => TrialStatus::InProgress,
    }
}

/// TrialStatus からDB の status 文字列へ変換する
pub fn trial_status_to_str(status: &TrialStatus) -> &'static str {
    match status {
        TrialStatus::InProgress => "in_progress",
        TrialStatus::Completed => "completed",
    }
}

impl TrialRow {
    /// TrialRow と関連する Steps からドメインの Trial を構築する
    pub fn into_trial(self, steps: Vec<Step>) -> Trial {
        Trial::from_raw(
            TrialId(self.id),
            ProjectId(self.project_id),
            self.name,
            self.memo,
            parse_trial_status(&self.status),
            steps,
            self.created_at,
            self.updated_at,
        )
    }
}

impl StepRow {
    /// StepRow と関連する Parameters からドメインの Step を構築する
    pub fn into_step(self, parameters: Vec<Parameter>) -> Step {
        Step::from_raw(
            StepId(self.id),
            TrialId(self.trial_id),
            self.name,
            self.position,
            self.started_at,
            self.completed_at,
            parameters,
            self.created_at,
            self.updated_at,
        )
    }
}

impl From<ParameterRow> for Parameter {
    fn from(row: ParameterRow) -> Self {
        let content: ParameterContent =
            serde_json::from_value(row.content).expect("Invalid ParameterContent JSON in DB");
        Parameter::from_raw(
            ParameterId(row.id),
            StepId(row.step_id),
            content,
            row.created_at,
            row.updated_at,
        )
    }
}
