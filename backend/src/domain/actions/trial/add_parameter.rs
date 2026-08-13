//! Step にパラメーターを追加するアクション

use crate::domain::models::parameter::{Parameter, ParameterContent};
use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::{
    parameter_validator, step_existence_validator, step_status_validator, trial_status_validator,
};

pub use parameter_validator::Error as ParameterValidationError;

pub struct Command {
    pub step_id: StepId,
    pub content: ParameterContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
    InvalidParameter(ParameterValidationError),
}

impl From<trial_status_validator::Error> for Error {
    fn from(_: trial_status_validator::Error) -> Self {
        Error::TrialAlreadyCompleted
    }
}

impl From<step_existence_validator::Error> for Error {
    fn from(_: step_existence_validator::Error) -> Self {
        Error::StepNotFound
    }
}

impl From<step_status_validator::Error> for Error {
    fn from(_: step_status_validator::Error) -> Self {
        Error::StepAlreadyCompleted
    }
}

/// バリデーション
pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)?;
    step_existence_validator::require_exists(state, &command.step_id)?;
    let step = state
        .step(&command.step_id)
        .expect("step existence already validated");
    step_status_validator::require_in_progress(step)?;
    parameter_validator::validate(&command.content).map_err(Error::InvalidParameter)?;
    Ok(())
}

/// 状態遷移（validate成功前提）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    let step = state
        .steps_mut()
        .iter_mut()
        .find(|s| s.id() == &command.step_id)
        .expect("validated to exist");
    step.add_parameter(Parameter::new(step.id().clone(), command.content));

    state
}

/// validate + execute
pub fn run(state: Trial, command: Command) -> Result<Trial, Error> {
    validate(&state, &command)?;
    Ok(execute(state, command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::models::parameter::ParameterValue;
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;

    fn trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    #[test]
    fn test_add_parameter_appends_to_step() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            step_id: step_id.clone(),
            content: ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.parameters().len(), 1);
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let (mut trial, step_id) = trial_with_step();
        trial.complete(None);
        let command = Command {
            step_id,
            content: ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        };

        let result = run(trial, command);
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_not_found() {
        let (trial, _) = trial_with_step();
        let command = Command {
            step_id: StepId::new(),
            content: ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        };

        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepNotFound));
    }

    #[test]
    fn test_returns_error_when_step_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let completed_step = Step::from_raw(
            StepId::new(),
            trial.id().clone(),
            "こね".to_string(),
            0,
            Some(crate::domain::timezone::JstDateTime::now()),
            Some(crate::domain::timezone::JstDateTime::now()),
            Vec::new(),
        );
        let step_id = completed_step.id().clone();
        trial.add_step(completed_step);
        let command = Command {
            step_id,
            content: ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        };

        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_parameter_invalid() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            step_id,
            content: ParameterContent::KeyValue {
                key: "強力粉".to_string(),
                value: ParameterValue::Quantity {
                    amount: 300.0,
                    unit: "   ".to_string(),
                },
            },
        };

        let result = run(trial, command);
        assert_eq!(
            result,
            Err(Error::InvalidParameter(
                ParameterValidationError::EmptyQuantityUnit
            ))
        );
    }
}
