//! Trial GraphQL 型
//!
//! ドメインモデルの Trial/Step/Parameter をラップした GraphQL 型。

use async_graphql::{Enum, InputObject, Json, MaybeUndefined, Object, ID};
use chrono::{DateTime, FixedOffset};

use crate::domain::models::parameter::{Parameter as DomainParameter, ParameterContent};
use crate::domain::models::step::Step as DomainStep;
use crate::domain::models::trial::{Trial as DomainTrial, TrialStatus as DomainTrialStatus};

/// GraphQL 用の TrialStatus 型
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
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

/// GraphQL 用の Parameter 型
///
/// ドメインモデルを直接公開せず、ラッパー型として定義する。
pub struct Parameter(pub DomainParameter);

#[Object]
impl Parameter {
    /// パラメーターID
    async fn id(&self) -> ID {
        ID(self.0.id().0.to_string())
    }

    /// パラメーターの内容（JSONスカラーとして入出力）
    async fn content(&self) -> Json<ParameterContent> {
        Json(self.0.content().clone())
    }
}

impl From<DomainParameter> for Parameter {
    fn from(parameter: DomainParameter) -> Self {
        Self(parameter)
    }
}

/// GraphQL 用の Step 型
///
/// ドメインモデルを直接公開せず、ラッパー型として定義する。
pub struct Step(pub DomainStep);

#[Object]
impl Step {
    /// StepID
    async fn id(&self) -> ID {
        ID(self.0.id().0.to_string())
    }

    /// Step名
    async fn name(&self) -> &str {
        self.0.name()
    }

    /// Trial内での位置（0始まり）
    async fn position(&self) -> i16 {
        self.0.position()
    }

    /// 開始日時（JST）
    async fn started_at(&self) -> Option<DateTime<FixedOffset>> {
        self.0.started_at().copied()
    }

    /// 完了日時（JST）
    async fn completed_at(&self) -> Option<DateTime<FixedOffset>> {
        self.0.completed_at().copied()
    }

    /// 完了済みかどうか
    async fn is_completed(&self) -> bool {
        self.0.is_completed()
    }

    /// この Step に紐づく Parameter 一覧
    async fn parameters(&self) -> Vec<Parameter> {
        self.0
            .parameters()
            .iter()
            .cloned()
            .map(Parameter::from)
            .collect()
    }
}

impl From<DomainStep> for Step {
    fn from(step: DomainStep) -> Self {
        Self(step)
    }
}

/// GraphQL 用の Trial 型
///
/// ドメインモデルを直接公開せず、ラッパー型として定義する。
pub struct Trial(pub DomainTrial);

#[Object]
impl Trial {
    /// TrialID
    async fn id(&self) -> ID {
        ID(self.0.id().0.to_string())
    }

    /// 紐づくプロジェクトID
    async fn project_id(&self) -> ID {
        ID(self.0.project_id().0.to_string())
    }

    /// Trial名
    async fn name(&self) -> Option<&str> {
        self.0.name()
    }

    /// メモ
    async fn memo(&self) -> Option<&str> {
        self.0.memo()
    }

    /// ステータス
    async fn status(&self) -> TrialStatus {
        (*self.0.status()).into()
    }

    /// この Trial に紐づく Step 一覧
    async fn steps(&self) -> Vec<Step> {
        self.0.steps().iter().cloned().map(Step::from).collect()
    }
}

impl From<DomainTrial> for Trial {
    fn from(trial: DomainTrial) -> Self {
        Self(trial)
    }
}

/// Trial作成時の入力
#[derive(InputObject)]
pub struct CreateTrialInput {
    pub project_id: ID,
    pub name: Option<String>,
    pub memo: Option<String>,
}

/// Trial更新時の入力
///
/// 未指定（undefined）のフィールドは変更なし、null 指定でクリアする。
#[derive(InputObject)]
pub struct UpdateTrialInput {
    pub name: MaybeUndefined<String>,
    pub memo: MaybeUndefined<String>,
}

/// Step追加時の入力
#[derive(InputObject)]
pub struct AddStepInput {
    pub name: String,
    pub started_at: Option<DateTime<FixedOffset>>,
    #[graphql(default)]
    pub parameters: Vec<Json<ParameterContent>>,
}

/// Step更新時の入力
///
/// `started_at` は未指定（undefined）で変更なし、null 指定でクリアする。
#[derive(InputObject)]
pub struct UpdateStepInput {
    pub name: Option<String>,
    pub started_at: MaybeUndefined<DateTime<FixedOffset>>,
    #[graphql(default)]
    pub add_parameters: Vec<Json<ParameterContent>>,
    #[graphql(default)]
    pub remove_parameter_ids: Vec<ID>,
}
