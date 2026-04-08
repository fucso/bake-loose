//! complete_step アクション - Step を完了状態にする

use chrono::{DateTime, Utc};

use crate::domain::models::step::StepId;
use crate::domain::models::trial::Trial;
use crate::domain::validators::trial::{
    step_existence_validator, step_status_validator, trial_status_validator,
};

/// complete_step コマンド
pub struct Command {
    pub step_id: StepId,
    pub completed_at: Option<DateTime<Utc>>,
}

/// complete_step エラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TrialAlreadyCompleted,
    StepNotFound,
    StepAlreadyCompleted,
}

/// バリデーション
pub fn validate(state: &Trial, command: &Command) -> Result<(), Error> {
    trial_status_validator::require_in_progress(state)
        .map_err(|_| Error::TrialAlreadyCompleted)?;
    let step = step_existence_validator::require_exists(state, &command.step_id)
        .map_err(|_| Error::StepNotFound)?;
    step_status_validator::require_in_progress(step)
        .map_err(|_| Error::StepAlreadyCompleted)?;
    Ok(())
}

/// 状態遷移（validate 成功前提）
pub fn execute(mut state: Trial, command: Command) -> Trial {
    let completed_at = command.completed_at.unwrap_or_else(Utc::now);

    if let Some(step) = state
        .steps_mut()
        .iter_mut()
        .find(|s| s.id() == &command.step_id)
    {
        step.complete(completed_at);
    }

    state.touch();
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
    use crate::domain::models::trial::Trial;

    fn make_trial_with_step() -> (Trial, StepId) {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let step = Step::new(trial.id().clone(), "捏ね".to_string(), 0);
        let step_id = step.id().clone();
        trial.add_step(step);
        (trial, step_id)
    }

    #[test]
    fn test_complete_step_success() {
        let (trial, step_id) = make_trial_with_step();
        let command = Command {
            step_id: step_id.clone(),
            completed_at: None,
        };
        let result = run(trial, command).unwrap();
        let step = result.steps().iter().find(|s| s.id() == &step_id).unwrap();
        assert!(step.completed_at().is_some());
    }

    #[test]
    fn test_returns_error_when_trial_completed() {
        let (mut trial, step_id) = make_trial_with_step();
        trial.complete();
        let command = Command {
            step_id,
            completed_at: None,
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::TrialAlreadyCompleted));
    }

    #[test]
    fn test_returns_error_when_step_not_found() {
        let (trial, _) = make_trial_with_step();
        let command = Command {
            step_id: StepId::new(),
            completed_at: None,
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepNotFound));
    }

    #[test]
    fn test_returns_error_when_step_already_completed() {
        let mut trial = Trial::new(ProjectId::new(), None, None);
        let mut step = Step::new(trial.id().clone(), "捏ね".to_string(), 0);
        let step_id = step.id().clone();
        step.complete(Utc::now());
        trial.add_step(step);

        let command = Command {
            step_id,
            completed_at: None,
        };
        let result = run(trial, command);
        assert_eq!(result, Err(Error::StepAlreadyCompleted));
    }
}
