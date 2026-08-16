//! TrialMutation リゾルバー

use async_graphql::{Context, ErrorExtensions, Json, MaybeUndefined, Object, Result, ID};
use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

// ParameterContent は Trial の GraphQL Json スカラーの入出力形状そのものであり、
// フラットな引数へ分解する対象ではない値オブジェクトのため、そのまま利用する。
use crate::domain::models::parameter::{Parameter as DomainParameter, ParameterContent};
use crate::domain::models::step::Step as DomainStep;
use crate::domain::models::trial::Trial as DomainTrial;
use crate::presentation::graphql::context::ContextExt;
use crate::presentation::graphql::error::{GraphQLError, UserFacingError};
use crate::presentation::graphql::types::trial::{
    AddStepInput, CreateTrialInput, Parameter, Step, Trial, UpdateStepInput, UpdateTrialInput,
};
use crate::use_case::trial::{
    add_parameter, add_step, complete_step, complete_trial, create_trial, remove_parameter,
    update_parameter, update_step, update_trial,
};

fn parse_uuid(id: &ID, label: &str) -> Result<Uuid> {
    Uuid::parse_str(&id.0)
        .map_err(|_| async_graphql::Error::new(format!("Invalid {label} ID format")))
}

fn to_double_option<T>(value: MaybeUndefined<T>) -> Option<Option<T>> {
    match value {
        MaybeUndefined::Undefined => None,
        MaybeUndefined::Null => Some(None),
        MaybeUndefined::Value(v) => Some(Some(v)),
    }
}

/// ユースケースの返却が不変条件を満たさない場合の内部エラー
///
/// ユースケースが検証済みの trial_id/step_id/parameter_id を渡している前提のため
/// 通常は必ず見つかるが、将来ユースケースの返却仕様が変わった場合に
/// panic せず内部エラーとして返す。
fn missing_after_use_case(context: &str) -> async_graphql::Error {
    log::error!("{context}");
    GraphQLError::new("内部エラーが発生しました", "INTERNAL_ERROR").extend()
}

/// ユースケースが返した Trial から最後に追加された Step を取り出す（`addStep` 専用）
fn last_step(trial: &DomainTrial) -> Result<DomainStep> {
    trial.steps().last().cloned().ok_or_else(|| {
        missing_after_use_case("step must be appended after add_step but trial has no steps")
    })
}

/// ユースケースが返した Trial から特定の Step を取り出す
fn find_step(trial: &DomainTrial, step_id: Uuid) -> Result<DomainStep> {
    trial
        .steps()
        .iter()
        .find(|s| s.id().0 == step_id)
        .cloned()
        .ok_or_else(|| missing_after_use_case(&format!("step {step_id} not found after use case")))
}

/// ユースケースが返した Trial から特定の Step 内の Parameter を取り出す
fn find_parameter(
    trial: &DomainTrial,
    step_id: Uuid,
    parameter_id: Uuid,
) -> Result<DomainParameter> {
    let step = find_step(trial, step_id)?;

    step.parameters()
        .iter()
        .find(|p| p.id().0 == parameter_id)
        .cloned()
        .ok_or_else(|| {
            missing_after_use_case(&format!(
                "parameter {parameter_id} not found in step {step_id} after use case"
            ))
        })
}

/// Trial 関連のミューテーション
#[derive(Default)]
pub struct TrialMutation;

