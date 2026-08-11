//! TrialMutation リゾルバー

use async_graphql::{Context, ErrorExtensions, MaybeUndefined, Object, Result, ID};
use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::domain::actions::trial::{
    add_step as add_step_action, update_trial as update_trial_action,
};
use crate::domain::models::parameter::{ParameterContent, ParameterId};
use crate::domain::models::project::ProjectId;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::TrialId;
use crate::domain::timezone::JstDateTime;
use crate::presentation::graphql::context::ContextExt;
use crate::presentation::graphql::error::UserFacingError;
use crate::presentation::graphql::types::trial::{
    AddStepInput, CreateTrialInput, Step, Trial, UpdateStepInput, UpdateTrialInput,
};
use crate::use_case::trial::{
    add_step, complete_step, complete_trial, create_trial, update_step, update_trial,
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

/// Trial 関連のミューテーション
#[derive(Default)]
pub struct TrialMutation;

#[Object]
impl TrialMutation {
    /// Trialを作成する
    async fn create_trial(&self, ctx: &Context<'_>, input: CreateTrialInput) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;
        let project_id = ProjectId(parse_uuid(&input.project_id, "project")?);

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
        let trial_id = TrialId(parse_uuid(&id, "trial")?);

        let command = update_trial_action::Command {
            name: to_double_option(input.name),
            memo: to_double_option(input.memo),
        };
        let use_case_input = update_trial::Input { trial_id, command };

        let trial = update_trial::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// Trialを完了状態にする
    async fn complete_trial(&self, ctx: &Context<'_>, id: ID) -> Result<Trial> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = TrialId(parse_uuid(&id, "trial")?);

        let trial = complete_trial::execute(&mut uow, complete_trial::Input { trial_id })
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        Ok(trial.into())
    }

    /// Trialに新しいStepを追加する
    async fn add_step(&self, ctx: &Context<'_>, trial_id: ID, input: AddStepInput) -> Result<Step> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = TrialId(parse_uuid(&trial_id, "trial")?);

        let command = add_step_action::Command {
            name: input.name,
            started_at: input.started_at.map(JstDateTime::from_fixed_offset),
        };
        let use_case_input = add_step::Input { trial_id, command };

        let trial = add_step::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        let step = trial
            .steps()
            .last()
            .cloned()
            .expect("step must be appended after add_step");

        Ok(step.into())
    }

    /// Stepの内容を更新する
    async fn update_step(
        &self,
        ctx: &Context<'_>,
        trial_id: ID,
        step_id: ID,
        input: UpdateStepInput,
    ) -> Result<Step> {
        let mut uow = ctx.create_unit_of_work()?;
        let trial_id = TrialId(parse_uuid(&trial_id, "trial")?);
        let step_id = StepId(parse_uuid(&step_id, "step")?);

        let add_parameters: Vec<ParameterContent> =
            input.add_parameters.into_iter().map(|p| p.0).collect();
        let remove_parameter_ids = input
            .remove_parameter_ids
            .iter()
            .map(|id| parse_uuid(id, "parameter").map(ParameterId))
            .collect::<Result<Vec<_>>>()?;

        let use_case_input = update_step::Input {
            trial_id,
            step_id: step_id.clone(),
            name: input.name,
            started_at: to_double_option(input.started_at)
                .map(|opt| opt.map(JstDateTime::from_fixed_offset)),
            add_parameters,
            remove_parameter_ids,
        };

        let trial = update_step::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        let step = trial
            .steps()
            .iter()
            .find(|s| s.id() == &step_id)
            .cloned()
            .expect("step must exist after update_step");

        Ok(step.into())
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
        let trial_id = TrialId(parse_uuid(&trial_id, "trial")?);
        let step_id = StepId(parse_uuid(&step_id, "step")?);

        let use_case_input = complete_step::Input {
            trial_id,
            step_id: step_id.clone(),
            completed_at: completed_at.map(JstDateTime::from_fixed_offset),
        };

        let trial = complete_step::execute(&mut uow, use_case_input)
            .await
            .map_err(|e| e.to_user_facing().extend())?;

        let step = trial
            .steps()
            .iter()
            .find(|s| s.id() == &step_id)
            .cloned()
            .expect("step must exist after complete_step");

        Ok(step.into())
    }
}
