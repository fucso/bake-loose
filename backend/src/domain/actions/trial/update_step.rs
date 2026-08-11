//! Step の name/started_at を更新するアクション

use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::timezone::JstDateTime;
use crate::domain::validators::trial::{
    step_existence_validator, step_name_validator, step_status_validator, trial_status_validator,
};

pub use step_name_validator::Error as StepNameValidationError;

/// 指定したフィールドのみを部分更新する（`None` は未指定＝変更なし）
pub struct Command {
    pub step_id: StepId,
    pub name: Option<String>,
    /// None: 変更なし / Some(None): クリア / Some(Some(t)): t に設定
    pub started_at: Option<Option<JstDateTime>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
    InvalidStepName(StepNameValidationError),
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

    if let Some(name) = &command.name {
        step_name_validator::validate(name).map_err(Error::InvalidStepName)?;
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

    if let Some(name) = command.name {
        step.set_name(name);
    }
    if let Some(started_at) = command.started_at {
        step.start(started_at);
    }

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
    use crate::domain::models::project::ProjectId;
    use crate::domain::models::step::Step;

    fn trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "こね".to_string(), 0, None);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    fn base_command(step_id: StepId) -> Command {
        Command {
            step_id,
            name: None,
            started_at: None,
        }
    }

    #[test]
    fn test_update_step_name() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            name: Some("新名称".to_string()),
            ..base_command(step_id.clone())
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.name(), "新名称");
    }

    #[test]
    fn test_update_step_started_at_clear() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            started_at: Some(None),
            ..base_command(step_id.clone())
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.started_at(), None);
    }

    #[test]
    fn test_update_step_started_at_sets_value() {
        let (trial, step_id) = trial_with_step();
        let new_started_at =
            crate::domain::timezone::JstDateTime::now() - chrono::Duration::hours(1);
        let command = Command {
            started_at: Some(Some(new_started_at)),
            ..base_command(step_id.clone())
        };

        let updated = run(trial, command).unwrap();
        let step = updated.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert_eq!(step.started_at(), Some(&new_started_at));
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let (mut trial, step_id) = trial_with_step();
        trial.complete();

        let result = run(trial, base_command(step_id));
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_not_found() {
        let (trial, _) = trial_with_step();

        let result = run(trial, base_command(StepId::new()));
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

        let result = run(trial, base_command(step_id));
        assert_eq!(result, Err(Error::StepAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_new_name_empty() {
        let (trial, step_id) = trial_with_step();
        let command = Command {
            name: Some("".to_string()),
            ..base_command(step_id)
        };

        let result = run(trial, command);
        assert!(matches!(result, Err(Error::InvalidStepName(_))));
    }
}