#[Object]
impl TrialMutation {
    /// Trialを作成する
    async fn create_trial(&self, ctx: &Context<'_>, input: CreateTrialInput) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;
        let project_id = parse_uuid(&input.project_id, "project")?;

        let use_case_input = create_trial::Input {
            project_id,
            name: input.name,
            memo: input.memo,
        };

        let trial = create_trial::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// Trialのname/memoを更新する
    async fn update_trial(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateTrialInput,
    ) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = parse_uuid(&id, "trial")?;

        let use_case_input = update_trial::Input {
            trial_id,
            name: to_double_option(input.name),
            memo: to_double_option(input.memo),
        };

        let trial = update_trial::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// Trialを完了状態にする
    async fn complete_trial(
        &self,
        ctx: &Context<'_>,
        id: ID,
        completed_at: Option<DateTime<FixedOffset>>,
    ) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = parse_uuid(&id, "trial")?;

        let use_case_input = complete_trial::Input {
            trial_id,
            completed_at,
        };

        let trial = complete_trial::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// Trialに新しいStepを追加する
    async fn add_step(&self, ctx: &Context<'_>, trial_id: ID, input: AddStepInput) -> Result<Step> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = parse_uuid(&trial_id, "trial")?;

        let use_case_input = add_step::Input {
            trial_id,
            name: input.name,
            started_at: input.started_at,
        };

        let trial = add_step::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(last_step(&trial)?.into())
    }

    /// Stepのname/started_atを更新する（パラメーターの追加・削除は addParameter/removeParameter で行う）
    async fn update_step(
        &self,
        ctx: &Context<'_>,
        trial_id: ID,
        step_id: ID,
        input: UpdateStepInput,
    ) -> Result<Step> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = parse_uuid(&trial_id, "trial")?;
        let step_id = parse_uuid(&step_id, "step")?;

        let use_case_input = update_step::Input {
            trial_id,
            step_id,
            name: input.name,
            started_at: to_double_option(input.started_at),
        };

        let trial = update_step::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(find_step(&trial, step_id)?.into())
    }

    /// Stepにパラメーターを追加する
    async fn add_parameter(
        &self,
        ctx: &Context<'_>,
        trial_id: ID,
        step_id: ID,
        content: Json<ParameterContent>,
    ) -> Result<Parameter> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = parse_uuid(&trial_id, "trial")?;
        let step_id = parse_uuid(&step_id, "step")?;

        let use_case_input = add_parameter::Input {
            trial_id,
            step_id,
            content: content.0,
        };

        let trial = add_parameter::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        // 追加されたパラメーターは Step の末尾に積まれる（domain::actions::trial::add_parameter の実装に依存）
        let step = find_step(&trial, step_id)?;
        let parameter = step.parameters().last().cloned().ok_or_else(|| {
            missing_after_use_case("parameter must be appended after add_parameter")
        })?;

        Ok(parameter.into())
    }

    /// Stepからパラメーターを削除する
    async fn remove_parameter(
        &self,
        ctx: &Context<'_>,
        trial_id: ID,
        step_id: ID,
        parameter_id: ID,
    ) -> Result<Step> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = parse_uuid(&trial_id, "trial")?;
        let step_id = parse_uuid(&step_id, "step")?;
        let parameter_id = parse_uuid(&parameter_id, "parameter")?;

        let use_case_input = remove_parameter::Input {
            trial_id,
            step_id,
            parameter_id,
        };

        let trial = remove_parameter::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(find_step(&trial, step_id)?.into())
    }

    /// 設定済みParameterの内容を更新する（末端の値のみ。種類の変更はできない）
    async fn update_parameter(
        &self,
        ctx: &Context<'_>,
        trial_id: ID,
        step_id: ID,
        parameter_id: ID,
        content: Json<ParameterContent>,
    ) -> Result<Parameter> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = parse_uuid(&trial_id, "trial")?;
        let step_id = parse_uuid(&step_id, "step")?;
        let parameter_id = parse_uuid(&parameter_id, "parameter")?;

        let use_case_input = update_parameter::Input {
            trial_id,
            step_id,
            parameter_id,
            content: content.0,
        };

        let trial = update_parameter::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(find_parameter(&trial, step_id, parameter_id)?.into())
    }

    /// Stepを完了状態にする
    async fn complete_step(
        &self,
        ctx: &Context<'_>,
        trial_id: ID,
        step_id: ID,
        completed_at: Option<DateTime<FixedOffset>>,
    ) -> Result<Step> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = parse_uuid(&trial_id, "trial")?;
        let step_id = parse_uuid(&step_id, "step")?;

        let use_case_input = complete_step::Input {
            trial_id,
            step_id,
            completed_at,
        };

        let trial = complete_step::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(find_step(&trial, step_id)?.into())
    }
}
