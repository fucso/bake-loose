//! Step からパラメーターを削除するアクション

use crate::domain::models::parameter::ParameterId;
use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::{
    step_existence_validator, step_status_validator, trial_status_validator,
};

pub struct Command {
    pub step_id: StepId,
    pub parameter_id: ParameterId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
    ParameterNotFound,
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

    if step.parameter(&command.parameter_id).is_none() {
        return Err(Error::ParameterNotFound);
    }
    Ok(())
}

/// 状態遷移（validate成功前提）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    let step = state
        .steps_mut()
        .iter_mut()
        .find(|s| s.id() == &command.step_id)
        .expect("validated to exist");
    step.remove_parameter(&command.parameter_id);

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
    use crate::domain::models::parameter::{Parameter, ParameterContent};
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;

    fn trial_with_step_and_parameter() -> (Trial, StepId, ParameterId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let mut step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        let parameter = Parameter::new(
            step_id.clone(),
            ParameterContent::Text {
                value: "打ち粉を追加".to_string(),
            },
        );
        let parameter_id = parameter.id().clone();
        step.add_parameter(parameter);
        trial.add_step(step);
        (trial, step_id, parameter_id)
    }

    #[test]
    fn test_remove_parameter_removes_from_step() {
        let (trial, step_id, parameter_id) = trial_with_step_and_parameter();
        let command = Command {
            step_id: step_id.clone(),
            parameter_id,
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.parameters().is_empty());
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let (mut trial, step_id, parameter_id) = trial_with_step_and_parameter();
        trial.complete(None);
        let command = Command {
            step_id,
            parameter_id,
        };

        let result = run(trial, command);
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_not_found() {
        let (trial, _, parameter_id) = trial_with_step_and_parameter();
        let command = Command {
            step_id: StepId::new(),
            parameter_id,
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
            parameter_id: ParameterId::new(),
        };

        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_parameter_not_found() {
        let (trial, step_id, _) = trial_with_step_and_parameter();
        let command = Command {
            step_id,
            parameter_id: ParameterId::new(),
        };

        let result = run(trial, command);
        assert_eq!(result, Err(Error::ParameterNotFound));
    }
}
