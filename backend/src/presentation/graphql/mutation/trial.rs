//! Trial ミューテーションリゾルバー
//!
//! トライアルに関するミューテーションを処理する。

use async_graphql::{Context, ErrorExtensions, Object, Result, ID};
use uuid::Uuid;

use crate::domain::actions::trial::create_trial as create_trial_action;
use crate::domain::models::parameter::{DurationValue, ParameterContent, ParameterValue};
use crate::domain::models::project::ProjectId;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::TrialId;
use crate::presentation::graphql::context::ContextExt;
use crate::presentation::graphql::error::UserFacingError;
use crate::presentation::graphql::types::trial::{
    AddStepInput, CompleteStepInput, CreateTrialInput, ParameterInput, Trial, UpdateStepInput,
    UpdateTrialInput,
};
use crate::use_case::trial::{
    add_step, complete_step, complete_trial, create_trial, update_step, update_trial,
};

/// Trial 関連のミューテーション
#[derive(Default)]
pub struct TrialMutation;

#[Object]
impl TrialMutation {
    /// トライアルを作成する
    async fn create_trial(&self, ctx: &Context<'_>, input: CreateTrialInput) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;

        let project_id = parse_project_id(&input.project_id)?;
        let steps = input
            .steps
            .unwrap_or_default()
            .into_iter()
            .map(|s| {
                let parameters = s
                    .parameters
                    .unwrap_or_default()
                    .into_iter()
                    .map(convert_parameter_input)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(create_trial_action::StepInput {
                    name: s.name,
                    started_at: s.started_at,
                    parameters,
                })
            })
            .collect::<Result<Vec<_>, async_graphql::Error>>()?;

        let uc_input = create_trial::Input {
            project_id,
            name: input.name,
            memo: input.memo,
            steps,
        };

        let trial = create_trial::execute(&mut uow, uc_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// トライアルを更新する
    async fn update_trial(&self, ctx: &Context<'_>, input: UpdateTrialInput) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;

        let trial_id = parse_trial_id(&input.id)?;

        let uc_input = update_trial::Input {
            trial_id,
            name: input.name,
            memo: input.memo,
        };

        let trial = update_trial::execute(&mut uow, uc_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// トライアルを完了する
    async fn complete_trial(&self, ctx: &Context<'_>, id: ID) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;

        let trial_id = parse_trial_id(&id)?;

        let uc_input = complete_trial::Input { trial_id };

        let trial = complete_trial::execute(&mut uow, uc_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// ステップを追加する
    async fn add_step(&self, ctx: &Context<'_>, input: AddStepInput) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;

        let trial_id = parse_trial_id(&input.trial_id)?;
        let parameters = input
            .parameters
            .unwrap_or_default()
            .into_iter()
            .map(convert_to_add_step_parameter)
            .collect::<Result<Vec<_>, _>>()?;

        let uc_input = add_step::Input {
            trial_id,
            name: input.name,
            started_at: input.started_at,
            parameters,
        };

        let trial = add_step::execute(&mut uow, uc_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// ステップを更新する
    async fn update_step(&self, ctx: &Context<'_>, input: UpdateStepInput) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;

        let trial_id = parse_trial_id(&input.trial_id)?;
        let step_id = parse_step_id(&input.step_id)?;

        let add_parameters = input
            .add_parameters
            .unwrap_or_default()
            .into_iter()
            .map(convert_to_update_step_parameter)
            .collect::<Result<Vec<_>, _>>()?;

        let remove_parameter_ids = input
            .remove_parameter_ids
            .unwrap_or_default()
            .into_iter()
            .map(|id| parse_parameter_id(&id))
            .collect::<Result<Vec<_>, _>>()?;

        let uc_input = update_step::Input {
            trial_id,
            step_id,
            name: input.name,
            started_at: input.started_at.map(Some),
            add_parameters,
            remove_parameter_ids,
        };

        let trial = update_step::execute(&mut uow, uc_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// ステップを完了する
    async fn complete_step(&self, ctx: &Context<'_>, input: CompleteStepInput) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;

        let trial_id = parse_trial_id(&input.trial_id)?;
        let step_id = parse_step_id(&input.step_id)?;

        let uc_input = complete_step::Input {
            trial_id,
            step_id,
            completed_at: input.completed_at,
        };

        let trial = complete_step::execute(&mut uow, uc_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }
}

// --- ヘルパー関数 ---

fn parse_trial_id(id: &ID) -> Result<TrialId, async_graphql::Error> {
    let uuid =
        Uuid::parse_str(&id.0).map_err(|_| async_graphql::Error::new("Invalid trial ID format"))?;
    Ok(TrialId(uuid))
}

fn parse_project_id(id: &ID) -> Result<ProjectId, async_graphql::Error> {
    let uuid = Uuid::parse_str(&id.0)
        .map_err(|_| async_graphql::Error::new("Invalid project ID format"))?;
    Ok(ProjectId(uuid))
}

fn parse_step_id(id: &ID) -> Result<StepId, async_graphql::Error> {
    let uuid =
        Uuid::parse_str(&id.0).map_err(|_| async_graphql::Error::new("Invalid step ID format"))?;
    Ok(StepId(uuid))
}

fn parse_parameter_id(
    id: &ID,
) -> Result<crate::domain::models::parameter::ParameterId, async_graphql::Error> {
    let uuid = Uuid::parse_str(&id.0)
        .map_err(|_| async_graphql::Error::new("Invalid parameter ID format"))?;
    Ok(crate::domain::models::parameter::ParameterId(uuid))
}

/// GraphQL ParameterInput をドメインの ParameterContent に変換する
fn convert_parameter_content(
    input: ParameterInput,
) -> Result<ParameterContent, async_graphql::Error> {
    if let Some(kv) = input.key_value {
        let value = if let Some(text) = kv.text_value {
            ParameterValue::Text { value: text }
        } else if let Some(q) = kv.quantity {
            ParameterValue::Quantity {
                amount: q.amount,
                unit: q.unit,
            }
        } else {
            return Err(async_graphql::Error::new(
                "keyValue には textValue または quantity を指定してください",
            ));
        };
        Ok(ParameterContent::KeyValue { key: kv.key, value })
    } else if let Some(d) = input.duration {
        Ok(ParameterContent::Duration {
            duration: DurationValue::new(d.value, d.unit),
            note: d.note,
        })
    } else if let Some(tm) = input.time_marker {
        Ok(ParameterContent::TimeMarker {
            at: DurationValue::new(tm.value, tm.unit),
            note: tm.note,
        })
    } else if let Some(text) = input.text {
        Ok(ParameterContent::Text { value: text })
    } else {
        Err(async_graphql::Error::new(
            "パラメーターの種類を1つ指定してください",
        ))
    }
}

fn convert_parameter_input(
    input: ParameterInput,
) -> Result<create_trial_action::ParameterInput, async_graphql::Error> {
    let content = convert_parameter_content(input)?;
    Ok(create_trial_action::ParameterInput { content })
}

fn convert_to_add_step_parameter(
    input: ParameterInput,
) -> Result<add_step::ParameterInput, async_graphql::Error> {
    let content = convert_parameter_content(input)?;
    Ok(add_step::ParameterInput { content })
}

fn convert_to_update_step_parameter(
    input: ParameterInput,
) -> Result<update_step::ParameterInput, async_graphql::Error> {
    let content = convert_parameter_content(input)?;
    Ok(update_step::ParameterInput { content })
}
