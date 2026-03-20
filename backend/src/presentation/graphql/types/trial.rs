//! Trial GraphQL 型
//!
//! ドメインモデルの Trial, Step, Parameter をラップした GraphQL 型。

use async_graphql::{Enum, InputObject, Object, ID};
use chrono::{DateTime, Utc};

use crate::domain::models::parameter::Parameter as DomainParameter;
use crate::domain::models::step::Step as DomainStep;
use crate::domain::models::trial::{Trial as DomainTrial, TrialStatus as DomainTrialStatus};

use super::parameter_content::ParameterContent;

/// GraphQL 用の Trial 型
pub struct Trial(pub DomainTrial);

#[Object]
impl Trial {
    async fn id(&self) -> ID {
        ID(self.0.id().0.to_string())
    }

    async fn project_id(&self) -> ID {
        ID(self.0.project_id().0.to_string())
    }

    async fn name(&self) -> Option<String> {
        self.0.name().map(|s| s.to_string())
    }

    async fn memo(&self) -> Option<String> {
        self.0.memo().map(|s| s.to_string())
    }

    async fn status(&self) -> TrialStatus {
        self.0.status().clone().into()
    }

    async fn steps(&self) -> Vec<Step> {
        self.0.steps().iter().map(|s| Step(s.clone())).collect()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        *self.0.created_at()
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        *self.0.updated_at()
    }
}

impl From<DomainTrial> for Trial {
    fn from(trial: DomainTrial) -> Self {
        Self(trial)
    }
}

/// GraphQL 用の TrialStatus enum
#[derive(Enum, Copy, Clone, PartialEq, Eq)]
pub enum TrialStatus {
    InProgress,
    Completed,
}

impl From<DomainTrialStatus> for TrialStatus {
    fn from(status: DomainTrialStatus) -> Self {
        match status {
            DomainTrialStatus::InProgress => TrialStatus::InProgress,
            DomainTrialStatus::Completed => TrialStatus::Completed,
        }
    }
}

/// GraphQL 用の Step 型
pub struct Step(pub DomainStep);

#[Object]
impl Step {
    async fn id(&self) -> ID {
        ID(self.0.id().0.to_string())
    }

    async fn name(&self) -> &str {
        self.0.name()
    }

    async fn position(&self) -> i32 {
        self.0.position() as i32
    }

    async fn started_at(&self) -> Option<DateTime<Utc>> {
        self.0.started_at().copied()
    }

    async fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.0.completed_at().copied()
    }

    async fn parameters(&self) -> Vec<Parameter> {
        self.0
            .parameters()
            .iter()
            .map(|p| Parameter(p.clone()))
            .collect()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        *self.0.created_at()
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        *self.0.updated_at()
    }
}

/// GraphQL 用の Parameter 型
pub struct Parameter(pub DomainParameter);

#[Object]
impl Parameter {
    async fn id(&self) -> ID {
        ID(self.0.id().0.to_string())
    }

    async fn content(&self) -> ParameterContent {
        self.0.content().clone().into()
    }

    async fn created_at(&self) -> DateTime<Utc> {
        *self.0.created_at()
    }

    async fn updated_at(&self) -> DateTime<Utc> {
        *self.0.updated_at()
    }
}

/// Trial 作成時の入力
#[derive(InputObject)]
pub struct CreateTrialInput {
    pub project_id: ID,
    pub name: Option<String>,
    pub memo: Option<String>,
    pub steps: Option<Vec<StepInput>>,
}

/// Step 入力
#[derive(InputObject)]
pub struct StepInput {
    pub name: String,
    pub started_at: Option<DateTime<Utc>>,
    pub parameters: Option<Vec<ParameterInput>>,
}

/// Parameter 入力
#[derive(InputObject)]
pub struct ParameterInput {
    pub key_value: Option<KeyValueInput>,
    pub duration: Option<DurationInput>,
    pub time_marker: Option<TimeMarkerInput>,
    pub text: Option<String>,
}

/// KeyValue 入力
#[derive(InputObject)]
pub struct KeyValueInput {
    pub key: String,
    pub text_value: Option<String>,
    pub quantity: Option<QuantityInput>,
}

/// Quantity 入力
#[derive(InputObject)]
pub struct QuantityInput {
    pub amount: f64,
    pub unit: String,
}

/// Duration 入力
#[derive(InputObject)]
pub struct DurationInput {
    pub value: f64,
    pub unit: String,
    pub note: String,
}

/// TimeMarker 入力
#[derive(InputObject)]
pub struct TimeMarkerInput {
    pub value: f64,
    pub unit: String,
    pub note: String,
}

/// Trial 更新時の入力
#[derive(InputObject)]
pub struct UpdateTrialInput {
    pub id: ID,
    pub name: Option<String>,
    pub memo: Option<String>,
}

/// Step 追加時の入力
#[derive(InputObject)]
pub struct AddStepInput {
    pub trial_id: ID,
    pub name: String,
    pub started_at: Option<DateTime<Utc>>,
    pub parameters: Option<Vec<ParameterInput>>,
}

/// Step 更新時の入力
#[derive(InputObject)]
pub struct UpdateStepInput {
    pub trial_id: ID,
    pub step_id: ID,
    pub name: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub add_parameters: Option<Vec<ParameterInput>>,
    pub remove_parameter_ids: Option<Vec<ID>>,
}

/// Step 完了時の入力
#[derive(InputObject)]
pub struct CompleteStepInput {
    pub trial_id: ID,
    pub step_id: ID,
    pub completed_at: Option<DateTime<Utc>>,
}
